//! The *consumer* half of the button config. Getting the file loaded and held
//! lives in `resources::button_config` (the `ButtonConfig` asset, its
//! `AssetLoader`, `ButtonConfigHandle`, and `load_button_config`); what's left
//! here is the part that isn't about a resource at all - turning loaded
//! bindings into `InputMap` components on entities.

use crate::prelude::*;

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
