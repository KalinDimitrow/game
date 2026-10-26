---
tags: [bevy, assets, scenes]
---

# 06. Assets & Scenes

Once you're spawning more than a handful of hardcoded shapes, you need files: textures, meshes, and whole saved/loaded worlds. Bevy's asset system is async by default (loading never blocks a frame) and scenes are just serialized ECS worlds.

## Representative examples — Assets

- `asset/asset_loading.rs` — the several ways to load an asset
- `asset/custom_asset.rs`, `asset/custom_asset_reader.rs` — your own asset types and sources
- `asset/hot_asset_reloading.rs` — edit a file on disk, see it update live
- `asset/embedded_asset.rs` — bake an asset into the binary
- `asset/asset_processing.rs` — preprocess assets at build/import time
- `asset/multi_asset_sync.rs` — wait for a batch of assets to finish loading
- `asset/web_asset.rs` — load from a URL

## Representative examples — Scenes

- `scene/world_serialization.rs` — save/load an entire world to a file (this is what `.scn.ron` files are)
- `scene/bsn.rs` — BSN, a more ergonomic scene-composition syntax

## Used in `game`

`systems/terrain/mod.rs::spawn_test_world` spawns `DynamicWorldRoot(asset_server.load("worlds/test_world.scn.ron"))` — `scene/world_serialization.rs`'s loading half, same as `WorldAssetRoot` already does per-tile for glTF scenes (`spawn_terrain`) and per-character-mesh (`character_controller`). (Earlier revisions of this note pointed at `load_scene_example.scn.ron`/`main.rs` and a BSN `three_d_scene_example.rs` — both gone; this is the current reference.)

### Experiment: describing a whole scene (terrain + physics + light + character + camera) in one `.scn.ron`

`test_world.scn.ron` pushes `scene/world_serialization.rs` as far as it goes for this project: a hand-authored RON file carrying tiles, their collision, a light, and later the player character/camera — built up incrementally over one long session (see `docs/Game/01 Cleanup Backlog.md` for the narrative). Findings, since none of this is obvious from the examples alone:

- **Mesh references work as path strings.** A `Handle<T>` field (e.g. `WorldAssetRoot`'s `Handle<WorldAsset>`) deserializes from `Path("models/x.glb#Scene0")` in the RON — confirmed against `bevy_world_serialization`'s own `HandleDeserializeProcessor`. This is also how one `.scn.ron` can reference another as a sub-scene (same mechanism, different target file).
- **Physics components need a `Reflect`-friendly stand-in.** `avian3d::Collider` is explicitly documented as not `Reflect` ("if you need to reflect it, use `ColliderConstructor`"). `RigidBody`/`LockedAxes`/`ColliderConstructor` all *are* reflectable and usable directly in RON.
- **No material authoring.** `StandardMaterial` has no registered file-loader of its own — a RON-described entity only ever gets whatever material its referenced glTF already baked in. A runtime tint/override (this project's checkerboard terrain tint) has no RON form and has to stay a Rust system reacting to `WorldInstanceReady`.
- **No `InputMap` authoring.** Reflectable in principle, but its fields are `HashMap<Action, Vec<Box<dyn Buttonlike>>>` — boxed trait objects aren't something to hand-write in RON. Attached reactively in Rust instead (`Added<Character>`/`Added<Camera3d>`), regardless of whether the entity came from RON or code.
- **ECS hierarchy (`ChildOf`) under a physics body doesn't propagate `GlobalTransform` reliably.** See [[03 Transforms and Camera]] for the specific gotcha — it bit a RON-authored child the same way it would a hand-coded one.
- **Entity key `0` in the RON is rejected** ("attempting to deserialize an invalid entity") — start numbering from `1`.

Net effect on code volume: moving the character/camera into RON didn't *remove* Rust code, it relocated it — from "spawn these components" to "attach this InputMap / sync this transform / attach this animation graph once the right component shows up." For a one-off player and a handful of terrain tiles, that's a wash at best. The case where this would actually pay for itself is many near-identical, data-varying instances (enemy/prop prefabs) referencing a shared `.scn.ron`, closer to the original motivating idea than to the terrain/player test this ended up covering.

## Where this fits

- ⬅ Comes from: [[04 2D Rendering]], [[05 3D Rendering]]
- ➡ Leads to: [[07 glTF and 3D Models]] (the most common "real" asset type), [[08 Reflection]] (scene serialization is built on reflection)
- 🔗 Related: [[16 Async Tasks and Remote Protocol]] (asset loading is itself async), [[21 Showcase Games]] (`showcase/loading_screen.rs` waits on asset+pipeline readiness)

[[00 Bevy Capabilities Map|← Back to map]]
