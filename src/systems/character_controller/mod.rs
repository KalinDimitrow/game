use avian3d::prelude::{Collider, LinearVelocity, LockedAxes, RigidBody};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::components::{CameraMovement, Character, CharacterMovement, ThirdPersonCamera};
use crate::systems::terrain::{GRID_DEPTH, GRID_WIDTH};

const MOVE_SPEED: f32 = 6.0;
const JUMP_SPEED: f32 = 6.0;
const MOUSE_SENSITIVITY: f32 = 0.005;
const PITCH_LIMIT: f32 = 1.4;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CharacterMovement>::default());
        app.add_systems(Startup, spawn_character);
        app.add_systems(Update, (move_character, follow_character));
    }
}

fn spawn_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let spawn = Vec3::new(GRID_WIDTH as f32 / 2.0, 3.0, GRID_DEPTH as f32 / 2.0);

    let mut movement_input = InputMap::<CharacterMovement>::default();
    movement_input
        .insert(CharacterMovement::Left, KeyCode::KeyA)
        .insert(CharacterMovement::Right, KeyCode::KeyD)
        .insert(CharacterMovement::Forward, KeyCode::KeyW)
        .insert(CharacterMovement::Backward, KeyCode::KeyS)
        .insert(CharacterMovement::Jump, KeyCode::Space);

    commands.spawn((
        Character,
        RigidBody::Dynamic,
        Collider::capsule(0.4, 1.0),
        LockedAxes::ROTATION_LOCKED,
        Mesh3d(meshes.add(Capsule3d::new(0.4, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(220, 90, 70))),
        Transform::from_translation(spawn),
        movement_input,
    ));

    commands.spawn((
        Camera3d::default(),
        ThirdPersonCamera::default(),
        Transform::from_xyz(spawn.x, spawn.y + 2.0, spawn.z + 6.0).looking_at(spawn, Vec3::Y),
        InputMap::default().with_dual_axis(CameraMovement::Pan, MouseMove::default()),
    ));
}

fn move_character(
    character: Single<(&mut LinearVelocity, &ActionState<CharacterMovement>), With<Character>>,
    camera: Single<&ThirdPersonCamera>,
) {
    let (mut velocity, action_state) = character.into_inner();

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
}

fn follow_character(
    character: Single<&Transform, (With<Character>, Without<Camera3d>)>,
    camera: Single<
        (&mut Transform, &mut ThirdPersonCamera, &ActionState<CameraMovement>),
        (With<Camera3d>, Without<Character>),
    >,
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
