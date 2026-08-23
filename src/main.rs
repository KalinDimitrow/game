use bevy::prelude::*;
use game::components::*;
use game::systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HelloPlugin)
        .run();
}

fn hello_world() {
    println!("hello world!");
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Startup, add_people);
        // app.add_systems(Update, hello_world);
        app.add_systems(Update,  (update_people, greet_people).chain());
    }
}




