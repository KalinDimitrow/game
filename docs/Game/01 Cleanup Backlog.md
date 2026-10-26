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

Added 2026-10-24, from splitting `utils::system_config` into its own directory (`button_config.rs` + `graphics_config.rs`, renamed from `display_config`) and starting the same treatment for levels:

- [ ] **Finish migrating `terrain::level_config` into `utils::level_map`.** `utils::level_map::LevelMap` is a from-scratch rebuild of `terrain::level_config::LevelConfig`'s parse/validate logic, one field at a time (currently just `name`+`grid`), developed against `data/levels/example_level.ron` (see [[Level Format]]) rather than inline RON strings. `terrain::level_config` keeps driving `spawn_terrain` untouched until `LevelMap` covers the tile registry, overrides, and points of interest too — then swap it in and delete the old module instead of running both.

Added 2026-10-25, from a long session pushing `DynamicWorldRoot`/`.scn.ron` as far as it goes — see [[06 Assets and Scenes]] for the technical findings (path-referenced meshes, `ColliderConstructor` standing in for `Collider`, no material/`InputMap` authoring) and [[03 Transforms and Camera]] + [[10 Animation]] for two Bevy gotchas (`GlobalTransform` not propagating to a child of a `RigidBody`, `Added<AnimationPlayer>` missing its one-frame window) hit along the way:

- [ ] **Decide whether `test_world.scn.ron` replaces `spawn_terrain`/`spawn_character`, or gets deleted.** Both are currently disabled (commented out of their `Plugin::build`, not removed) in favor of `TerrainPlugin::spawn_test_world` loading `data/assets/worlds/test_world.scn.ron`, which now describes the terrain tiles, their physics, a light, the player character, and the camera as `DynamicWorld` RON instead of Rust. It works — mesh, physics, input, and animation all end up attached — but getting there added *new* permanent Rust (`attach_character_input`, `sync_character_visual_transform`, `attach_character_animation_graph` in `character_controller/mod.rs`) rather than removing any; nothing is actually more data-driven than `LevelConfig` already made it. The honest value case is many near-identical, data-varying instances referencing a shared prefab file (enemies, props) — not the one-off player/terrain this ended up testing. Either commit to that direction and build it out, or re-enable `spawn_terrain`/`spawn_character`, delete `test_world.scn.ron`/`spawn_test_world`, and keep the three systems above only if something still needs them.

Added 2026-10-26, from fixing `attach_character_input`'s every-frame polling (moved to observers) and noticing `spawn_character_visual_when_ready`/`attach_character_animation_graph` have the same "runs every `Update` frame, early-returns until some async thing is ready" shape:

- [ ] **DEFERRED to the second wave of refactoring - add a loading state and move asset-readiness systems off `Update` polling.** `spawn_character_visual_when_ready` (waits on 4 glTF handles) and `attach_character_animation_graph` (waits on an `AnimationPlayer` plus `CharacterAnimations`) both run every frame and mostly no-op until their dependencies exist - functionally fine, but it's the same shape `attach_character_input` had, just not fixable with an observer here (there's no single discrete event to react to; it's "wait for N async things," not "wait for one component to appear"). The fix is a `GameState::Loading`/`GameState::Playing` split (see [[15 State Management]]):
  - Both systems stay `run_if(in_state(Loading))` as-is - including the animation-graph attach, since there's no reason it has to happen *during* play; it's just one more thing to finish before leaving `Loading`, same as the mesh/physics. Neither system should ever run once `Playing` starts.
  - The transition out of `Loading` only fires once everything is ready: glTF handles loaded, `Character` exists, and the visual's `AnimationPlayer` + `AnimationGraphHandle` are both attached.
  - The readiness check itself can be genuinely event-driven instead of re-polling `is_loaded_with_dependencies()` every frame - `bevy_asset::AssetEvent::LoadedWithDependencies { id }` fires exactly once per asset when it (and its dependencies) finish loading; accumulate which of the 4 handles have reported in via `MessageReader<AssetEvent<Gltf>>` instead.
  - Also the natural hook point for an actual loading screen (something to show *during* `Loading`), rather than the current silent gap before the character/camera appear.
