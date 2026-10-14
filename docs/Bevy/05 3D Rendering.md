---
tags: [bevy, 3d, rendering, pbr]
---

# 05. 3D Rendering

Bevy's deep end: physically-based rendering, lighting, shadows, and a long tail of modern-renderer features (SSAO, SSR, volumetric fog, meshlets...). Most of the engine's rendering complexity lives here — the 68 examples in `3d/` are more than double any other category.

## Representative examples (the "basics" slice)

- `3d/3d_scene.rs` — the canonical starting point: shapes + lighting
- `3d/3d_shapes.rs` — built-in primitive meshes
- `3d/pbr.rs` — physically-based material properties
- `3d/lighting.rs`, `3d/spotlight.rs`, `3d/rect_light.rs` — light types
- `3d/parenting.rs` — transform hierarchy in practice
- `3d/texture.rs` — texture-mapped materials
- `3d/generate_custom_mesh.rs` — build a mesh from code

## Representative examples (once the basics feel small)

- `3d/anti_aliasing.rs`, `3d/tonemapping.rs`, `3d/color_grading.rs`, `3d/bloom_3d.rs` — post-processing
- `3d/ssao.rs`, `3d/ssr.rs`, `3d/volumetric_fog.rs`, `3d/fog_volumes.rs` — advanced lighting effects
- `3d/shadow_biases.rs`, `3d/pcss.rs`, `3d/contact_shadows.rs` — shadow tuning
- `3d/deferred_rendering.rs`, `3d/order_independent_transparency.rs` — pipeline-level choices
- `3d/skybox.rs`, `3d/reflection_probes.rs`, `3d/irradiance_volumes.rs` — environment lighting
- `3d/meshlet.rs`, `3d/solari.rs` — experimental high-end rendering (meshlets, raytracing)
- `3d/mesh_ray_cast.rs` — CPU-side ray casting against meshes

## Used in `game`

`src/main.rs::setup` spawns a `PointLight` and a `Camera3d` — almost exactly `3d/lighting.rs` + `3d/3d_scene.rs`'s camera, minus the mesh. `materialize_shapes` turns `SpawnFloor`/`SpawnCube` placeholder components into real `Mesh3d`/`MeshMaterial3d` entities, which is the same pattern as `3d/generate_custom_mesh.rs` but driven by a marker-component query instead of running once at startup.

## Where this fits

- ⬅ Comes from: [[03 Transforms and Camera]]
- ➡ Leads to: [[06 Assets and Scenes]]
- 🔗 Related: [[07 glTF and 3D Models]] (real-world meshes instead of primitives), [[13 Shaders and Custom Rendering]] (everything here eventually wants a custom shader), [[14 Gizmos and Debug Drawing]] (debugging what you're rendering)

[[00 Bevy Capabilities Map|← Back to map]]
