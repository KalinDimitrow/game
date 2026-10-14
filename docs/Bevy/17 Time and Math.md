---
tags: [bevy, time, math]
---

# 17. Time & Math

The cross-cutting toolbox. Technically available from the very first frame (`Time` is a resource in every app), but called out here because this is roughly where projects start deliberately reaching for it — fixed timesteps for simulation, curves for animation/easing, bounding volumes for picking/culling.

## Representative examples — Time

- `time/time.rs` — how `Time` works inside the ECS
- `time/timers.rs` — ticking `Timer` resources
- `time/virtual_time.rs` — `Time<Virtual>` for pause/slow-mo/speed-up without touching real wall-clock time

## Representative examples — Math

- `math/cubic_splines.rs` — cubic curve construction
- `math/bounding_2d.rs` — bounding volumes and intersection tests
- `math/custom_primitives.rs` — add your own primitive shape type
- `math/random_sampling.rs` — sample random points from primitives
- `movement/physics_in_fixed_timestep.rs` — the "industry-standard" input/physics/render split using a fixed timestep (lives in `movement/` but is really a Time+Math use case)
- `movement/smooth_follow.rs` — interpolated following (lives in `math/` README section, file under `movement/`)

## Where this fits

- ⬅ Comes from: [[16 Async Tasks and Remote Protocol]]
- ➡ Leads to: [[18 Diagnostics and Dev Tools]]
- 🔗 Related: [[02 ECS Core]] (`ecs/fixed_timestep.rs` is the ECS-level version of the same idea), [[10 Animation]] (easing curves), [[14 Gizmos and Debug Drawing]] (visualizing primitives/bounding volumes)

[[00 Bevy Capabilities Map|← Back to map]]
