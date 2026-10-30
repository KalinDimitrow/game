use crate::prelude::*;
use crate::utils::system_config::ButtonConfig;

/// Keeps the `ButtonConfig` handle alive and findable. A strong handle also
/// lives in `PendingAssets`, but that one is for the loading gate - this is
/// the one anything else reads back through.
#[derive(Resource)]
pub struct ButtonConfigHandle(pub Handle<ButtonConfig>);

// Kicks off the load and registers it with the loading gate, so a missing or
// malformed `button_config.ron` fails startup loudly (via
// `check_pending_assets`) instead of silently leaving the player with no
// bindings.
pub fn load_button_config(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut pending: ResMut<PendingAssets>,
) {
    let handle: Handle<ButtonConfig> = asset_server.load(BUTTON_CONFIG_ASSET_PATH);
    pending.track(handle.clone());
    commands.insert_resource(ButtonConfigHandle(handle));
}

// Builds the `InputMap` from the loaded asset, and rebuilds it in place when
// the file changes on disk.
//
// Driven by `AssetEvent` rather than run once in a later state, because
// `Added` and `Modified` want the identical handling - "(re)build the map
// from whatever the asset says now" - so an event-driven version gets
// hot-reload for free instead of needing a second code path for it. The
// `InputMap` is replaced on the existing entity rather than respawned, so
// `ActionState<PlayerAction>` (a required component, carrying
// just-pressed/held state) survives a reload mid-press.
//
// Hot-reload only actually fires with bevy's `file_watcher` feature enabled;
// without it this still handles `Added`, which is all startup needs.
pub fn apply_button_config(
    mut commands: Commands,
    mut events: MessageReader<AssetEvent<ButtonConfig>>,
    configs: Res<Assets<ButtonConfig>>,
    existing: Query<Entity, With<InputMap<PlayerAction>>>,
) {
    for event in events.read() {
        let (AssetEvent::Added { id } | AssetEvent::Modified { id }) = event else {
            continue;
        };
        let Some(config) = configs.get(*id) else {
            continue;
        };

        let mut input_map = InputMap::<PlayerAction>::default();
        for (action, key) in &config.bindings {
            input_map.insert(*action, *key);
        }

        // `ActionState<PlayerAction>` is added automatically as a required
        // component of `InputMap`, so the spawn branch gets one for free and
        // the insert branch leaves the existing one untouched.
        if let Ok(entity) = existing.single() {
            commands.entity(entity).insert(input_map);
        } else {
            commands.spawn(input_map);
        }
    }
}
