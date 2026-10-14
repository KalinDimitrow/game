/// set up a simple 3D scene
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::components::{CameraMovement, SpawnFloor, SpawnCube};

pub const WORLD_FILE_PATH: &str = "load_scene_example.scn.ron";

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

pub fn setup(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>
) {
    commands.spawn(DynamicWorldRoot(asset_server.load(WORLD_FILE_PATH)));
    // This will capture the total continuous value, for direct use.
    // Note that you can also use discrete gesture-like motion,
    // via the `MouseMoveDirection` enum.
    // let input_map = InputMap::default().with_dual_axis(CameraMovement::Pan, MouseMove::default());
    // commands.spawn(Camera2d).insert(input_map);

    // commands.spawn((
    //     Sprite::default(),
    //     Transform::from_scale(Vec3::new(100., 100., 1.)),
    // ));

    // commands.spawn(
    //     (
    //     // #CircularBase
    //     Mesh3d(meshes.add(Circle::new(4.0))),
    //     MeshMaterial3d::<StandardMaterial>(materials.add(Color::WHITE)),
    //     Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
    //     )
    // );
    // commands.spawn(
    //     (
    //     // #Cube
    //     Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    //     MeshMaterial3d::<StandardMaterial>(materials.add(Color::srgb_u8(124, 144, 255))),
    //     Transform::from_xyz(0.0, 0.5, 0.0)
    //     )
    // );
    commands.spawn(
    (
        PointLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0)
        )
    );
    commands.spawn(
        (
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        InputMap::default().with_dual_axis(CameraMovement::Pan, MouseMove::default())
    )
    );
}

pub fn materialize_shapes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // Find entities that have our placeholder but haven't been given a mesh yet
    floors: Query<(Entity, &SpawnFloor), Without<Mesh3d>>,
    cubes: Query<Entity, (With<SpawnCube>, Without<Mesh3d>)>,
) {
    // Turn Floor Placeholders into Circles
    for (entity, floor) in &floors {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Circle::new(floor.radius))),
            MeshMaterial3d::<StandardMaterial>(materials.add(Color::WHITE)),
        ));
    }

    // Turn Cube Placeholders into Cubes
    for entity in &cubes {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d::<StandardMaterial>(materials.add(Color::srgb_u8(124, 144, 255))),
        ));
    }
}
