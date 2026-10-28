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
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Character;

/// Marks the child entity holding the character's glTF scene, so movement
/// systems can rotate it to face the direction of travel without touching
/// `Character`'s own transform, whose rotation is locked for physics.
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct CharacterVisual;

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

/// Which locomotion animation `Character` should currently be playing. Set by
/// `move_character`, read by `apply_character_animation` - kept as its own
/// component (rather than re-deriving it from velocity every frame in the
/// animation system) so the animation only restarts on an actual state change.
#[derive(Component, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component, Default, PartialEq)]
pub enum CharacterAnimationState {
    #[default]
    Idle,
    Walking,
    // No real jump clip exists in the available assets (see the cleanup
    // backlog) - this reuses the running animation as a stand-in.
    Jumping,
}

/// Three stages, not two - `Loading` and `Initialize` split apart because
/// they wait on different *kinds* of fact:
/// - `Loading`: waiting on assets to finish their background load, tracked
///   with an [`AssetBarrier`](crate::resources::AssetBarrier) (see its doc
///   comment, and `docs/Bevy/06 Assets and Scenes.md`).
///   Tried collapsing this into a blocking `AssetServer::wait_for_asset` call
///   in `Startup` instead (no async phase to represent at all) - it
///   deadlocks every time: the system that actually finalizes a load into
///   `Assets<T>` and flips its load state (`handle_internal_asset_events`)
///   runs in `PreUpdate`, which can't start until `Startup` returns. Blocking
///   inside `Startup` waits on something that can only ever happen *after*
///   `Startup` finishes - confirmed hung for 45+ seconds before reverting.
///   `multi_asset_sync.rs`'s async variant avoids this by spawning the wait
///   on `AsyncComputeTaskPool` - a genuinely separate thread, not the one
///   running the schedule - so this stays an `Update`-polled state instead.
/// - `Initialize`: assets are confirmed loaded, so one-time setup that needs
///   them (building an `AnimationGraph`) can run in `OnEnter` with no
///   readiness guard at all - entering the state *is* the guarantee. Also
///   where setup still waits on entity-level facts (an entity spawning,
///   which has no equivalent "finished" event - only a repeated query) that
///   a state transition can't front-load a guarantee for.
/// - `Playing`: gameplay.
///
/// `CharacterControllerPlugin` drives Loading -> Initialize -> Playing today;
/// any future plugin with its own loading-time dependency can reuse these
/// same three stages instead of growing its own ad-hoc readiness check.
#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    Loading,
    Initialize,
    Playing,
}
