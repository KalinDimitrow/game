---
tags: [game, design, assets, state]
---

# 11 — Staged Asset Loading

Design plan for replacing the single `PendingAssets` gate with an ordered,
multi-stage load. Drafted 2026-10-30, revised the same day, nothing implemented
yet. Formalizes item 3 of [[03 Phase 0 - Foundation]] ("Loading game assets").

The ask, as stated: load in stages, the last being gameplay, with intermediate
stages because some assets depend on others; a descriptor saying what's
required; possibly a generated asset→stage map from the asset build script;
and a clean stage-transition mechanism.

One part of that premise needs adjusting before any of it gets built, so that
comes first.

> **Revision, same day.** The first draft used a hand-rolled driver over custom
> schedules, and treated hot reload as a second code path alongside the stages.
> Both were replaced after the observation that **a stage sequence is a state
> machine, and state machines transition backwards too** — so hot reload is a
> rewind through the same stages, not a parallel mechanism. That removes the
> dual-path design entirely and makes real Bevy `States` the better substrate.
> The superseded options are kept at the end of the transition section, because
> the reasoning for *why* they lost is the useful part. Affected sections are
> marked.

## The triage: Bevy already resolves asset→asset dependencies

`recursive_dependency_load_state` is *transitive*. This is already load-bearing
in the current code — `terrain::spawn_test_world` tracks exactly one handle for
`test_world.scn.ron`, and that one handle covers all 7 glTF scenes it
references plus the character mesh underneath them (see `PendingAssets`'s doc
comment, and [[06 Assets and Scenes]]). Any dependency a *loader* declares, via
`LoadContext::load`, is already waited on with no staging whatsoever.

So "asset A references asset B" is, on its own, not a reason for a stage. If it
were, stages would be a worse reimplementation of something the engine does
better. What *does* need a stage is a dependency that crosses out of the asset
system — five kinds, and only these:

| Kind | Example in this project | Does Bevy cover it? |
|---|---|---|
| Loader dependency | `.scn.ron` → `.glb` → textures | **Yes**, transitively. No stage. |
| Name not known until a load finishes | a unit roster index that lists which unit defs exist | **No.** Inherently a stage boundary. |
| Runtime-derived asset | `AnimationGraph` built from 3 loaded glTFs | **No.** This is today's `Initialize`. |
| Logical-name resolution | a unit def saying `script: "aggressive"`, resolved against a registry rather than a path | **No**, if it's a name. **Yes**, if you make it a path and `load_context.load` it. |
| Entity-level fact | the scene instance spawned, so an `AnimationPlayer` now exists | **No.** Not an asset fact at all. Today's `CharacterSceneSpawned`. |
| GPU pipeline warm | shaders compile lazily on first draw; "loaded" ≠ "renders without a hitch" | **No.** `showcase/loading_screen.rs` is the reference. |

Which gives the actual definition to build against:

> **A stage boundary exists where code must run between two loads** — because
> the next load can't be *named*, or the next asset doesn't exist on disk and
> has to be *derived*, or the thing being waited for isn't an asset at all.

That definition is worth more than it looks, because it also tells you what the
*number* of stages is: it's the number of distinct derive/resolve steps you
have written. Which is a compile-time quantity. More on that below.

## Stage model

A stage is a label plus two hook points and a gate:

```mermaid
flowchart LR
    A[Begin<br/>OnEnter, once] --> B{handles<br/>all Loaded?}
    B -- no --> B
    B -- failed --> F[Err: stage + path]
    B -- yes --> C[Settle<br/>every frame in-state]
    C --> D{blockers<br/>empty?}
    D -- no --> C
    D -- yes --> E[next stage]
    E -. asset changed:<br/>rewind .-> A
```

- **Begin** — runs once, when the stage becomes current. Its systems *name*
  loads and track them on the stage. This is the hook that makes dynamic
  content work: a `Begin` system can read assets loaded by any earlier stage,
  so a roster index loaded in stage N tells stage N+1 what to load.
