use bevy::prelude::*;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

/// Counts asset loads still in flight, as one atomic rather than a set of
/// handle ids to tick off. Modeled on bevy's own `asset/multi_asset_sync.rs`
/// example, minus its async half (nothing here needs to `.await` the
/// barrier, so there's no `event_listener::Event` and no
/// `AsyncComputeTaskPool` task - just `is_ready()`, polled from a system).
///
/// Each load is tagged with an [`AssetBarrierGuard`] via
/// `AssetServer::load_builder().with_guard(..)`; `bevy_asset` drops that
/// guard when the load finishes. The reason this is worth having over
/// accumulating `AssetEvent::LoadedWithDependencies` ids (what this replaced):
/// the guard is dropped when a load **fails**, too - confirmed in
/// `bevy_asset`'s `spawn_load_task`, which does `if let Err(err) = ... { error!(..) }`
/// and then `drop(guard)` unconditionally. `LoadedWithDependencies` only
/// ever fires on success, so a missing or corrupt file used to leave the
/// game stuck in `GameState::Loading` forever with nothing but an error log.
///
/// # Ordering caveat
///
/// `is_ready()` becoming true does **not** by itself mean `Assets<T>::get()`
/// will return the asset: the guard is dropped on the IO task pool thread as
/// soon as the load task finishes, while the asset is only inserted into
/// `Assets<T>` later, by `handle_internal_asset_events` in `PreUpdate`. What
/// makes the current usage safe is schedule order - the check runs in
/// `Update` and the code that actually reads the asset runs in
/// `OnEnter(..)`, i.e. in `StateTransition`, which `bevy_state` installs
/// immediately *after* `PreUpdate`. So the earliest an `OnEnter` can observe
/// the transition is the next frame, by which point `PreUpdate` has already
/// inserted the asset. Reading the asset from a system that runs in the same
/// `Update` as the check would race.
#[derive(Resource)]
pub struct AssetBarrier(Arc<AtomicU32>);

/// Held for the duration of one asset load; decrements [`AssetBarrier`]'s
/// count on drop, whether that load succeeded or failed.
pub struct AssetBarrierGuard(Arc<AtomicU32>);

impl AssetBarrier {
    /// Returns a barrier plus the guard to hand to each load. The count
    /// starts at 1 - held by the returned guard - so the barrier can't read
    /// as ready while loads are still being queued up; dropping that
    /// original guard once every load is registered is what arms it.
    pub fn new() -> (AssetBarrier, AssetBarrierGuard) {
        let count = Arc::new(AtomicU32::new(1));
        (AssetBarrier(count.clone()), AssetBarrierGuard(count))
    }

    /// True once every guard has been dropped - i.e. every tagged load has
    /// either finished or failed.
    pub fn is_ready(&self) -> bool {
        self.0.load(Ordering::Acquire) == 0
    }
}

impl Clone for AssetBarrierGuard {
    fn clone(&self) -> Self {
        self.0.fetch_add(1, Ordering::AcqRel);
        AssetBarrierGuard(self.0.clone())
    }
}

impl Drop for AssetBarrierGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

/// The mesh/skeleton and each animation are separate glTF assets - Bevy
/// matches clips to scene entities by bone name, not by source file, so
/// these just need to be loaded independently and combined once all of them
/// are ready. Populated by `systems::character_controller::load_character_gltf_handles`.
#[derive(Resource)]
pub struct CharacterGltfHandles {
    pub mesh: Handle<Gltf>,
    pub idle: Handle<Gltf>,
    pub walking: Handle<Gltf>,
    pub jumping: Handle<Gltf>,
}

/// Indexed by `CharacterAnimationState` variant order: Idle, Walking, Jumping.
/// Built by `systems::character_controller::build_character_animation_graph`
/// once every handle in `CharacterGltfHandles` has finished loading.
#[derive(Resource)]
pub struct CharacterAnimations {
    pub indices: Vec<AnimationNodeIndex>,
    pub graph_handle: Handle<AnimationGraph>,
}
