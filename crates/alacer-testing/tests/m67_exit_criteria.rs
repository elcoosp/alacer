//! M67 exit-criteria integration tests (§13.3, ADR-0073): the adapter's
//! full-tree echo joins the causal log. M59's pre-registered kill rule
//! cashes — a consumer prices the adapter activation's timing and the
//! a11y sink's full-tree fallback (the AT connected or reconnected; the
//! app answered with the whole resolved tree) and the answer was traced
//! and gone — a trail reader could not answer "why did the app just
//! push the whole tree, and how big was it" from the log alone. The a11y
//! tier grows its own record ([`alacer_core::inspect::A11yEchoRecord`],
//! the tree named by its node count — never the payload, the lengths
//! rule) and the thread trail below, the route-drop trail's shape: the
//! note site is the exact place the echo resolves — the sink's
//! full-tree build — and the dispatching pipeline drains it at the tick's
//! start, ahead of the frame's routes (a connect predates the asks), into
//! [`alacer_core::inspect::InspectEvent::A11yEcho`], dev-mode and
//! bounded, silent while nobody listens.
//!
//! Every test runs the real pipeline (A4): the echo routed the exact way
//! the platform's activation route rides it — the erased `A11ySink`
//! downcast to the [`AccessKitSink`], the full-tree build — through the
//! harness's trail door.

use alacer_a11y::AccessKitSink;
use alacer_core::inspect::InspectEvent;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label, Panel};

/// The echo route the platform's activation response rides: reach the
/// erased a11y sink, downcast, build the full tree — the m9 suite's own
/// mechanism, now the note site's caller (ADR-0073).
fn with_sink<R>(harness: &mut TestHarness, f: impl FnOnce(&mut AccessKitSink) -> R) -> R {
    let sink = harness
        .pipeline_mut()
        .a11y_sink_mut()
        .as_any_mut()
        .and_then(|any| any.downcast_mut::<AccessKitSink>())
        .expect("the harness carries the AccessKit sink — the windowed composition");
    f(sink)
}

/// The echo route, end to end: the trail installed first (the note
/// rides the observer's own enable — the trail door's contract), then
/// the activation echo the platform answers, then the tick whose start
/// drain streams it, then the drained window oldest first.
fn route_echo(harness: &mut TestHarness) -> Option<accesskit::TreeUpdate> {
    let _trail = harness.trail();
    with_sink(harness, |sink| sink.full_tree_update())
}

/// The drained window after one echo route: the trail reinstalled, the
/// frame ticked, the facts handed over oldest first. A window without
/// a new build behind it answers no echo (the fact streamed once).
fn drained_events(harness: &mut TestHarness) -> Vec<InspectEvent> {
    let trail = harness.trail();
    let _ = harness.tick();
    trail.drain()
}

/// The resolved tree's node count — the a11y snapshot's own size, the
/// count the echo must name honestly.
fn snapshot_nodes(harness: &TestHarness) -> usize {
    harness
        .pipeline()
        .a11y_snapshot()
        .map(|tree| tree.walk(None).len())
        .expect("the tree is resolved after settle")
}

/// A small resolved tree: the root column, a heading label, and three
/// filler panels — enough for the echo's count to be a real answer
/// (five nodes, the mount's own shape).
fn mount_filler_tree(harness: &mut TestHarness) {
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Label::new("The echo demo").boxed());
    for _ in 0..3 {
        let _ = harness.add_child(root, Panel::new(0.2, alacer_widgets::theme::PANEL).boxed());
    }
    harness.settle();
}

#[test]
fn m67_the_echo_names_the_tree_the_activation_answered() {
    let mut harness = TestHarness::new(240.0, 320.0);
    mount_filler_tree(&mut harness);

    // The AT connects: the platform answers from the sink's resolved
    // tree — the same builder the deltas use — and the echo resolves
    // where the build happens (the trail riding from the route's own
    // install).
    let built = route_echo(&mut harness).expect("the tree is resolved");

    // The fact streams at the next tick's start drain, ahead of the
    // frame's routes: the connect predates the asks, the echo reads
    // first.
    let events = drained_events(&mut harness);
    let Some(InspectEvent::A11yEcho(record)) = events.first().cloned() else {
        let head = if events.is_empty() {
            "(empty)".to_string()
        } else {
            "another fact".to_string()
        };
        panic!("the a11y echo leads the drain, found {head}");
    };
    assert_eq!(
        record.nodes,
        built.nodes.len(),
        "the echo names the very tree the activation answered"
    );
    assert!(
        record.nodes > 0,
        "the resolved tree is not empty — the echo names a real size"
    );
}

