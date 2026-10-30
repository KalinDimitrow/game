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
//
// Registered with `PendingAssets`, which it wasn't before: `GameState::Loading`
// used to wait only on the character's animation glTFs, so the world - and the
// 7 glTF scenes it references - could still be loading when the state said
// loading was done. Tracking is recursive (`recursive_dependency_load_state`),
// so this one handle covers every tile and the character mesh underneath it.
fn spawn_test_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut pending: ResMut<PendingAssets>,
) {
    let world = asset_server.load(TEST_WORLD_PATH);
    pending.track(world.clone());
    commands.spawn((Transform::IDENTITY, DynamicWorldRoot(world)));
}
