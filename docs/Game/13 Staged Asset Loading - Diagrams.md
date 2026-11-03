---
tags: [game, design, assets, state, diagrams]
---

# 13 — Staged Asset Loading: Diagrams

Structural and behavioural diagrams for the draft in `src/systems/asset_loading/`,
which implements the machine track (steps 1–4) of [[11 Staged Asset Loading]].
Drawn 2026-11-02 against the code as written, not against the design doc — where
the two differ (`poll_gate` being its own system, `despawn_on_replay` instead of
`DespawnOnExit`), these follow the code and say so.

Read in order: the mapping note first, then §1–§3 for *what the pieces are*, §4
for *what a frame does*. The sequence diagrams are the ones that actually explain
the design; the rest is vocabulary for reading them.

Generic types are written Rust-style in prose (`Handle<Gltf>`) and bracket-free
inside the diagrams (`Handle of Gltf`), because Mermaid labels and angle brackets
don't mix across renderers.

## How to read a "class diagram" of an ECS

There are no classes here, and forcing the UML vocabulary onto ECS is most of why
this is hard to picture. The honest mapping:

| UML idea | What it is in this module |
|---|---|
| Class with fields | A `Resource` struct (`StageGate`, `RewindRequest`) — one global instance, not one per caller |
| Class with methods | Only `StageGate`/`RewindRequest`/`LoadStage` have real `impl` blocks, and they hold *no* behaviour beyond accessors |
| Method call | Almost never. The behaviour lives in **systems** — free functions the scheduler calls once per frame with their parameters injected |
| Object (instance) | A resource value, or an entity plus its components |
| Association, "has a" | A system *parameter*. `poll_gate(ResMut<StageGate>, Res<AssetServer>, …)` is the only thing connecting those two types |
| Interface | A `SystemSet` (`LoadSet`) — a named slot in the frame that other plugins order against |
| Component (UML, deployable) | A Bevy `Plugin` |
| Static call graph | Doesn't exist. Order comes from `configure_sets(..).chain()`, not from who calls whom |

So the useful reading is: **four data types, one ordering contract, six systems
plus one run condition, and a frame that runs them in a fixed order.** Nothing in
this module calls anything else in this module.

## 1. Class diagram — the types

```mermaid
classDiagram
    direction TB

    class LoadStage {
        <<enumeration, States>>
        Boot
        Index
        Declarations
        Resolve
        Payload
        Derive
        World
        Instantiate
        Warm
        Settled
        +ORDER$
        +position() usize
        +next() LoadStage
        +earliest(other) LoadStage
    }

    class StageGate {
        <<Resource>>
        -handles: PendingAssets
        -blockers: HashSet~str~
        -assets_ready: bool
        +track(handle)
        +block(reason)
        +release(reason)
        +is_ready() bool
        +waiting_on() Vec~str~
    }

    class PendingAssets {
        <<Resource>>
        -handles: Vec~UntypedHandle~
        +track(handle)
        +progress(asset_server) AssetLoadProgress
    }

    class AssetLoadProgress {
        <<enumeration>>
        Loading
        Ready
        Failed
    }

    class RewindRequest {
        <<Resource>>
        -target: Option~LoadStage~
        +to(stage)
        +requested() Option~LoadStage~
    }

    class LoadSet {
        <<enumeration, SystemSet>>
        Poll
        Watch
        Settle
        Advance
    }

    class AssetLoadingPlugin {
        <<Plugin>>
        +build(app)
    }

    class DriverSystems {
        <<free functions>>
        poll_gate() Result
        advance_stage()
        apply_rewind()
        clear_gate()
    }

    class ExtensionPoints {
        <<free functions, for domain plugins>>
        settling(stage) RunCondition
        rewind_on_change(stage) System
        despawn_on_replay() System
    }

    StageGate *-- PendingAssets : delegates the fold to
    PendingAssets ..> AssetLoadProgress : returns
    RewindRequest o-- LoadStage : holds target
    LoadStage ..> LoadStage : next() and earliest()

    AssetLoadingPlugin ..> LoadStage : init_state
    AssetLoadingPlugin ..> StageGate : init_resource
    AssetLoadingPlugin ..> RewindRequest : init_resource
    AssetLoadingPlugin ..> LoadSet : configure_sets, chained
    AssetLoadingPlugin ..> DriverSystems : add_systems

    DriverSystems ..> StageGate : reads and writes
    DriverSystems ..> RewindRequest : consumes
    ExtensionPoints ..> StageGate : reads
    ExtensionPoints ..> RewindRequest : writes
    ExtensionPoints ..> LoadSet : placed in
```

