use avian3d::prelude::LinearVelocity;
use bevy::world_serialization::WorldInstanceReady;

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
        app.add_observer(attach_character_movement_input);
        app.add_observer(attach_camera_movement_input);
        app.add_observer(mark_character_scene_spawned);
        // Both of these live in `resources::character_animations` - they exist
        // to produce `CharacterGltfHandles` and `CharacterAnimations` - but the
        // scheduling stays here, so the character's asset load order is visible
        // next to the systems that consume it.
        //
        // No `Loading` system of its own anymore: `load_character_gltf_handles`
        // registers its handles with `PendingAssets` and `CorePlugin`'s
        // `check_pending_assets` drives the transition for every plugin at once.
        app.add_systems(Startup, load_character_gltf_handles);
        app.add_systems(OnEnter(GameState::Initialize), build_character_animation_graph);
        app.add_systems(
            Update,
            finish_character_initialization.run_if(in_state(GameState::Initialize)),
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

// `WorldInstanceReady` fires on the entity owning the `WorldAssetRoot`, so for
// the character that's the `CharacterVisual` itself (entity 10 in
// `test_world.scn.ron`) once its nested glTF scene instance - and therefore
// the `AnimationPlayer` inside it - exists.
//
// This is the "there's no event for an entity existing" gap in `GameState`'s
// old doc comment, closed: `check_character_ready` used to poll for a
// `Character`, which was really standing in for "the world spawned" (a
// `Character` only ever comes from the scene). That worked, but made an entity
// query the de-facto readiness signal for an asset load. `bevy_world_serialization`
// publishes the actual completion event; this just narrows it to the instance
// we care about, since every `WorldAssetRoot` in the scene fires one.
fn mark_character_scene_spawned(
    trigger: On<WorldInstanceReady>,
    visuals: Query<(), With<CharacterVisual>>,
    mut commands: Commands,
) {
    if visuals.contains(trigger.entity) {
        commands.entity(trigger.entity).insert(CharacterSceneSpawned);
    }
}

// Attaches the animation graph to the character's `AnimationPlayer` and hands
// off to `Playing` - one system where there used to be two chained ones
// (`attach_character_animation_graph` + `check_character_ready`), since the
// transition is just "the attach succeeded" and splitting them meant the
// second had to re-derive what the first already knew.
//
// Deliberately order-independent rather than relying on `CharacterSceneSpawned`
// and `CharacterAnimations` arriving in a particular order: the graph is built
// in `OnEnter(Initialize)` (i.e. `StateTransition`) while the scene instance
// spawns in `SpawnScene`, two different schedules, and the margin between them
// is not something worth depending on. Returning early and retrying next frame
// costs a couple of frames during `Initialize` and nothing after.
//
// Same reason it queries "has a player but no graph yet" instead of
// `Added<AnimationPlayer>`: `Added` is only true for the one frame the
// component appeared, which isn't guaranteed to be a frame where the graph
// also exists - confirmed previously via a throwaway diagnostic that the
// player existed in the world the whole time, just never on the same frame as
// `Added` and a ready `CharacterAnimations` at once.
fn finish_character_initialization(
    mut commands: Commands,
    animations: Option<Res<CharacterAnimations>>,
    spawned: Query<(), With<CharacterSceneSpawned>>,
    mut players: Query<(Entity, &mut AnimationPlayer), Without<AnimationGraphHandle>>,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result<(), BevyError> {
    let Some(animations) = animations else {
        return Ok(());
    };
    if spawned.is_empty() {
        return Ok(());
    }

    let idle = *animations
        .nodes
        .get(&CharacterAnimationState::default())
        .ok_or("no animation node for the default `CharacterAnimationState`")?;

    let mut attached = false;
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut player, idle, std::time::Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph_handle.clone()))
            .insert(transitions);
        attached = true;
    }

    // Only advance once a player was actually set up. `CharacterSceneSpawned`
    // says the instance spawned, which is one sync point ahead of this query
    // seeing its entities.
    if attached {
        next_state.set(GameState::Playing);
    }

    Ok(())
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

    // One map lookup where this used to be a `match` whose arms had to agree
    // with `CharacterAnimations.indices`'s ordering by convention alone.
    let Some(&index) = animations.nodes.get(state) else {
        warn!("no animation node for {state:?}");
        return;
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
