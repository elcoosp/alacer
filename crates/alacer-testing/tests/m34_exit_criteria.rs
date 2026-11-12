//! M34 exit-criteria integration tests (§14.2, ADR-0040): the backend
//! features are true. `backend-wgpu` and `backend-softbuffer` claimed a
//! capability the tree never delivered — the platform's backend modules,
//! enum, and construction paths compiled unconditionally, so every
//! single-backend (and no-backend) build failed at `cargo check` with
//! unresolved crates. M34 makes the features own their backends: each
//! present path's module, variant, construction arm, and resize path
//! compiles with its feature; a build with a backend compiled out still
//! builds the pipeline, the window plumbing, and — when the consumer
//! asks for the missing path — the typed refusal
//! ([`PlatformError::BackendUnavailable`], a fact about the build, never
//! a panic, never a munged fallback), and CI gains the feature matrix
//! that keeps the four combinations honest.
//!
//! The suite pins the contract the way the harness layer is allowed to
//! (the DAG drives everything except the facade): the cfg discipline in
//! the platform's own source, the CI job that exercises all four
//! combinations, and the facade's feature forwarding the consumer
//! configures through. The refusal's in-process shape — the display
//! naming its gate, the chain ending at the build fact — is pinned where
//! the type lives, in `alacer-platform`'s own suite (the m34 tests run
//! under every combination there: `cargo test -p alacer-platform` in the
//! features job, `--all-features` in the main one).

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves")
}

fn read_workspace_file(path: &str) -> String {
    let path = workspace_root().join(path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("required file {} missing: {error}", path.display()))
}

#[test]
fn m34_the_platform_compiles_its_backends_with_their_features() {
    // The cfg discipline in the platform's own source: each present
    // path's module and re-exports gate on the feature that owns them,
    // the backend enum's variants are the compiled backends, and the
    // preference resolution carries the typed refusal arm.
    let platform = read_workspace_file("crates/alacer-platform/src/lib.rs");
    for fragment in [
        "#[cfg(feature = \"backend-softbuffer\")]\npub mod softbuffer;",
        "#[cfg(feature = \"backend-wgpu\")]\npub mod wgpu_backend;",
        "fn build_wgpu_backend",
        "fn build_softbuffer_backend",
        "PlatformError::BackendUnavailable",
    ] {
        assert!(
            platform.contains(fragment),
            "the platform's feature truth is missing: {fragment}"
        );
    }

    // The no-backend build keeps its refusal honest: the wildcard arm
    // rides the same match the compiled arms ride.
    assert!(
        platform.contains(
            "#[cfg(not(all(feature = \"backend-wgpu\", feature = \"backend-softbuffer\")))]"
        ),
        "the preference resolution's refusal arm is gated to the builds that need it"
    );
}

#[test]
fn m34_ci_exercises_the_whole_feature_matrix() {
    // The gate that keeps the four combinations honest: the features
    // job checks the no-backend build (and runs its refusal tests),
    // each backend alone, and the facade's software-only and
    // pipeline-only builds.
    let ci = read_workspace_file(".github/workflows/ci.yml");
    for fragment in [
        "backend feature matrix (M34, ADR-0040)",
        "cargo check -p alacer-platform --no-default-features",
        "cargo test -p alacer-platform --no-default-features",
        "cargo check -p alacer-platform --no-default-features --features backend-wgpu",
        "cargo check -p alacer-platform --no-default-features --features backend-softbuffer",
        "cargo check -p alacer --no-default-features --features backend-softbuffer",
        "cargo check -p alacer --no-default-features",
    ] {
        assert!(
            ci.contains(fragment),
            "the CI feature matrix is missing: {fragment}"
        );
    }
}

#[test]
fn m34_the_facade_forwards_the_features_the_consumer_configures() {
    // The facade's feature forwarding is the consumer's interface: the
    // default carries both backends, each backend forwards to the
    // platform's own, and hot-reload stays independent of the present
    // paths.
    let facade = read_workspace_file("crates/alacer/Cargo.toml");
    for fragment in [
        "default = [\"backend-wgpu\", \"backend-softbuffer\"]",
        "backend-wgpu = [\"alacer-platform/backend-wgpu\"]",
        "backend-softbuffer = [\"alacer-platform/backend-softbuffer\"]",
        "hot-reload = [\"dep:subsecond\"]",
    ] {
        assert!(
            facade.contains(fragment),
            "the facade's feature forwarding drifted: {fragment}"
        );
    }
}
