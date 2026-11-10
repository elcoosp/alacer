//! M18 exit-criteria integration tests (§11.6, ADR-0024): the announcement
//! seam joins the invalidation currency. The app's voice — state changes
//! that own no widget — ships as the reserved live-region node in pass 9's
//! snapshot: the same delta, the same `Change::A11Y` masks, no second
//! channel. The region is present from the first frame, last write wins,
//! and the refusals (empty text, no root) are honest — no masks, an idle
//! frame. The surface work rides the same milestone: the last two `String`
//! error leaves carry their causes, the dead convenience APIs are gone,
//! the living documents re-agree, and the corpus grows 53 → 54 by the
//! coordinated rule.
//!
//! Every test runs the real pipeline (A4) with the real sink — the exact
//! route an assistive technology's adapter drains.

use accesskit::Live;
use alacer_a11y::{AccessKitSink, find_in_update};
use alacer_core::announce::ANNOUNCEMENT_NODE_HASH;
use alacer_core::announce::Politeness;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label};
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

fn parse_workspace_json(path: &str) -> serde_json::Value {
    let text = read_workspace_file(path);
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path} is not valid JSON: {error}"))
}

/// A mounted app whose announcement routes drive the pipeline — the
/// example's window, assembled the consumer's way.
fn mount_app(harness: &mut TestHarness) {
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let _ = harness.add_child(root, Label::new("The announcements").boxed());
    harness.settle();
}

/// The live-region node in the sink's last `TreeUpdate`, with the
/// properties an adapter forwards to the AT.
fn region_in_last_update(
    harness: &mut TestHarness,
) -> (accesskit::Live, accesskit::Role, Option<String>) {
    let sink = harness
        .pipeline_mut()
        .a11y_sink_mut()
        .as_any_mut()
        .and_then(|any| any.downcast_mut::<AccessKitSink>())
        .expect("the harness carries the AccessKit sink — the windowed composition");
    let update = sink
        .last_update()
        .expect("the a11y pass produced an update");
    let region = find_in_update(update, ANNOUNCEMENT_NODE_HASH)
        .expect("the reserved region rides the update");
    (
        region
            .live()
            .expect("the region carries its aria-live priority"),
        region.role(),
        region.label().map(String::from),
    )
}

#[test]
fn m18_an_announcement_rides_the_sink_delta() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_app(&mut harness);

    // The region is the root's last child from the first frame, named
    // empty — announcing never moves the tree's shape.
    let tree = harness
        .pipeline()
        .a11y_snapshot()
        .expect("the snapshot is built");
    let root = tree.root.expect("a root is mounted");
    let root_node = tree.by_id(root).expect("the root node");
    assert_eq!(
        root_node.children.last().copied(),
        Some(ANNOUNCEMENT_NODE_HASH),
        "the reserved region is the root's last child from the first frame"
    );

    let accepted = harness.pipeline_mut().announce("Saved", Politeness::Polite);
    assert!(accepted, "a rooted, non-empty announcement is accepted");
    let report = harness.tick();
    assert!(
        report.a11y_ran && !report.layout_ran && !report.paint_ran,
        "the announcement is the a11y pass's trigger and nothing else runs"
    );
    assert_eq!(
        report.a11y_delta.updated.len(),
        1,
        "the delta re-emits the region and nothing else"
    );
    assert!(
        report.a11y_delta.updated.contains(&ANNOUNCEMENT_NODE_HASH),
        "the region's id is the delta's content"
    );

    let (live, role, label) = region_in_last_update(&mut harness);
    assert_eq!(live, Live::Polite, "the polite priority rides the update");
    assert_eq!(
        role,
        accesskit::Role::Status,
        "the ARIA pairing: a polite announcement speaks as a status region"
    );
    assert_eq!(label.as_deref(), Some("Saved"), "the AT reads the text");
}

#[test]
fn m18_assertive_interrupts_as_an_alert() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_app(&mut harness);

    harness
        .pipeline_mut()
        .announce("Connection lost", Politeness::Assertive);
    harness.tick();
    let (live, role, label) = region_in_last_update(&mut harness);
    assert_eq!(live, Live::Assertive, "the AT reads the interrupt priority");
    assert_eq!(
        role,
        accesskit::Role::Alert,
        "the ARIA pairing: an assertive announcement speaks as an alert"
    );
    assert_eq!(label.as_deref(), Some("Connection lost"));
}

#[test]
fn m18_rapid_announcements_replace_last_write_wins() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_app(&mut harness);

    // Two calls before one tick: the region is one node, the batch is one
    // spoken fact — the second announcement replaces the first, never an
    // unbounded queue inside a frame.
    harness.pipeline_mut().announce("Saved", Politeness::Polite);
    harness
        .pipeline_mut()
        .announce("3 items left", Politeness::Polite);
    let report = harness.tick();
    assert!(report.a11y_ran);
    assert_eq!(
        report.a11y_delta.updated.len(),
        1,
        "the two calls ride one region delta"
    );
    let (_, _, label) = region_in_last_update(&mut harness);
    assert_eq!(label.as_deref(), Some("3 items left"), "last write wins");
}