- **Gate** — the driver folds `recursive_dependency_load_state` over the
  stage's tracked handles. Not a user hook; it's `PendingAssets::progress`
  as it exists today, scoped to one stage.
- **Settle** — runs every frame *after* the gate is green, until nothing is
  blocking. Derived assets get built here, and entity-level waits live here.

**Settle running only once the gate is green is the important property**, and
it's one the current code already paid for: `build_character_animation_graph`
has no readiness guard because entering `Initialize` already proves the loads
succeeded. Keeping the gate ahead of Settle preserves that — derive steps stay
guard-free, and a missing file is still an `Err` with a path in it rather than
an `.expect()` in an unrelated-looking function.

### Completion is a set of named blockers, not a boolean

A stage is done when its handles are loaded **and** its blocker set is empty:

```rust
/// The current stage's gate. One of these, not one per stage: `OnExit` clears
/// it, so it only ever describes the stage being entered - which is also what
/// makes re-entering a stage after a rewind clean.
#[derive(Resource, Default)]
pub struct LoadGate {
    handles: Vec<UntypedHandle>,
    blockers: HashSet<&'static str>,
}

impl LoadGate {
    /// Same role `PendingAssets::track` has today, scoped to one stage.
    pub fn track(&mut self, handle: impl Into<UntypedHandle>) {
        self.handles.push(handle.into());
    }

    /// Holds the stage open under a name. The name is what the loading screen
    /// and the log line show, so it should read as the thing being waited for.
    pub fn block(&mut self, reason: &'static str) {
        self.blockers.insert(reason);
    }

    pub fn release(&mut self, reason: &'static str) {
        self.blockers.remove(reason);
    }
}
```

Why blockers rather than "each stage has a `ready` condition": it makes stage
completion **open for extension without the driver knowing anything**. Any
plugin can hold any stage open — `CharacterControllerPlugin` blocks
`instantiate` on `"character-animation-player"` and releases it when the player
is attached, and the driver never learns that characters exist. It also hands
you the loading-screen text and the "what are we stuck on?" log line for free,
which a boolean cannot. "All handles loaded" is then just the one blocker the
driver owns itself.

This is the generalization of what `finish_character_initialization` does today
by returning early and retrying — same retry loop, but the reason it's waiting
becomes a value rather than an implicit early-return.

## The transition mechanism

**Revised 2026-10-30**, after the observation that a stage sequence is a state
machine, and a state machine's transitions go backwards as well as forwards.
That turns out to be the requirement that settles this section, so the original
four-way comparison is recorded at the end for the record rather than up front.

