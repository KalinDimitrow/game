use avian3d::prelude::{Collider, LinearVelocity, LockedAxes, RigidBody};
use bevy::world_serialization::WorldInstanceReady;

use crate::prelude::*;
use crate::systems::terrain::{LevelConfig, spawn_terrain};

const MOVE_SPEED: f32 = 6.0;
const JUMP_SPEED: f32 = 6.0;
const MOUSE_SENSITIVITY: f32 = 0.005;
const PITCH_LIMIT: f32 = 1.4;
// Radians/sec-ish slerp rate for turning the visual to face travel direction -
// fast enough to feel responsive without snapping instantly on direction changes.
const VISUAL_TURN_SPEED: f32 = 10.0;

// character.glb here is from the Blender FBX pipeline
// (convert_fbx_to_glb.sh), which bakes the source .fbx's unit scale
// directly into vertex data on import - mesh-local bounding box is
// already ~1.61 units tall (meters), unlike the old assimp pipeline's
// raw/un-unit-converted ~161 units. Scaled to match the capsule
// collider's 1.8 (length 1.0 + 2 * radius 0.4).
const CHARACTER_SCALE: f32 = 1.8 / 1.61;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CharacterMovement>::default());
        app.add_systems(Startup, spawn_character.after(spawn_terrain));
        app.add_systems(
            Update,
            (
                (move_character, apply_character_animation, orient_character_visual).chain(),
                follow_character,
                spawn_character_visual_when_ready.run_if(not(resource_exists::<CharacterAnimations>)),
            ),
        );
    }
}

// The mesh/skeleton and each animation are separate glTF assets (see
// `CHARACTER_MESH_PATH` and friends) - Bevy matches clips to scene
// entities by bone name, not by source file, so these just need to be
// loaded independently and combined once all of them are ready.
#[derive(Resource)]
struct CharacterGltfHandles {
    mesh: Handle<Gltf>,
    idle: Handle<Gltf>,
    walking: Handle<Gltf>,
    jumping: Handle<Gltf>,
}

#[derive(Resource)]
struct CharacterAnimations {
    // Indexed by `CharacterAnimationState` variant order: Idle, Walking, Jumping.
    indices: Vec<AnimationNodeIndex>,
    graph_handle: Handle<AnimationGraph>,
}

fn spawn_character(mut commands: Commands, asset_server: Res<AssetServer>, level: Res<LevelConfig>) {
    // Falls back to the grid center if the level doesn't define a `PlayerSpawn`
    // POI - a level file omitting it is plausible author input, not a bug.
    let (grid_x, grid_z) = level
        .player_spawn()
        .unwrap_or((level.grid_width / 2, level.grid_depth / 2));
    let spawn = Vec3::new(grid_x as f32 * level.cell_size, 3.0, grid_z as f32 * level.cell_size);

    let mut movement_input = InputMap::<CharacterMovement>::default();
    movement_input
        .insert(CharacterMovement::Left, KeyCode::KeyA)
        .insert(CharacterMovement::Right, KeyCode::KeyD)
        .insert(CharacterMovement::Forward, KeyCode::KeyW)
        .insert(CharacterMovement::Backward, KeyCode::KeyS)
        .insert(CharacterMovement::Jump, KeyCode::Space);

    // No visible mesh here - the capsule is purely the physics collider. The
    // character model is spawned as a child once its glTF scene finishes
    // loading (see `spawn_character_visual_when_ready`).
    commands.spawn((
        Character,
        CharacterAnimationState::default(),
        RigidBody::Dynamic,
        Collider::capsule(0.4, 1.0),
        LockedAxes::ROTATION_LOCKED,
        Transform::from_translation(spawn),
        movement_input,
    ));

    commands.spawn((
        Camera3d::default(),
        ThirdPersonCamera::default(),
        Transform::from_xyz(spawn.x, spawn.y + 2.0, spawn.z + 6.0).looking_at(spawn, Vec3::Y),
        InputMap::default().with_dual_axis(CameraMovement::Pan, MouseMove::default()),
    ));

    commands.insert_resource(CharacterGltfHandles {
        mesh: asset_server.load(CHARACTER_MESH_PATH),
        idle: asset_server.load(CHARACTER_IDLE_PATH),
        walking: asset_server.load(CHARACTER_WALKING_PATH),
        jumping: asset_server.load(CHARACTER_JUMPING_PATH),
    });
}

