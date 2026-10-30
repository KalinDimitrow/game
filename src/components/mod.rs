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

/// Marks a [`CharacterVisual`] whose nested glTF scene instance has finished
/// spawning, so its `AnimationPlayer` is known to exist. Inserted by
/// `character_controller::mark_character_scene_spawned`, an observer on
/// `bevy_world_serialization`'s `WorldInstanceReady`.
///
/// Not reflectable/authorable: this is a runtime fact about a load, never
/// something `test_world.scn.ron` should be able to assert.
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

/// Which locomotion animation `Character` should currently be playing. Set by
/// `move_character`, read by `apply_character_animation` - kept as its own
/// component (rather than re-deriving it from velocity every frame in the
/// animation system) so the animation only restarts on an actual state change.
/// `Hash`/`Eq` because this is also the key type for
/// [`CharacterGltfHandles::animations`](crate::resources::CharacterGltfHandles)
/// and [`CharacterAnimations::nodes`](crate::resources::CharacterAnimations) -
/// keying those by the state that plays a clip is what replaced a `Vec`
/// indexed by this enum's variant order.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
#[reflect(Component, Default, Debug, PartialEq, Hash)]
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
/// - `Loading`: waiting on assets to finish their background load. Every
///   plugin registers its own handles into
///   [`PendingAssets`](crate::resources::PendingAssets) during `Startup`, and
///   `systems::core::check_pending_assets` drives the transition off
///   `AssetServer`'s own load state (see `PendingAssets`'s doc comment, and
///   `docs/Bevy/06 Assets and Scenes.md`).
///   Tried collapsing this into a blocking `AssetServer::wait_for_asset` call
///   in `Startup` instead (no async phase to represent at all) - it
///   deadlocks every time: the system that actually finalizes a load into
///   `Assets<T>` and flips its load state (`handle_internal_asset_events`)
///   runs in `PreUpdate`, which can't start until `Startup` returns. Blocking
///   inside `Startup` waits on something that can only ever happen *after*
///   `Startup` finishes - confirmed hung for 45+ seconds before reverting.
///   It's only safe from a genuinely separate thread (e.g.
///   `AsyncComputeTaskPool`), never from inside the schedule - so this stays
///   an `Update`-polled state instead.
/// - `Initialize`: assets are confirmed loaded *and not failed*, so one-time
///   setup that needs them (building an `AnimationGraph`) can run in
///   `OnEnter` - entering the state *is* the guarantee. Also where setup
///   waits on entity-level facts that asset readiness can't front-load: the
///   scene spawner only spawns `test_world.scn.ron`'s entities once the asset
///   is loaded, so the character's `AnimationPlayer` appears a frame or two
///   into this state, not on entry.
/// - `Playing`: gameplay.
///
/// `CharacterControllerPlugin` drives Initialize -> Playing today, but
/// `Loading` is no longer its private concern: `PendingAssets` is pooled
/// across plugins, so `TerrainPlugin`'s world load gates startup too (it
/// didn't before - see that resource's doc comment).
///
/// There is no `LoadFailed` variant: a failed load is reported by
/// `check_pending_assets` returning `Err`, which routes through bevy's error
/// handler rather than parking the app in a state with nothing to render.
#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    Loading,
    Initialize,
    Playing,
}
