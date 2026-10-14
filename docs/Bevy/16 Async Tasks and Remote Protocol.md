---
tags: [bevy, async, networking, tooling]
---

# 16. Async Tasks & Remote Protocol

Two related "the main loop isn't enough" capabilities: offloading slow work to background tasks without blocking frames, and exposing/controlling a running app from an *external* process via the Bevy Remote Protocol (BRP).

## Representative examples — Async Tasks

- `async_tasks/async_compute.rs` — `AsyncComputeTaskPool` for longer-running work
- `async_tasks/async_channel_pattern.rs` — communicate results back via channels
- `async_tasks/external_source_external_thread.rs` — a plain OS thread feeding data in via a channel

## Representative examples — Remote Protocol (BRP)

- `remote/server.rs` — a Bevy app that exposes itself to BRP for live editing
- `remote/app_under_test.rs` — an app meant to be controlled externally
- `remote/client.rs` — a standalone CLI client that talks BRP
- `remote/integration_test.rs` — connect via BRP, find a button, click it — i.e. black-box UI testing of a running app

## Where this fits

- ⬅ Comes from: [[15 State Management]]
- ➡ Leads to: [[17 Time and Math]]
- 🔗 Related: [[08 Reflection]] (BRP is reflection exposed over the network), [[06 Assets and Scenes]] (asset loading is itself async), [[18 Diagnostics and Dev Tools]] (BRP is effectively an external dev tool)

[[00 Bevy Capabilities Map|← Back to map]]