The earlier draft rejected "one state per stage" because the stage list had to
be dynamic, and then [§On a dynamic number of stages](#on-a-dynamic-number-of-stages)
argued the list should be a compile-time `const`. Those two positions can't both
stand. With a `const` stage list, a plain `States` enum is back on the table —
and it is the only candidate where a **backward** transition is a primitive
instead of something bolted on:

```rust
/// Deliberately *not* a `SubStates` of `GameState::Loading` - see the hazard
/// below, which is the whole reason this is its own state.
#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum LoadStage {
    #[default]
    Boot,
    Index,
    Content,
    Derive,
    Instantiate,
    Warm,
    /// Terminal: everything loaded, derived, instantiated and warm.
    Settled,
}

impl LoadStage {
    pub const ORDER: &'static [LoadStage] = &[
        Self::Boot, Self::Index, Self::Content,
        Self::Derive, Self::Instantiate, Self::Warm, Self::Settled,
    ];

    pub fn position(self) -> usize {
        Self::ORDER.iter().position(|s| *s == self).expect("in ORDER")
    }

    pub fn next(self) -> LoadStage {
        *Self::ORDER.get(self.position() + 1).unwrap_or(&Self::Settled)
    }

    /// Whichever of the two comes first. Used to coalesce rewind requests.
    pub fn earliest(self, other: LoadStage) -> LoadStage {
        if other.position() < self.position() { other } else { self }
    }
}
```

What this buys, all of it native:

| Need | Native mechanism |
|---|---|
| Begin hook | `OnEnter(LoadStage::Content)` |
| Settle hook | `Update.run_if(settling(LoadStage::Derive))` |
| **Backward transition** | `next_state.set(LoadStage::Derive)` |
| **Undo on leaving a stage** | `OnExit(..)`, and `DespawnOnExit<LoadStage>` for entities (`bevy_state-0.20.0/src/state_scoped.rs:145`) |
| Ordering against asset insertion | `StateTransition` is installed after `PreUpdate`, so an `OnEnter` can never observe a load before `Assets<T>` has it — the caveat [[06 Assets and Scenes]] documents, handled for free |

Registration stays as cheap as the custom-schedule version:

```rust
// CharacterControllerPlugin::build
app.add_systems(OnEnter(LoadStage::Content), load_character_gltf_handles);
app.add_systems(Update, build_character_animation_graph.run_if(settling(LoadStage::Derive)));
app.add_systems(Update, finish_character_initialization.run_if(settling(LoadStage::Instantiate)));
```

`settling` is what preserves the guard-free-derive property — it's `in_state`
plus "this stage's handles are all loaded", so a derive step still needs no
readiness check of its own:

```rust
/// A derive step gated on this cannot observe a half-loaded stage, which is
/// what lets `build_character_animation_graph` keep having no readiness guard.
pub fn settling(stage: LoadStage) -> impl Fn(Res<State<LoadStage>>, Res<LoadGate>) -> bool + Clone {
    move |current, gate| *current.get() == stage && gate.is_ready()
}
```

### The driver shrinks to two systems

No exclusive world access, no `try_run_schedule`, no `resource_scope` trap —
all of that existed only to hand-roll what `StateTransition` already does:

```rust
fn advance_stage(
    current: Res<State<LoadStage>>,
    gate: Res<LoadGate>,
    asset_server: Res<AssetServer>,
    mut next: ResMut<NextState<LoadStage>>,
) -> Result<(), BevyError> {
    let stage = *current.get();
    if stage == LoadStage::Settled {
        return Ok(());
    }

    match gate.progress(&asset_server) {
        AssetLoadProgress::Loading => return Ok(()),
        AssetLoadProgress::Failed(failures) => {
            return Err(format!(
                "load stage {stage:?}: {} asset(s) failed:\n  {}",
                failures.len(),
                failures.join("\n  ")
            )
            .into());
        }
        AssetLoadProgress::Ready => {}
    }

    // Settle systems ran this frame (same `Update`, ordered before this);
    // if nothing is still holding the stage open, move on.
    if gate.blockers_clear() {
        next.set(stage.next());
    }
    Ok(())
}
```

…plus `apply_rewind`, in the hot-reload section below. `LoadGate` is the
per-stage `handles` + `blockers` from the stage model above, reset in
`OnExit(..)` — which is also what makes re-entry clean.

### The hazard: `LoadStage` must not be a `SubStates` of `GameState::Loading`

This is the one thing that will break the whole design if it's got wrong, and
it's non-obvious.

If stages are a sub-state of `GameState::Loading`, then rewinding to re-run a
derive step means **re-entering `GameState::Loading`** — which exits
`GameState::Playing`, which despawns everything carrying
`DespawnOnExit(GameState::Playing)` and re-runs every `OnEnter(Playing)` setup.
Editing one texture would reset the match.

So `LoadStage` is an independent `States` with a terminal `Settled` variant, and
`GameState::Playing` is entered once, on `OnEnter(LoadStage::Settled)`, and
never left by a rewind. A hot reload moves `LoadStage` backwards while
gameplay keeps running.

The consequence to accept: gameplay must tolerate a derived resource being
briefly absent or mid-rebuild. The current code already does, and not by
accident — `apply_character_animation` takes `Option<Res<CharacterAnimations>>`
and early-returns, as does `finish_character_initialization`. Those guards were
written for the cold-load race and happen to be exactly what a rewind needs,
so this is a property the codebase already has rather than one to add.

### For the record: the options this replaced

- **One `GameState` variant per stage** — what exists today. Rejected at 6
  stages for the usual reason (adding a stage edits the enum plus every gate),
  but the mechanism was right; the fix is a *separate* state enum, not a
  non-state.
- **`SubStates` carrying an index** (`LoadPhase(usize)` under `Loading`) —
  rejected: you can't write `OnEnter(LoadPhase(3))` against a data-driven list,
  and sub-stating under `Loading` is precisely the hazard above.
- **Plain resource + one-shot systems (`SystemId`)** — fully dynamic, but every
  hook needs registering and storing by hand, and multi-system stages lose
  normal ordering and parallelism.
- **Plain resource + one custom schedule per hook per stage** — the original
  recommendation here, keyed by an interned `StageLabel(&'static str)` and run
  via `World::try_run_schedule` (which tolerates a missing schedule:
  `bevy_ecs-0.20.0/src/world/mod.rs:3983`). It works, and it's the right answer
  *if* the stage list ever has to be runtime data. It isn't, so it was paying
  for dynamism it didn't use — and it would have had to reimplement backward
  transitions, `OnExit` cleanup and the `PreUpdate`/`StateTransition` ordering
  guarantee by hand.

## The descriptor

Three separate questions got bundled into "a descriptor"; they have different
answers.

### 1. Stage order → a Rust `const`, not a data file

That's the `LoadStage` enum and its `ORDER` const from the transition section
above — the same `(name, value)` table idea `CONFIG_READERS`,
`CHARACTER_ANIMATIONS` and `KEYCODES` already use in `utils::constants`, just
expressed as a `States` enum because that's what the state machine needs.
Reasons to keep it in Rust rather than RON:

- **A stage without Rust systems does nothing.** Adding a stage means adding
  systems to its `OnEnter`/`settling(..)` hooks — it's inherently a code
  change, so putting the order in a data file creates a file that can't be
  meaningfully edited on its own, plus a new failure mode (a RON stage nobody
  registered systems for).
- **Bootstrapping.** A load plan that is itself a loaded asset has to answer
  "which stage loads the load plan?". Making it a `const` deletes the question.
- **`States` requires it.** `OnEnter`/`OnExit`/`in_state` all need a type with
  known variants. This is the constraint that decided the mechanism; see the
  transition section.

The counter-argument, recorded in case it ever applies: if a non-programmer
ever needs to reorder stages without a rebuild, RON wins — and that would mean
going back to the custom-schedule mechanism, since a `States` enum can't be
data. That isn't the case now and probably never is, because the order follows
the derive steps.

### 2. Stage membership → system registration, not a generated map

This is the part of the original plan I'd drop. "Which assets belong to stage
N" is already fully expressed by *which stage's `OnEnter` system loads them* —
`load_character_gltf_handles` registered on `OnEnter(LoadStage::Content)` **is**
the statement that the character clips are content-stage assets. A generated
asset→stage map would be a second, derived copy of that, and a build script
can't produce it anyway: it would have to infer design intent (is this mesh
needed at boot or at level 3?) from a directory listing.

