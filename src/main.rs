use bevy::asset::AssetPlugin;
use bevy::window::WindowPlugin;
use game::prelude::*;
use game::systems_prelude::*;
use game::utils::system_config::{system_config_reader, SystemConfig};

fn main() -> Result<(), BevyError> {
    // Read before the `App` exists: window creation happens as part of
    // `DefaultPlugins`, so the size has to be known up front rather than
    // applied as a Startup system (which would resize an already-open window).
    let system_config = system_config_reader(CONFIG_READERS)?;
    let SystemConfig { button: button_config, graphics: graphics_config } = system_config;
    let resolution = graphics_config.resolution.clone();

    App::new()
        .insert_resource(button_config)
        .insert_resource(graphics_config)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: ASSET_ROOT_PATH.to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .init_state::<GameState>()
        .add_plugins(CorePlugin)
        .add_plugins(TerrainPlugin)
        .add_plugins(CharacterControllerPlugin)
        .run();

    Ok(())
}