#[test]
fn m18_the_handler_route_queues_and_the_tick_routes() {
    // The consumer's door: an event handler cannot hold the pipeline, so
    // the ambient `announce` fn queues the speech act (the clipboard-seam
    // shape, ADR-0020; the input-queue shape, ADR-0014) and the next tick
    // drains it into the same reserved region — one destination, no second
    // channel.
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_app(&mut harness);

    let accepted = alacer_core::announce::announce("3 items left", Politeness::Polite);
    assert!(accepted, "the queued route accepts a non-empty act");
    let report = harness.tick();
    assert!(report.a11y_ran, "the drain is the pass's trigger");
    assert_eq!(
        report.a11y_delta.updated.len(),
        1,
        "the drained act rides one region delta"
    );
    let (_, _, label) = region_in_last_update(&mut harness);
    assert_eq!(label.as_deref(), Some("3 items left"));

    // The empty refusal answers at the queue, before any frame.
    assert!(!alacer_core::announce::announce("  ", Politeness::Polite));
    assert_eq!(harness.settle(), 0, "a refused act queues nothing");
}

#[test]
fn m18_refusals_stay_honest_at_the_announcement_tier() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_app(&mut harness);
    let counters = harness.counters();

    // An empty announcement speaks nothing — refused, no masks, idle.
    assert!(!harness.pipeline_mut().announce("", Politeness::Polite));
    assert!(!harness.pipeline_mut().announce("   ", Politeness::Polite));
    assert_eq!(
        harness.settle(),
        0,
        "a refused announcement queues no masks"
    );
    assert!(harness.pipeline().announcement().is_none());

    // Before a root is mounted there is no tree to attach the region to —
    // the same honest refusal on a fresh pipeline.
    let mut fresh = TestHarness::new(200.0, 100.0);
    assert!(!fresh.pipeline_mut().announce("Saved", Politeness::Polite));
    assert!(fresh.pipeline().announcement().is_none());

    assert_eq!(
        harness.counters().a11y_runs,
        counters.a11y_runs,
        "no a11y pass ran for a refusal"
    );
}

#[test]
fn m18_the_vocabulary_lives_in_core() {
    // One definition, no widget dialect: the reserved hash, the
    // politeness, and the replace rule live beside the seams the a11y
    // contract already owns; the pipeline only consumes them.
    let announce = read_workspace_file("crates/alacer-core/src/announce.rs");
    assert!(
        announce.contains("ANNOUNCEMENT_NODE_HASH"),
        "the reserved identity is a named constant, one definition"
    );
    assert!(
        announce.contains("Last write wins"),
        "the replace rule is recorded in core, one definition"
    );
    let passes = read_workspace_file("crates/alacer-core/src/passes.rs");
    assert!(
        passes.contains("pub fn announce("),
        "the route is the pipeline's — no second channel"
    );
    assert!(
        !passes.contains("has_queued_input"),
        "the dead convenience read is gone (the platform pushes and never drains)"
    );
    let sink = read_workspace_file("crates/alacer-a11y/src/lib.rs");
    assert!(
        sink.contains("map_announcement_role"),
        "the sink maps the ARIA pairing, polite status / assertive alert"
    );
}

#[test]
fn m18_the_last_string_leaves_grow_their_chains() {
    // The M16/M17 error-chain discipline completes: PlatformError::Softbuffer
    // and BackendError::SurfaceCreation carry their structured causes, and
    // the dead convenience APIs are gone from the facade and the view.
    let platform = read_workspace_file("crates/alacer-platform/src/lib.rs");
    assert!(
        platform.contains("Softbuffer(::softbuffer::SoftBufferError)"),
        "the softbuffer variant carries the structured present-path error"
    );
    let backend = read_workspace_file("crates/alacer-platform/src/wgpu_backend.rs");
    assert!(
        backend.contains("SurfaceCreation(wgpu::CreateSurfaceError)"),
        "the surface-creation variant carries the structured wgpu error"
    );
    let hot = read_workspace_file("crates/alacer/src/hot.rs");
    assert!(
        !hot.contains("run_hot_window_with_channel"),
        "the dead convenience assembly is gone — the example wires the contract it needs"
    );
    let view_hot = read_workspace_file("crates/alacer-view/src/hot.rs");
    assert!(
        !view_hot.contains("pub const fn registry"),
        "the dead registry accessor is gone"
    );
}

#[test]
fn m18_the_announce_case_stays_published_as_the_corpus_moves() {
    // The announce criterion case stays published with its baseline and
    // its regression contract; every M-suite pin moves with the corpus —
    // 57 since M22's measurement run (the refused-press route joined there).
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let id = "announce/route_100_announcements_1k";

    let text = serde_json::to_string(&baselines).expect("baselines serialize");
    assert!(
        text.contains(id),
        "{id} must be published in baselines.json"
    );
    let text = serde_json::to_string(&contracts).expect("contracts serialize");
    assert!(text.contains(id), "{id} must carry a regression contract");

    for suite in [
        "crates/alacer-testing/tests/m5_exit_criteria.rs",
        "crates/alacer-testing/tests/m6_exit_criteria.rs",
        "crates/alacer-testing/tests/m7_exit_criteria.rs",
        "crates/alacer-testing/tests/m8_exit_criteria.rs",
        "crates/alacer-testing/tests/m9_exit_criteria.rs",
        "crates/alacer-testing/tests/m10_exit_criteria.rs",
        "crates/alacer-testing/tests/m11_exit_criteria.rs",
        "crates/alacer-testing/tests/m12_exit_criteria.rs",
        "crates/alacer-testing/tests/m13_exit_criteria.rs",
        "crates/alacer-testing/tests/m14_exit_criteria.rs",
        "crates/alacer-testing/tests/m15_exit_criteria.rs",
    ] {
        let source = read_workspace_file(suite);
        assert!(
            source.contains("const EXPECTED_BENCH_COUNT: usize = 60;"),
            "{suite} must pin the 60-case corpus (moved with the corpus at M22)"
        );
    }
}
