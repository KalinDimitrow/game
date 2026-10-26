---
tags: [bevy, state]
---

# 15. State Management

Once there's a menu, a gameplay screen, and a pause overlay, you need a formal way to say "we are in state X" and run systems only in the states where they make sense. Built on top of ECS resources and schedules — no separate state machine engine.

## Representative examples

- `state/states.rs` — the baseline: Menu → InGame transition
- `state/sub_states.rs` — hierarchical states (e.g. Paused as a sub-state of InGame)
- `state/computed_states.rs` — derive a state from other state/data instead of setting it directly
- `state/custom_transitions.rs` — hook custom logic into state-transition schedules
- `ecs/state_scoped.rs` — auto-despawn entities when entering/exiting a state (lives in `ecs/` but is squarely a state-management tool)

## Used in `game`

Not yet - currently planned, not implemented (see `docs/Game/01 Cleanup Backlog.md`, 2026-10-26 entry). `character_controller`'s asset-loading systems (`spawn_character_visual_when_ready`, `attach_character_animation_graph`) currently run every `Update` frame and early-return until their async dependencies (glTF handles, an `AnimationPlayer`) are ready — functionally fine, but the idiomatic fix is `state/states.rs`'s pattern: a `GameState::Loading` → `GameState::Playing` transition, where one cheap system does the readiness *check* each frame (`run_if(in_state(Loading))`) and the actual one-time setup moves to `OnEnter(Playing)` instead of living inside an early-return. Also the natural hook point for an actual loading screen (`ecs/state_scoped.rs` for despawning it on exit).

## Where this fits

- ⬅ Comes from: [[14 Gizmos and Debug Drawing]]
- ➡ Leads to: [[16 Async Tasks and Remote Protocol]]
- 🔗 Related: [[11 UI]] (menu/pause screens are the most common reason to reach for states), [[12 Audio]] (`audio/soundtrack.rs` switches music by state), [[02 ECS Core]] (states are resources + schedules under the hood)

[[00 Bevy Capabilities Map|← Back to map]]
