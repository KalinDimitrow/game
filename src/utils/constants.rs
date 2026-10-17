//! Path constants used across the crate - config files, the asset root, and
//! individual asset paths. Gameplay tuning constants (speeds, grid size, etc.)
//! stay local to the system that uses them; only paths live here.

pub const BUTTON_CONFIG_PATH: &str = "data/config/button_config.ron";
pub const DISPLAY_CONFIG_PATH: &str = "data/config/display.ron";

// Passed to `AssetPlugin { file_path, .. }` in `main.rs`. `FOX_PATH` and
// `TERRAIN_TILE_PATH` below are relative to this root, not to the crate root -
// `asset_server.load(FOX_PATH)` resolves to `{ASSET_ROOT_PATH}/{FOX_PATH}`.
pub const ASSET_ROOT_PATH: &str = "data/assets";

pub const FOX_PATH: &str = "models/character/Fox.glb";
pub const TERRAIN_TILE_PATH: &str = "models/terrain/ground_tile.glb";
