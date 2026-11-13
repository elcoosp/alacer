//! M53 exit-criteria integration tests (§13.3, ADR-0059): the wheel's
//! line count joins the scroll record. ADR-0058's second kill rule
//! cashes: a consumer needs the wheel's line count named — the
//! coalesced run's notional lines — so the record grows the count
//! beside the wheel species ("the run is the coalescer's story, not a
//! unit ask; today the distance is derivable from the ask and the
//! landing" — the distance the clamp's landing cannot reconstruct).
//! The count prices the run by the platform's own convention — one
//! line = 40 px, §10.2 — so a notch answers 1.0 and the trackpad's
//! pixel runs answer their honest fraction. The count rides the
//! honest absence everywhere else: the AT's unit scrolls ask by their
//! unit (M52), the value steps by the page, the app's own door and
//! the content's own clamp (M51) are stories with no wheel run behind
//! them, and the non-finite run prices no count at all (the refusal
//! names the wall).
//!
//! Every test runs the real pipeline (A4) with a real list driven by
//! the real pointer route and the real AT action route — the exact
//! paths a mouse wheel and a screen reader's scroll command ride.

use alacer_core::geom::{Point, Vector};
use alacer_core::inspect::{InspectEvent, InspectObserver, ScrollAction, ScrollOutcome};
use alacer_core::widget::{A11yAction, EventResponse, Role, ScrollUnit};
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};

/// The scroll-trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the run's facts are the assertion source).
struct TrailObserver {
    events: std::collections::VecDeque<InspectEvent>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            events: std::collections::VecDeque::with_capacity(64),
        }
    }

    /// The streamed scroll facts, oldest first.
    fn scrolls(&self) -> Vec<alacer_core::ScrollRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::Scroll(record) => Some(*record),
                _ => None,
            })
            .collect()
    }
}

impl InspectObserver for TrailObserver {
    fn observe(&mut self, event: InspectEvent) {
        if self.events.len() >= 64 {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the m-suite's shape, exactly.
struct Share(std::rc::Rc<std::cell::RefCell<TrailObserver>>);

impl Share {
    fn new() -> Self {
        Self(std::rc::Rc::new(std::cell::RefCell::new(
            TrailObserver::new(),
        )))
    }
}

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

/// The streamed scroll facts, oldest first.
fn scroll_records(observer: &Share) -> Vec<alacer_core::ScrollRecord> {
    observer.0.borrow().scrolls()
}

/// Empties the shared ring between beats: each assertion reads the
/// facts the ask under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// A list mounted through the real pipeline with a content height the
/// viewport can scroll: 500 px of content in a 100 px viewport — a
/// 400 px span, the walls at 0 and 400 (the m50 suite's shape). The
/// node rides back for the wheel beats: the scroll container's own
/// bounds translate with the offset (compose moves the viewport), so a
/// beat aims at the bounds' current center, wherever the scroll put
/// them.
fn mount_scrollable_list(
    observer: &Share,
) -> (
    TestHarness,
    alacer_widgets::ListHandle,
    alacer_core::arena::NodeId,
) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let list = List::new(100.0);
    let handle = list.handle();
    let node = harness.add_child(root, list.boxed());
    handle.set_content_height(500.0);
    harness.settle();
    (harness, handle, node)
}

/// One AT ask routed through the real action route (the exact path a
/// screen reader's scroll command rides): the consumed verdict.
fn at_ask(harness: &mut TestHarness, action: A11yAction) -> bool {
    let id = harness.query(Role::ScrollArea).named("list").first().id;
    harness.pipeline_mut().dispatch_a11y_action(id, action)
}

/// One wheel routed through the real pointer route with the run's own
/// delta, aimed at the list's *current* bounds (the scroll container's
/// own bounds translate with the offset — compose moves the viewport —
/// so the beat reads them per ask): the response the route gave. The
/// delta is y-signed as the platform delivers it (negative scrolls
/// down, §10.2).
fn wheel(
    harness: &mut TestHarness,
    node: alacer_core::arena::NodeId,
    delta_y: f32,
) -> EventResponse {
    let bounds = harness.bounds(node);
    let position = Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + bounds.height() * 0.5,
    );
    harness.queue_pointer(alacer_core::events::PointerEvent::Wheel {
        delta: Vector::new(0.0, delta_y),
        position,
    });
    harness.settle();
    EventResponse::Handled
}

#[test]
fn m53_the_wheel_run_prices_its_lines() {
    let observer = Share::new();
    let (mut harness, _handle, node) = mount_scrollable_list(&observer);

    // The one-notch run: -40 px is one line by the platform's own
    // convention (one line = 40 px, §10.2) — the count rides beside
    // the wheel species, the magnitude the pointer asked by.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, -40.0);
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "one run, one fact");
    assert_eq!(records[0].action, ScrollAction::Wheel);
    assert_eq!(records[0].outcome, ScrollOutcome::Delivered);
    assert_eq!(records[0].offset, 40.0);
    assert_eq!(
        records[0].lines,
        Some(1.0),
        "one notch prices one line (the platform's own convention, ADR-0059)"
    );