fn spawn_character_visual_when_ready(
    mut commands: Commands,
    gltf_handles: Res<CharacterGltfHandles>,
    asset_server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    character: Single<Entity, With<Character>>,
) {
    let handles = [
        &gltf_handles.mesh,
        &gltf_handles.idle,
        &gltf_handles.walking,
        &gltf_handles.jumping,
    ];
    if !handles.iter().all(|handle| asset_server.is_loaded_with_dependencies(*handle)) {
        return;
    }

    let mesh_gltf = gltfs
        .get(&gltf_handles.mesh)
        .expect("character mesh glTF should be loaded by now");
    let idle_gltf = gltfs
        .get(&gltf_handles.idle)
        .expect("idle glTF should be loaded by now");
    let walking_gltf = gltfs
        .get(&gltf_handles.walking)
        .expect("walking glTF should be loaded by now");
    let jumping_gltf = gltfs
        .get(&gltf_handles.jumping)
        .expect("jumping glTF should be loaded by now");

    // Each animation-only file has exactly one clip, named after its own
    // source file by the conversion script - take whichever one it is
    // rather than hardcoding an exact internal name.
    let idle_clip = idle_gltf
        .named_animations
        .values()
        .next()
        .expect("idle glTF should have an animation clip")
        .clone();
    let walking_clip = walking_gltf
        .named_animations
        .values()
        .next()
        .expect("walking glTF should have an animation clip")
        .clone();
    let jumping_clip = jumping_gltf
        .named_animations
        .values()
        .next()
        .expect("jumping glTF should have an animation clip")
        .clone();

    // Order matches `CharacterAnimationState`'s variant order (Idle, Walking,
    // Jumping). Bevy matches each clip's curves to scene entities by bone
    // name (AnimationTargetId), not by which file the clip came from, so
    // clips loaded from entirely separate glTF files apply correctly to the
    // mesh's skeleton once spawned below.
    let (graph, node_indices) = AnimationGraph::from_clips([idle_clip, walking_clip, jumping_clip]);
    let graph_handle = graphs.add(graph);

    commands.insert_resource(CharacterAnimations {
        indices: node_indices,
        graph_handle,
    });

    commands.entity(*character).with_children(|parent| {
        parent
            .spawn((
                CharacterVisual,
                WorldAssetRoot(
                    mesh_gltf
                        .default_scene
                        .clone()
                        .expect("character mesh glTF should have a default scene"),
                ),
                Transform::from_xyz(0.0, -0.9, 0.0).with_scale(Vec3::splat(CHARACTER_SCALE)),
            ))
            .observe(on_character_scene_ready);
    });
}

// Fires once the character's scene (and its `AnimationPlayer`) has actually
// spawned - glTF scene loading is async, so this can't just happen in the
// system above.
fn on_character_scene_ready(
    _ready: On<WorldInstanceReady>,
    mut commands: Commands,
    animations: Res<CharacterAnimations>,
    player: Single<(Entity, &mut AnimationPlayer)>,
) {
    let (entity, mut player) = player.into_inner();
    let mut transitions = AnimationTransitions::new();
    transitions
        .play(&mut player, animations.indices[0], std::time::Duration::ZERO)
        .repeat();

    commands
        .entity(entity)
        .insert(AnimationGraphHandle(animations.graph_handle.clone()))
        .insert(transitions);
}

fn move_character(
    character: Single<
        (
            &mut LinearVelocity,
            &ActionState<CharacterMovement>,
            &mut CharacterAnimationState,
        ),
        With<Character>,
    >,
    camera: Single<&ThirdPersonCamera>,
) {
    let (mut velocity, action_state, mut animation_state) = character.into_inner();

    // Movement is relative to the camera's yaw, not the capsule's own rotation
    // (which is locked) - standard third-person feel.
    let forward = Quat::from_rotation_y(camera.yaw) * Vec3::NEG_Z;
    let right = Quat::from_rotation_y(camera.yaw) * Vec3::X;

    let mut direction = Vec3::ZERO;
    if action_state.pressed(&CharacterMovement::Forward) {
        direction += forward;
    }
    if action_state.pressed(&CharacterMovement::Backward) {
        direction -= forward;
    }
    if action_state.pressed(&CharacterMovement::Right) {
        direction += right;
    }
    if action_state.pressed(&CharacterMovement::Left) {
        direction -= right;
    }

    let direction = direction.normalize_or_zero();
    velocity.0.x = direction.x * MOVE_SPEED;
    velocity.0.z = direction.z * MOVE_SPEED;

    // Grounded heuristic, not a real ground check: good enough for a collision
    // testbed, not for shipping. See the cleanup backlog.
    let grounded = velocity.0.y.abs() < 0.05;
    if grounded && action_state.just_pressed(&CharacterMovement::Jump) {
        velocity.0.y = JUMP_SPEED;
    }

    let airborne = velocity.0.y.abs() >= 0.05;
    let desired_state = if airborne {
        CharacterAnimationState::Jumping
    } else if direction.length_squared() > 0.0 {
        CharacterAnimationState::Walking
    } else {
        CharacterAnimationState::Idle
    };
    // Only write on an actual change - `Changed<CharacterAnimationState>` (read
    // by `apply_character_animation`) fires on every write, not just value
    // changes, and replaying an already-playing clip restarts it from frame 0.
    if *animation_state != desired_state {
        *animation_state = desired_state;
    }
}

