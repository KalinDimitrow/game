/// set up a simple 3D scene
use bevy::prelude::*;
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::prelude::MouseMove;
use crate::components::{CameraMovement, Person};

pub fn scene() -> impl SceneList {
    bsn_list! [
        // (
        //     // #CircularBase
        //     Mesh3d(asset_value(Circle::new(4.0)))
        //     MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
        //     Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
        // ),
        // (
        //     // #Cube
        //     Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
        //     MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
        //     Transform::from_xyz(0.0, 0.5, 0.0)
        // ),
        // (
        //     PointLight {
        //         shadow_maps_enabled: true,
        //     }
        //     Transform::from_xyz(4.0, 8.0, 4.0)
        // ),
        // (
        //     Camera3d
        //     template_value(Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y))
        //     // input_map
        // )
    ]
}

pub fn update_people(mut query: Query<&mut crate::components::Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Elaina Proctor" {
            name.0 = "Elaina Hume".to_string();
            break; // We don't need to change any other names.
        }
    }
}
