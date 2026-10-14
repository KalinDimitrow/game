use bevy::prelude::*;
use game::systems::core::CorePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CorePlugin)
        .run();
}
