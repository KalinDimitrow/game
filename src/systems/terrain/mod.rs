use avian3d::prelude::PhysicsPlugins;

use crate::prelude::*;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());
        app.add_systems(Startup, spawn_test_world);
    }
}

fn spawn_test_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut pending: ResMut<PendingAssets>,
) {
    let world = asset_server.load(TEST_WORLD_PATH);
    // pending.track(world.clone());
    commands.spawn((Transform::IDENTITY, DynamicWorldRoot(world)));
}
