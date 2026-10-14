use bevy::prelude::*;
use leafwing_input_manager::Actionlike;

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
#[actionlike(DualAxis)]
pub enum CameraMovement {
    Pan,
}

// Buttonlike actions a player can trigger, independent of which physical key/button
// is bound to them. The binding (action -> key) is loaded at runtime, so it can be
// remapped without touching this enum.
#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum PlayerAction {
    Confirm,
    Pause,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SpawnFloor {
    pub radius: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SpawnCube;