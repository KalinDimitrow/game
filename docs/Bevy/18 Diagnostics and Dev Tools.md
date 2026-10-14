---
tags: [bevy, diagnostics, dev-tools]
---

# 18. Diagnostics & Dev Tools

Tooling for understanding your own app as it grows past the point where `println!` is enough: FPS/frame-time tracking, custom metrics, log layers, and editor-ish overlays.

## Representative examples

- `diagnostics/log_diagnostics.rs` — log FPS and friends to the console
- `diagnostics/custom_diagnostic.rs` — register your own diagnostic
- `diagnostics/enabling_disabling_diagnostic.rs` — toggle diagnostics at runtime
- `dev_tools/fps_overlay.rs` — an on-screen FPS overlay
- `dev_tools/infinite_grid.rs` — editor-style ground-plane grid
- `dev_tools/schedule_data.rs` — dump a schedule's structure to a file, for understanding system ordering
- `app/log_layers.rs`, `app/log_layers_ecs.rs` — custom `tracing` log layers, including piping log data into the ECS
- `tools/gamepad_viewer.rs` — visualize gamepad button/stick/trigger state live

## Where this fits

- ⬅ Comes from: [[17 Time and Math]]
- ➡ Leads to: [[19 Stress Tests and Testbeds]]
- 🔗 Related: [[14 Gizmos and Debug Drawing]] (gizmos are the drawing half of debugging; this is the metrics half), [[16 Async Tasks and Remote Protocol]] (BRP overlaps with "external dev tool")

[[00 Bevy Capabilities Map|← Back to map]]
