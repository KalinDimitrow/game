pub mod button_config;
pub mod input;

pub use self::button_config::*;
pub use self::input::*;

use crate::prelude::*;
// use crate::systems::asset_loading::LoadStage;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CameraMovement>::default());
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
        app.init_asset::<ButtonConfig>();
        app.init_asset_loader::<ButtonConfigLoader>();
        app.add_systems(Update, apply_button_config);
        app.add_systems(
            Update,
            player_action_system.run_if(in_state(GameState::Playing)),
        );
    }
}