The one place a *runtime* asset→stage mapping is genuinely needed is the rewind
lookup in the hot-reload section — and there it's per asset *type* per consuming
stage, declared in one line of plugin wiring, not per file.

### 3. Asset inventory → *this* is what's worth generating

What a build script genuinely knows, and Rust otherwise doesn't, is **what
files exist**. That's worth generating, for three reasons, one of which is a
trap specific to this project:

- **`load_folder` is silently broken on wasm.** It looks like the native answer
  to "load everything in `units/`", and on desktop it works —
  `LoadedFolder.handles` is `#[dependency]`, so one folder handle covers the
  whole directory recursively under `recursive_dependency_load_state`. But
  `bevy_asset`'s `HttpWasmAssetReader::read_directory` logs
  `"Reading directories is not supported"` and returns an **empty** stream
  (`bevy_asset-0.20.0/src/io/wasm.rs:132`). You get zero assets and a
  successfully-loaded folder. `Cargo.toml` has a `wasm-release` profile, so
  this is a live concern, not a hypothetical.
- **Determinate progress.** A loading bar needs a denominator before the loads
  start. Generated inventory gives it; runtime directory discovery can't.
- **Typos become compile errors.** A wrong path in `CHARACTER_ANIMATIONS` is
  currently a runtime load failure.