`ORDER` is a `&'static [LoadStage]` const — the single source of truth for both
"what comes next" and "is this stage earlier than that one", which is why
`next()` and `earliest()` read it rather than restating the sequence as a
`match`.

The two generic extension points lose their parameters in the box above, so in
full: `rewind_on_change::<A: Asset>(stage)` returns a system watching one asset
type, and `despawn_on_replay::<M: Component>` *is* a system, registered per
marker component rather than called.

Three things the diagram is trying to make visible:

- **`StageGate` is one resource, not one per stage.** `clear_gate` on `OnExit`
  resets it, so it only ever describes the stage *being entered*. That single
  fact is what makes replaying a stage clean, and it's why there's no
  stage-to-gate multiplicity anywhere.
- **`assets_ready` is a cache, not a source of truth.** It's recomputed once per
  frame by `poll_gate`, which exists so `settling(..)` can be a cheap
  `Fn(Res<State<LoadStage>>, Res<StageGate>) -> bool` instead of re-folding load
  state once per settle system per frame.
- **`StageGate` delegating to `PendingAssets` is draft scaffolding.** Migration
  step 1 moves `PendingAssets`'s body into `StageGate` and deletes the type. The
  composition arrow is temporary; the `handles` + `blockers` split is not.

### The state machine the enum encodes

```mermaid
stateDiagram-v2
    direction LR
    [*] --> Boot
    Boot --> Index
    Index --> Declarations
    Declarations --> Resolve
    Resolve --> Payload
    Payload --> Derive
    Derive --> World
    World --> Instantiate
    Instantiate --> Warm
    Warm --> Settled
    Settled --> Settled : terminal, advance_stage returns early

    Settled --> Derive : apply_rewind, a Gltf changed
    Settled --> Boot : apply_rewind, button_config changed
    Instantiate --> Index : apply_rewind, assets.dsc.ron changed
```

Forward edges are `advance_stage` (gate green, no blockers). Backward edges are
`apply_rewind`, and they re-walk the *same* edges a cold load walks — that's the
whole point: hot reload has no code of its own. `Settled` is terminal for forward
progress only, which is what lets `GameState::Playing` be entered once and never
left by a reload.

## 2. Object diagram — two snapshots

UML object diagrams show instances with real values. Both snapshots are the world
state during one `Update`, i.e. what `advance_stage` saw.

### Snapshot A — cold load, held at `Instantiate`

```mermaid
flowchart TB
    subgraph W["World, frame N"]
        S["state: State of LoadStage<br/>= Instantiate"]
        G["gate: StageGate<br/>handles = one Handle of Scene, test_world.scn.ron<br/>blockers = {character-animation-player}<br/>assets_ready = true"]
        R["rewind: RewindRequest<br/>= None"]
        N["next: NextState of LoadStage<br/>= Unchanged"]
        E1["entity 42<br/>Character, SceneRoot"]
        E2["entity 57<br/>AnimationPlayer<br/>spawned by the scene"]
        RES["CharacterAnimations<br/>resource, built in Derive"]
    end

    G -- "assets_ready = true, so settling(Instantiate) passes" --> SET["finish_character_initialization<br/>runs this frame"]
    E2 -.-> SET
    RES -.-> SET
    SET -- "found the AnimationPlayer, so gate.release(..)" --> G
    G -- "blockers now empty at Advance time, so next.set(Warm)" --> N
```

Read it as the answer to "what is the game stuck on?" — which here is a *value*
(`blockers`), printable on a loading screen via `waiting_on()`. That's the
argument for named blockers over a per-stage `ready` boolean: a boolean can't tell
you what it's waiting for, and the driver would have to learn that characters
exist in order to ask.

### Snapshot B — the frame a rewind is requested

