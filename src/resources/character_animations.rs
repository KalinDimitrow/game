use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::CharacterAnimationState;

#[derive(Resource, Default)]
pub struct CharacterGltfHandles {
    pub animations: HashMap<CharacterAnimationState, Handle<Gltf>>,
}

#[derive(Resource)]
pub struct CharacterAnimations {
    pub nodes: HashMap<CharacterAnimationState, AnimationNodeIndex>,
    pub graph_handle: Handle<AnimationGraph>,
}

