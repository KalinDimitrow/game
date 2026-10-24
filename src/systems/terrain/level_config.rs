//! Data-driven level/terrain description: grid size, per-cell tile/height/
//! walkability overrides, and points of interest (spawn points, objectives).
//! Parsed from RON at `LEVEL_PATH`, same read/parse/validate shape as
//! `utils::system_config`'s readers - kept separate rather than folded into
//! `SystemConfig` since it lives in a different directory (`data/levels/`,
//! not `data/config/`) and is consumed differently (inserted directly as its
//! own resource, not read ahead of `App` construction).

use bevy::ecs::error::BevyError;
use serde::Deserialize;
use std::collections::HashMap;

use crate::prelude::*;

#[derive(Deserialize)]
struct LevelConfigFile {
    name: String,
    grid: GridFile,
    terrain: TerrainFile,
    #[serde(default)]
    points_of_interest: Vec<PointOfInterest>,
}

#[derive(Deserialize)]
struct GridFile {
    width: usize,
    depth: usize,
    cell_size: f32,
}

#[derive(Deserialize)]
struct TerrainFile {
    tiles: Vec<TileDefFile>,
    default_tile: String,
    #[serde(default)]
    overrides: Vec<CellOverrideFile>,
}

#[derive(Deserialize)]
struct TileDefFile {
    id: String,
    path: String,
}

#[derive(Deserialize)]
struct CellOverrideFile {
    x: usize,
    z: usize,
    tile: String,
    height: f32,
    walkable: bool,
}

fn resolve(file: LevelConfigFile) -> Result<LevelConfig, BevyError> {
    let GridFile { width, depth, cell_size } = file.grid;
    if width == 0 || depth == 0 {
        return Err(
            format!("level grid width/depth must be non-zero, got {width}x{depth}").into(),
        );
    }
    if cell_size <= 0.0 {
        return Err(format!("level grid cell_size must be positive, got {cell_size}").into());
    }

    let tile_paths: HashMap<String, String> = file
        .terrain
        .tiles
        .into_iter()
        .map(|tile| (tile.id, tile.path))
        .collect();

    let default_tile_path = tile_paths.get(&file.terrain.default_tile).cloned().ok_or_else(|| {
        format!(
            "level's default_tile `{}` is not in its tile registry",
            file.terrain.default_tile
        )
    })?;

    let mut cell_overrides = HashMap::new();
    for cell in file.terrain.overrides {
        if cell.x >= width || cell.z >= depth {
            return Err(format!(
                "level terrain override at ({}, {}) is outside the {width}x{depth} grid",
                cell.x, cell.z
            )
            .into());
        }
        let tile_path = tile_paths
            .get(&cell.tile)
            .cloned()
            .ok_or_else(|| format!("level terrain override references unknown tile `{}`", cell.tile))?;
        cell_overrides.insert(
            (cell.x, cell.z),
            CellOverride {
                tile_path,
                height: cell.height,
                walkable: cell.walkable,
            },
        );
    }

    for poi in &file.points_of_interest {
        let (x, z) = poi.position;
        if x >= width || z >= depth {
            return Err(format!(
                "level point of interest {:?} at ({x}, {z}) is outside the {width}x{depth} grid",
                poi.kind
            )
            .into());
        }
    }

    Ok(LevelConfig {
        name: file.name,
        grid_width: width,
        grid_depth: depth,
        cell_size,
        default_tile_path,
        cell_overrides,
        points_of_interest: file.points_of_interest,
    })
}

fn parse_level_config(contents: &str) -> Result<LevelConfig, BevyError> {
    let file: LevelConfigFile =
        ron::from_str(contents).map_err(|e| format!("failed to parse level config: {e}"))?;
    resolve(file)
}

pub fn load_level_config() -> Result<LevelConfig, BevyError> {
    let contents = std::fs::read_to_string(LEVEL_PATH)
        .map_err(|e| format!("failed to read level config at `{LEVEL_PATH}`: {e}"))?;
    let level =
        parse_level_config(&contents).map_err(|e| format!("{e} (in `{LEVEL_PATH}`)"))?;
    Ok(level)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"(
        name: "test_level",
        grid: (width: 4, depth: 3, cell_size: 1.0),
        terrain: (
            tiles: [(id: "ground", path: "models/terrain/ground_tile.glb")],
            default_tile: "ground",
            overrides: [(x: 1, z: 1, tile: "ground", height: 0.5, walkable: false)],
        ),
        points_of_interest: [
            (kind: PlayerSpawn, position: (0, 0)),
            (kind: EnemySpawn("north_gate"), position: (3, 0)),
            (kind: Objective("core"), position: (0, 2)),
        ],
    )"#;

    #[test]
    fn parses_valid_config() {
        let level = parse_level_config(VALID).expect("valid config should parse");
        assert_eq!((level.grid_width, level.grid_depth), (4, 3));
        assert_eq!(level.default_tile_path, "models/terrain/ground_tile.glb");
        assert_eq!(level.cell_overrides.len(), 1);
        assert_eq!(level.player_spawn(), Some((0, 0)));
    }

    #[test]
    fn malformed_ron_is_an_error_not_a_panic() {
        assert!(parse_level_config("not valid ron at all {{{").is_err());
    }

    #[test]
    fn zero_grid_size_is_rejected() {
        let ron = r#"(
            name: "bad",
            grid: (width: 0, depth: 3, cell_size: 1.0),
            terrain: (tiles: [(id: "ground", path: "x.glb")], default_tile: "ground", overrides: []),
            points_of_interest: [],
        )"#;
        assert!(parse_level_config(ron).is_err());
    }

    #[test]
    fn unknown_default_tile_is_rejected() {
        let ron = r#"(
            name: "bad",
            grid: (width: 4, depth: 3, cell_size: 1.0),
            terrain: (tiles: [(id: "ground", path: "x.glb")], default_tile: "missing", overrides: []),
            points_of_interest: [],
        )"#;
        assert!(parse_level_config(ron).is_err());
    }

    #[test]
    fn out_of_bounds_override_is_rejected() {
        let ron = r#"(
            name: "bad",
            grid: (width: 2, depth: 2, cell_size: 1.0),
            terrain: (
                tiles: [(id: "ground", path: "x.glb")],
                default_tile: "ground",
                overrides: [(x: 5, z: 0, tile: "ground", height: 0.0, walkable: true)],
            ),
            points_of_interest: [],
        )"#;
        assert!(parse_level_config(ron).is_err());
    }

    #[test]
    fn out_of_bounds_point_of_interest_is_rejected() {
        let ron = r#"(
            name: "bad",
            grid: (width: 2, depth: 2, cell_size: 1.0),
            terrain: (tiles: [(id: "ground", path: "x.glb")], default_tile: "ground", overrides: []),
            points_of_interest: [(kind: PlayerSpawn, position: (9, 9))],
        )"#;
        assert!(parse_level_config(ron).is_err());
    }
}
