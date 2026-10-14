---
tags: [game, roadmap]
---

# Phase 2 — AI Scripting (the actual game)

This is the differentiator. Everything here reuses Phase 1's mechanics — it doesn't replace them, it exposes them to player-authored code instead of Rust call sites.

9. **Scripting runtime & sandboxing** *(added — not on the original list but load-bearing for the whole concept)* — embed a script language. Rhai is the natural fit for Bevy/Rust: pure Rust, no FFI/`unsafe`, and supports per-call instruction-count limits — a buggy or adversarial player script must not be able to hang a tick (cf. Screeps' CPU-budget model). Lua (`mlua`) is the alternative if player-facing syntax matters more than embedding simplicity.
10. **Script API: sensors & actions** — read-only sensors (nearby enemies, own HP/cooldowns, terrain info) and callable actions (`move_to`, `attack`, `use_skill`), bound to the systems built in Phase 1.
11. **Script editor & tooling (UI)** — in-game text editor panel, per-unit script assignment, error/debug output, hot-reload without restarting the level. The most complex single piece of UI in the game — lands here, ahead of the rest of [[08 Phase 5 - UI and Feedback]].
12. **Script persistence** *(added)* — save/load player-authored scripts to disk. Keep separate from general save/load state ([[09 Phase 6 - Persistence and Polish]]) — scripts are the player's actual creative output, not incidental state.

→ [[06 Phase 3 - Waves and Opposition]]
