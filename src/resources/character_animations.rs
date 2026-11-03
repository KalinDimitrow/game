use bevy::prelude::*;
use std::collections::HashMap;

use super::PendingAssets;
use crate::components::CharacterAnimationState;
use crate::utils::constants::CHARACTER_ANIMATIONS;

#[derive(Resource)]
pub struct CharacterGltfHandles {
    pub animations: HashMap<CharacterAnimationState, Handle<Gltf>>,
}

#[derive(Resource)]
pub struct CharacterAnimations {
    pub nodes: HashMap<CharacterAnimationState, AnimationNodeIndex>,
    pub graph_handle: Handle<AnimationGraph>,
}

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
