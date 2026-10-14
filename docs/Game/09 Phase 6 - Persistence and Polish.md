---
tags: [game, roadmap]
---

# Phase 6 — Persistence & Polish

None of this blocks the core loop — it layers on afterward.

19. **Save/load game state** — level progress, unlocked units, meta-progression. Separate from script persistence ([[05 Phase 2 - AI Scripting]]), which matters enough to exist much earlier.
20. **Animations** — layered onto the existing placeholder meshes (`SpawnFloor`/`SpawnCube` in `systems/scene.rs`). Doesn't block logic, correctly last.
21. **Audio** *(added)* — SFX for attacks/hits/UI, music.
22. **VFX/particles** *(added)* — hit sparks, projectile trails. Cosmetic only.
