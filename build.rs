use std::path::PathBuf;
use std::process::Command;

/// Make `./target/<profile>/game` launchable without going through `cargo run`.
///
/// `bevy/dynamic_linking` turns both Bevy and libstd into shared objects, and
/// Cargo normally papers over that by setting `LD_LIBRARY_PATH` for the process
/// *it* spawns. Launching the binary yourself (or from an IDE run config) gets
/// no such help, so the two NEEDED libraries have to be reachable via rpath:
///
/// * `libbevy_dylib-<hash>.so` — Cargo uplifts this into target/<profile>/ but
///   drops the hash from the filename, while the loader searches for the SONAME,
///   which keeps it. Only target/<profile>/deps/ has a correctly-named copy.
/// * `libstd-<hash>.so` — lives in the toolchain, not in target/ at all.
///
/// `profile.dev.rpath` alone does not cover either case: it emits `$ORIGIN/`
/// (wrong name) plus a `$ORIGIN/../../..`-style path to the toolchain that is
/// off by a directory and resolves to nothing.
///
/// These go through `rustc-link-arg-bins` rather than `rustflags` so they stay
/// out of the dependency fingerprint — via rustflags, every change here would
/// rebuild all of Bevy.
fn main() {
    println!("cargo::rustc-link-arg-bins=-Wl,-rpath,$ORIGIN/deps");

    if let Some(dir) = host_std_lib_dir() {
        println!("cargo::rustc-link-arg-bins=-Wl,-rpath,{}", dir.display());
    }

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=RUSTC");
}

/// Absolute path to the active toolchain's shared-library directory, as an
/// absolute path so it does not depend on where the binary sits. Returns `None`
/// if `rustc` cannot be queried, in which case we simply skip the flag rather
/// than fail the build.
fn host_std_lib_dir() -> Option<PathBuf> {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let target = std::env::var("TARGET").ok()?;

    let output = Command::new(rustc).arg("--print").arg("sysroot").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let sysroot = String::from_utf8(output.stdout).ok()?;
    let dir = PathBuf::from(sysroot.trim())
        .join("lib")
        .join("rustlib")
        .join(target)
        .join("lib");

    dir.is_dir().then_some(dir)
}
