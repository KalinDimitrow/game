use bevy::prelude::*;
use game::components::*;
use game::systems::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HelloPlugin)
        .add_plugins(InputManagerPlugin::<CameraMovement>::default())
        .run();
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        // app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        // app.add_systems(Startup, add_people);
        // app.add_systems(Update, hello_world);
        // app.add_systems(Update,  (update_people, greet_people).chain());
        app.add_systems(Update, (keyboard_input_system, pan_camera, materialize_shapes));
        app.add_systems(Startup, (scene.spawn(), setup));
        app.register_type::<MeshMaterial3d<StandardMaterial>>();
        app.register_type::<SpawnFloor>();
        app.register_type::<SpawnCube>();
    }
}
