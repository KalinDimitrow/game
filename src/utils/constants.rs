//! Path constants used across the crate - config files, the asset root, and
//! individual asset paths. Gameplay tuning constants (speeds, grid size, etc.)
//! stay local to the system that uses them; only paths live here.

use crate::components::CharacterAnimationState;
use crate::utils::system_config::{graphics_config_reader, ConfigReader};

pub const GRAPHICS_CONFIG_PATH: &str = "data/config/display.ron";

// Every file `system_config_reader` knows how to read, paired with the
// reader that parses it - same (name, value) shape as `KEYCODES` in
// `resources::button_config`. Every reader shares `ConfigReader`'s
// signature, so `system_config_reader` can call the one next to a given path
// directly instead of matching on some tag to find it. Adding a new config
// file is a one-line change here rather than touching the reader itself.
//
// Only graphics is left here, down from graphics + button. What belongs in
// this table is config that has to be read *before the `App` exists*, which
// is really just "whatever window creation needs". `button_config.ron` has no
// such constraint - its only consumer is a system - so it moved to the asset
// system and gets hot-reload for free; see `resources::button_config`.
pub const CONFIG_READERS: &[(&str, ConfigReader)] =
    &[(GRAPHICS_CONFIG_PATH, graphics_config_reader)];

// Passed to `AssetPlugin { file_path, .. }` in `main.rs`. The asset paths
// below are relative to this root, not to the crate root -
// `asset_server.load(TEST_WORLD_PATH)` resolves to
// `{ASSET_ROOT_PATH}/{TEST_WORLD_PATH}`.
pub const ASSET_ROOT_PATH: &str = "data/assets";

// Registered in `main.rs` as a second `AssetSource` under `CONFIG_SOURCE_ID`,
// rather than folded under `ASSET_ROOT_PATH`. Config stays where it already
// is (`data/config/`, next to `display.ron`) instead of being moved under
// `data/assets/` just to become loadable, and `AssetSource` is the seam that
// makes the same path work off a non-filesystem reader later - wasm has no
// `std::fs`, and this crate has a `wasm-release` profile.
pub const CONFIG_SOURCE_ID: &str = "config";
pub const CONFIG_SOURCE_PATH: &str = "data/config";

// `config://` resolves to `CONFIG_SOURCE_PATH`. Loaded through `AssetServer`,
// not read with `std::fs` like `GRAPHICS_CONFIG_PATH` above: that one has to
// exist before the window does, this one doesn't.
pub const BUTTON_CONFIG_ASSET_PATH: &str = "config://button_config.ron";

// Terrain, physics, the light, and the character/camera all come from here -
// this is the only way any of them get into the world (see `systems::terrain`).
pub const TEST_WORLD_PATH: &str = "worlds/test_world.scn.ron";

// The character's mesh/skeleton (`character.glb`) is deliberately absent:
// `test_world.scn.ron` references it itself via entity 10's `WorldAssetRoot`,
// so it loads as a dependency of the world rather than being named twice in
// two places that could drift apart.
//
// Each animation is converted to its own .glb (tools/convert_fbx_to_glb.sh)
// rather than pre-merged into character.glb - Bevy matches animation clips to
// scene entities by bone name, not by source file, so they load independently
// and get combined into one `AnimationGraph` at runtime.
//
// A `(state, path)` table rather than one const per clip: adding a clip is one
// line here, instead of a const + a resource field + an argument in the right
// ordinal position in `AnimationGraph::from_clips` + a `match` arm. Same
// (name, value) shape as `CONFIG_READERS` above and `KEYCODES` in
// `resources::button_config`.
pub const CHARACTER_ANIMATIONS: &[(CharacterAnimationState, &str)] = &[
    (
        CharacterAnimationState::Idle,
        "models/characters/basic_character/Idle.glb",
    ),
    (
        CharacterAnimationState::Walking,
        "models/characters/basic_character/Walking.glb",
    ),
    // No real jump clip exists in the available assets (see the cleanup
    // backlog) - `Unarmed_Jump` is the closest stand-in.
    (
        CharacterAnimationState::Jumping,
        "models/characters/basic_character/Unarmed_Jump.glb",
    ),
];
