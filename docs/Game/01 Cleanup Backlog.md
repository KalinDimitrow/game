---
tags: [game, backlog]
---

# Cleanup Backlog

Small, mechanical tasks. None of these require deciding what the game actually is — good picks for a low-focus session, one at a time. See [[00 Roadmap Overview]] for the file references.

- [ ] **Migrate `keyboard_input_system` off raw `ButtonInput<KeyCode>`.** It directly reads A/D to rotate the camera (`systems/core/input.rs:39-84`), which predates the action-based convention already established by `CameraMovement`/`PlayerAction`. Add a buttonlike `CameraMovement` variant (or reuse `Pan`) bound in `assets/button_config.ron`, move the rotation logic into `pan_camera` or a sibling system, then delete `keyboard_input_system` and the now-unused `keyboard_event_system`.
- [x] ~~Decide the fate of the tutorial leftovers.~~ Deleted 2026-10-14 — `Person`/`Name`/`GreetTimer` and `add_people`/`greet_people`/`update_people` are gone from the repo.
- [x] ~~Resolve the duplicate scene definition.~~ Deleted 2026-10-14 — the dead `scene()`/`bsn_list!` function and all commented-out scaffolding in `setup()` are gone; `systems/core/scene.rs` now has one source of truth (`DynamicWorldRoot` loaded from `assets/load_scene_example.scn.ron`).
- [ ] **Give `PlayerAction::Pause` somewhere to go.** Right now it's only logged (`systems/core/input.rs:17-19`). This naturally gets wired up in [[08 Phase 5 - UI and Feedback]], once there's a pause state/menu to transition to — low priority until then.
- [ ] **Resolve the two-camera stopgap.** Added 2026-10-14: `core::scene::setup()` still spawns a `Camera3d` with a leafwing `InputMap` (now `Camera.is_active = false`), purely so `pan_camera`'s `Single<...>` query doesn't panic now that `terrain::spawn_terrain`'s `FreeCamera` is the one actually rendering. Pick one camera system — either drop the orbit camera and `pan_camera` entirely, or make `pan_camera` tolerate zero/many cameras (`Option<Single<...>>` or a plain `Query`) and decide whether `FreeCamera` or the leafwing-driven orbit cam is the real one going forward.
