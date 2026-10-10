use bevy::prelude::*;
use leafwing_input_manager::Actionlike;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Person;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Name(pub String);

#[derive(Resource)]
pub struct GreetTimer(pub Timer);

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
#[actionlike(DualAxis)]
pub enum CameraMovement {
    Pan,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SpawnFloor {
    pub radius: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SpawnCube;