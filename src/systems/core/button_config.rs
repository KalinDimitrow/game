//! The *consumer* half of the button config, plus the one system that puts the
//! load on a stage. The type, its `AssetLoader` and `ButtonConfigHandle` live
//! in `resources::button_config`; what's here is the part that isn't about a
//! resource at all - naming the load to the load machine, and turning loaded
//! bindings into `InputMap` components on entities.

use crate::prelude::*;
use crate::systems::asset_loading::StageGate;

pub fn load_button_config(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut gate: ResMut<StageGate>,
) {
    let handle: Handle<ButtonConfig> = asset_server.load(BUTTON_CONFIG_ASSET_PATH);
    gate.track(handle.clone());
    commands.insert_resource(ButtonConfigHandle(handle));
}

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

#[cfg(test)]
mod tests {
    use bevy::asset::io::AssetSourceBuilder;
    use bevy::asset::AssetPlugin;
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::resources::ButtonConfigLoader;
    use crate::systems::asset_loading::{AssetLoadingPlugin, LoadStage};

    /// The staged machine against the real `button_config.ron`, which the
    /// `AssetLoadingPlugin` unit tests can't cover - they deliberately use
    /// handles no file backs. No window and no renderer, but a real
    /// `AssetPlugin` reading the repo's own `data/config/`, so this is the one
    /// test that fails if the path constant, the `config://` source or the
    /// stage registration break.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        // Same two-step as `main.rs`: the source has to be registered before
        // `AssetPlugin` builds, or `config://` doesn't resolve.
        app.register_asset_source(
            CONFIG_SOURCE_ID,
            AssetSourceBuilder::platform_default(CONFIG_SOURCE_PATH, None),
        );
        app.add_plugins(AssetPlugin {
            file_path: ASSET_ROOT_PATH.to_string(),
            ..default()
        });
        app.add_plugins(AssetLoadingPlugin);
        app.init_asset::<ButtonConfig>();
        app.init_asset_loader::<ButtonConfigLoader>();
        app.add_systems(OnEnter(LoadStage::Boot), load_button_config);
        app
    }

    /// Also pins the ordering fact the registration depends on: the initial
    /// `OnEnter(Boot)` runs *within* the first `app.update()`, in the
    /// `StateTransition` that precedes `PreStartup` - so the load is named
    /// earlier than the `Startup` system this replaced, not later.
    #[test]
    fn boot_loads_the_real_button_config_and_then_advances() {
        let mut app = test_app();

        assert!(
            app.world().get_resource::<ButtonConfigHandle>().is_none(),
            "nothing should have run before the first update"
        );

        app.update();
        assert!(
            app.world().get_resource::<ButtonConfigHandle>().is_some(),
            "OnEnter(Boot) should have run in the first update, ahead of PreStartup"
        );
        // Whether the gate is already green here is a race with the IO task, so
        // this asserts only what the schedule guarantees: a transition asked
        // for in frame N lands at the start of frame N+1.
        assert_eq!(
            *app.world().resource::<State<LoadStage>>().get(),
            LoadStage::Boot,
            "the machine cannot have left Boot within the frame it entered it"
        );

        // Real IO, so poll until the load lands rather than assuming a frame
        // count. The cap turns a never-loading asset into a failure.
        for _ in 0..600 {
            if *app.world().resource::<State<LoadStage>>().get() != LoadStage::Boot {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
            app.update();
        }

        let handle = &app.world().resource::<ButtonConfigHandle>().0;
        let config = app
            .world()
            .resource::<Assets<ButtonConfig>>()
            .get(handle)
            .expect("Boot advanced, so the config must be in Assets<ButtonConfig>");
        assert!(
            !config.bindings.is_empty(),
            "data/config/button_config.ron should parse to at least one binding"
        );

        app.update();
        assert_ne!(
            *app.world().resource::<State<LoadStage>>().get(),
            LoadStage::Boot,
            "the machine should have moved past Boot once the gate went green"
        );
    }
}
