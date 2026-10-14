---
tags: [bevy, app, window]
---

# 01. Foundations — App & Window

Every Bevy project starts here: build an `App`, decide which plugins it needs, and (usually) open a window. This is the only stage where "no rendering at all" is a legitimate, well-supported configuration — useful for servers, CLI tools, and tests.

## Representative examples

- `app/empty.rs` — an `App` that does nothing; the literal floor
- `app/empty_defaults.rs` — same, but with `DefaultPlugins` (window, rendering, input, audio...)
- `app/plugin.rs` — write your own `Plugin`
- `app/plugin_group.rs` — bundle plugins together
- `app/headless.rs` / `app/headless_renderer.rs` — run without a window, optionally still rendering to an image
- `app/custom_loop.rs` — replace the default runner to drive `App::update()` yourself
- `app/settings.rs` — persist app settings across runs
- `app/without_winit.rs` — run without the winit event loop entirely
- `window/window_settings.rs` — title, size, resizability, decorations
- `window/multiple_windows.rs` — more than one OS window from one app
- `window/transparent_window.rs`, `window/custom_cursor_image.rs` — window chrome tricks

## Where this fits

- ➡ Leads to: [[02 ECS Core]] — once the app boots, you need a way to put things in it
- 🔗 Related: [[16 Async Tasks and Remote Protocol]] (custom runners/headless apps often pair with external control), [[20 Platform Targets]] (window setup differs on wasm/mobile)

[[00 Bevy Capabilities Map|← Back to map]]
