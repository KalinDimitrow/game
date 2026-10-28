use avian3d::prelude::LinearVelocity;

use crate::prelude::*;

const MOVE_SPEED: f32 = 6.0;
const JUMP_SPEED: f32 = 6.0;
const MOUSE_SENSITIVITY: f32 = 0.005;
const PITCH_LIMIT: f32 = 1.4;
// Radians/sec-ish slerp rate for turning the visual to face travel direction -
// fast enough to feel responsive without snapping instantly on direction changes.
const VISUAL_TURN_SPEED: f32 = 10.0;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CharacterMovement>::default());
        app.add_systems(Startup, load_character_gltf_handles);
        app.add_observer(attach_character_movement_input);
        app.add_observer(attach_camera_movement_input);
        app.add_systems(
            Update,
            check_character_assets_loaded.run_if(in_state(GameState::Loading)),
        );
        app.add_systems(OnEnter(GameState::Initialize), build_character_animation_graph);
        app.add_systems(
            Update,
            (attach_character_animation_graph, check_character_ready)
                .chain()
                .run_if(in_state(GameState::Initialize)),
        );
        app.add_systems(
            Update,
            (
                (
                    move_character,
                    apply_character_animation,
                    sync_character_visual_transform,
                    orient_character_visual,
                )
                    .chain(),
                follow_character,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// Attaches input bindings the instant `Character`/`Camera3d` is actually
// inserted - `InputMap` itself is never authored as RON data (its fields are
// boxed trait objects, not something to hand-write), so this is the bridge
// that makes `test_world.scn.ron`'s character/camera controllable without
// putting input config in the scene file.
//
// Observers rather than polling `Added<_>` every frame: each component type
// gets its own discrete, one-time event here (no other async condition to
// wait on, unlike `attach_character_animation_graph` below), which is exactly
// what observers are for - and `Camera3d` is Bevy's type, not ours, so a
// component hook (single-slot, exclusive) would risk colliding with one Bevy
// itself adds in a future version. Two observers, not one, because
// `On<Add<T>>` is monomorphic over a single component type.
fn attach_character_movement_input(trigger: On<Add<Character>>, mut commands: Commands) {
    let mut movement_input = InputMap::<CharacterMovement>::default();
    movement_input
        .insert(CharacterMovement::Left, KeyCode::KeyA)
        .insert(CharacterMovement::Right, KeyCode::KeyD)
        .insert(CharacterMovement::Forward, KeyCode::KeyW)
        .insert(CharacterMovement::Backward, KeyCode::KeyS)
        .insert(CharacterMovement::Jump, KeyCode::Space);
    commands.entity(trigger.entity).insert(movement_input);
}

fn attach_camera_movement_input(trigger: On<Add<Camera3d>>, mut commands: Commands) {
    commands.entity(trigger.entity).insert(
        InputMap::<CameraMovement>::default().with_dual_axis(CameraMovement::Pan, MouseMove::default()),
    );
}

// Keeps `CharacterVisual` positioned on `Character` without relying on ECS
// parent/child + `GlobalTransform` propagation - confirmed (via a throwaway
// diagnostic, since removed) that propagation never recomputed a
// `DynamicWorld`-spawned mesh parented under a `RigidBody`, even after 15+
// seconds and forcing `Transform` to re-register as changed. Matches
// avian3d's own documented stance on rigid-body hierarchies being flat:
// "write your own system to handle hierarchies differently." `Character`
// never rotates (`LockedAxes::ROTATION_LOCKED`), so a plain translation
// offset is enough - no rotation composition needed.
const CHARACTER_VISUAL_OFFSET: Vec3 = Vec3::new(0.0, -0.9, 0.0);

fn sync_character_visual_transform(
    character: Single<&Transform, (With<Character>, Without<CharacterVisual>)>,
    mut visual: Single<&mut Transform, (With<CharacterVisual>, Without<Character>)>,
) {
    visual.translation = character.translation + CHARACTER_VISUAL_OFFSET;
}

// Loads the mesh + 3 animation-only glTF files, each tagged with a clone of
// the same `AssetBarrierGuard` so `check_character_assets_loaded` can wait on
// all 4 with a single atomic read - see `AssetBarrier`'s doc comment for why
// that beats accumulating `AssetEvent`s (chiefly: it also releases on a
// *failed* load, which the event-based version never did).
//
// `mesh` isn't read by anything in this file anymore (`test_world.scn.ron`'s
// own `CharacterVisual` loads it independently via its own `WorldAssetRoot`,
// and `AssetServer` dedupes by path so this isn't a second read) - it's
// still here so "character assets are loaded" means all of them, not just
// the animation clips.
fn load_character_gltf_handles(mut commands: Commands, asset_server: Res<AssetServer>) {
    let (barrier, guard) = AssetBarrier::new();

    let load = |path: &'static str| {
        asset_server
            .load_builder()
            .with_guard(guard.clone())
            .load::<Gltf>(path)
    };
    commands.insert_resource(CharacterGltfHandles {
        mesh: load(CHARACTER_MESH_PATH),
        idle: load(CHARACTER_IDLE_PATH),
        walking: load(CHARACTER_WALKING_PATH),
        jumping: load(CHARACTER_JUMPING_PATH),
    });

    commands.insert_resource(barrier);
    // `guard` (the original, holding the initial count of 1) drops here, now
    // that every load has registered its own clone - that's what arms the
    // barrier.
}

// Builds the `AnimationGraph` from the 3 animation-only glTF files. Runs in
// `OnEnter(Initialize)`, not `Update` - no readiness guard needed here at
// all, since entering `Initialize` already guarantees (via
// `check_character_assets_loaded`) that every `CharacterGltfHandles` handle
// has finished loading. Runs exactly once, ever.
//
// (Tried collapsing this and `check_character_assets_loaded` into one
// blocking `Startup` system instead, via `AssetServer::wait_for_asset` +
// `bevy::tasks::block_on` - see `GameState`'s doc comment for why that
// deadlocks every time and was reverted.)
//
// Used to also spawn `CharacterVisual` itself (as a fallback for when
// nothing else had), back when `spawn_character` could produce a `Character`
// with no visual of its own - that fallback parented the mesh under
// `Character` via `with_children`, which would have silently hit the exact
// `GlobalTransform`-under-a-`RigidBody` bug `sync_character_visual_transform`
// exists to work around (see its doc comment). `spawn_character` is deleted
// and `test_world.scn.ron` always provides its own standalone `CharacterVisual`
// now, so that branch was both dead and a landmine - removed rather than kept
// "just in case."
fn build_character_animation_graph(
    mut commands: Commands,
    gltf_handles: Res<CharacterGltfHandles>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
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
    // mesh's skeleton once `attach_character_animation_graph` below attaches
    // this to it.
    let (graph, node_indices) = AnimationGraph::from_clips([idle_clip, walking_clip, jumping_clip]);
    let graph_handle = graphs.add(graph);

    commands.insert_resource(CharacterAnimations {
        indices: node_indices,
        graph_handle,
    });
}

// Picks up the character's `AnimationPlayer` once its glTF scene has
// actually finished spawning (async, so can't happen in the system above) -
// queries for "has a player but no graph yet" instead of `Added<AnimationPlayer>`,
// because `Added` is only true for the one frame the component appeared, and
// that frame isn't guaranteed to be the same frame `CharacterAnimations`
// finishes building (it depends on 3 separate async glTF loads) - confirmed
// via a throwaway diagnostic that the player existed in the world the whole
// time, just never on the same frame as `Added` and a ready `CharacterAnimations`
// at once. No `Added<AnimationGraphHandle>`-style window to miss this way.
fn attach_character_animation_graph(
    mut commands: Commands,
    animations: Option<Res<CharacterAnimations>>,
    mut players: Query<(Entity, &mut AnimationPlayer), Without<AnimationGraphHandle>>,
) {
    let Some(animations) = animations else {
        return;
    };
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut player, animations.indices[0], std::time::Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph_handle.clone()))
            .insert(transitions);
    }
}

