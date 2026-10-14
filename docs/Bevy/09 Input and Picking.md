---
tags: [bevy, input, picking]
---

# 09. Input & Picking

The scene exists and is populated — now make it respond to the player. "Input" is raw device events (keyboard/mouse/gamepad/touch); "picking" is the higher-level "what's under the cursor" layer built on top of it.

## Representative examples — Input

- `input/keyboard_input.rs`, `input/keyboard_input_events.rs` — press/release vs. the full event stream
- `input/keyboard_modifiers.rs` — ctrl/shift-aware input
- `input/mouse_input.rs`, `input/mouse_input_events.rs`, `input/mouse_grab.rs` — mouse buttons, movement, cursor locking
- `input/gamepad_input.rs`, `input/gamepad_rumble.rs` — gamepad support, including force feedback
- `input/touch_input.rs` — touch presses/releases/cancels

## Representative examples — Picking

- `picking/simple_picking.rs` — the minimal "click to spawn something" loop
- `picking/mesh_picking.rs`, `picking/sprite_picking.rs` — pick 3D meshes or 2D sprites
- `picking/dragdrop_picking.rs`, `picking/draggable_slider.rs` — drag interactions via picking
- `picking/debug_picking.rs` — visualize what picking is doing

## Used in `game`

`src/systems/input.rs` has `keyboard_input_system` (raw `ButtonInput<KeyCode>`/`ButtonInput<Key>`, matching `input/keyboard_input.rs`) and `pan_camera` (reads `ActionState<CameraMovement>` from the `leafwing-input-manager` crate — a layer *above* Bevy's own input examples, since Bevy itself has no built-in action-mapping abstraction; leafwing-input-manager fills that specific gap).

## Where this fits

- ⬅ Comes from: [[07 glTF and 3D Models]], [[08 Reflection]]
- ➡ Leads to: [[10 Animation]] — input often drives what gets animated
- 🔗 Related: [[03 Transforms and Camera]] (camera controllers consume input), [[11 UI]] (UI has its own input/navigation layer)

[[00 Bevy Capabilities Map|← Back to map]]
