use bevy::prelude::*;

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
/// Built by `systems::character_controller::spawn_character_visual_when_ready`
/// once every handle in `CharacterGltfHandles` has finished loading.
#[derive(Resource)]
pub struct CharacterAnimations {
    pub indices: Vec<AnimationNodeIndex>,
    pub graph_handle: Handle<AnimationGraph>,
}
