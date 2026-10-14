---
tags: [bevy, reflection]
---

# 08. Reflection

Rust has no built-in runtime type introspection; Bevy's `Reflect` trait adds it. This is what makes scene serialization, the remote protocol, and editor-style tooling possible without every type author hand-writing serialization code.

## Representative examples

- `reflection/reflection.rs` — the baseline: dynamically read/write a Rust value's fields
- `reflection/reflection_types.rs` — the different flavors of reflectable types
- `reflection/dynamic_types.rs` — `DynamicStruct`/`DynamicEnum` and friends
- `reflection/mutation_by_reflection.rs` — mutate data you don't have a concrete type for
- `reflection/serialization.rs` — serialize/deserialize via reflection, no serde derive needed on the type
- `reflection/function_reflection.rs` — call functions dynamically
- `reflection/generic_reflection.rs` — registering generic types for reflection

## Used in `game`

`src/main.rs` calls `app.register_type::<MeshMaterial3d<StandardMaterial>>()`, `register_type::<SpawnFloor>()`, `register_type::<SpawnCube>()` — this registration step is exactly what `reflection/reflection_types.rs` explains, and it's a prerequisite for the scene-loading in [[06 Assets and Scenes]] to be able to deserialize those component types from the `.scn.ron` file.

## Where this fits

- ⬅ Comes from: [[06 Assets and Scenes]]
- ➡ Leads to: [[09 Input and Picking]]
- 🔗 Related: [[16 Async Tasks and Remote Protocol]] (BRP — Bevy Remote Protocol — is reflection exposed over a network connection), [[02 ECS Core]] (component registration)

[[00 Bevy Capabilities Map|← Back to map]]
