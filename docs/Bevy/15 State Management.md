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

## Where this fits

- ⬅ Comes from: [[14 Gizmos and Debug Drawing]]
- ➡ Leads to: [[16 Async Tasks and Remote Protocol]]
- 🔗 Related: [[11 UI]] (menu/pause screens are the most common reason to reach for states), [[12 Audio]] (`audio/soundtrack.rs` switches music by state), [[02 ECS Core]] (states are resources + schedules under the hood)

[[00 Bevy Capabilities Map|← Back to map]]
