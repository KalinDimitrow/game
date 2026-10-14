---
tags: [game, roadmap]
---

# Phase 0 — Foundation

No gameplay yet — this is the substrate everything else attaches components and queries to.

1. **Core data model** *(added)* — `Unit`, `Enemy`, `Team`/faction tags, base placement components. Not on the original list but implicit in all of it: every later system queries this schema, so it has to exist first.
2. **Terrain & collision** — ground representation, static obstacles, spatial queries. Needed for placement, line-of-sight, and pathing later.
3. **Loading game assets** — mesh/material/RON content pipeline. Already partially exists (`systems/core/scene.rs`, `DynamicWorldRoot`) — formalize rather than rebuild.
4. **Loading & switching levels** — a level = terrain + spawn points + wave definitions, data-driven via RON, consistent with the existing scene-loading convention.

→ [[04 Phase 1 - Core Combat Loop]]
