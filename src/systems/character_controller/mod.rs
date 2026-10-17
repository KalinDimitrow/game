use avian3d::prelude::{Collider, LinearVelocity, LockedAxes, RigidBody};
use bevy::world_serialization::WorldInstanceReady;

use crate::prelude::*;
use crate::systems::terrain::{GRID_DEPTH, GRID_WIDTH};

const MOVE_SPEED: f32 = 6.0;
const JUMP_SPEED: f32 = 6.0;
const MOUSE_SENSITIVITY: f32 = 0.005;
const PITCH_LIMIT: f32 = 1.4;

// Fox.glb's mesh-local bounding box is ~79 units tall (measured from the
// glTF's accessor min/max, origin at its feet) vs. the capsule collider's
// 1.8 (length 1.0 + 2 * radius 0.4) - this scale brings the model to the
// same height as the capsule from the previous iteration.
const FOX_SCALE: f32 = 1.8 / 79.03;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CharacterMovement>::default());
        app.add_systems(Startup, spawn_character);
        app.add_systems(
            Update,
            (
                (move_character, apply_character_animation).chain(),
                follow_character,
                spawn_fox_visual_when_ready.run_if(not(resource_exists::<CharacterAnimations>)),
            ),
        );
    }
}

#[derive(Resource)]
struct FoxHandle(Handle<Gltf>);

#[derive(Resource)]
struct CharacterAnimations {
    // Indexed by `CharacterAnimationState` variant order: Idle, Walking, Jumping.
    indices: Vec<AnimationNodeIndex>,
    graph_handle: Handle<AnimationGraph>,
}

fn spawn_character(mut commands: Commands, asset_server: Res<AssetServer>) {
    let spawn = Vec3::new(GRID_WIDTH as f32 / 2.0, 3.0, GRID_DEPTH as f32 / 2.0);

    let mut movement_input = InputMap::<CharacterMovement>::default();
    movement_input
        .insert(CharacterMovement::Left, KeyCode::KeyA)
        .insert(CharacterMovement::Right, KeyCode::KeyD)
        .insert(CharacterMovement::Forward, KeyCode::KeyW)
        .insert(CharacterMovement::Backward, KeyCode::KeyS)
        .insert(CharacterMovement::Jump, KeyCode::Space);

    // No visible mesh here - the capsule is purely the physics collider. The
    // Fox model is spawned as a child once its glTF scene finishes loading
    // (see `spawn_fox_visual_when_ready`).
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

    commands.insert_resource(FoxHandle(asset_server.load(FOX_PATH)));
}

fn spawn_fox_visual_when_ready(
    mut commands: Commands,
    fox_handle: Res<FoxHandle>,
    asset_server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    character: Single<Entity, With<Character>>,
) {
    if !asset_server.is_loaded_with_dependencies(&fox_handle.0) {
        return;
    }

    let fox = gltfs
        .get(&fox_handle.0)
        .expect("fox glTF should be loaded by now");

    // Order matches `CharacterAnimationState`'s variant order (Idle, Walking,
    // Jumping) - "Survey" is the closest thing Fox.glb has to an idle pose.
    let (graph, node_indices) = AnimationGraph::from_clips([
        fox.named_animations["Survey"].clone(),
        fox.named_animations["Walk"].clone(),
        fox.named_animations["Run"].clone(),
    ]);
    let graph_handle = graphs.add(graph);

    commands.insert_resource(CharacterAnimations {
        indices: node_indices,
        graph_handle,
    });

    commands.entity(*character).with_children(|parent| {
        parent
            .spawn((
                WorldAssetRoot(
                    fox.default_scene
                        .clone()
                        .expect("fox glTF should have a default scene"),
                ),
                Transform::from_xyz(0.0, -0.9, 0.0).with_scale(Vec3::splat(FOX_SCALE)),
            ))
            .observe(on_fox_scene_ready);
    });
}

// Fires once the Fox's scene (and its `AnimationPlayer`) has actually spawned
// - glTF scene loading is async, so this can't just happen in the system above.
fn on_fox_scene_ready(
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
// player doesn't exist until the Fox's glTF scene finishes loading (which can
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
