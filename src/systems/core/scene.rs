use crate::components::{CameraMovement, SpawnCube, SpawnFloor};
/// set up a simple 3D scene
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub const WORLD_FILE_PATH: &str = "load_scene_example.scn.ron";

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(DynamicWorldRoot(asset_server.load(WORLD_FILE_PATH)));
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        InputMap::default().with_dual_axis(CameraMovement::Pan, MouseMove::default()),
    ));
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
