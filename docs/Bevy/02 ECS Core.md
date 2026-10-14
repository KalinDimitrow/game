---
tags: [bevy, ecs]
---

# 02. ECS Core

Bevy's Entity Component System is the vocabulary everything downstream is written in: entities, components, systems, queries, resources, events, schedules. Nothing in rendering, UI, or audio makes sense without this.

## Representative examples

- `ecs/ecs_guide.rs` — the canonical "start here" tour of the whole ECS
- `ecs/startup_system.rs` — run-once-at-boot systems
- `ecs/hierarchy.rs` — parent/child entity relationships
- `ecs/relationships.rs` — define custom relationships beyond parent/child
- `ecs/change_detection.rs` — react only when a component/resource actually changed
- `ecs/observers.rs` / `ecs/observer_propagation.rs` — reactive, event-driven systems
- `ecs/message.rs` — typed events between systems
- `ecs/one_shot_systems.rs` — run a system on demand instead of scheduling it
- `ecs/system_piping.rs` — chain systems, propagate errors
- `ecs/run_conditions.rs` — gate systems on conditions
- `ecs/fixed_timestep.rs` — decouple simulation rate from frame rate
- `ecs/error_handling.rs` — return/handle `Result` from systems
- `ecs/state_scoped.rs` — auto-despawn entities on state transitions (bridges to [[15 State Management]])

## Used in `game`

`src/systems/mod.rs` has `add_people`/`greet_people`/`update_people` as a near-literal rerun of the "Hello World ECS" pattern from the Bevy book, plus a `RunSystemOnce` unit test — the project's only ECS example so far is this one.

## Where this fits

- ⬅ Comes from: [[01 Foundations - App and Window]]
- ➡ Leads to: [[03 Transforms and Camera]] — the first "real" components you'll spawn are spatial ones
- 🔗 Related: [[08 Reflection]] (reflection is what lets ECS data be inspected/serialized generically), [[15 State Management]] (states are implemented on top of ECS resources/schedules)

[[00 Bevy Capabilities Map|← Back to map]]
