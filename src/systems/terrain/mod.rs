use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::prelude::*;

pub const TERRAIN_TILE_PATH: &str = "models/terrain/ground_tile.glb";

// Rectangular, not square, so it's visually obvious which axis is which while
// testing grid/tile-based collision later.
pub const GRID_WIDTH: usize = 28;
pub const GRID_DEPTH: usize = 16;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FreeCameraPlugin);
        app.add_systems(Startup, spawn_terrain);
    }
}

fn spawn_terrain(mut commands: Commands, asset_server: Res<AssetServer>) {
    // One glTF scene load, instanced across the grid - each tile is a separate
    // entity/transform so a later collision system can key off grid cell (i, j).
    let tile_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(TERRAIN_TILE_PATH));
    for j in 0..GRID_DEPTH {
        for i in 0..GRID_WIDTH {
            commands.spawn((
                Transform::from_xyz(i as f32, 0.0, j as f32),
                WorldAssetRoot(tile_scene.clone()),
            ));
        }
    }

    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::new(-0.3, -0.5, -0.2), Vec3::Y),
    ));

    let grid_center = Vec3::new(GRID_WIDTH as f32 / 2.0, 0.0, GRID_DEPTH as f32 / 2.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(grid_center.x, 14.0, GRID_DEPTH as f32 + 10.0)
            .looking_at(grid_center, Vec3::Y),
        FreeCamera {
            walk_speed: 8.0,
            run_speed: 24.0,
            ..default()
        },
    ));
}
