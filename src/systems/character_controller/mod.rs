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
        app.add_systems(
            Update,
            finish_character_initialization.run_if(in_state(GameState::Playing)),
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

const CHARACTER_VISUAL_OFFSET: Vec3 = Vec3::new(0.0, -0.9, 0.0);

fn sync_character_visual_transform(
    character: Single<&Transform, (With<Character>, Without<CharacterVisual>)>,
    mut visual: Single<&mut Transform, (With<CharacterVisual>, Without<Character>)>,
) {
    visual.translation = character.translation + CHARACTER_VISUAL_OFFSET;
}

fn mark_character_scene_spawned(
    trigger: On<WorldInstanceReady>,
    visuals: Query<(), With<CharacterVisual>>,
    mut commands: Commands,
) {
    if visuals.contains(trigger.entity) {
        commands.entity(trigger.entity).insert(CharacterSceneSpawned);
    }
}

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
    // testbed, not for shipping.
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

    if *animation_state != desired_state {
        *animation_state = desired_state;
    }
}

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