`build.rs` already exists (for rpath), so this is an addition to it:

```rust
/// Emit `const` path lists for convention-named asset groups, so a stage's
/// `OnEnter` systems can iterate a directory's contents without `load_folder`.
fn emit_asset_inventory() {
    // (const name, directory under data/assets, extension suffix)
    const GROUPS: &[(&str, &str, &str)] = &[
        ("CHARACTER_CLIPS", "models/characters/basic_character", ".glb"),
        ("UNIT_DEFS", "units", ".unit.ron"),
        ("SCRIPTS", "scripts", ".script.ron"),
    ];

    let mut out = String::new();
    for (name, dir, ext) in GROUPS {
        let paths = collect_sorted(Path::new("data/assets").join(dir), ext);
        out.push_str(&format!("pub const {name}: &[&str] = &[\n"));
        for path in paths {
            // Asset-server-relative, forward slashes, sorted for determinism.
            out.push_str(&format!("    {path:?},\n"));
        }
        out.push_str("];\n");
    }

    let dest = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("asset_inventory.rs");
    std::fs::write(dest, out).expect("failed to write asset inventory");
    println!("cargo::rerun-if-changed=data/assets");
}
```

Consumed as `include!(concat!(env!("OUT_DIR"), "/asset_inventory.rs"));` from a
`resources::inventory` module, and used from a stage's `OnEnter`:

```rust
// OnEnter(LoadStage::Content)
fn load_unit_defs(asset_server: Res<AssetServer>, mut gate: ResMut<LoadGate>) {
    for path in inventory::UNIT_DEFS {
        gate.track(asset_server.load::<UnitDef>(*path));
    }
}
```

Note the ordering constraint this implies: generated inventory is a *sorted*
list, so iteration order is stable across machines. Worth keeping, because
"loads in directory order" is the kind of thing that works until it doesn't.

### 4. Runtime-discovered names → one stage's `OnEnter` reading the last one's assets

The one case the inventory genuinely can't cover: a manifest asset whose
*contents* name what to load. That's what the `Index` stage exists for — it
loads indices, and `OnEnter(LoadStage::Content)` reads them:

```rust
// OnEnter(LoadStage::Content)
fn load_indexed_units(
    asset_server: Res<AssetServer>,
    indices: Res<Assets<UnitIndex>>,
    index_handle: Res<UnitIndexHandle>,
    mut gate: ResMut<LoadGate>,
) {
    // No readiness guard: `Index` completed, and `StateTransition` runs after
    // `PreUpdate`, so the asset is in `Assets<T>` by the time `OnEnter` sees it.
    let index = indices.get(&index_handle.0).expect("Index stage completed");
    for entry in &index.units {
        gate.track(asset_server.load::<UnitDef>(&entry.path));
    }
}
```

## On a dynamic number of stages

I'd not build it, and the reasoning is the stage definition above: a stage
exists because *code* runs between two loads, and code is compile-time. So the
stage count is bounded by the derive steps you've written, and `LoadStage` can
be an enum — which, per the transition section, is what makes the whole
state-machine mechanism (and therefore the rewind-based hot reload) available
at all. This conclusion is load-bearing for the rest of the design, not just a
scoping preference.

What is genuinely dynamic is covered without a variable stage list:

- **Stage contents** — how many unit defs, which level's meshes. Handled by
  `OnEnter` systems reading indices.
- **Skipped stages** — a stage with no registered systems tracks nothing and
  blocks nothing, so `advance_stage` moves through it the frame it's entered.
  No special case needed.

