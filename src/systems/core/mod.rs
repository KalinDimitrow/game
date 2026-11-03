pub mod button_config;
pub mod input;

pub use self::button_config::*;
pub use self::input::*;

use crate::prelude::*;
use crate::systems::asset_loading::LoadStage;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CameraMovement>::default());
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());

        // The loading gate (`resources::pending_assets`). `PendingAssets` is
        // pooled across plugins, so core owns the scheduling rather than
        // whichever plugin happens to load something - that's what lets
        // `TerrainPlugin`'s world load gate startup too, instead of only
        // `CharacterControllerPlugin`'s glTFs.
        app.init_resource::<PendingAssets>();
        app.add_systems(
            Update,
            check_pending_assets.run_if(in_state(GameState::Loading)),
        );

        // Asset and loader come from `resources::button_config`; the two
        // systems below are this module's, since neither produces a resource -
        // one names a load to the load machine, the other writes `InputMap`s
        // onto entities.
        app.init_asset::<ButtonConfig>();
        app.init_asset_loader::<ButtonConfigLoader>();
        // The first asset to load through `AssetLoadingPlugin` rather than
        // `PendingAssets`, and `Boot` is the stage the design assigns it: the
        // loading screen needs its own assets before it can report on anything
        // else. `GameState::Loading` no longer waits on this file - `Boot` does,
        // and it holds the whole machine until the config is in `Assets<T>`.
        //
        // Earlier than the `Startup` this replaces, not later:
        // `StateTransition` runs once *before* `PreStartup`
        // (`bevy_state-0.20.0/src/state/transitions.rs:43`), so the initial
        // `OnEnter(LoadStage::Boot)` has already happened by the time `Startup`
        // would have run.
        app.add_systems(OnEnter(LoadStage::Boot), load_button_config);
        // Deliberately not state-gated: this is also the hot-reload path, so
        // it has to keep listening for `Modified` long after loading is done.
        app.add_systems(Update, apply_button_config);

        // Gated on `Playing` now that the `InputMap` arrives with an asset
        // rather than existing from `Startup`: there's no `ActionState` for
        // this system's `Single` to find until `apply_button_config` has run,
        // and reacting to a pause/confirm mid-load wouldn't mean anything.
        app.add_systems(
            Update,
            player_action_system.run_if(in_state(GameState::Playing)),
        );
    }
}
