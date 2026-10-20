use avian3d::prelude::{Collider, PhysicsPlugins, RigidBody};
use bevy::world_serialization::WorldInstanceReady;
use std::collections::HashMap;

use crate::prelude::*;

pub mod level_config;
pub use self::level_config::*;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());
        app.add_systems(Startup, spawn_terrain);
    }
}

// Fallible Startup system: a missing/malformed level file is a content-author
// problem, not a bug - same reasoning as `load_button_config`/`load_display_config`.
// `pub(crate)` rather than private: `character_controller::spawn_character` orders
// itself after this system, since it reads the `LevelConfig` resource inserted here.
pub(crate) fn spawn_terrain(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) -> Result<(), BevyError> {
    let level = load_level_config()?;
    info!("loading level \"{}\"", level.name);

    // Chessboard tint, alternating per grid cell by (i + j) parity - swapped
    // onto each tile's mesh once its scene instance finishes spawning (the
    // `.observe` below), since glTF scene spawning is async and the mesh/
    // material entities don't exist yet at the time of this `commands.spawn`.
    let light_tile = standard_materials.add(StandardMaterial {
        base_color: Color::srgb(0.82, 0.82, 0.76),
        ..default()
    });
    let dark_tile = standard_materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.22, 0.2),
        ..default()
    });

    // One glTF scene load per distinct tile asset the level references,
    // instanced across the grid - each tile is a separate entity/transform so
    // a later collision system can key off grid cell (i, j).
    let mut tile_scenes: HashMap<&str, Handle<WorldAsset>> = HashMap::new();
    for j in 0..level.grid_depth {
        for i in 0..level.grid_width {
            let (tile_path, height) = match level.cell_overrides.get(&(i, j)) {
                Some(cell) => (cell.tile_path.as_str(), cell.height),
                None => (level.default_tile_path.as_str(), 0.0),
            };
            let scene = tile_scenes
                .entry(tile_path)
                .or_insert_with(|| {
                    asset_server.load(GltfAssetLabel::Scene(0).from_asset(tile_path.to_string()))
                })
                .clone();
            let tile_material = if (i + j) % 2 == 0 { light_tile.clone() } else { dark_tile.clone() };
            commands
                .spawn((
                    Transform::from_xyz(i as f32 * level.cell_size, height, j as f32 * level.cell_size),
                    WorldAssetRoot(scene),
                ))
                .observe(
                    move |ready: On<WorldInstanceReady>,
                          children: Query<&Children>,
                          mut mesh_materials: Query<&mut MeshMaterial3d<StandardMaterial>>| {
                        for descendant in children.iter_descendants(ready.entity) {
                            if let Ok(mut material) = mesh_materials.get_mut(descendant) {
                                material.0 = tile_material.clone();
                            }
                        }
                    },
                );
        }
    }

    // Single flat collider spanning the whole grid, rather than one per tile -
    // every tile is coplanar right now so per-tile colliders would just be
    // overhead. Revisit per-cell colliders once terrain has real height/obstacles
    // - the level format already carries per-cell `height`/`walkable` (see
    // `level_config`), nothing reads them for collision yet.
    let grid_size = Vec3::new(
        level.grid_width as f32 * level.cell_size,
        0.1,
        level.grid_depth as f32 * level.cell_size,
    );
    let grid_center = Vec3::new(
        grid_size.x / 2.0 - level.cell_size / 2.0,
        0.0,
        grid_size.z / 2.0 - level.cell_size / 2.0,
    );
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(grid_size.x, grid_size.y, grid_size.z),
        Transform::from_translation(grid_center - Vec3::Y * 0.05),
    ));

    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::new(-0.3, -0.5, -0.2), Vec3::Y),
    ));

    commands.insert_resource(level);
    Ok(())
}
