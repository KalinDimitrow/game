pub mod button_config;
pub mod input;

pub use self::button_config::*;
pub use self::input::*;

use crate::prelude::*;

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

        // Asset, loader and load system all come from
        // `resources::button_config`; only `apply_button_config` below is this
        // module's, since it writes `InputMap`s onto entities rather than
        // producing a resource.
        app.init_asset::<ButtonConfig>();
        app.init_asset_loader::<ButtonConfigLoader>();
        app.add_systems(Startup, load_button_config);
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
