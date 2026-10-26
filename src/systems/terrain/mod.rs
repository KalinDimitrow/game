use avian3d::prelude::PhysicsPlugins;

use crate::prelude::*;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());
        app.add_systems(Startup, spawn_test_world);
    }
}

// Loads the whole scene - terrain tiles, their physics, the light, and the
// character/camera - from `DynamicWorld` RON instead of spawning each of
// those from Rust. Spawns at the origin since the RON's own entities carry
// absolute positions.
fn spawn_test_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Transform::IDENTITY,
        DynamicWorldRoot(asset_server.load("worlds/test_world.scn.ron")),
    ));
}
