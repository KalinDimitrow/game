use bevy::prelude::*;
use leafwing_input_manager::Actionlike;
use strum::EnumIter;

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
#[actionlike(DualAxis)]
pub enum CameraMovement {
    Pan,
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect, EnumIter)]
pub enum PlayerAction {
    Confirm,
    Pause,
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum CharacterMovement {
    Left,
    Right,
    Forward,
    Backward,
    Jump,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Character;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct CharacterVisual;

#[derive(Component)]
pub struct CharacterSceneSpawned;

/// Orbit state for the camera following `Character`, driven by `CameraMovement::Pan`.
#[derive(Component, Reflect)]
#[reflect(Component, Default)]
pub struct ThirdPersonCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: -0.3,
            distance: 6.0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
#[reflect(Component, Default, Debug, PartialEq, Hash)]
pub enum CharacterAnimationState {
    #[default]
    Idle,
    Walking,
    // No real jump clip exists in the available assets - this reuses the
    // running animation as a stand-in.
    Jumping,
}

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    Loading,
    Initialize,
    Playing,
}
