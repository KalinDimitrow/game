---
tags: [bevy, performance, testing]
---

# 19. Stress Tests & Testbeds

Once the game works, the question becomes "does it still work at scale." This category exists to benchmark and stress specific engine subsystems in isolation — recommended to run in `--release` mode, since the whole point is performance measurement.

## Representative examples

- `stress_tests/many_cubes.rs` — per-entity draw overhead; has a `sphere` mode to test frustum culling
- `stress_tests/many_foxes.rs` — skinned-mesh animation performance at scale (configurable count)
- `stress_tests/many_lights.rs` — point-light rendering limits
- `stress_tests/many_sprites.rs`, `bevymark.rs` — 2D sprite throughput
- `stress_tests/many_cameras_lights.rs` — multi-camera/light overhead
- `stress_tests/many_components.rs` — raw ECS scale (entities/components/systems)
- `stress_tests/transform_hierarchy.rs` — transform propagation performance through deep hierarchies
- `stress_tests/text_pipeline.rs`, `many_glyphs.rs`, `many_text.rs` — text rendering throughput
- `testbed/*` — interactive testbeds for exploring a subsystem live, rather than a fixed benchmark

## Where this fits

- ⬅ Comes from: [[18 Diagnostics and Dev Tools]]
- ➡ Leads to: [[20 Platform Targets]]
- 🔗 Related: [[02 ECS Core]] (`many_components.rs` stresses raw ECS), [[05 3D Rendering]] / [[04 2D Rendering]] (most stress tests target one of these two render paths), [[10 Animation]] (`many_foxes.rs`, `many_morph_targets.rs`)

[[00 Bevy Capabilities Map|← Back to map]]
