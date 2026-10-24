//! Path constants used across the crate - config files, the asset root, and
//! individual asset paths. Gameplay tuning constants (speeds, grid size, etc.)
//! stay local to the system that uses them; only paths live here.

use crate::utils::system_config::{button_config_reader, display_config_reader, ConfigReader};

pub const BUTTON_CONFIG_PATH: &str = "data/config/button_config.ron";
pub const DISPLAY_CONFIG_PATH: &str = "data/config/display.ron";
pub const LEVEL_PATH: &str = "data/levels/level_01.ron";

// Every file `system_config_reader` knows how to read, paired with the
// reader that parses it - same (name, value) shape as `KEYCODES` in
// `utils::system_config`. Every reader shares `ConfigReader`'s signature, so
// `system_config_reader` can call the one next to a given path directly
// instead of matching on some tag to find it. Adding a new config file is a
// one-line change here rather than touching the reader itself.
pub const CONFIG_READERS: &[(&str, ConfigReader)] =
    &[(BUTTON_CONFIG_PATH, button_config_reader), (DISPLAY_CONFIG_PATH, display_config_reader)];

// Passed to `AssetPlugin { file_path, .. }` in `main.rs`. The `CHARACTER_*`
// paths below are relative to this root, not to the crate root -
// `asset_server.load(CHARACTER_MESH_PATH)` resolves to
// `{ASSET_ROOT_PATH}/{CHARACTER_MESH_PATH}`. Terrain tile paths aren't a
// constant here anymore - they're read from `LEVEL_PATH`'s RON (see
// `systems::terrain::level_config`), also resolved against this same root.
pub const ASSET_ROOT_PATH: &str = "data/assets";

// The character's mesh/skeleton and each of its animations are converted
// as separate .glb files (tools/convert_fbx_to_glb.sh) rather than
// pre-merged into one - Bevy matches animation clips to scene entities by
// bone name, not by source file, so `character_controller` loads all of
// these independently and combines them at runtime.
pub const CHARACTER_MESH_PATH: &str = "models/characters/basic_character/character.glb";
pub const CHARACTER_IDLE_PATH: &str = "models/characters/basic_character/Idle.glb";
pub const CHARACTER_WALKING_PATH: &str = "models/characters/basic_character/Walking.glb";
pub const CHARACTER_JUMPING_PATH: &str = "models/characters/basic_character/Unarmed_Jump.glb";
