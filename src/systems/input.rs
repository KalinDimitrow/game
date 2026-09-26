use bevy::{input::keyboard::*, prelude::*};
use leafwing_input_manager::prelude::*;
use crate::components::CameraMovement;

pub fn keyboard_event_system(mut keyboard_inputs: MessageReader<KeyboardInput>) {
    for keyboard_input in keyboard_inputs.read() {
        info!("{:?}", keyboard_input);
    }
}

pub fn pan_camera(query: Single<(&mut Transform, &ActionState<CameraMovement>), With<Camera3d>>) {
    const CAMERA_PAN_RATE: f32 = 0.5;

    let (mut camera_transform, action_state) = query.into_inner();

    let camera_pan_vector = action_state.axis_pair(&CameraMovement::Pan);

    // Because we're moving the camera, not the object, we want to pan in the opposite direction.
    // However, UI coordinates are inverted on the y-axis, so we need to flip y a second time.
    // camera_transform.translation.x -= CAMERA_PAN_RATE * camera_pan_vector.x;
    // camera_transform.translation.y += CAMERA_PAN_RATE * camera_pan_vector.y;
    let rotation = Quat::from_rotation_y(-0.005 * camera_pan_vector.x)
        * Quat::from_rotation_x(-0.005 * camera_pan_vector.y);
    // camera_transform.rotate_y(-0.005 * camera_pan_vector.x);
    // camera_transform.rotate_x(-0.005 * camera_pan_vector.y);
    camera_transform.rotate_around(Vec3::ZERO, rotation);
    camera_transform.look_at(Vec3::ZERO, Vec3::Y);
}

pub fn keyboard_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    key_input: Res<ButtonInput<Key>>,
    mut query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>
) {
    // KeyCode is used when you want the key location across different keyboard layouts
    // See https://w3c.github.io/uievents-code/#code-value-tables for the locations
    // if keyboard_input.pressed(KeyCode::KeyA) {
    //     info!("'A' currently pressed");
    // }
    //
    if keyboard_input.pressed(KeyCode::KeyA) {
        info!("'A' just pressed");
        for mut tranform in &mut query {
            tranform.rotate_y(0.5 * time.delta_secs());
        }
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        info!("'A' just pressed");
        for mut tranform in &mut query {
            tranform.rotate_y(-0.5 * time.delta_secs());
        }
    }
    // if keyboard_input.just_released(KeyCode::KeyA) {
    //     info!("'A' just released");
    // }

    // Key is used when you want a specific key, no matter where it is located.
    // This is useful for symbols that have a specific connotation, e.g. '?' for
    // a help menu or '+'/'-' for zoom
    // let key = Key::Character("?".into());
    // if key_input.pressed(key.clone()) {
    //     info!("'?' currently pressed");
    //     for mut tranform in &mut query {
    //         tranform.rotate_y(0.5 * time.delta_secs());
    //     }
    // }
    // if key_input.just_pressed(key.clone()) {
    //     info!("'?' just pressed");
    // }
    // if key_input.just_released(key) {
    //     info!("'?' just released");
    // }
}