```mermaid
flowchart TB
    subgraph W2["World, frame M, Idle.glb was just saved"]
        S2["state: State of LoadStage<br/>= Settled"]
        G2["gate: StageGate<br/>handles = empty<br/>blockers = empty<br/>assets_ready = true"]
        R2["rewind: RewindRequest<br/>= Some(Derive)"]
        N2["next: NextState of LoadStage<br/>= Pending(Derive)"]
    end

    EV["three AssetEvent of Gltf, Modified<br/>an editor fires several per save"] --> WATCH["rewind_on_change for Gltf, target Derive<br/>in LoadSet::Watch"]
    WATCH -- "request.to(Derive) three times,<br/>coalesced by earliest()" --> R2
    S2 --> ADV["advance_stage<br/>stage is Settled, returns early"]
    R2 --> APPLY["apply_rewind<br/>Derive is earlier than Settled,<br/>so next.set(Derive)"]
    APPLY --> N2
```

Note `handles = empty` at `Settled`: the gate was emptied by `clear_gate` on the
way out of every stage, so **the gate's strong handles only live as long as the
stage does**. Anything that must outlive its stage needs another owner — the world
scene survives because its handle is also on a `SceneRoot` component, and
`CharacterAnimations` survives because it's a resource. Worth knowing before
adding a stage that loads something nothing else holds onto.

## 3. Component diagram — who wires what

```mermaid
flowchart TB
    subgraph Driver["AssetLoadingPlugin: owns the machine, knows no content"]
        direction TB
        D1["init_state LoadStage<br/>init_resource StageGate and RewindRequest"]
        D2["configure_sets: Poll, Watch, Settle, Advance, chained"]
        D3["poll_gate, advance_stage, apply_rewind"]
        D4["clear_gate on OnExit of every stage in ORDER"]
    end

    subgraph Domain["Domain plugins: own the content, know nothing of each other"]
        direction TB
        P1["CharacterControllerPlugin<br/>OnEnter(Payload): load clips, gate.track<br/>Settle(Derive): build the animation graph<br/>Settle(Instantiate): block then release character-animation-player<br/>Watch: rewind_on_change for Gltf, target Derive"]
        P2["TerrainPlugin<br/>OnEnter(World): load test_world.scn.ron, gate.track<br/>OnEnter(Instantiate): despawn_on_replay then spawn"]
        P3["CorePlugin and UI<br/>OnEnter(Boot): button_config, gate.track<br/>Watch: rewind_on_change for ButtonConfig, target Boot"]
    end

    subgraph Bevy["Bevy, providing the primitives"]
        direction TB
        B1["StateTransition schedule<br/>after PreUpdate, before Update"]
        B2["AssetServer<br/>recursive_dependency_load_state"]
        B3["Assets of T, and AssetEvent of T"]
        B4["file_watcher<br/>queue_ancestors, transitive"]
    end

    Domain -- "add_systems to OnEnter, LoadSet::Settle, LoadSet::Watch" --> Driver
    Driver -- "exposes LoadStage, StageGate, LoadSet, settling,<br/>rewind_on_change, despawn_on_replay" --> Domain
    Driver --> B1
    Driver --> B2
    Domain --> B2
    B4 --> B3
    B3 --> Domain
```

The dependency arrow worth staring at is the one that **isn't** there: nothing
goes from `Driver` to a specific domain plugin. `advance_stage` never learns that
characters or terrain exist — it compares a cached bool and a set emptiness. All
coupling runs domain → driver, through four extension points:

1. `OnEnter(stage)` — name the loads, `gate.track(..)`, `gate.block(..)`.
2. `LoadSet::Settle` + `run_if(settling(stage))` — derive, and `gate.release(..)`.
3. `LoadSet::Watch` — `rewind_on_change::<A>(consuming_stage)`.
4. `OnEnter(stage)` + `despawn_on_replay::<M>` — make a spawning stage replay-safe.

`.in_set(LoadSet::Settle)` is not decoration. It is what puts a settle system
*ahead of* `advance_stage` in the same frame, so a blocker released this frame is
seen this frame rather than next.

## 4. Sequence diagrams

### 4a. The anatomy of one frame

Everything else depends on this ordering, and most of the design's non-obvious
properties are consequences of it.

