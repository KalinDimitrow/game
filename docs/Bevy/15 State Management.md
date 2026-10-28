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

`components::GameState` — **three** states, not the usual two, initialized in `main.rs` via `init_state::<GameState>()`:

- **`Loading`**: `character_controller::check_character_assets_loaded` waits on the 4 `CharacterGltfHandles` glTF handles via `resources::AssetBarrier` — the `asset/multi_asset_sync.rs` pattern (see [[06 Assets and Scenes]] for the mechanics and the one ordering caveat). One atomic read per frame; calls `next_state.set(Initialize)` once every guarded load has finished **or failed**.
- **`Initialize`**: `build_character_animation_graph` runs in `OnEnter(Initialize)` — a genuine `state/custom_transitions.rs`-style one-shot, with **no readiness guard at all**, because entering this state already proves the assets are loaded. `attach_character_animation_graph` and `check_character_ready` then run every `Update` frame `.run_if(in_state(Initialize))`, waiting on a *different kind* of fact - not "has an asset loaded" but "has an entity been spawned with this component" (`Character` existing, the visual's `AnimationPlayer` appearing) - which has no event to subscribe to, only a query checked repeatedly until it's observed. `check_character_ready` calls `next_state.set(Playing)` once both are true.
- **`Playing`**: `move_character`/`apply_character_animation`/`sync_character_visual_transform`/`orient_character_visual`/`follow_character`, all gated `.run_if(in_state(Playing))`.

The `Loading`/`Initialize` split exists specifically because those two kinds of "waiting" are different: one has a completion *event*, the other only has entity state to *observe*, and only the former can be turned into a zero-guard one-shot by moving it into `OnEnter`. Confirmed via a throwaway log that both transitions fire exactly once, in order. No loading-screen UI yet (`ecs/state_scoped.rs` would be the tool for despawning one on exit) - the state machine is real, nothing renders for `Loading`/`Initialize` yet.

### Dead end, worth remembering: don't `block_on` an asset load inside `Startup`

Tried collapsing `Loading` entirely by having a `Startup` system call `bevy::tasks::block_on(asset_server.wait_for_asset(&handle))` directly - `wait_for_asset` is real, public, documented API, and `bevy_asset`'s own source explicitly says blocking on a deferred load this way is the *intended* alternative to a (deliberately unprovided) `load_async`. It deadlocks every time regardless, for a structural reason, not a bug: the system that actually finalizes a background-loaded asset into `Assets<T>` and flips its load state (`handle_internal_asset_events`) runs in `PreUpdate`, and `PreUpdate` cannot start until `Startup` returns. Blocking inside `Startup` waits on something that, by construction, can only happen *after* `Startup` finishes - confirmed hung for 45+ seconds (not just slow) before reverting. The engine's own `asset/multi_asset_sync.rs` example avoids this correctly: its async variant `.await`s the same kind of future from inside a task spawned on `AsyncComputeTaskPool` - a genuinely separate thread, not the one running the schedule - then checks a plain `AtomicBool` from a normal `Update` system. `block_on` is fine from a background task; it's not fine from inside the schedule itself.

## Where this fits

- ⬅ Comes from: [[14 Gizmos and Debug Drawing]]
- ➡ Leads to: [[16 Async Tasks and Remote Protocol]]
- 🔗 Related: [[11 UI]] (menu/pause screens are the most common reason to reach for states), [[12 Audio]] (`audio/soundtrack.rs` switches music by state), [[02 ECS Core]] (states are resources + schedules under the hood)

[[00 Bevy Capabilities Map|← Back to map]]
