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

`src/systems/input.rs::pan_camera` rotates the camera around `Vec3::ZERO` using `ActionState<CameraMovement>` from `leafwing-input-manager` — a hand-rolled, simplified cousin of `camera/pan_orbit_camera_minimal.rs`.

## Where this fits

- ⬅ Comes from: [[02 ECS Core]]
- ➡ Leads to: [[04 2D Rendering]] and [[05 3D Rendering]] — now that things can be placed and viewed, render them
- 🔗 Related: [[09 Input and Picking]] (camera controllers are driven by input), [[10 Animation]] (animated transforms)

[[00 Bevy Capabilities Map|← Back to map]]
