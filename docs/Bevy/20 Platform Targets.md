---
tags: [bevy, wasm, mobile, no_std]
---

# 20. Platform Targets

Porting beyond "native desktop." Each target has its own constraints: wasm runs in a browser with no filesystem and (usually) WebGL2/WebGPU limits; mobile needs a different app-lifecycle model and touch-first input; `no_std` strips the Rust standard library entirely for embedded-class environments.

## What's here

- `examples/wasm/` — currently empty in-tree (the README's "Wasm" section is about build/run instructions, not example files); see the platform-specific build docs in `examples/README.md` for `wasm-bindgen`/`wasm-server-runner` setup, WebGL2 vs WebGPU, and asset loading caveats in-browser
- `examples/mobile/` — currently empty in-tree; Android/iOS setup covered in `examples/README.md` (`cargo-apk` for Android, Xcode project for iOS)
- `no_std/library/src/lib.rs` — a library crate written to compile under both `std` and `no_std`
- `reflection/auto_register_static/src/lib.rs` — automatic reflection type registration for platforms without the `inventory` crate's support (a `no_std`/constrained-platform concern)

## Where this fits

- ⬅ Comes from: [[19 Stress Tests and Testbeds]]
- ➡ Leads to: [[21 Showcase Games]]
- 🔗 Related: [[01 Foundations - App and Window]] (window/runner setup differs most here), [[12 Audio]] (browser audio has its own unlock-on-interaction quirks, noted in the README)

[[00 Bevy Capabilities Map|← Back to map]]
