use bevy::prelude::*;
use std::collections::HashMap;

/// A spawn/objective marker placed on the grid. The `EnemySpawn`/`Objective`
/// variants carry an id so later systems (wave composition, win/lose
/// conditions) can reference a specific gate/base by name instead of by
/// coordinates.
#[derive(serde::Deserialize, Debug)]
pub enum PointOfInterestKind {
    PlayerSpawn,
    EnemySpawn(String),
    Objective(String),
}

#[derive(serde::Deserialize)]
pub struct PointOfInterest {
    pub kind: PointOfInterestKind,
    pub position: (usize, usize),
}

/// A validated, resolved per-cell exception to the default tile - the tile
/// `id` from the RON file has already been looked up to an asset path by the
/// time this exists, so callers never need the tile registry itself.
pub struct CellOverride {
    pub tile_path: String,
    pub height: f32,
    pub walkable: bool,
}

/// The fully-resolved, validated level: everything `terrain::spawn_terrain`
/// needs to build the grid, plus the points of interest other systems (e.g.
/// `character_controller::spawn_character`) place things at. Parsed/validated
/// by `systems::terrain::level_config`.
#[derive(Resource)]
pub struct LevelConfig {
    pub name: String,
    pub grid_width: usize,
    pub grid_depth: usize,
    pub cell_size: f32,
    pub default_tile_path: String,
    pub cell_overrides: HashMap<(usize, usize), CellOverride>,
    pub points_of_interest: Vec<PointOfInterest>,
}

impl LevelConfig {
    /// Grid coordinates of the first `PlayerSpawn` POI, if the level defines one.
    pub fn player_spawn(&self) -> Option<(usize, usize)> {
        self.points_of_interest.iter().find_map(|poi| {
            matches!(poi.kind, PointOfInterestKind::PlayerSpawn).then_some(poi.position)
        })
    }
}

/// The mesh/skeleton and each animation are separate glTF assets - Bevy
/// matches clips to scene entities by bone name, not by source file, so
/// these just need to be loaded independently and combined once all of them
/// are ready. Populated by `systems::character_controller::spawn_character`.
#[derive(Resource)]
pub struct CharacterGltfHandles {
    pub mesh: Handle<Gltf>,
    pub idle: Handle<Gltf>,
    pub walking: Handle<Gltf>,
    pub jumping: Handle<Gltf>,
}

/// Indexed by `CharacterAnimationState` variant order: Idle, Walking, Jumping.
/// Built by `systems::character_controller::spawn_character_visual_when_ready`
/// once every handle in `CharacterGltfHandles` has finished loading.
#[derive(Resource)]
pub struct CharacterAnimations {
    pub indices: Vec<AnimationNodeIndex>,
    pub graph_handle: Handle<AnimationGraph>,
}
