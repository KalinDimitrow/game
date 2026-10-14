---
tags: [bevy, gltf, assets]
---

# 07. glTF & 3D Models

The specific, very common case of "assets" that is bringing in real models made in Blender/etc. rather than primitives generated in code. glTF is Bevy's first-class 3D interchange format.

## Representative examples

- `gltf/load_gltf.rs` — load and render a glTF file as a scene (the entry point)
- `gltf/load_gltf_extras.rs` — read custom glTF "extras" metadata
- `gltf/query_gltf_primitives.rs` — inspect the mesh primitives inside a loaded glTF
- `gltf/update_gltf_scene.rs` — modify a glTF-spawned scene after the fact
- `gltf/edit_material_on_gltf.rs` — override materials post-spawn
- `gltf/gltf_skinned_mesh.rs` — skinned (riggable) meshes from glTF
- `tools/scene_viewer/main.rs` — a general-purpose glTF viewer, useful as a standalone debugging tool

## Where this fits

- ⬅ Comes from: [[06 Assets and Scenes]]
- ➡ Leads to: [[09 Input and Picking]] — once there's a real scene, you want to interact with it
- 🔗 Related: [[10 Animation]] (`animation/animated_mesh.rs` plays a skinned glTF fox), [[05 3D Rendering]] (glTF materials map onto PBR)

[[00 Bevy Capabilities Map|← Back to map]]
