pub mod button_config;
pub mod input;
pub mod people;
pub mod scene;

pub use self::button_config::*;
pub use self::input::*;
pub use self::people::*;
pub use self::scene::*;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::components::{CameraMovement, PlayerAction, SpawnCube, SpawnFloor};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CameraMovement>::default());
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());

        // app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        // app.add_systems(Startup, add_people);
        // app.add_systems(Update, (update_people, greet_people).chain());
        app.add_systems(
            Update,
            (
                keyboard_input_system,
                pan_camera,
                materialize_shapes,
                player_action_system,
            ),
        );
        app.add_systems(Startup, (scene.spawn(), setup, load_button_config));
        app.register_type::<MeshMaterial3d<StandardMaterial>>();
        app.register_type::<SpawnFloor>();
        app.register_type::<SpawnCube>();
    }
}
