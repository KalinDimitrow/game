---
tags: [bevy, animation]
---

# 10. Animation

Moving things over time in a structured way — skeletal animation from glTF, procedural transform/color animation, and the graph system for blending multiple animations together.

## Representative examples

- `animation/animated_transform.rs` — animate a `Transform` purely from code, no asset needed
- `animation/animated_mesh.rs` — play a skinned glTF animation (the fox)
- `animation/animated_mesh_control.rs` — drive that animation with keyboard input
- `animation/animation_graph.rs` — blend multiple animation clips together
- `animation/animation_masks.rs` — restrict an animation to part of a skeleton
- `animation/easing_functions.rs`, `animation/eased_motion.rs` — easing curves for non-linear motion
- `animation/color_animation.rs` — animate colors across color spaces
- `animation/morph_targets.rs` — blend-shape style deformation
- `animation/animated_ui.rs` — animate UI node properties, not just world objects

## Where this fits

- ⬅ Comes from: [[09 Input and Picking]]
- ➡ Leads to: [[11 UI]]
- 🔗 Related: [[07 glTF and 3D Models]] (skinned meshes are the usual animation source), [[17 Time and Math]] (easing curves live in bevy's math/curve types), [[19 Stress Tests and Testbeds]] (`stress_tests/many_foxes.rs`, `many_morph_targets.rs` benchmark this)

[[00 Bevy Capabilities Map|← Back to map]]
