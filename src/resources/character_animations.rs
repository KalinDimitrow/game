use bevy::prelude::*;
use std::collections::HashMap;

use super::PendingAssets;
use crate::components::CharacterAnimationState;
use crate::utils::constants::CHARACTER_ANIMATIONS;

/// One glTF per animation clip, keyed by the state that plays it. Populated by
/// [`load_character_gltf_handles`] from `CHARACTER_ANIMATIONS`.
///
/// The mesh/skeleton isn't here: it arrives as a loader dependency of
/// `test_world.scn.ron` (entity 10's `WorldAssetRoot`), which [`PendingAssets`]
/// tracks recursively. The separate `mesh` handle this used to carry existed
/// only to pad the old barrier, was read by nothing, and tracked the wrong
/// asset anyway - `character.glb`, the root `Gltf`, where the scene spawns
/// `character.glb#Scene0`, a distinct `AssetId`.
#[derive(Resource)]
pub struct CharacterGltfHandles {
    pub animations: HashMap<CharacterAnimationState, Handle<Gltf>>,
}

/// Built by [`build_character_animation_graph`] once every handle in
/// [`CharacterGltfHandles`] has loaded.
///
/// Keyed by the state that plays each clip, rather than a `Vec` indexed by
/// `CharacterAnimationState`'s variant order. That ordinal agreement was
/// unchecked and had to be kept in sync by comment across three places at
/// once: the load order, `AnimationGraph::from_clips`'s argument order, and a
/// `match` in `apply_character_animation`. A map makes the key the thing the
/// lookup is actually about.
#[derive(Resource)]
pub struct CharacterAnimations {
    pub nodes: HashMap<CharacterAnimationState, AnimationNodeIndex>,
    pub graph_handle: Handle<AnimationGraph>,
}

// Loads one animation-only glTF per `CHARACTER_ANIMATIONS` entry, registering
// each with `PendingAssets` so `CorePlugin`'s `check_pending_assets` waits on
// them. Scheduled on `Startup` by `CharacterControllerPlugin`.
//
// Driven off the table rather than four named consts and four named fields, so
// the set of clips is described in exactly one place (`CHARACTER_ANIMATIONS`)
// instead of being restated here, in `CharacterGltfHandles`, in the
// `AnimationGraph`'s argument order, and in a `match` downstream.
//
// The mesh/skeleton isn't loaded here. `test_world.scn.ron`'s `CharacterVisual`
// references it via its own `WorldAssetRoot`, so it's a dependency of the world
// handle `terrain::spawn_test_world` tracks - covered by the gate without being
// named twice. The old `mesh` field was read by nothing and tracked
// `character.glb` where the scene actually spawns `character.glb#Scene0`.
pub fn load_character_gltf_handles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut pending: ResMut<PendingAssets>,
) {
    let animations: HashMap<_, _> = CHARACTER_ANIMATIONS
        .iter()
        .map(|&(state, path)| {
            let handle: Handle<Gltf> = asset_server.load(path);
            pending.track(handle.clone());
            (state, handle)
        })
        .collect();

    commands.insert_resource(CharacterGltfHandles { animations });
}

// Builds the `AnimationGraph` from the animation-only glTF files. Runs in
// `OnEnter(Initialize)`, not `Update`: entering `Initialize` already means
// every tracked load finished *successfully* (`check_pending_assets` returns
// `Err` on a failure instead of advancing), so there's no readiness guard
// here. Runs exactly once, ever.
//
// (Tried collapsing this and the readiness check into one blocking `Startup`
// system instead, via `AssetServer::wait_for_asset` + `bevy::tasks::block_on`
// - see `GameState`'s doc comment for why that deadlocks every time and was
// reverted.)
//
// Still fallible rather than `.expect()`-ing, even with that guarantee: the
// expects it replaced asserted exactly this invariant back when it *wasn't*
// true (the `AssetBarrier` released on failed loads too, so a missing
// `Idle.glb` reached this function and panicked). Returning `Err` means a
// regression in the gate shows up as an error with a state name in it, not a
// panic in an unrelated-looking function.
//
// Built node-by-node rather than with `AnimationGraph::from_clips`, because
// `from_clips` returns a `Vec` whose order is the caller's problem to keep
// aligned with `CharacterAnimationState`. `add_clip` returns one index at a
// time, so each can be stored under the state that plays it - which also
// means the iteration order of `animations` is irrelevant here.
pub fn build_character_animation_graph(
    mut commands: Commands,
    gltf_handles: Res<CharacterGltfHandles>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) -> Result<(), BevyError> {
    let mut graph = AnimationGraph::new();
    let root = graph.root;
    let mut nodes = HashMap::new();

    for (&state, handle) in &gltf_handles.animations {
        let gltf = gltfs.get(handle).ok_or_else(|| {
            format!("character glTF for {state:?} is not in `Assets<Gltf>` on entering Initialize")
        })?;

        // Each animation-only file has exactly one clip, named after its own
        // source file by the conversion script - take whichever one it is
        // rather than hardcoding an exact internal name.
        let clip = gltf
            .named_animations
            .values()
            .next()
            .ok_or_else(|| format!("character glTF for {state:?} has no animation clip"))?
            .clone();

        // Bevy matches a clip's curves to scene entities by bone name
        // (`AnimationTargetId`), not by which file the clip came from, so
        // clips from entirely separate glTF files all apply correctly to the
        // mesh's skeleton once `finish_character_initialization` attaches
        // this graph to its `AnimationPlayer`.
        nodes.insert(state, graph.add_clip(clip, 1.0, root));
    }

    let graph_handle = graphs.add(graph);
    commands.insert_resource(CharacterAnimations { nodes, graph_handle });

    Ok(())
}