    // The multi-notch run: a coalesced -80 px asks two lines — the
    // run's own magnitude, not the landing's.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, -80.0);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].offset, 120.0, "40 + the run's 80 px");
    assert_eq!(
        records[0].lines,
        Some(2.0),
        "the coalesced run prices its own magnitude"
    );

    // The trackpad's pixel run: a -100 px flick prices its honest
    // fraction — 2.5 lines — never a rounded notch count.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, -100.0);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].offset, 220.0);
    assert_eq!(
        records[0].lines,
        Some(2.5),
        "the pixel run prices its honest fraction (the convention's own arithmetic)"
    );

    // The reverse run: scrolling up prices the same magnitude — the
    // count is the run's size, the direction rides the offsets.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, 80.0);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].offset, 140.0, "220 - the run's 80 px");
    assert_eq!(
        records[0].lines,
        Some(2.0),
        "the count prices the magnitude — the direction rides the offsets"
    );
}

#[test]
fn m53_a_pinned_run_still_names_the_run_it_was() {
    let observer = Share::new();
    let (mut harness, handle, node) = mount_scrollable_list(&observer);

    // The run past the wall: a -10_000 px slam prices 250 lines — the
    // count computed before the clamp arithmetic, the landing pinned
    // at the max. The count is the ask's story; the landing, the
    // clamp's — one record carries both.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, -10_000.0);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the slam streamed its fact");
    assert_eq!(records[0].outcome, ScrollOutcome::Delivered);
    assert_eq!(
        records[0].offset, 400.0,
        "the landing pins at content minus viewport"
    );
    assert_eq!(
        records[0].pinned,
        Some(alacer_core::RangeBound::Max),
        "the wall the clamp pinned names itself (M50)"
    );
    assert_eq!(
        records[0].lines,
        Some(250.0),
        "the run prices what the pointer asked — the clamp never rewrote it"
    );
    assert_eq!(
        records[0].requested,
        Some(10_000.0),
        "the ask beside the landing (the M44 rule at the scroll tier)"
    );
    assert_eq!(handle.offset(), 400.0);
}

#[test]
fn m53_a_non_finite_run_prices_no_count() {
    let observer = Share::new();
    let (mut harness, handle, node) = mount_scrollable_list(&observer);

    // The NaN run: the refusal names the wall (M50), the ask rides
    // its absence (the M44 rule), and the count rides the same
    // absence — a count of a run that never was would be the lie the
    // additive field's contract forbids.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, f32::NAN);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the refusal streams its fact");
    assert_eq!(
        records[0].outcome,
        ScrollOutcome::Refused(alacer_core::ScrollRefusal::NonFiniteAsk),
        "the funnel carries only finite asks — the wall is its own fact"
    );
    assert_eq!(
        records[0].requested, None,
        "the wire cannot carry an infinite ask honestly"
    );
    assert_eq!(
        records[0].lines, None,
        "the non-finite run prices no count — the absence the refusal names"
    );
    assert_eq!(
        records[0].offset, 0.0,
        "the state keeps the offset the ask found"
    );
    assert_eq!(handle.offset(), 0.0, "the corruption is gone");

    // The next good run prices like nothing happened.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, -40.0);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].lines, Some(1.0), "the list still scrolls");
}

