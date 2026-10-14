---
tags: [game, roadmap]
---

# Phase 1 — Core Combat Loop (hardcoded)

Build the combat sandbox with ordinary Rust systems before any scripting exists. You need something to validate feel against, and Phase 2's scripting API will just expose these same hooks to player scripts instead of calling them directly from Rust — so nothing here is wasted once scripting lands.

5. **Skill & stats system** — HP, armor type, attack damage/type, cooldowns, movement speed. The numeric substrate everything below reads from.
6. **Pathing & movement** — enemy waypoint/navmesh pathing toward the player's base, plus free-roam movement for player units. Two movement *models*, one motion primitive.
7. **Projectiles & melee attacks** — generic "ability execution": target acquisition, range/cooldown check, hit resolution (hitscan vs. melee vs. travelling projectile).
8. **Damage & armor system** — resolves an attack's hit into HP loss via armor/damage-type math.

End of this phase = a fully playable, not-yet-scriptable combat sandbox. See [[10 First Playable Milestone]].

→ [[05 Phase 2 - AI Scripting]]
