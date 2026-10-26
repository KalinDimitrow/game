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
- [x] ~~One flat collider for the whole terrain grid.~~ Moot since 2026-10-26 — `terrain::spawn_terrain` (which had this single-cuboid-for-the-whole-grid behavior) is deleted. The current `test_world.scn.ron` tiles each carry their own `ColliderConstructor`, scaled by the tile's own `Transform.scale` — per-tile by construction, not something to revisit later.

Added 2026-10-17, from building `button_config.rs` and now `display_config.rs` as two independent config loaders:

- [ ] **Unify config loading.** `button_config` and `display_config` (now `graphics_config`, since folded into `utils::system_config` on 2026-10-24) each hand-roll the same read-file → parse-RON → wrap-errors-with-path shape, independently, with their own private parsed-file struct — worth pulling into one non-Bevy loader that reads every config file up front and hands back a single struct/map. (A third example, `terrain/level_config.rs`, used to motivate this too; it's deleted as of 2026-10-26 along with `spawn_terrain`/`spawn_character`, so this is back to just the two system configs — lower urgency than when a third was actively showing up, but still a real duplication.)

Added 2026-10-25, from a long session pushing `DynamicWorldRoot`/`.scn.ron` as far as it goes — see [[06 Assets and Scenes]] for the technical findings (path-referenced meshes, `ColliderConstructor` standing in for `Collider`, no material/`InputMap` authoring) and [[03 Transforms and Camera]] + [[10 Animation]] for two Bevy gotchas (`GlobalTransform` not propagating to a child of a `RigidBody`, `Added<AnimationPlayer>` missing its one-frame window) hit along the way:

- [x] ~~Decide whether `test_world.scn.ron` replaces `spawn_terrain`/`spawn_character`, or gets deleted.~~ Decided 2026-10-26 — committed to the RON direction. `spawn_terrain`, `spawn_character`, `terrain/level_config.rs`, the `LevelConfig`/`CellOverride`/`PointOfInterest`/`PointOfInterestKind` types, `LEVEL_PATH`, and `data/levels/level_01.ron` are all deleted. `TerrainPlugin::spawn_test_world` loading `data/assets/worlds/test_world.scn.ron` is now the only way terrain/character/camera get into the world. `utils::level_map` (added 2026-10-24) was being built specifically to replace `terrain::level_config` for `spawn_terrain`'s benefit; with `spawn_terrain` gone, its target consumer no longer existed either, so it (and `docs/Game/Level Format.md`, and its own backlog entry) were deleted along with everything else here rather than left to describe a plan that no longer applies.

Added 2026-10-26, from fixing `attach_character_input`'s every-frame polling (moved to observers) and noticing `spawn_character_visual_when_ready`/`attach_character_animation_graph` have the same "runs every `Update` frame, early-returns until some async thing is ready" shape:

- [ ] **DEFERRED to the second wave of refactoring - add a loading state and move asset-readiness systems off `Update` polling.** `spawn_character_visual_when_ready` (waits on 4 glTF handles) and `attach_character_animation_graph` (waits on an `AnimationPlayer` plus `CharacterAnimations`) both run every frame and mostly no-op until their dependencies exist - functionally fine, but it's the same shape `attach_character_input` had, just not fixable with an observer here (there's no single discrete event to react to; it's "wait for N async things," not "wait for one component to appear"). The fix is a `GameState::Loading`/`GameState::Playing` split (see [[15 State Management]]):
  - Both systems stay `run_if(in_state(Loading))` as-is - including the animation-graph attach, since there's no reason it has to happen *during* play; it's just one more thing to finish before leaving `Loading`, same as the mesh/physics. Neither system should ever run once `Playing` starts.
  - The transition out of `Loading` only fires once everything is ready: glTF handles loaded, `Character` exists, and the visual's `AnimationPlayer` + `AnimationGraphHandle` are both attached.
  - The readiness check itself can be genuinely event-driven instead of re-polling `is_loaded_with_dependencies()` every frame - `bevy_asset::AssetEvent::LoadedWithDependencies { id }` fires exactly once per asset when it (and its dependencies) finish loading; accumulate which of the 4 handles have reported in via `MessageReader<AssetEvent<Gltf>>` instead.
  - Also the natural hook point for an actual loading screen (something to show *during* `Loading`), rather than the current silent gap before the character/camera appear.
