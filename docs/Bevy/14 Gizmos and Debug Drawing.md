---
tags: [bevy, gizmos, debugging]
---

# 14. Gizmos & Debug Drawing

Immediate-mode drawing — lines, shapes, text — for visualizing things that don't need a persistent mesh: axes, light ranges, transform handles, debug overlays. Cheap to add, and the natural companion to any custom rendering or physics-ish work.

## Representative examples

- `gizmos/2d_gizmos.rs`, `gizmos/3d_gizmos.rs` — the baseline immediate-mode drawing API
- `gizmos/axes.rs` — draw coordinate axes on an entity
- `gizmos/light_gizmos.rs` — visualize light ranges/directions
- `gizmos/transform_gizmo.rs` — an interactive translate/rotate/scale handle
- `gizmos/anchored_text_gizmos.rs`, `2d_text_gizmos.rs`, `3d_text_gizmos.rs` — text as a gizmo
- `math/render_primitives.rs` — render every math primitive as both a mesh and a gizmo, side by side

## Where this fits

- ⬅ Comes from: [[13 Shaders and Custom Rendering]]
- ➡ Leads to: [[15 State Management]]
- 🔗 Related: [[18 Diagnostics and Dev Tools]] (`dev_tools/infinite_grid.rs`, `fps_overlay.rs` are gizmo-adjacent), [[17 Time and Math]] (gizmos are the usual way to visualize math primitives)

[[00 Bevy Capabilities Map|← Back to map]]