The one case that would actually need unbounded stages is **recursively nested
manifests** (an index that can name another index, to arbitrary depth). If that
ever becomes real, the fix is a *self-transition* rather than a growing list:
re-set `NextState` to the stage you're already in, which re-runs `OnExit` (which
clears the gate) and `OnEnter` (which re-reads the now-larger index), with an
iteration cap to turn a cyclic index into an error instead of a hang. The
backward-transition machinery already needed for hot reload covers this case
too — it's the same primitive with a target of "here".

## Hot reload is a backward transition, not a second code path

**Revised 2026-10-30.** The original version of this section proposed splitting
every derive step in two: the real work in `Update` reacting to
`AssetEvent::Added | Modified` (the shape `apply_button_config` already has),
plus a `Settle` system that only waits for the output to exist. That was wrong,
or at least needlessly expensive, and the reason is worth stating because it
generalizes:

> A second code path that only runs in a situation you rarely test will rot
> without telling you.

Hot reload is a dev-loop feature. A dual-path design means the hot-reload path
is exercised only when someone happens to edit a file while the game is
running, so it can break silently for weeks. **Replaying the stages instead
means hot reload and cold load are the same code.** If the reload path breaks,
startup breaks, and you find out on the next `cargo run`. That is a much
stronger guarantee than "we wrote both paths carefully", and it's worth more
than the handful of lines the dual-path version saved.

So: on an asset change, set `LoadStage` **back** to the earliest stage whose
work consumed that asset, and let the machine run forward again.

```rust
/// Coalesces every rewind asked for this frame. Editors fire several
/// `Modified` events per save, and a multi-file save can name several stages.
#[derive(Resource, Default)]
pub struct RewindRequest(Option<LoadStage>);

impl RewindRequest {
    pub fn to(&mut self, stage: LoadStage) {
        self.0 = Some(match self.0 {
            Some(current) => current.earliest(stage),
            None => stage,
        });
    }
}

/// One instance per (asset type, consuming stage).
fn rewind_on_change<A: Asset>(
    stage: LoadStage,
) -> impl FnMut(MessageReader<AssetEvent<A>>, ResMut<RewindRequest>) + Clone {
    move |mut events, mut request| {
        if events.read().any(|e| matches!(e, AssetEvent::Modified { .. })) {
            request.to(stage);
        }
    }
}

fn apply_rewind(
    mut request: ResMut<RewindRequest>,
    current: Res<State<LoadStage>>,
    mut next: ResMut<NextState<LoadStage>>,
) {
    let Some(target) = request.0.take() else {
        return;
    };
    // Only ever backwards. A change to something a later stage consumes will be
    // picked up when the machine reaches that stage anyway.
    if target.position() < current.get().position() {
        next.set(target);
    }
}
```

Registered per consuming stage, which is one line each:

```rust
app.add_systems(Update, rewind_on_change::<Gltf>(LoadStage::Derive));
app.add_systems(Update, rewind_on_change::<ButtonConfig>(LoadStage::Boot));
```

### Why the rewind target is the *consuming* stage, not the loading stage

Editing `Idle.glb` does not require re-running `Content`: `AssetServer` has
already reloaded the clip itself by the time the event fires. What's stale is
the `AnimationGraph` *derived* from it. So the target is `Derive`. Getting this
backwards isn't harmful, just wasteful — re-running `Content` re-issues
`asset_server.load` calls that return cached handles.

That asymmetry is the safety valve for the whole design: **over-rewinding is
always safe, under-rewinding leaves something stale.** So when a mapping is
unclear, pick the earlier stage. It also means the per-type registration above
can stay coarse — "any `Gltf` changed → rewind to `Derive`" is fine even once
there are glTFs that nothing derives from.

### Verified: the events arrive on the handle you actually tracked