// Separate `Query`s rather than `Single`s here on purpose: the animation
// player doesn't exist until the character's glTF scene finishes loading (which can
// take several frames), so "no match yet" is an expected, non-error state -
// `Single` would panic on it instead of just skipping.
fn apply_character_animation(
    character: Query<&CharacterAnimationState, (With<Character>, Changed<CharacterAnimationState>)>,
    animations: Option<Res<CharacterAnimations>>,
    mut player: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    let Some(animations) = animations else {
        return;
    };
    let Ok(state) = character.single() else {
        return;
    };
    let Ok((mut player, mut transitions)) = player.single_mut() else {
        return;
    };

    let index = match state {
        CharacterAnimationState::Idle => animations.indices[0],
        CharacterAnimationState::Walking => animations.indices[1],
        CharacterAnimationState::Jumping => animations.indices[2],
    };
    transitions
        .play(&mut player, index, std::time::Duration::from_millis(200))
        .repeat();
}

// Rotates the visual child (not `Character` itself, whose rotation is locked
// for physics - see `LockedAxes::ROTATION_LOCKED` in `spawn_character`) to
// face the current horizontal movement direction, slerping toward it so
// direction changes turn smoothly instead of snapping.
fn orient_character_visual(
    character: Single<&LinearVelocity, With<Character>>,
    mut visual: Query<&mut Transform, With<CharacterVisual>>,
    time: Res<Time>,
) {
    // The visual spawns as a child only once its glTF scene has loaded, so
    // "no match yet" is expected, not an error - same reasoning as
    // `apply_character_animation`'s player query.
    let Ok(mut transform) = visual.single_mut() else {
        return;
    };

    let horizontal = Vec3::new(character.0.x, 0.0, character.0.z);
    if horizontal.length_squared() < 1e-4 {
        return;
    }

    // The character.glb rest pose faces +Z rather than Bevy's usual -Z forward
    // (an artifact of the FBX->glb pipeline's axis conversion - see
    // `CHARACTER_SCALE`'s comment) - aligning -Z here pointed the model's back
    // at the travel direction, making walking look like reverse/moonwalking.
    let target_rotation = Quat::from_rotation_arc(Vec3::Z, horizontal.normalize());
    let turn = (VISUAL_TURN_SPEED * time.delta_secs()).min(1.0);
    transform.rotation = transform.rotation.slerp(target_rotation, turn);
}

type OrbitCameraQuery<'w> = (
    &'w mut Transform,
    &'w mut ThirdPersonCamera,
    &'w ActionState<CameraMovement>,
);

fn follow_character(
    character: Single<&Transform, (With<Character>, Without<Camera3d>)>,
    camera: Single<OrbitCameraQuery<'_>, (With<Camera3d>, Without<Character>)>,
) {
    let (mut camera_transform, mut orbit, action_state) = camera.into_inner();
    let pan = action_state.axis_pair(&CameraMovement::Pan);

    orbit.yaw -= pan.x * MOUSE_SENSITIVITY;
    orbit.pitch = (orbit.pitch - pan.y * MOUSE_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);

    let rotation = Quat::from_euler(EulerRot::YXZ, orbit.yaw, orbit.pitch, 0.0);
    let look_target = character.translation + Vec3::Y * 0.5;

    camera_transform.translation = look_target + rotation * Vec3::new(0.0, 0.0, orbit.distance);
    *camera_transform = camera_transform.looking_at(look_target, Vec3::Y);
}
