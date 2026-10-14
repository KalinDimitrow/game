---
tags: [bevy, audio]
---

# 12. Audio

Playing sound — effects, music, and spatial (positional) audio. Smaller and more self-contained than most other stages; it mostly plugs into events and game state rather than other rendering systems.

## Representative examples

- `audio/audio.rs` — load and play a file, the baseline
- `audio/audio_control.rs` — pause/volume/speed control over a playing sound
- `audio/play_sound_effect.rs` — fire a one-shot sound in response to a game event
- `audio/soundtrack.rs` — switch background music based on game state
- `audio/spatial_audio_2d.rs`, `audio/spatial_audio_3d.rs` — positional audio tied to a moving emitter
- `audio/pitch.rs` — play a raw generated pitch, no asset file
- `audio/decodable.rs` — implement a custom audio source type

## Where this fits

- ⬅ Comes from: [[11 UI]]
- ➡ Leads to: [[13 Shaders and Custom Rendering]]
- 🔗 Related: [[15 State Management]] (`soundtrack.rs` switches tracks on state), [[03 Transforms and Camera]] (spatial audio follows a `Transform`)

[[00 Bevy Capabilities Map|← Back to map]]
