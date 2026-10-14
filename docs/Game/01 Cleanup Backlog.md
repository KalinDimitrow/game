---
tags: [game, backlog]
---

# Cleanup Backlog

Small, mechanical tasks. None of these require deciding what the game actually is — good picks for a low-focus session, one at a time. See [[00 Roadmap Overview]] for the file references.

- [ ] **Migrate `keyboard_input_system` off raw `ButtonInput<KeyCode>`.** It directly reads A/D to rotate the camera (`systems/input.rs:39-84`), which predates the action-based convention already established by `CameraMovement`/`PlayerAction`. Add a buttonlike `CameraMovement` variant (or reuse `Pan`) bound in `assets/button_config.ron`, move the rotation logic into `pan_camera` or a sibling system, then delete `keyboard_input_system` and the now-unused `keyboard_event_system`.
- [ ] **Decide the fate of the tutorial leftovers.** `Person`/`Name`/`GreetTimer` (`components/mod.rs`) and `add_people`/`greet_people`/`update_people` (`systems/people.rs`) are fully disconnected from `main.rs` (commented out at `main.rs:19-22`) but still compile and have a test. Either delete them outright, or keep deliberately as a documented ECS-basics reference with a comment saying so.
- [ ] **Resolve the duplicate scene definition.** `scene()` in `systems/scene.rs:8-34` (`bsn_list!`) is dead code — `setup()` instead spawns a `DynamicWorldRoot` from `assets/load_scene_example.scn.ron`. Pick one source of truth for scene content and delete the other.
- [ ] **Give `PlayerAction::Pause` somewhere to go.** Right now it's only logged (`systems/input.rs:17-19`). This naturally gets wired up in [[08 Phase 5 - UI and Feedback]], once there's a pause state/menu to transition to — low priority until then.
