---
tags: [game, concept]
---

# Concept

A tower-defense variant: instead of building and upgrading static towers, the player starts with a fixed roster of `n` distinct units that move freely over the map and have their own special attacks. Instead of controlling them directly in real time, the player authors a behavior script for each unit; the scripts run autonomously once a wave starts. The skill being tested is writing good unit AI, not reflexes or base-building economy.

This resolves the genre fork that was open in earlier planning ([[00 Roadmap Overview]]) — what's left open now is scope, not identity.

Open sub-decisions (narrower than before, none of them block Phase 0/1 engineering — see [[03 Phase 0 - Foundation]]):

- [ ] Scripting language for player-authored AI — leaning **Rhai** (pure Rust, sandboxable, no FFI, good Bevy precedent) over Lua (`mlua`, needs FFI) or a custom DSL (full control, much more work). See [[05 Phase 2 - AI Scripting]].
- [ ] Number and identity of starting units (`n`) and their special attacks — pure content/balance, doesn't block engine work.
- [ ] Single-player only vs. anything leaderboard/sharing-adjacent — affects whether [[07 Phase 4 - Meta Rules]]'s score system needs a backend at all.

→ [[03 Phase 0 - Foundation]] for the build order.
