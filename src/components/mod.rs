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

// Default-mapped to A/D/W/S/Space in `systems/character_controller`. Buttonlike,
// same reasoning as `PlayerAction`: systems react to these, never to raw KeyCodes.
#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum CharacterMovement {
    Left,
    Right,
    Forward,
    Backward,
    Jump,
}

/// Marks the player-controlled capsule so camera/movement systems can find it.
#[derive(Component)]
pub struct Character;

/// Orbit state for the camera following `Character`, driven by `CameraMovement::Pan`.
#[derive(Component)]
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

/// Which locomotion animation `Character` should currently be playing. Set by
/// `move_character`, read by `apply_character_animation` - kept as its own
/// component (rather than re-deriving it from velocity every frame in the
/// animation system) so the animation only restarts on an actual state change.
#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharacterAnimationState {
    #[default]
    Idle,
    Walking,
    // No real jump clip exists in the available assets (see the cleanup
    // backlog) - this reuses the running animation as a stand-in.
    Jumping,
}
