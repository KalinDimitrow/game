use bevy::asset::{RecursiveDependencyLoadState, UntypedHandle};
use bevy::prelude::*;

use crate::components::GameState;

/// Every asset handle `GameState::Loading` should wait on, pooled across
/// plugins: each registers its own loads during `Startup`, and
/// [`check_pending_assets`] folds `AssetServer`'s load state over the whole
/// list. So "the game finished loading" is one fact about one list, not a
/// readiness check per plugin - which is what let `TerrainPlugin`'s world load
/// join the gate (see "what this fixes" below).
///
/// # Why not the `AssetBarrier` this replaced
///
/// The previous version was a hand-rolled `Arc<AtomicU32>` tagged onto each
/// load via `AssetServer::load_builder().with_guard(..)`, modeled on bevy's
/// `asset/multi_asset_sync.rs`, with the guard's `Drop` decrementing the
/// count. `AssetServer` already tracks all of this, and three things make it
/// the better source of truth:
///
/// - **It tells success apart from failure.** The guard's `Drop` runs either
///   way (`bevy_asset`'s `spawn_load_task` does `drop(guard)` unconditionally
///   after logging), so `is_ready()` only ever meant "settled", never
///   "usable". Every reader downstream had to `.expect()` on an invariant the
///   barrier didn't actually provide - a missing `Idle.glb` released the
///   barrier, entered `Initialize`, then panicked in
///   `build_character_animation_graph`. `RecursiveDependencyLoadState` carries
///   `Failed(Arc<AssetLoadError>)`, so that becomes an
///   [`AssetLoadProgress::Failed`] with a path in it.
/// - **It covers dependencies.** The guard is dropped as soon as *that one*
///   asset's load task ends; anything the asset referenced is a separate task
///   with no guard on it. That was invisible for the character glTFs (their
///   clips are labeled sub-assets inside one file) but made the pattern
///   useless for `test_world.scn.ron`, whose 7 glTF scenes are loader
///   dependencies - guarding it would have proven only that the RON parsed.
///   `recursive_dependency_load_state` is transitive, which is why tracking
///   the one world handle here now covers every tile and the character mesh.
/// - **No ordering caveat.** The guard dropped on the IO task pool thread,
///   ahead of the asset landing in `Assets<T>`, so reading an asset was only
///   safe by schedule accident (check in `Update`, read in a later
///   `OnEnter`). Load state flips in `handle_internal_asset_events`
///   (`PreUpdate`), where `process_asset_load` sets `Loaded` *and* inserts
///   into `Assets<T>` under one `AssetInfos` write lock - so "loaded" and
///   "gettable" are now the same instant, and the race the old doc comment
///   had to warn about doesn't exist.
///
/// # What this fixes beyond the barrier
///
/// `GameState::Loading` used to wait only on the 4 character glTFs. The world
/// (`terrain::spawn_test_world`) was loaded unguarded, so the state meant
/// "the animation clips settled", not "the world is ready" - what covered the
/// gap was `check_character_ready` polling for a `Character`, which only ever
/// comes from the scene. That made an entity query load-bearing for asset
/// readiness by accident, and left the original failure mode intact one state
/// later: a missing `test_world.scn.ron` released the barrier, entered
/// `Initialize`, and polled forever for a `Character` that could never spawn.
#[derive(Resource, Default)]
pub struct PendingAssets(Vec<UntypedHandle>);

/// Outcome of folding `AssetServer`'s load state over every handle in
/// [`PendingAssets`].
pub enum AssetLoadProgress {
    /// At least one load is still in flight and none have failed.
    Loading,
    /// Every load finished successfully; the assets are in `Assets<T>` now.
    Ready,
    /// One or more loads failed, one `path: error` string each.
    Failed(Vec<String>),
}

impl PendingAssets {
    /// Registers a handle to wait on. Takes the handle by value and keeps it:
    /// a strong handle here also keeps the asset alive, which matters for the
    /// world - its only other owner is a component on an entity.
    pub fn track(&mut self, handle: impl Into<UntypedHandle>) {
        self.0.push(handle.into());
    }

    /// Folds every tracked handle's `RecursiveDependencyLoadState` into one
    /// answer. Failures win over still-loading, so a broken asset surfaces
    /// immediately rather than after the slowest sibling finishes.
    pub fn progress(&self, asset_server: &AssetServer) -> AssetLoadProgress {
        let mut failures = Vec::new();
        let mut still_loading = false;

        for handle in &self.0 {
            match asset_server.recursive_dependency_load_state(handle.id()) {
                RecursiveDependencyLoadState::Loaded => {}
                RecursiveDependencyLoadState::NotLoaded
                | RecursiveDependencyLoadState::Loading => still_loading = true,
                RecursiveDependencyLoadState::Failed(error) => {
                    // The id's own path - for a failed *dependency* that's the
                    // parent that pulled it in, and `error`'s `Display` names
                    // the dependency itself, so both ends end up in the message.
                    let path = asset_server
                        .get_path(handle.id())
                        .map_or_else(|| "<unknown path>".to_string(), |path| path.to_string());
                    failures.push(format!("`{path}`: {error}"));
                }
            }
        }

        if !failures.is_empty() {
            AssetLoadProgress::Failed(failures)
        } else if still_loading {
            AssetLoadProgress::Loading
        } else {
            AssetLoadProgress::Ready
        }
    }
}

// Drives `Loading` -> `Initialize` off `AssetServer`'s own load state, folded
// over every handle any plugin registered in `PendingAssets` during `Startup`.
// One system for the whole app, not one readiness check per plugin.
//
// Fallible on purpose. A failed load returns `Err`, which routes through
// bevy's error handler with the offending path in the message. The two
// alternatives are both worse: a `GameState::LoadFailed` variant is a state
// with nothing to render, and the `AssetBarrier` version this replaced just
// advanced anyway and left the next state to `.expect()` on an invariant the
// barrier never actually provided (see [`PendingAssets`]'s doc comment).
pub fn check_pending_assets(
    pending: Res<PendingAssets>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result<(), BevyError> {
    match pending.progress(&asset_server) {
        AssetLoadProgress::Loading => {}
        AssetLoadProgress::Ready => next_state.set(GameState::Initialize),
        AssetLoadProgress::Failed(failures) => {
            return Err(format!(
                "{} asset(s) failed to load:\n  {}",
                failures.len(),
                failures.join("\n  ")
            )
            .into());
        }
    }

    Ok(())
}
