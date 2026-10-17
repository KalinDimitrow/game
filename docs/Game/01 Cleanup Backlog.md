---
tags: [game, backlog]
---

# Cleanup Backlog

Small, mechanical tasks. None of these require deciding what the game actually is — good picks for a low-focus session, one at a time. See [[00 Roadmap Overview]] for the file references.

- [x] ~~Decide the fate of the tutorial leftovers.~~ Deleted 2026-10-14 — `Person`/`Name`/`GreetTimer` and `add_people`/`greet_people`/`update_people` are gone from the repo.
- [x] ~~Resolve the duplicate scene definition.~~ Deleted 2026-10-14 — the dead `scene()`/`bsn_list!` function and all commented-out scaffolding in `setup()` are gone; `systems/core/scene.rs` now has one source of truth (`DynamicWorldRoot` loaded from `assets/load_scene_example.scn.ron`).
- [x] ~~Migrate `keyboard_input_system` off raw `ButtonInput<KeyCode>`.~~ Deleted 2026-10-15 — `keyboard_input_system`, `keyboard_event_system`, and the old orbit-around-origin `pan_camera` are all gone (superseded by `systems/character_controller`'s third-person camera, which reuses `CameraMovement::Pan` instead).
- [x] ~~Resolve the two-camera stopgap.~~ Also resolved 2026-10-15 as a side effect of the above — `core::scene.rs`'s inactive leftover camera is gone entirely; the character controller's third-person camera is now the only one.
- [ ] **Give `PlayerAction::Pause` somewhere to go.** Right now it's only logged (`systems/core/input.rs`). This naturally gets wired up in [[08 Phase 5 - UI and Feedback]], once there's a pause state/menu to transition to — low priority until then.

Added 2026-10-15, from building the collision/animation testbed in `systems/character_controller` and `systems/terrain` — the user's noted most of this will get refactored anyway, so treat these as a map of what's known-rough rather than an urgent queue:

- [ ] **Grounded check is a velocity heuristic, not a real one.** `move_character` treats `|velocity.y| < 0.05` as "on the ground" — no actual raycast/contact check. Works for a flat floor, will misbehave on slopes or under any downward force that isn't gravity.
- [ ] **Jump has no real animation.** No jump clip exists in any locally available asset, so jumping just reuses the `Run` clip as a stand-in (`systems/character_controller/mod.rs`). Revisit if a better-suited character asset ever shows up.
- [ ] **Fox is a placeholder, not a humanoid.** It's the only rigged+animated character asset available in the local Bevy examples checkout; a running fox is much longer nose-to-tail than the capsule collider is wide, so the visual and the collider don't really match shape-wise. `FOX_SCALE`/offset are tuned to match the capsule's *height* only.
- [ ] **One flat collider for the whole terrain grid.** `terrain::spawn_terrain` spans the entire 28×16 grid with a single `Collider::cuboid` rather than per-tile colliders. Fine while every tile is coplanar — revisit per [[03 Phase 0 - Foundation]] once terrain has real height variation.

Added 2026-10-17, from building `button_config.rs` and now `display_config.rs` as two independent config loaders:

- [ ] **Unify config loading.** `button_config` (a Bevy Startup system), `display_config` (a plain function called from `main()`, read before the `App` exists), and now `terrain/level_config.rs` (a plain function called from the `spawn_terrain` Startup system) each hand-roll the same read-file → parse-RON → wrap-errors-with-path shape, independently, with their own private parsed-file struct. The third one has now shown up — worth pulling this into one non-Bevy loader that reads every config file up front and hands back a single struct/map, instead of every new config getting its own bespoke loader.
