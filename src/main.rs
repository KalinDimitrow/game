use bevy::prelude::*;
use game::systems::core::CorePlugin;
use game::systems::terrain::TerrainPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CorePlugin)
        .add_plugins(TerrainPlugin)
        .run();
}
