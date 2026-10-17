use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use game::systems::character_controller::CharacterControllerPlugin;
use game::systems::core::CorePlugin;
use game::systems::terrain::TerrainPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "data/assets".to_string(),
            ..default()
        }))
        .add_plugins(CorePlugin)
        .add_plugins(TerrainPlugin)
        .add_plugins(CharacterControllerPlugin)
        .run();
}
