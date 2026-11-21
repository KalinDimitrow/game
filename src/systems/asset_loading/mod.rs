
use std::collections::HashMap;
use bevy::asset::RecursiveDependencyLoadState;
use crate::prelude::*;
use bevy::prelude::*;
use bevy::tasks::futures_lite::StreamExt;
use strum::{EnumIter, IntoEnumIterator, FromRepr};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Boot,
    InGame,
}

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter, FromRepr)]
#[source(GameState = GameState::Loading)]
#[repr(usize)]
pub enum AssetLoadingStates {
    #[default]
    Phase1,       // Normal gameplay, no menus open
    Phase2,     // Game paused menu open
    Phase3,  // Inventory panel open
}

impl AssetLoadingStates {
    pub fn next(&self) -> Option<Self> {
        Self::from_repr(*self as usize + 1)
    }
}


#[derive(Resource, Default)]
pub struct AssetHolder {
    handles: Vec<UntypedHandle>,
}

impl AssetHolder {
    pub fn add(&mut self, handles: &mut Vec<UntypedHandle>) {
        self.handles.append(handles);
    }
}
#[derive(Resource, Default)]
pub struct AssetTracker {
    handles: Vec<UntypedHandle>,
    last: usize,
}

impl AssetTracker {
    pub fn initiate_phase(&mut self, asset_server: &mut AssetServer, mut commands: Commands, gltf_handles: Res<CharacterGltfHandles>,
                          gltfs: Res<Assets<Gltf>>,
                          mut graphs: ResMut<Assets<AnimationGraph>>, phase: AssetLoadingStates) {
        match phase {
            AssetLoadingStates::Phase1 => {self.initiate_phase1(asset_server, commands, gltf_handles, gltfs, graphs)},
            AssetLoadingStates::Phase2 => {self.initiate_phase2(asset_server, commands, gltf_handles, gltfs, graphs)},
            AssetLoadingStates::Phase3 => {self.initiate_phase3(asset_server, commands, gltf_handles, gltfs, graphs)},
        }
        self.last = if self.handles.len() > 0 {self.handles.len() - 1} else {self.handles.len()};
    }
    pub fn ready(&mut self, asset_server: &AssetServer) -> bool {
        if self.handles.is_empty() {
            return true;
        }

        let mut first:usize = 0;
        while first <= self.last {
            let mut id = self.handles[first].id();
            while let RecursiveDependencyLoadState::Loaded = asset_server.recursive_dependency_load_state(id) && first < self.last {
                self.handles.swap(first, self.last);
                self.last -= 1;
                id = self.handles[first].id();
            }

            match asset_server.recursive_dependency_load_state(id) {
                RecursiveDependencyLoadState::Loaded | RecursiveDependencyLoadState::NotLoaded | RecursiveDependencyLoadState::Loading => {first += 1;},
                RecursiveDependencyLoadState::Failed(error) => {
                    first += 1;
                }
            }
        }

        self.last == 0
            && matches!(
            asset_server.recursive_dependency_load_state(self.handles[0].id()),
            RecursiveDependencyLoadState::Loaded
        )
    }

    fn initiate_phase1(&mut self, asset_server: &mut AssetServer, mut commands: Commands,     gltf_handles: Res<CharacterGltfHandles>,
                       gltfs: Res<Assets<Gltf>>,
                       mut graphs: ResMut<Assets<AnimationGraph>>,) {
        self.handles.push(asset_server.load::<ButtonConfig>(BUTTON_CONFIG_ASSET_PATH).into());
    }

    fn initiate_phase2(&mut self, asset_server: &mut AssetServer, mut commands: Commands,     gltf_handles: Res<CharacterGltfHandles>,
                       gltfs: Res<Assets<Gltf>>,
                       mut graphs: ResMut<Assets<AnimationGraph>>,) {
        let animations: HashMap<_, _> = CHARACTER_ANIMATIONS
            .iter()
            .map(|&(state, path)| {
                let handle: Handle<Gltf> = asset_server.load(path);
                self.handles.push(handle.clone().into());
                (state, handle)
            })
            .collect();
        commands.insert_resource(CharacterGltfHandles { animations });
    }

    fn initiate_phase3(&mut self, asset_server: &mut AssetServer, mut commands: Commands, gltf_handles: Res<CharacterGltfHandles>,
                       gltfs: Res<Assets<Gltf>>,
                       mut graphs: ResMut<Assets<AnimationGraph>>,) {
        let mut graph = AnimationGraph::new();
        let root = graph.root;
        let mut nodes = HashMap::new();

        for (&state, handle) in &gltf_handles.animations {
            let gltf = gltfs.get(handle).ok_or_else(|| {
                format!("character glTF for {state:?} is not in `Assets<Gltf>` on entering Initialize")
            }).unwrap();

            // Each animation-only file has exactly one clip, named after its own
            // source file by the conversion script - take whichever one it is
            // rather than hardcoding an exact internal name.
            let clip = gltf
                .named_animations
                .values()
                .next()
                .ok_or_else(|| format!("character glTF for {state:?} has no animation clip")).unwrap()
                .clone();

            // Bevy matches a clip's curves to scene entities by bone name
            // (`AnimationTargetId`), not by which file the clip came from, so
            // clips from entirely separate glTF files all apply correctly to the
            // mesh's skeleton once `finish_character_initialization` attaches
            // this graph to its `AnimationPlayer`.
            nodes.insert(state, graph.add_clip(clip, 1.0, root));
        }

        let graph_handle = graphs.add(graph);
        let animations = CharacterAnimations { nodes, graph_handle };
            commands.insert_resource(animations);

    }
}
pub struct AssetLoaderPlugin;

impl AssetLoaderPlugin {
    fn init_resources(&self, app: &mut App) {
        app.init_resource::<AssetTracker>()
            .init_resource::<AssetHolder>()
            .init_resource::<CharacterGltfHandles>()
            .add_sub_state::<AssetLoadingStates>();
    }

    fn configure_sets(&self, app: &mut App) {}

    fn configure_scheduler(&self, app: &mut App) {
        for state in AssetLoadingStates::iter() {
            app.add_systems(OnEnter(state), initiate_phase);
            app.add_systems(Update, poll_phase.run_if(in_state(state)));
            app.add_systems(OnExit(state), clean_phase);
        }
    }
}

pub fn initiate_phase(
    mut asset_server: ResMut<AssetServer>,
    mut tracker: ResMut<AssetTracker>,
    mut commands: Commands,
    gltf_handles: Res<CharacterGltfHandles>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    phase: Res<State<AssetLoadingStates>>,
) -> Result<(), BevyError> {
    tracker.initiate_phase(&mut asset_server, commands, gltf_handles, gltfs, graphs, *phase.get());

    Ok(())
}

pub fn poll_phase(
    asset_server: Res<AssetServer>,
    mut tracker: ResMut<AssetTracker>,
    phase: Res<State<AssetLoadingStates>>,
    mut next_phase: ResMut<NextState<AssetLoadingStates>>,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result<(), BevyError> {
    if tracker.ready(&asset_server) {
        if let Some(next) = phase.get().next() {
            next_phase.set(next);
        } else {
            next_state.set(GameState::Playing);
        }
    }

    Ok(())
}

pub fn clean_phase(
    mut asset_holder: ResMut<AssetHolder>,
    mut tracker: ResMut<AssetTracker>,
) -> Result<(), BevyError> {
    asset_holder.add(&mut tracker.handles);
    Ok(())
}

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        self.init_resources(app);
        self.configure_sets(app);
        self.configure_scheduler(app);
    }
}

#[cfg(test)]
mod tests {
}
