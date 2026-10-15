pub mod button_config;
pub mod input;

pub use self::button_config::*;
pub use self::input::*;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::components::{CameraMovement, PlayerAction};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CameraMovement>::default());
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());

        app.add_systems(Update, player_action_system);
        app.add_systems(Startup, load_button_config);
    }
}