```mermaid
sequenceDiagram
    autonumber
    participant Sched as Bevy scheduler
    participant AS as AssetServer
    participant ST as StateTransition
    participant Gate as StageGate
    participant Settle as settle systems
    participant Adv as advance_stage
    participant Rew as apply_rewind

    Sched->>AS: PreUpdate, handle_internal_asset_events
    Note over AS: load state flips to Loaded and the asset lands in<br/>Assets of T under one lock, so "loaded" and<br/>"gettable" are the same instant

    Sched->>ST: StateTransition, after PreUpdate and before Update
    ST->>Gate: OnExit of the previous stage, clear_gate resets to default
    ST->>Gate: OnEnter of the new stage, domain systems track handles and arm blockers
    Note over ST,Gate: an OnEnter can never observe a load before<br/>Assets of T has it, free from this ordering

    Sched->>Gate: Update, LoadSet::Poll, poll_gate
    Gate->>AS: recursive_dependency_load_state per tracked handle
    AS-->>Gate: Loading, Ready, or Failed
    Gate->>Gate: assets_ready = progress is Ready

    Sched->>Rew: LoadSet::Watch, rewind_on_change drains its AssetEvents
    Sched->>Settle: LoadSet::Settle, run_if settling(stage)
    Note over Settle: gated on assets_ready, so a derive step cannot<br/>observe a half-loaded stage and needs no<br/>readiness guard of its own
    Settle->>Gate: block(..) or release(..)

    Sched->>Adv: LoadSet::Advance, advance_stage
    Adv->>Gate: is_ready() and blockers empty?
    Adv-->>ST: next.set(stage.next())
    Sched->>Rew: LoadSet::Advance, apply_rewind, chained after
    Rew-->>ST: next.set(target), overwrites, so the rewind wins
    Note over ST: the transition lands at the start of frame N+1
```

Consequences to carry into the rest of the diagrams:

- **One stage per frame, offset by one frame.** A transition asked for in frame
  N's `Update` lands at the start of frame N+1, so ten empty stages take ten
  frames.
- **The frame that lands in a stage also runs one settle call** — the `OnEnter`
  and one full `Update` both happen in it. Tests counting settle calls start at
  one, not zero.
- **Rewind beats forward progress** on a frame where both want `NextState`,
  because `(advance_stage, apply_rewind).chain()` makes the rewind the last write.
  That's the conservative direction: over-rewinding is wasteful, under-rewinding
  leaves something stale.

### 4b. Cold load — `Payload` into `Derive`

```mermaid
sequenceDiagram
    autonumber
    participant CC as CharacterControllerPlugin systems
    participant ST as StateTransition
    participant Gate as StageGate
    participant AS as AssetServer
    participant Adv as advance_stage

    rect rgb(235,240,248)
    Note over ST,Adv: frame N, entering Payload
    ST->>Gate: OnExit(Resolve), clear_gate
    ST->>CC: OnEnter(Payload), load_character_gltf_handles
    CC->>AS: load the four character Gltf files
    AS-->>CC: four handles, loads in flight
    CC->>Gate: track(handle), four times
    Gate->>AS: poll_gate, recursive_dependency_load_state
    AS-->>Gate: Loading
    Gate->>Gate: assets_ready = false
    Note over CC: settling(Payload) is false, so no settle system runs
    Adv->>Gate: is_ready()? no, hold the stage
    end

    rect rgb(235,240,248)
    Note over ST,Adv: frames N+1 onward, the loads finish
    Gate->>AS: poll_gate
    AS-->>Gate: Ready, transitively, covering labeled sub-assets
    Gate->>Gate: assets_ready = true
    Adv->>Gate: is_ready() yes, blockers empty yes
    Adv->>ST: next.set(Derive)
    end

    rect rgb(232,245,236)
    Note over ST,Adv: the next frame, Derive
    ST->>Gate: OnExit(Payload), clear_gate, dropping the four strong handles
    ST->>CC: OnEnter(Derive), nothing registered
    Gate->>Gate: poll_gate over an empty list, so Ready
    CC->>CC: Settle, build_character_animation_graph
    Note over CC: reads Assets of Gltf with no readiness check,<br/>because settling(Derive) already proved<br/>the loads succeeded
    CC->>CC: insert_resource(CharacterAnimations)
    Adv->>ST: next.set(World)
    end
```

