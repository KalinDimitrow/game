---
tags: [bevy, transforms, camera]
---

# 03. Transforms & Camera

Before anything can be meaningfully rendered, you need to place it in space and have something looking at it. `Transform` (local) and `GlobalTransform` (computed, hierarchy-aware) are the backbone; cameras decide what part of the world — and how — gets drawn.

## Representative examples

- `transforms/transform.rs` — translate/rotate/scale, all at once
- `transforms/translation.rs`, `transforms/scale.rs`, `transforms/3d_rotation.rs` — each axis in isolation
- `transforms/align.rs` — axis-alignment helper
- `camera/pan_orbit_camera_cad.rs` / `pan_orbit_camera_minimal.rs` — the classic "orbit a 3D scene" controller
- `camera/free_camera_controller.rs` — fly-cam style controller
- `camera/camera_orbit.rs` — pitch/yaw/roll orbiting
- `camera/first_person_view_model.rs` — separate FOV for world model vs. view model
- `camera/custom_projection.rs` — write your own projection math
- `camera/projection_zoom.rs` — zoom for both ortho and perspective
- `camera/2d_top_down_camera.rs`, `camera/2d_screen_shake.rs` — 2D-specific camera behavior
- `3d/split_screen.rs`, `3d/two_passes.rs` — multiple cameras rendering to one window
- `3d/orthographic.rs` — isometric/CAD-style views

## Used in `game`

`character_controller/mod.rs::follow_character` is the current camera controller — a hand-rolled, simplified cousin of `camera/pan_orbit_camera_minimal.rs`, orbiting around `Character`'s position using `ActionState<CameraMovement>` from `leafwing-input-manager`. (An earlier `src/systems/input.rs::pan_camera`, orbiting a fixed `Vec3::ZERO`, is gone — this note was stale.)

### Gotcha: `GlobalTransform` doesn't reliably propagate to a non-physics child of a `RigidBody`

Hit this giving the player character a visual mesh as a `ChildOf` child of its (physics-driven) capsule, per [[06 Assets and Scenes]]'s RON-scene experiment — though it's not RON-specific, it'd bite the same way hand-coded. The capsule's own `GlobalTransform` stays correct every frame (avian3d syncs it directly from the physics solver), but the *child* mesh's `GlobalTransform` never got recomputed by Bevy's standard hierarchy propagation — confirmed stuck at its initial value for 15+ seconds, even after forcibly re-marking the child's `Transform` as `Changed` every frame. A sibling entity (the camera, parented only to a static root) propagated fine the whole time, isolating the problem to children of a *moving rigid body* specifically.

`avian3d`'s own docs for `PhysicsTransformPlugin` say as much directly: *"the engine treats nested rigid bodies as a flat structure... moving the parent will not affect the child... write your own system to handle hierarchies differently."* Fix: don't rely on `ChildOf`/propagation for anything visual attached to a `RigidBody` — make it a standalone entity and write a plain system that copies the parent's `Transform` across every frame (see `character_controller::sync_character_visual_transform`).

## Where this fits

- ⬅ Comes from: [[02 ECS Core]]
- ➡ Leads to: [[04 2D Rendering]] and [[05 3D Rendering]] — now that things can be placed and viewed, render them
- 🔗 Related: [[09 Input and Picking]] (camera controllers are driven by input), [[10 Animation]] (animated transforms)

[[00 Bevy Capabilities Map|← Back to map]]
