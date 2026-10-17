use avian3d::prelude::{Collider, PhysicsPlugins, RigidBody};

use crate::prelude::*;

// Rectangular, not square, so it's visually obvious which axis is which while
// testing grid/tile-based collision later.
pub const GRID_WIDTH: usize = 28;
pub const GRID_DEPTH: usize = 16;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());
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

    // Single flat collider spanning the whole grid, rather than one per tile -
    // every tile is coplanar right now so per-tile colliders would just be
    // overhead. Revisit per-cell colliders once terrain has real height/obstacles.
    let grid_center = Vec3::new(GRID_WIDTH as f32 / 2.0 - 0.5, 0.0, GRID_DEPTH as f32 / 2.0 - 0.5);
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(GRID_WIDTH as f32, 0.1, GRID_DEPTH as f32),
        Transform::from_translation(grid_center - Vec3::Y * 0.05),
    ));

    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::new(-0.3, -0.5, -0.2), Vec3::Y),
    ));
}
