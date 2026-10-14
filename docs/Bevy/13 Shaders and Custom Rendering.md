---
tags: [bevy, shaders, rendering, wgsl]
---

# 13. Shaders & Custom Rendering

The point where the built-in `StandardMaterial`/`ColorMaterial` stop being enough and you start writing WGSL yourself, or even bypassing the standard material system to write render phases directly. `shader/` (19 examples) is the approachable half; `shader_advanced/` (13 examples) is the "you're basically writing part of the renderer" half.

## Representative examples — Shaders (approachable)

- `shader/shader_material.rs`, `shader/shader_material_2d.rs` — a custom shader + material, 3D and 2D
- `shader/extended_material.rs` — extend `StandardMaterial` instead of replacing it
- `shader/animate_shader.rs` — pass time into a shader for animated effects
- `shader/shader_defs.rs` — compile-time toggles inside one shader (`#ifdef`-equivalent)
- `shader/array_texture.rs`, `shader/storage_buffer.rs` — custom data bindings
- `shader/compute_shader_game_of_life.rs` — compute shaders, not just vertex/fragment
- `shader/automatic_instancing.rs` — Bevy auto-batches identical meshes into one draw call

## Representative examples — Advanced (mid/low-level render API)

- `shader_advanced/custom_render_phase.rs`, `custom_phase_item.rs` — build a render phase from scratch
- `shader_advanced/manual_material.rs` — implement a material without the high-level helpers
- `shader_advanced/specialized_mesh_pipeline.rs` — a fully custom mesh pipeline
- `shader_advanced/custom_post_processing.rs` — a post-processing pass as its own render pass
- `shader_advanced/mesh_shader_intro.rs` — task/amplification/mesh shaders (GPU-driven geometry)

## Where this fits

- ⬅ Comes from: [[12 Audio]]
- ➡ Leads to: [[14 Gizmos and Debug Drawing]]
- 🔗 Related: [[05 3D Rendering]] and [[04 2D Rendering]] (this is "level 2" of both), [[19 Stress Tests and Testbeds]] (custom pipelines are usually perf-motivated)

[[00 Bevy Capabilities Map|← Back to map]]
