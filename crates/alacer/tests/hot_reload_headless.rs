//! The hot-reload demo's headless proof, run as a test: the §13.2 contract
//! — a patch swaps code, never state — executed by the example itself in
//! its `--headless` mode, no window, no patcher, the assert-and-exit shape
//! the CI hot-patch job already drives the example with.
//!
//! The test executes the built example binary with `--headless` and
//! asserts the proof's own success line. Machines without the built
//! example skip with the reason named — the honest refusal, the wgpu
//! fuzz's own discipline, never a fake pass. CI's `hot-patch` job builds
//! the examples first, so the proof runs for real there.

/// The example binary's path, resolved from this test's crate — built by
/// `cargo build -p alacer --example hot_reload` (the CI hot-patch job's
/// first step).
fn example_binary() -> Option<std::path::PathBuf> {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(manifest).join("../../target/debug/examples/hot_reload");
    path.is_file().then_some(path)
}

#[test]
fn the_hot_reload_example_proves_itself_headless() {
    let Some(binary) = example_binary() else {
        eprintln!(
            "SKIPPED: the hot_reload example is not built — run `cargo build \
             -p alacer --example hot_reload` (the CI hot-patch job builds it)"
        );
        return;
    };

    let output = std::process::Command::new(&binary)
        .arg("--headless")
        .output()
        .expect("the example binary spawns");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "the headless proof failed (exit {:?}): {stdout}{stderr}",
        output.status.code()
    );
    // The proof's own report: the patch's shape and the state that
    // survived it, printed for the run's transcript.
    for expected in ["created: 0", "removed: 0", "patched:", "state intact"] {
        assert!(
            stdout.contains(expected),
            "the proof's report is missing `{expected}`: {stdout}"
        );
    }
}
