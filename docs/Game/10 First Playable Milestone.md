---
tags: [game, milestone]
---

# First Playable Milestone

The end of [[04 Phase 1 - Core Combat Loop]]: a hardcoded, non-scriptable combat sandbox.

- One level, one wave, enemies pathing from a spawn point to a base.
- 2–3 placeholder player units with a single hardcoded "attack nearest enemy in range" behavior — no scripting yet, that's [[05 Phase 2 - AI Scripting]].
- Damage/armor math resolves hits; the base loses HP when an enemy reaches it.

Why hardcode the AI first instead of jumping straight to scripting: the combat numbers (damage, range, cooldowns, enemy speed) need to feel right before investing in the much larger scripting/sandboxing/editor stack on top of them. If the combat loop isn't fun with perfect AI, no amount of scripting flexibility fixes that.
