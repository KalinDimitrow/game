use bevy::asset::io::AssetSourceBuilder;
use bevy::asset::AssetPlugin;
use bevy::window::WindowPlugin;
use game::prelude::*;
use game::systems_prelude::*;
use game::utils::system_config::{system_config_reader, SystemConfig};

fn main() -> Result<(), BevyError> {
    // Read before the `App` exists: window creation happens as part of
    // `DefaultPlugins`, so the size has to be known up front rather than
    // applied as a Startup system (which would resize an already-open window).
    // This is the whole reason `CONFIG_READERS` still exists - the button
    // config has no such constraint and is an asset now.
    let system_config = system_config_reader(CONFIG_READERS)?;
    let SystemConfig { graphics: graphics_config } = system_config;
    let resolution = graphics_config.resolution.clone();

    App::new()
        .insert_resource(graphics_config)
        // Must precede `AssetPlugin` (added below as part of `DefaultPlugins`):
        // registered sources are built when that plugin builds, and ignored
        // after. Gives `button_config.ron` a `config://` path without moving
        // the file under `ASSET_ROOT_PATH`.
        .register_asset_source(
            CONFIG_SOURCE_ID,
            AssetSourceBuilder::platform_default(CONFIG_SOURCE_PATH, None),
        )
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
        .add_plugins(AssetLoadingPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(TerrainPlugin)
        .add_plugins(CharacterControllerPlugin)
        .run();

    Ok(())
}
