//! First step of moving level data out of `systems::terrain::level_config`
//! and into `utils`, mirroring `utils::system_config`'s directory shape.
//! Built up incrementally rather than ported in one go -
//! `systems::terrain::level_config`/`LevelConfig` stay in place and keep
//! driving `spawn_terrain` until `LevelMap` reaches the same functionality,
//! at which point one replaces the other instead of both existing side by
//! side.

use bevy::ecs::error::BevyError;
use serde::Deserialize;

/// A level's grid shape: how many cells across and deep, and the
/// world-space size of one cell. First piece of `LevelMap` - the tile
/// registry, cell overrides, and points of interest land in later steps.
pub struct LevelMap {
    pub name: String,
    pub grid_width: usize,
    pub grid_depth: usize,
    pub cell_size: f32,
}

/// Mirrors only the fields `LevelMap` currently models. Serde ignores
/// unrecognized input fields by default (no `deny_unknown_fields`), so this
/// can parse a full level RON file - like `data/levels/example_level.ron`,
/// which also carries `terrain`/`points_of_interest` - and just pick out
/// `name`/`grid`, leaving the rest for later steps to add fields for.
#[derive(Deserialize)]
struct LevelMapFile {
    name: String,
    grid: GridFile,
}

#[derive(Deserialize)]
struct GridFile {
    width: usize,
    depth: usize,
    cell_size: f32,
}

/// Parses and validates in one step, same shape as
/// `system_config`'s `parse_button_config`/`parse_graphics_config` - a
/// `LevelMap` never exists in an unvalidated state.
pub fn parse_level_map(contents: &str) -> Result<LevelMap, BevyError> {
    let file: LevelMapFile =
        ron::from_str(contents).map_err(|e| format!("failed to parse level map: {e}"))?;
    let level = LevelMap {
        name: file.name,
        grid_width: file.grid.width,
        grid_depth: file.grid.depth,
        cell_size: file.grid.cell_size,
    };
    validate_level_map(&level)?;
    Ok(level)
}

/// Checks the invariants anything building a grid from a `LevelMap` relies
/// on: both dimensions need at least one cell, and a cell size of zero or
/// less would collapse the grid to a point.
pub fn validate_level_map(level: &LevelMap) -> Result<(), BevyError> {
    if level.grid_width == 0 || level.grid_depth == 0 {
        return Err(format!(
            "level grid width/depth must be non-zero, got {}x{}",
            level.grid_width, level.grid_depth
        )
        .into());
    }
    if level.cell_size <= 0.0 {
        return Err(format!("level grid cell_size must be positive, got {}", level.cell_size).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Embedded at compile time rather than read from disk at test-time, so
    // `cargo test` doesn't depend on the working directory it's run from -
    // same reasoning as using `include_str!` for any other test fixture.
    // This is the same file `docs/Game/Level Format.md` documents; if the
    // format gains a field, add it there first and let this test tell you
    // what `LevelMap` doesn't parse yet.
    const EXAMPLE_LEVEL: &str = include_str!("../../../data/levels/example_level.ron");

    fn make(grid_width: usize, grid_depth: usize, cell_size: f32) -> LevelMap {
        LevelMap { name: "test".to_string(), grid_width, grid_depth, cell_size }
    }

    #[test]
    fn valid_level_map_passes() {
        assert!(validate_level_map(&make(4, 3, 1.0)).is_ok());
    }

    #[test]
    fn zero_width_is_rejected() {
        assert!(validate_level_map(&make(0, 3, 1.0)).is_err());
    }

    #[test]
    fn zero_depth_is_rejected() {
        assert!(validate_level_map(&make(4, 0, 1.0)).is_err());
    }

    #[test]
    fn non_positive_cell_size_is_rejected() {
        assert!(validate_level_map(&make(4, 3, 0.0)).is_err());
        assert!(validate_level_map(&make(4, 3, -1.0)).is_err());
    }

    #[test]
    fn parses_valid_level_map() {
        let ron = r#"(name: "test_level", grid: (width: 4, depth: 3, cell_size: 1.0))"#;
        let level = parse_level_map(ron).expect("valid level map should parse");
        assert_eq!(level.name, "test_level");
        assert_eq!((level.grid_width, level.grid_depth), (4, 3));
    }

    #[test]
    fn malformed_ron_is_an_error_not_a_panic() {
        assert!(parse_level_map("not valid ron at all {{{").is_err());
    }

    #[test]
    fn invalid_grid_is_rejected_even_if_ron_is_well_formed() {
        let ron = r#"(name: "bad", grid: (width: 0, depth: 3, cell_size: 1.0))"#;
        assert!(parse_level_map(ron).is_err());
    }

    #[test]
    fn parses_the_example_level_fixture() {
        let level = parse_level_map(EXAMPLE_LEVEL).expect("example_level.ron should parse");
        assert_eq!(level.name, "example_level");
        assert_eq!((level.grid_width, level.grid_depth), (5, 4));
    }
}
