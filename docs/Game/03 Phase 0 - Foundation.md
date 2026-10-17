---
tags: [game, roadmap]
---

# Phase 0 — Foundation

No gameplay yet — this is the substrate everything else attaches components and queries to.

1. **Core data model** *(added)* — `Unit`, `Enemy`, `Team`/faction tags, base placement components. Not on the original list but implicit in all of it: every later system queries this schema, so it has to exist first.
2. **Terrain & collision** — ground representation, static obstacles, spatial queries. Needed for placement, line-of-sight, and pathing later. Started 2026-10-14: `systems/terrain/` tiles a real glTF floor asset from disk (`data/assets/models/terrain/ground_tile.glb`) into a 28×16 rectangular grid, one entity per cell, with a free-flying camera to look at it. Picked a grid specifically so collision/placement can be added as cheap per-cell lookups later instead of general spatial queries — still no actual collision yet, just the addressable grid. 2026-10-17: grid size and tile asset are no longer hardcoded — both now come from the level RON described in item 4 below. Per-cell overrides carry `height`/`walkable`, but nothing consumes them for collision yet; still one flat `Collider::cuboid` for the whole grid.
3. **Loading game assets** — mesh/material/RON content pipeline. Already partially exists (`systems/core/scene.rs`, `DynamicWorldRoot`, and now `systems/terrain/`'s glTF loading via `WorldAssetRoot`) — formalize rather than rebuild.
4. **Loading & switching levels** — a level = terrain + spawn points + wave definitions, data-driven via RON, consistent with the existing scene-loading convention. Started 2026-10-17: `systems/terrain/level_config.rs` loads `data/levels/level_01.ron` (grid size/cell size, a tile registry + sparse per-cell overrides, and `points_of_interest` for spawn/objective markers) into a validated `LevelConfig` resource. `terrain::spawn_terrain` builds the grid from it; `character_controller::spawn_character` reads its `PlayerSpawn` POI for the player's start position. Wave definitions and actual level *switching* (vs. just loading the one file) are still open.

→ [[04 Phase 1 - Core Combat Loop]]
