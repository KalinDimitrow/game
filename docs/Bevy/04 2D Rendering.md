---
tags: [bevy, 2d, rendering]
---

# 04. 2D Rendering

Sprites, 2D meshes, and the handful of post-processing effects Bevy supports in 2D. The lightest-weight rendering path — good for the first "I actually see something on screen" moment.

## Representative examples

- `2d/sprite.rs` — render a single sprite
- `2d/move_sprite.rs`, `2d/rotation.rs`, `2d/rotate_to_cursor.rs` — move/rotate sprites
- `2d/sprite_sheet.rs`, `2d/sprite_animation.rs`, `2d/texture_atlas.rs` — sprite sheets and frame animation
- `2d/sprite_slice.rs` — 9-patch scalable UI-ish sprites
- `2d/tilemap_chunk.rs` — tilemap rendering
- `2d/mesh2d.rs`, `2d/mesh2d_manual.rs` — 2D meshes, including "do it yourself" low-level
- `2d/text2d.rs` — text as a world-space 2D object (vs. UI text)
- `2d/bloom_2d.rs`, `2d/wireframe_2d.rs`, `2d/transparency_2d.rs` — 2D post-processing/render modes
- `2d/cpu_draw.rs` — manually read/write texture pixels

## Where this fits

- ⬅ Comes from: [[03 Transforms and Camera]]
- ➡ Leads to: [[06 Assets and Scenes]] — sprites are textures, which are assets
- 🔗 Related: [[05 3D Rendering]] (shares materials/shader concepts), [[13 Shaders and Custom Rendering]] (`sprite_material.rs` is a custom shader for sprites), [[09 Input and Picking]] (`picking/sprite_picking.rs`)

[[00 Bevy Capabilities Map|← Back to map]]