`Derive` loads nothing and still gets a stage, because *code must run between two
loads*: the `AnimationGraph` doesn't exist on disk. That is the stage-boundary
criterion from [[11 Staged Asset Loading]] §triage, and the reason the stage count
is a compile-time quantity — it's the number of derive/resolve steps you have
written in Rust.

### 4c. A blocker holding a stage open

```mermaid
sequenceDiagram
    autonumber
    participant ST as StateTransition
    participant CC as CharacterControllerPlugin
    participant Gate as StageGate
    participant Scene as scene spawn, by Bevy
    participant Adv as advance_stage
    participant UI as loading screen

    ST->>CC: OnEnter(Instantiate)
    CC->>Gate: block("character-animation-player")
    Note over CC,Gate: armed in OnEnter, never in Startup: a Startup-armed<br/>blocker holds only the first pass and waves<br/>every replay straight through

    loop every frame until the player exists
        Gate->>Gate: poll_gate, assets_ready = true
        CC->>Scene: Settle, query for an AnimationPlayer
        Scene-->>CC: not yet
        UI->>Gate: waiting_on() returns character-animation-player
        Adv->>Gate: is_ready() yes, but blockers non-empty, so hold
    end

    Scene-->>CC: the AnimationPlayer now exists
    CC->>Gate: release("character-animation-player")
    Adv->>Gate: blockers empty, same frame, because Settle precedes Advance
    Adv->>ST: next.set(Warm)
```

This is the generalisation of what `finish_character_initialization` does today by
early-returning and retrying: the same retry loop, except *the reason it is
waiting becomes a value* instead of an implicit early return — which is where the
loading screen's text and the "what are we stuck on?" log line come from for free.

### 4d. Hot reload — a rewind through the same path

```mermaid
sequenceDiagram
    autonumber
    actor Dev
    participant FW as file_watcher
    participant AS as AssetServer
    participant Watch as rewind_on_change, Gltf to Derive
    participant Req as RewindRequest
    participant Rew as apply_rewind
    participant ST as StateTransition
    participant CC as CharacterControllerPlugin
    participant Game as gameplay, GameState::Playing

    Dev->>FW: save Idle.glb
    FW->>AS: reload_path, then queue_ancestors, transitively
    Note over AS: every loader-dependent path is queued too, so Modified<br/>fires for the tracked root as well as the leaf
    AS->>Watch: several AssetEvent Modified, PreUpdate into LoadSet::Watch
    Watch->>Req: to(Derive) per event, coalesced to Some(Derive)
    Note over Watch: Modified only, never Added: Added is the cold load<br/>the gate already waited on

    Rew->>Req: take() yields Some(Derive)
    Rew->>Rew: Derive is earlier than Settled, so the rewind stands
    Rew->>ST: next.set(Derive)

    ST->>CC: OnExit(Settled), clear_gate
    ST->>CC: OnEnter(Derive)
    CC->>CC: Settle, build_character_animation_graph again
    Note over CC: insert_resource overwrites, Assets::add returns a fresh<br/>handle, and the old graph drops with its last handle:<br/>idempotent for free
    CC->>ST: Derive, World, Instantiate, Warm, Settled

    Game-->>Game: keeps running throughout
    Note over Game: LoadStage is deliberately not a SubStates of<br/>GameState::Loading: otherwise a rewind would exit Playing,<br/>despawn everything marked DespawnOnExit(Playing) and<br/>reset the match over one texture edit
```

The cost of this model is re-entrancy, and it's a per-stage *discipline* rather
than a mechanism. `Instantiate` and `World` are the two stages where it isn't
free:

```mermaid
sequenceDiagram
    autonumber
    participant ST as StateTransition
    participant Despawn as despawn_on_replay for Character
    participant Spawn as spawn_character
    participant W as World

    rect rgb(235,240,248)
    Note over ST,W: first pass
    ST->>Despawn: OnEnter(Instantiate)
    Despawn->>W: query With Character, finds nothing, no-op
    ST->>Spawn: chained after
    Spawn->>W: spawn, one entity
    end

    rect rgb(250,238,232)
    Note over ST,W: after a rewind, the replay
    ST->>Despawn: OnEnter(Instantiate)
    Despawn->>W: despawns the previous pass's entity
    ST->>Spawn: chained after
    Spawn->>W: spawn, still one entity
    end
```

