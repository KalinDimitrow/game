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

## Used in `game`

`character_controller/mod.rs` builds one `AnimationGraph::from_clips([idle, walking, jumping])` from three *separate* glTF files (each a single-animation export, matched to the mesh's skeleton by bone name — `animation/animation_graph.rs`'s multi-clip-blending idea, but sourced cross-file rather than from one asset), then plays/switches nodes via `AnimationTransitions` driven by `CharacterAnimationState` — close to `animation/animated_mesh_control.rs`.

### Gotcha: `Added<AnimationPlayer>` can miss the component entirely

`AnimationPlayer` is inserted by the glTF loader once the skinned mesh's scene actually finishes spawning — asynchronous, so attaching the `AnimationGraphHandle` has to happen in a system that reacts to it appearing, not at spawn time. The natural first attempt, `Query<_, Added<AnimationPlayer>>`, is a trap here: `Added` is only true for the one frame the component was inserted, and that's not guaranteed to be the same frame the `AnimationGraph` (built from 3 independently-loading glTF files) finishes and becomes available — confirmed via a throwaway diagnostic that the `AnimationPlayer` existed in the world the whole time, just never on a frame where the system's `Added` filter also matched. Query `Without<AnimationGraphHandle>` instead ("a player that hasn't been equipped yet") — it keeps matching every frame until the graph is attached, with no window to miss.

## Where this fits

- ⬅ Comes from: [[09 Input and Picking]]
- ➡ Leads to: [[11 UI]]
- 🔗 Related: [[07 glTF and 3D Models]] (skinned meshes are the usual animation source), [[17 Time and Math]] (easing curves live in bevy's math/curve types), [[19 Stress Tests and Testbeds]] (`stress_tests/many_foxes.rs`, `many_morph_targets.rs` benchmark this)

[[00 Bevy Capabilities Map|← Back to map]]
