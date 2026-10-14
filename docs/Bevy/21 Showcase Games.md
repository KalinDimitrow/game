---
tags: [bevy, showcase, games]
---

# 21. Showcase Games & Full Demos

The capstone: complete, playable (or at least complete) demos that combine most of the earlier stages — ECS, rendering, input, UI, audio, state — into one coherent thing. The best place to see how the pieces actually fit together in practice, as opposed to one feature in isolation.

## Representative examples

- `showcase/alien_cake_addict.rs` — a small 3D game: movement, collision-ish logic, scoring
- `showcase/breakout.rs` — the classic arcade game: 2D physics-ish movement, collision, score/UI
- `showcase/game_menu.rs` — a full menu system ([[15 State Management]] + [[11 UI]] in practice)
- `showcase/mines.rs` — minesweeper built entirely in Bevy UI
- `showcase/loading_screen.rs` — wait for assets *and* render pipelines to be ready before showing gameplay ([[06 Assets and Scenes]] done properly)
- `showcase/contributors.rs` — bouncing-ball physics-ish visualization, mostly for fun
- `showcase/desk_toy.rs` — transparent, always-on-top window turned into a desktop pet ([[01 Foundations - App and Window]]'s window tricks, pushed to an extreme)

## Where this fits

- ⬅ Comes from: [[20 Platform Targets]] (and really, every earlier stage)
- 🔗 Related: everything above — this is the stage where [[02 ECS Core]], [[11 UI]], [[15 State Management]], [[12 Audio]], and [[09 Input and Picking]] all show up in the same file

[[00 Bevy Capabilities Map|← Back to map]]