Despawn on *entry*, not `DespawnOnExit(LoadStage::Instantiate)` as the design doc
sketched: `DespawnOnExit` fires on *any* exit, including the forward one into
`Warm`, so it would despawn the character on the way into gameplay rather than on
a replay. Entry-despawn is idempotent in the direction that matters and doesn't
care how the stage was re-entered. (`DespawnOnExit` is still the right tool for
something whose lifetime really is "while this stage is current" — a progress
widget.)

### 4e. Failure

```mermaid
sequenceDiagram
    autonumber
    participant Gate as poll_gate
    participant PA as PendingAssets::progress
    participant AS as AssetServer
    participant Err as Bevy error handler

    Gate->>PA: progress(asset_server)
    PA->>AS: recursive_dependency_load_state per handle
    AS-->>PA: Failed, carrying an AssetLoadError
    PA->>AS: get_path(id), the parent that pulled the dependency in
    PA-->>Gate: AssetLoadProgress::Failed, one path-and-error string each
    Gate-->>Err: Err("load stage Payload: 1 asset(s) failed to load: ...")
    Note over Gate,Err: failures win over still-loading, so a broken asset<br/>surfaces immediately rather than after the<br/>slowest sibling finishes
    Note over Err: no LoadFailed state, which would be a state with nothing<br/>to render, and no advancing anyway, which would leave<br/>the next stage to .expect() on an invariant<br/>the gate no longer provides
```

The stage name in the message is what's new relative to `check_pending_assets`:
the same failure is now attributed to the stage that asked for the load.

## Where the draft stops

Both omissions are deliberate and documented in `asset_loading/mod.rs`:

- **`OnEnter(LoadStage::Settled)` does not set `GameState::Playing` yet.** One
  line, and it's in the design, but adding it while `check_pending_assets` still
  drives `GameState` would give the app two drivers racing for the same state. It
  belongs with migration step 2, which collapses `GameState` to
  `Loading`/`Playing` and deletes `check_pending_assets` in the same change.
- **Exactly one stage has content: `Boot`.** `load_button_config`
  (`src/systems/core/button_config.rs:24`, registered at
  `src/systems/core/mod.rs:45`) loads `button_config.ron` and tracks it, so
  `Boot` genuinely holds the machine until the file is in `Assets<ButtonConfig>`.
  Every other stage is empty and passes through the frame it's entered — the
  designed behaviour for empty stages ("a stage with no registered systems tracks
  nothing and blocks nothing"), which is what lets the full stage list exist
  before every stage has content.

So the `Boot` row of §3 is real, and `boot_loads_the_real_button_config_and_then_advances`
covers it against the actual file. The `Payload`/`Derive`/`World`/`Instantiate`
rows, and therefore §4b–§4d, still show the *shape* registration will take, with
`CharacterControllerPlugin` and `TerrainPlugin` as the worked examples from the
migration plan — those three loads still run on `Startup` through `PendingAssets`
(`src/resources/character_animations.rs:59`, `src/systems/terrain/mod.rs:29`). The
driver itself — §1 through §4a — is real, and covered by the tests at the bottom
of `asset_loading/mod.rs`.

### One ordering fact worth pinning down

The initial `OnEnter(LoadStage::Boot)` runs **inside the first `app.update()`**,
in the `StateTransition` that precedes `PreStartup`
(`bevy_state-0.20.0/src/state/transitions.rs:43`) — not before the first update,
and not after `Startup`. Two consequences:

- A `Boot` load system runs *earlier* than the `Startup` system it replaces, so
  moving a load onto the stage never delays it.
- A stage system therefore cannot read anything a `Startup` system produced.
  That's why `load_button_config` has to issue the load itself rather than
  tracking a handle `Startup` created — the handle wouldn't exist yet.

## Where this fits

- ⬅ Describes: [[11 Staged Asset Loading]] (the design these diagram), §"Stage
  model", §"The transition mechanism", §"Hot reload is a backward transition"
- 🔗 Related: [[12 Asset Description]] (what flows through `Index`,
  `Declarations`, `Resolve` and `Payload`), [[15 State Management]] (the
  `GameState` machine this replaces), [[06 Assets and Scenes]] (the transitive
  dependency tracking §4b relies on)