// Transitions `Loading` -> `Initialize` once every glTF load tagged in
// `load_character_gltf_handles` has finished (or failed - see `AssetBarrier`).
// One atomic read per frame, and it stops running entirely once `Initialize`
// starts.
fn check_character_assets_loaded(
    barrier: Res<AssetBarrier>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if barrier.is_ready() {
        next_state.set(GameState::Initialize);
    }
}

// Transitions `Initialize` -> `Playing` once the remaining, entity-level
// facts are true: `Character` exists, and the visual's `AnimationPlayer` +
// `AnimationGraphHandle` are both attached (by `attach_character_animation_graph`
// just above, chained to run first so this can see the result the same
// frame). Unlike asset loading, there's no event for "an entity now has
// component X" - this has to stay a plain query, checked every frame, until
// it's observed; splitting `Loading`/`Initialize` apart couldn't remove this
// one; only the asset-readiness check above got to drop its guard.
fn check_character_ready(
    character: Query<(), With<Character>>,
    visual_ready: Query<(), (With<AnimationPlayer>, With<AnimationGraphHandle>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !character.is_empty() && !visual_ready.is_empty() {
        next_state.set(GameState::Playing);
    }
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
// for physics - see the `LockedAxes` component on `Character` in
// `test_world.scn.ron`) to face the current horizontal movement direction,
// slerping toward it so direction changes turn smoothly instead of snapping.
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
