use bevy::prelude::*;
use leafwing_input_manager::Actionlike;

#[derive(Component)]
pub struct Person;

#[derive(Component)]
pub struct Name(pub String);

#[derive(Resource)]
pub struct GreetTimer(pub Timer);

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
#[actionlike(DualAxis)]
pub enum CameraMovement {
    Pan,
}