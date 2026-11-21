use crate::components::CharacterAnimationState;
use crate::utils::system_config::{graphics_config_reader, ConfigReader};

pub const GRAPHICS_CONFIG_PATH: &str = "data/config/display.ron";

pub const CONFIG_READERS: &[(&str, ConfigReader)] =
    &[(GRAPHICS_CONFIG_PATH, graphics_config_reader)];

pub const ASSET_ROOT_PATH: &str = "data/assets";
pub const CONFIG_SOURCE_ID: &str = "config";
pub const CONFIG_SOURCE_PATH: &str = "data/config";
pub const BUTTON_CONFIG_ASSET_PATH: &str = "config://button_config.ron";
pub const TEST_WORLD_PATH: &str = "worlds/test_world.scn.ron";

pub const CHARACTER_ANIMATIONS: &[(CharacterAnimationState, &str)] = &[
    (
        CharacterAnimationState::Idle,
        "models/characters/basic_character/Idle.glb",
    ),
    (
        CharacterAnimationState::Walking,
        "models/characters/basic_character/Walking.glb",
    ),
    (
        CharacterAnimationState::Jumping,
        "models/characters/basic_character/Unarmed_Jump.glb",
    ),
];
