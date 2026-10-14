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

`src/main.rs` spawns `DynamicWorldRoot(asset_server.load(WORLD_FILE_PATH))` pointing at `load_scene_example.scn.ron` — this *is* `scene/world_serialization.rs`'s loading half. `three_d_scene_example.rs::scene()` uses the BSN-style `bsn_list!` macro (currently fully commented out), which maps directly to `scene/bsn.rs`.

## Where this fits

- ⬅ Comes from: [[04 2D Rendering]], [[05 3D Rendering]]
- ➡ Leads to: [[07 glTF and 3D Models]] (the most common "real" asset type), [[08 Reflection]] (scene serialization is built on reflection)
- 🔗 Related: [[16 Async Tasks and Remote Protocol]] (asset loading is itself async), [[21 Showcase Games]] (`showcase/loading_screen.rs` waits on asset+pipeline readiness)

[[00 Bevy Capabilities Map|← Back to map]]