#[test]
fn m53_the_stories_without_a_run_ride_the_absence() {
    let observer = Share::new();
    let (mut harness, handle, _node) = mount_scrollable_list(&observer);

    // The AT's unit scroll: the ask rides its unit (M52) and no run —
    // the count is the wheel door's story alone.
    clear_trail(&observer);
    assert!(at_ask(
        &mut harness,
        A11yAction::ScrollDown(ScrollUnit::Item)
    ));
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the unit scroll answered");
    assert_eq!(records[0].action, ScrollAction::ScrollDown);
    assert_eq!(records[0].unit, Some(ScrollUnit::Item));
    assert_eq!(
        records[0].lines, None,
        "the unit scroll names its unit — a second magnitude would be the same story told twice"
    );

    // The value step: the page is the step, not a run.
    clear_trail(&observer);
    assert!(at_ask(&mut harness, A11yAction::Increment));
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].action, ScrollAction::Increment);
    assert_eq!(records[0].lines, None, "the step is not a run");

    // The app's own door: a programmatic ask with no run in it.
    clear_trail(&observer);
    handle.scroll_to(200.0);
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].action, ScrollAction::ScrollTo);
    assert_eq!(
        records[0].lines, None,
        "the app door rides no run — the honest absence"
    );

    // The content's own clamp (M51): a story with no ask and no run.
    clear_trail(&observer);
    handle.set_content_height(150.0);
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the clamp answered");
    assert_eq!(records[0].action, ScrollAction::ContentClamp);
    assert_eq!(
        records[0].lines, None,
        "the clamp rides no run — the content's story"
    );
}

#[test]
fn m53_the_surface_rides_the_same_cut() {
    // The seam ships where the vocabulary lives: the record field in
    // core, the wheel door's pricing, the additive wire field, the
    // demonstration in the catalog.
    let inspect = read_workspace_file("crates/alacer-core/src/inspect.rs");
    assert!(
        inspect.contains("pub lines: Option<f32>"),
        "the record grows the count beside the wheel species"
    );
    let list = read_workspace_file("crates/alacer-widgets/src/list.rs");
    for fragment in [
        "let lines = delta.y.is_finite().then(|| delta.y.abs() / A11Y_LINE)",
        "lines,",
    ] {
        assert!(
            list.contains(fragment),
            "the wheel door prices the run where the delta is decoded: {fragment}"
        );
    }
    let wire = read_workspace_file("crates/alacer-inspect/src/wire.rs");
    assert!(
        wire.contains("lines: Option<f32>"),
        "the wire carries the additive lines field"
    );

    // The decision record itself documents the scope.
    let adr = read_workspace_file("docs/adr/ADR-0059-m53-scope.md");
    assert!(
        adr.contains("M53 scope") && adr.contains("lines"),
        "ADR-0059 documents the wheel's line-count scope"
    );
}

#[test]
fn m53_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // features bullet carries the fact, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M54",
        "sixty-one decision records",
        "Since M53 the wheel's coalesced runs name their line count too",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `wheel_lines` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("[`wheel_lines`](../crates/alacer/examples/wheel_lines.rs)"),
        "docs/examples.md carries the demonstration"
    );

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0059",
        "61 decision records",
        "the wheel's line count joins the scroll record",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The protocol document carries the wire vocabulary.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`lines`") && protocol.contains("`1.0`"),
        "the protocol document carries the lines field"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M53 — the wheel's line count joins the scroll record (ADR-0059)**"),
        "the CHANGELOG carries the M53 entry"
    );

    // Every pin the M-count move touched moved together — the suites'
    // document pins ride the same cut, the chain cannot drift.
    for suite in ["m7", "m19", "m24", "m35", "m43", "m49", "m50", "m51", "m52"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("M0 through M54"),
            "the {suite} suite's README pin moved with the documents"
        );
    }
    for suite in ["m19", "m24", "m35", "m43", "m49", "m50", "m51", "m52"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("61 decision records"),
            "the {suite} suite's architecture pin moved with the register"
        );
    }
    for suite in ["m24", "m35", "m43", "m49", "m50", "m51", "m52"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("sixty-one decision records"),
            "the {suite} suite's README status pin moved with the documents"
        );
    }
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
