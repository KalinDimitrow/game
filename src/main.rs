use bevy::asset::AssetPlugin;
use bevy::window::WindowPlugin;
use game::display_config::load_display_config;
use game::prelude::*;

fn main() {
    // Read before the `App` exists: window creation happens as part of
    // `DefaultPlugins`, so the size has to be known up front rather than
    // applied as a Startup system (which would resize an already-open window).
    let resolution = match load_display_config() {
        Ok(resolution) => resolution,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    App::new()
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
        .add_plugins(CorePlugin)
        .add_plugins(TerrainPlugin)
        .add_plugins(CharacterControllerPlugin)
        .run();
}