The obvious worry is that a stage tracks `test_world.scn.ron` while the file
that changed is a texture three levels down, so `AssetEvent::Modified` fires for
an id no stage ever tracked. That turns out not to be a problem:
`AssetInfos` keeps a `loader_dependents` reverse map, and the file watcher's
`queue_ancestors` walks it **transitively**, queueing every dependent path for
reload as well (`bevy_asset-0.20.0/src/server/mod.rs:1950-1961`, called from
`reload_path` at `:1982`). So changing a texture reloads the glTF, which
reloads the `.scn.ron` — and `Modified` fires for the tracked root too, not just
the leaf. Stage-level reload mapping is sound without reimplementing dependency
tracking.

(`loader_dependents` itself is private, so you can't query it directly. You
don't need to — you get the events.)

### What re-entrancy costs

This is the real price of the model, and it's a discipline rather than a
mechanism. A stage that cold-loads into a fresh world will, on replay, run
against a world that already holds the previous run's results:

- **Tracked handles must be cleared** on leaving a stage, or they accumulate
  duplicates across replays. That's `OnExit(stage)` resetting `LoadGate` —
  one system for all stages, generic over nothing.
- **Derive steps are naturally idempotent.** `commands.insert_resource(..)`
  overwrites; `Assets::add` returns a fresh handle and the old
  `AnimationGraph` drops when the last handle does. Nothing to change.
- **Spawning stages are not.** Anything `instantiate` spawns needs
  `DespawnOnExit(LoadStage::Instantiate)` or an equivalent, or a replay
  double-spawns it. Worth noting that today's `finish_character_initialization`
  is already safe here — it queries `Without<AnimationGraphHandle>`, so a
  second run is a no-op — but that was luck, and new instantiate-stage systems
  need it on purpose.
- **Blockers must be re-armed, not just released.** A plugin that blocks a
  stage in `OnEnter` and releases it in `Settle` works on replay; one that
  blocks it from `Startup` would only ever do so once.

None of that is free, but it's *local, inspectable* discipline — "this stage
must be safe to re-enter" is a property you can state and review per stage.
The dual-path alternative's requirement was "this reload system must stay
behaviourally identical to a different system somewhere else", which is neither
local nor checkable.

### Bonus: level switching falls out of this

[[03 Phase 0 - Foundation]] item 4 wants level *switching*, which under this
model is a rewind with a wider target: set `LoadStage` back to `Index`, point
the level index at a different file, and the machine reloads content, re-derives
and re-instantiates through the same path as startup. No separate
"unload the level" routine, and no third code path.

## Proposed stages for this project

| Stage | Purpose | Has content today? | Re-entrant? |
|---|---|---|---|
| `Boot` | Assets the loading screen itself needs (font, button config). Must finish before any progress can be *displayed*. | Yes — `button_config.ron` moves here | Trivially |
| `Index` | Manifests/indices: unit roster, script list, level index. | No — placeholder until [[05 Phase 2 - AI Scripting]] | Trivially |
| `Content` | Everything named statically or by an index. The big parallel stage. | Yes — character clips, `test_world.scn.ron` | Yes, loads are cached |
| `Derive` | Runtime-built assets: `AnimationGraph`, compiled scripts, name→handle registries. | Yes — `build_character_animation_graph` | Yes, `insert_resource` overwrites |
| `Instantiate` | Scene spawn + entity-level waits. | Yes — `finish_character_initialization` | **Needs care** — see re-entrancy costs |
| `Warm` | Pipeline/shader warm-up before gameplay. | No — add when content is big enough to stutter | Yes |
| `Settled` | Terminal. `OnEnter` sets `GameState::Playing` (once, ever). | — | — |

`Boot` existing at all is a consequence worth noting: a loading screen has
assets, so there must be a stage before the one it reports on. [[11 UI]] and
`showcase/loading_screen.rs` are the references for the screen itself.

The re-entrancy column is the new load-bearing one — it's the per-stage property
the rewind model asks you to state and review, and `Instantiate` is the only
current stage where it isn't free.

## Migration path

Each step is independently landable and leaves the app working.

1. **`PendingAssets` → `LoadGate`.** Mechanical: add the named-blocker set
   alongside the existing handle list, keep `progress` as-is. No behavior
   change, nothing else moves yet.
2. **Add the `LoadStage` enum + `advance_stage` + the `OnExit` gate reset**,
   with every current load system on `OnEnter(LoadStage::Content)` and the two
   `Initialize` systems on `settling(..)` of `Derive`/`Instantiate`.
   `GameState` collapses to `Loading`/`Playing` in the same step, since
   `Initialize` has nowhere left to live. This is the step that proves the
   machine, and it's a net deletion.
3. **Make `Instantiate` re-entrant** — `DespawnOnExit(LoadStage::Instantiate)`
   on anything it spawns, blockers armed in `OnEnter` rather than `Startup`.
   Do this *before* step 4, because step 4 is what starts exercising re-entry.
4. **Add `RewindRequest` + `apply_rewind` + one `rewind_on_change::<T>` per
   consuming stage.** Hot reload starts working here, through the same path as
   cold load. Verify by editing `button_config.ron` and an animation glTF with
   the app running and `file_watcher` enabled.
5. **Add `build.rs` inventory generation**, replace `CHARACTER_ANIMATIONS`'
   hand-written paths with a generated group.
6. **Add `Boot` + the loading screen**, driven by a `LoadProgress` resource
   (`stage`, `position`/`ORDER.len()`, `loaded`/`total`,
   `waiting_on: Vec<&'static str>` — the blocker names).
7. **Add `Index`/manifest loading** when unit defs exist, `Warm` when a
   first-frame stutter is actually observable, and level switching (a rewind to
   `Index`) when there's a second level.

Steps 1–4 are the plan proper and should land together-ish, since 4 is what
makes the whole thing worth doing. 5–7 are driven by content that doesn't exist
yet and shouldn't be built ahead of it.

### Testability

Worth calling out because it's a change from today, and the rewind model makes
it matter more: the machine is testable without files or a window. A
`MinimalPlugins` + `AssetPlugin` app, a gate blocked on a name a test system
releases on frame 3, and assertions on `State<LoadStage>` per frame covers
enter-once, blocker release and advance with no assets involved. **The rewind
path is the one that most needs tests** — set `RewindRequest` directly, assert
the state went backwards, assert it doesn't go backwards past a change in a
later stage, and assert a stage that spawns doesn't double-spawn across a
replay. That last one is the regression this design can actually suffer, so it
should be the first test written.

The current `GameState` machine has no equivalent test, which is why its two
regressions were both found by reading rather than by failing.

## What I'd deliberately not build

- **A dependency-graph solver.** `recursive_dependency_load_state` is the solver.
- **A reverse dependency map for rewind targeting.** `queue_ancestors` already
  propagates reloads up to the handle you tracked — see the verified note above.
- **A generated asset→stage map** — see descriptor §2.
- **`load_folder`** on anything that must work in `wasm-release`.
- **A separate hot-reload code path** — that's the point of the revision.
- **`LoadStage` as a `SubStates` of `GameState::Loading`** — see the hazard.

## Where this fits

- ⬅ Builds on: [[06 Assets and Scenes]] (the dependency-tracking mechanics and
  the `AssetBarrier` history), [[15 State Management]] (the current
  `Loading`/`Initialize`/`Playing` machine this replaces)
- ➡ Enables: [[05 Phase 2 - AI Scripting]] (script assets, a script registry,
  hot-reloaded scripts), [[03 Phase 0 - Foundation]] item 4 (level switching —
  a level change is a rewind to `LoadStage::Index`, i.e. the same primitive as
  hot reload with a wider target)
- 🔗 Related: [[11 UI]] (`showcase/loading_screen.rs`), [[20 Platform Targets]]
  (the wasm directory-listing trap), [[01 Cleanup Backlog]]

Note for whoever implements this: [[06 Assets and Scenes]] §"Waiting on a batch
of loads" and [[15 State Management]]'s "Used in `game`" both still describe the
`AssetBarrier`/`check_character_ready` design, removed 2026-10-28. Those
sections need rewriting against the current code before they're rewritten
against this plan.