#[test]
fn m67_the_echo_counts_the_resolved_tree() {
    let mut harness = TestHarness::new(240.0, 320.0);
    mount_filler_tree(&mut harness);

    let built = route_echo(&mut harness).expect("the tree is resolved");
    let events = drained_events(&mut harness);
    let Some(InspectEvent::A11yEcho(record)) = events
        .iter()
        .find(|event| matches!(event, InspectEvent::A11yEcho(_)))
        .cloned()
    else {
        panic!("the echo streamed for the build");
    };

    assert_eq!(
        record.nodes,
        snapshot_nodes(&harness),
        "the echo's count is the resolved tree's own size"
    );
    assert_eq!(
        record.nodes,
        built.nodes.len(),
        "the echo's count is the build's own answer, verbatim"
    );
}

#[test]
fn m67_the_echo_streams_once_per_build() {
    let mut harness = TestHarness::new(240.0, 320.0);
    mount_filler_tree(&mut harness);

    let _ = route_echo(&mut harness).expect("the tree is resolved");
    let first = drained_events(&mut harness);
    assert_eq!(
        first
            .iter()
            .filter(|event| matches!(event, InspectEvent::A11yEcho(_)))
            .count(),
        1,
        "the echo streamed on the first drain"
    );

    // The next frame answers no echo: the fact streamed once — the echo
    // is the build's own fact, not a frame-rate companion (the flood
    // rule; a connect is rare, the log stays quiet between connects).
    let second = drained_events(&mut harness);
    assert!(
        !second
            .iter()
            .any(|event| matches!(event, InspectEvent::A11yEcho(_))),
        "the echo never repeats without a new build"
    );

    // A reconnect builds again — and the echo answers again: every
    // activation names the tree it answered, one fact per build.
    let _ = route_echo(&mut harness).expect("the tree stays resolved");
    let third = drained_events(&mut harness);
    assert_eq!(
        third
            .iter()
            .filter(|event| matches!(event, InspectEvent::A11yEcho(_)))
            .count(),
        1,
        "the reconnect's build echoes once, the same honesty"
    );
}

#[test]
fn m67_the_unbuilt_tree_is_no_echo() {
    let mut harness = TestHarness::new(240.0, 320.0);

    // Before the first frame there is no resolved tree: the build
    // answers `None` (the m9 suite's own pin) and no echo queues — the
    // honest absence, the trail never inventing a size.
    let built = route_echo(&mut harness);
    assert!(built.is_none(), "no tree, no echo to name");
    assert!(
        !drained_events(&mut harness)
            .iter()
            .any(|event| matches!(event, InspectEvent::A11yEcho(_))),
        "an unanswered activation streams no fact"
    );
}

#[test]
fn m67_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // trail sentence carries the fact, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M155",
        "a hundred and sixty decision records",
        "Since M67 the adapter's echo streams its fact too",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `a11y_echo` |"),
        "the README's example table carries the demonstration"
    );

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0073",
        "a hundred and sixty decision records",
        "the adapter's full-tree echo joins the causal log",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The protocol document carries the wire vocabulary: the echo's
    // node count beside the kind.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`a11y_echo`") && protocol.contains("`nodes`"),
        "the protocol document carries the a11y echo's fact"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog
            .contains("**M67 — the adapter's full-tree echo joins the causal log (ADR-0073)**"),
        "the CHANGELOG carries the M67 entry"
    );

    // The catalog: the demonstration walks the echo beat.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("the tree the activation answered"),
        "docs/examples.md carries the echo demonstration"
    );

    // The sibling suites' pins ride the same cut (one sweep, no drift).
    let m66 = read_workspace_file("crates/alacer-testing/tests/m66_exit_criteria.rs");
    assert!(
        m66.contains("M0 through M155"),
        "the m66 suite's pins moved with the documents"
    );
}

/// Reads a workspace file relative to the manifest the suite runs from
/// (the m-suite's own helper shape: `CARGO_MANIFEST_DIR` walks up to
/// the workspace root).
fn read_workspace_file(path: &str) -> String {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest dir");
    let workspace = std::path::Path::new(&manifest)
        .ancestors()
        .nth(2)
        .expect("the suite rides two directories under the workspace root");
    std::fs::read_to_string(workspace.join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}
