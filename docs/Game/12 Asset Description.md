---
tags: [game, design, assets, data-model]
---

# 12 — Asset Description

Format for the data-driven content layer: one declared inventory of everything
the game can refer to — textures, models, animations, properties, skills, units,
worlds — with typed cross-references between entries, resolved and **fully
validated before the game starts operating**.

Drafted 2026-10-30, revised twice the same day: once to remove optionality from
the schema (see [§The governing rule](#the-governing-rule-everything-named-is-required)),
and again to settle the name. The whole format was first called "the catalogue"
and its root file `catalogue.ron`; both are retired in favor of **asset
description** (prose) and **`AssetDescription`** (the Rust-facing resolved
resource, specified in [[11 Staged Asset Loading]] alongside the loader that
produces it). Split out of that page, which now covers only the loading
*machinery*; this page covers what gets loaded and how it refers to itself.

Scope note: this page is about the **format and its invariants**, not Rust
types or parser structure — getting the data model right first is the point,
and implementation detail here would only invite designing the format around
the implementation. The one exception is the name `AssetDescription` itself,
kept consistent with [[11 Staged Asset Loading]] so the two pages describe one
thing rather than two similar-sounding ones.

## The governing rule: everything named is required

> **The asset description is a statement of requirement, not of possibility.**
> Every entry it declares must exist. Every reference it makes must resolve. A
> missing piece is an error, always — never a default, never a fallback, never
> a "maybe".

This is the invariant the whole format is built to protect, and two things
follow from it that shape every decision below:

- **No optional fields, and no defaults.** Not "absent means empty", not
  "absent means inherit", not "absent means use the obvious value". If a field
  is part of a kind, every entry of that kind writes it out. An empty set is
  written `[]` — a positive declaration of "none", distinguishable from a
  mistake, which an absent field is not.
- **If a field is meaningful for only some entries of a kind, those entries
  are a different kind.** This is the rule that keeps optionality from sneaking
  back in. A conditional field is a kind boundary that hasn't been drawn yet;
  a `type:`/`mode:` discriminator flag that changes which other fields are
  required is the same mistake wearing a different hat.

The first draft of this page violated both. It had `model: Option<ModelId>` on
units and a `template: bool` flag that changed which fields were required —
optionality introduced not by any content need but by an inheritance mechanism
added unprompted. See [§Deferred](#deliberately-deferred) for where that went.

## The mistake this corrects

An earlier draft said the descriptor should list **load roots** — assets
something explicitly loads — because everything reachable from a root comes
along transitively. It used a world file as the worked example: one root, nine
files.

That correctly describes Bevy's *file* loading and is the wrong model for a
content library. Making worlds the roots means shared content has no single
declaration site; content nothing has placed yet cannot exist, which kills the
world-editor case; verification inverts into "walk every world" and can only
validate what is already used; and the most-derived artifact in the system
becomes its entry point.

So the asset description enumerates **content entities**, not files and not
load roots. A file is an implementation detail *of* an entity. Worlds are the
last kind to load, not the first.

## Shape: typed logical IDs, not paths

Content refers to content by **typed logical ID** — `Model("knight")`,
`Animation("idle")` — never by path. A path appears exactly once, in the entry
that owns the file.

- **Moving a file touches one line**, in the entry that owns it.
- **References are type-checked.** A unit's `model` field names a model; it
  cannot name a skill.
- **The ID is what survives serialization.** World files, save files and a
  future script API name `"knight"`, not a path that may move between versions.
- **It is what every engine that solved this converged on** — see
  [§Precedent](#precedent).

## Format

### Root: `assets.dsc.ron`, standalone at the top of `data/assets/`

```ron
// data/assets/assets.dsc.ron
// The one hand-maintained entry point, and the only asset path that remains a
// compiled-in constant. Everything else is reachable from here.
(
    // The set of kinds is closed. The entries inside each file are open.
    kinds: [
        (kind: Texture,      file: "declarations/textures.ron"),
        (kind: Model,        file: "declarations/models.ron"),
        (kind: Animation,    file: "declarations/animations.ron"),
        (kind: AnimationSet, file: "declarations/animation_sets.ron"),
        (kind: Property,     file: "declarations/properties.ron"),
        (kind: Skill,        file: "declarations/skills.ron"),
        (kind: Unit,         file: "declarations/units.ron"),
        (kind: World,        file: "declarations/worlds.ron"),
    ],
)
```

`.dsc.ron` rather than plain `.ron`, matching the `.scn.ron` convention
`test_world.scn.ron` already uses — the file is unambiguously a *descriptor*
at a glance, same as `.scn.ron` reads as "scene" without opening it.

It sits directly under `data/assets/` — a sibling of `models/`, `worlds/`,
etc. — rather than in its own top-level directory, for two reasons:

- **It's content about content**, so it belongs inside the tree it describes,
  not beside it. `data/config/` earned a separate directory (and its own
  `AssetSource`, `config://`) because those files must be read with `std::fs`
  *before the `App` exists* — see `utils::system_config`'s module doc. Nothing
  about `assets.dsc.ron` has that constraint: it's loaded through the ordinary
  `AssetServer`, so it just needs a path under `ASSET_ROOT_PATH` like anything
  else. No new `AssetSource` to register.
- **Standalone at the top, not buried.** It's the one file in `data/assets/`
  that has to be found without already knowing the layout — everything else is
  reachable *from* it. Keeping it loose at the top, with the per-kind files one
  level down in `declarations/`, means `ls data/assets/` shows the entry point
  immediately instead of it being one anonymous file among the kind
  declarations it names. (`declarations/` rather than a repeat of the old
  "catalogue" name — it holds exactly what the `Declarations` load stage reads,
  so the directory and the stage now name each other.)

One file per kind rather than one monolith: merge conflicts stay local to the
kind being edited, and a kind's file can be split into a directory later without
changing the model. The root file existing at all is what makes the kind set
*declared* rather than implied by a directory scan — an unparsed kind would be a
silent hole in validation, which the governing rule forbids.

**How `assets.dsc.ron` itself gets parsed, gets its own load stage, and
survives being hot-edited is specified in [[11 Staged Asset Loading]] §3** — it
needs a dedicated `Asset` + `AssetLoader` pair to be readable at all (there is
no other way for the `Declarations` stage to ask "which files did the root
name?"), and that page is where the loading machinery, not the format, lives.

### File-owning kinds

```ron
// data/assets/declarations/animations.ron
(
    entries: [
        ("idle", (clip: "models/characters/basic_character/Idle.glb")),
        ("walk", (clip: "models/characters/basic_character/Walking.glb")),
        ("run",  (clip: "models/characters/basic_character/Running.glb")),
        ("jump", (clip: "models/characters/basic_character/Unarmed_Jump.glb")),
    ],
)
```

```ron
// data/assets/declarations/models.ron - a model is a scene file and nothing else.
(
    entries: [
        ("knight",      (scene: "models/characters/basic_character/character.glb#Scene0")),
        ("ground_tile", (scene: "models/terrain/ground_tile.glb#Scene0")),
    ],
)
```

`Model` carries only `scene` on purpose. The first draft put a `locomotion:`
clip-binding map here, which immediately created a conditional field — a terrain
tile has no locomotion — i.e. exactly the smell the governing rule names. The
bindings are their own kind:

```ron
// data/assets/declarations/animation_sets.ron
(
    entries: [
        ("knight_locomotion", (
            bindings: {
                Idle:    "idle",
                Walking: "walk",
                Jumping: "jump",
            },
        )),
    ],
)
```

Which also makes two models able to share a binding set, and one model able to
have several — neither of which was expressible with the map on the model.

### Referencing kinds

```ron
// data/assets/declarations/properties.ron - stats are declared, not ad-hoc keys.
(
    entries: [
        ("health",     (kind: Scalar, min: 1.0, max: 10000.0)),
        ("move_speed", (kind: Scalar, min: 0.0, max: 50.0)),
    ],
)
```

```ron
// data/assets/declarations/skills.ron
(
    entries: [
        ("basic_attack", (
            animation: "attack",
            icon: "ui/icons/sword",
            modifies: [("health", -10.0)],
        )),
    ],
)
```

```ron
// data/assets/declarations/units.ron - every field written out, for every entry.
(
    entries: [
        ("knight", (
            model: "knight",
            animation_set: "knight_locomotion",
            properties: [("health", 140.0), ("move_speed", 6.0)],
            skills: ["basic_attack"],
        )),
    ],
)
```

```ron
// data/assets/declarations/worlds.ron - the most derived kind, and the last to load.
(
    entries: [
        ("test_world", (
            scene: "worlds/test_world.scn.ron",
            // Declared so the loader knows what this world needs before the
            // scene is parsed, and so validation can prove the IDs exist.
            units: ["knight"],
            models: ["ground_tile"],
        )),
    ],
)
```

### Entries are a list, not a map

`entries` is a list of `(id, entry)` pairs rather than a RON map, because RON
resolves a duplicate map key by silently keeping the last one. Under the
governing rule a duplicate ID has to be an error, so the pairs stay a sequence
and the lookup table is built with an explicit duplicate check.

Same reasoning for `properties: [("health", 140.0)]` being pairs rather than a
map — a stat written twice is a mistake worth reporting, not worth resolving.

## Validation

One pass, after every declaration file is parsed. It must catch all of these,
and **report every problem it finds before failing once** — a validator that
dies on the first error turns one content pass into N edit-run cycles:

| Error | Example |
|---|---|
| Dangling reference | unit `knight` names skill `shield_bash`, which no entry declares |
| Missing file | animation `idle` points at a `.glb` that is not on disk |
| Duplicate ID | two `("knight", ..)` entries in `units.ron` |
| Duplicate key within an entry | `properties: [("health", 100.0), ("health", 140.0)]` |
| Wrong-kind reference | unit's `model` field names an animation |
| Unknown kind | `assets.dsc.ron` names a kind with no parser |
| Undeclared property | a unit sets `"stamina"`, which `properties.ron` does not declare |
| Out-of-range value | `health: -5.0` against the declared `min` |
| Reference cycle | see below |
| Undeclared file on disk | a file under `data/assets/` that no entry names |

Note the last one has no allow-list. The asset description names every asset
the game needs, so a file it does not name is by definition not needed and
should not ship. Running this against the repo as of 2026-10-30 reports three:
`models/character/Fox.glb`, `.../basic_character/Running.glb` and
`.../basic_character/Zombie_Attack.glb` are named by nothing. The correct
response is to declare them or delete them, which is the check working.

### What the topological sort is for

Cycle detection, and that is all — a smaller job than it first appeared, and
the reason is worth stating because it also cuts the loading work down:

> **Reference resolution needs validation, not ordering.** A lookup into a
> fully-parsed table does not care what order the table was filled in.

`unit "knight" → model "knight"` imposes no ordering: parse every declaration
file in parallel, then resolve every reference in one pass. Ordering would only
be required if one entry's *content* were computed from another's — which, with
inheritance deferred, nothing does.

So the sort runs over the reference graph purely to prove it is acyclic, and
to give error reports a sensible order. What to avoid is the version that looks
more rigorous: topologically sorting the reference graph and turning each depth
level into a load stage. Textures, models, animations and unit declarations are
mutually independent; depth-sorting them would load in four sequential waves
what should be one parallel one. Depth in the content graph is not the same axis
as "code must run between these two loads", which is what a stage boundary
actually is — see [[11 Staged Asset Loading]] §triage.

## Killing the constants

| Today, in `utils::constants` | After |
|---|---|
| `CHARACTER_ANIMATIONS` (3 paths + state bindings) | `Animation` entries + one `AnimationSet` entry |
| `TEST_WORLD_PATH` | a `World` entry |
| `ASSET_ROOT_PATH`, `CONFIG_SOURCE_*` | unchanged — these configure `AssetPlugin` before the `App` exists |
| `GRAPHICS_CONFIG_PATH`, `CONFIG_READERS` | unchanged — pre-`App` config; see `utils::system_config`'s module doc |
| — | **one new constant**: the path to `assets.dsc.ron` |

The honest end state is one constant, not zero — the bootstrap path cannot
itself be data, the same reason the stage list cannot be
([[11 Staged Asset Loading]] §1). The win is one, rather than a table that grows
with every asset.

Worth noting what else it removes: the binding from `CharacterAnimationState` to
a clip currently lives in three places that must agree — the constant's tuples,
the `AnimationGraph` node map, and the lookup in `apply_character_animation`.
As an `AnimationSet` entry it is declared once, and a second character with
different clip names needs no new Rust.

## Precedent

The convergent features across engines that solved this are worth copying
rather than rediscovering.

- **RimWorld's `Def` system** — the closest match. XML definitions with a
  `defName`, cross-referencing each other by name, and an explicit
  parse-all → resolve-cross-references → report-unresolved-as-a-batch pipeline.
  Its generated static accessors (`DefOf`) are a good idea worth stealing, so
  code naming a specific entry gets a compile-time symbol rather than a string
  literal. It *also* has abstract parent defs, which is the inheritance feature
  deferred below — worth knowing the precedent exists, and that it is the part
  that introduced optionality here.
- **Unreal's Asset Manager** — `PrimaryAssetId` is literally `Type:Name`,
  deliberately decoupled from the content path; soft references let content
  point at content without forcing a load; Asset Bundles are named subsets, the
  mechanism for "load this part now" on top of a complete registry.
- **Unity Addressables** — logical addresses instead of paths, labels and
  groups for subset selection, and a build-time content catalog shipped with
  the game: the same split between hand-authored intent and a generated,
  validated artifact. ("Content catalog" is Unity's own name for its generated
  artifact, kept verbatim here as their term, not ours.)
- **Godot** — `.tres`/`.tscn` resources referencing each other by UID rather
  than path, precisely so files can move.
- **Paradox grand-strategy titles** — large script databases with name-based
  cross-references, validated on load.
- **id Tech 4 `.def` declarations** — a decl manager resolving declarations by
  name and reporting missing ones, rather than paths compiled into code.

Common thread: typed logical IDs, references that don't force loads, one
validation pass that resolves names, and subset selection layered on top of a
complete asset description.

## Deliberately deferred

**Inheritance / templates.** Dropped from this draft, because it was the sole
source of optionality in the schema — a `template: bool` flag plus fields that
were required only for non-templates. If it is wanted later, the governing rule
dictates the shape: a **separate kind** (`UnitTemplate`) with its own complete
schema, and a unit referencing one unconditionally, so no field on either kind
becomes conditional. Expanding templates in a preprocessing step, so the
validated asset description only ever contains complete entries, is the other
option and is probably simpler. Either way it must not reintroduce a
conditional field.

**Subset selection.** The asset description declares everything that exists; it
does not say what to load *now*. "Load all of it" is correct while the content
fits in memory many times over. The seam for later is a `bundle` or `tags`
field plus a per-world declaration of which bundles it needs — Unreal's Asset
Bundles and Unity's labels — which slots in without touching the resolve pass.
Building it now would be speculative; declaring an unused field would be worse.

## Where this fits

- ⬅ Depends on: [[11 Staged Asset Loading]] (the stage machinery this loads
  through, and the home of `AssetDescription`'s loader — this format is what
  reshaped that page's stage list)
- ➡ Enables: [[03 Phase 0 - Foundation]] items 1 and 4 (core data model; level
  loading and switching), [[04 Phase 1 - Core Combat Loop]] (stats and skills as
  declared content rather than constants), [[05 Phase 2 - AI Scripting]] (a
  script API that names content by ID is the only kind that can be sandboxed
  and persisted)
- 🔗 Related: [[08 Reflection]] (RON ↔ struct deserialization),
  [[06 Assets and Scenes]] (`.scn.ron` world files, which become the `World`
  kind's payload)

[[00 Roadmap Overview|← Back to roadmap]]
