//! M54 exit-criteria integration tests (§13.3, ADR-0060): the value
//! step's page joins the scroll record. ADR-0058's first kill rule
//! cashes: a consumer needs the step's unit named — the value step's
//! page is a unit-like distance a reader may want attributed the same
//! way — so the record grows the unit beside the step species too
//! ("the kill rule's words named the AT's unit scrolls alone; the
//! step's page rides the absence until a consumer prices it"). The
//! step steps a page — always, by the action vocabulary's own contract
//! (M10, ADR-0016) — and the record names it now: `Some(ScrollUnit::Page)`
//! beside the `Increment`/`Decrement` species, the same field the unit
//! scrolls answer by (M52), the honest absence keeping its other
//! stories (the wheel's run — a run is not an ask, and M53 gave it its
//! own magnitude — the app door, the content clamp).
//!
//! Every test runs the real pipeline (A4) with a real list driven by
//! the real AT action route — the exact path a screen reader's scroll
//! command rides.

use alacer_core::geom::{Point, Vector};
use alacer_core::inspect::{InspectEvent, InspectObserver, ScrollAction, ScrollOutcome};
use alacer_core::widget::{A11yAction, EventResponse, Role, ScrollUnit};
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};

/// The scroll-trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the step's facts are the assertion source).
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
/// 400 px span, the walls at 0 and 400 (the m50 suite's shape).
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

/// One wheel routed through the real pointer route, aimed at the
/// list's *current* bounds (the scroll container's own bounds
/// translate with the offset — compose moves the viewport — so the
/// beat reads them per ask): the response the route gave.
fn wheel(harness: &mut TestHarness, node: alacer_core::arena::NodeId) -> EventResponse {
    let bounds = harness.bounds(node);
    let position = Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + bounds.height() * 0.5,
    );
    harness.queue_pointer(alacer_core::events::PointerEvent::Wheel {
        delta: Vector::new(0.0, -40.0),
        position,
    });
    harness.settle();
    EventResponse::Handled
}

#[test]
fn m54_the_value_step_names_its_page() {
    let observer = Share::new();
    let (mut harness, handle, _node) = mount_scrollable_list(&observer);

    // The increment: a page-wide step, the page it stepped by riding
    // beside the species — the kill rule's own words cashed ("the
    // step's page is a unit-like distance a reader may want
    // attributed the same way").
    clear_trail(&observer);
    assert!(at_ask(&mut harness, A11yAction::Increment));
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the step answered");
    assert_eq!(records[0].action, ScrollAction::Increment);
    assert_eq!(records[0].outcome, ScrollOutcome::Delivered);
    assert_eq!(records[0].offset, 100.0, "one page = the viewport (100 px)");
    assert_eq!(
        records[0].unit,
        Some(ScrollUnit::Page),
        "the step's page rides beside the species (ADR-0058's first kill rule)"
    );
    assert_eq!(
        records[0].lines, None,
        "the step carries the unit and no count — the count is the wheel's story (M53)"
    );

    // The decrement past the low wall: from a mid-range offset the
    // page-sized ask overshoots — the landing pins at zero, the wall
    // naming itself (M50), the page riding beside the species.
    clear_trail(&observer);
    handle.scroll_to(30.0);
    harness.settle();
    clear_trail(&observer);
    assert!(at_ask(&mut harness, A11yAction::Decrement));
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].action, ScrollAction::Decrement);
    assert_eq!(records[0].offset, 0.0, "30 - one page, clamped at zero");
    assert_eq!(
        records[0].unit,
        Some(ScrollUnit::Page),
        "the reverse step names the same page"
    );
    assert_eq!(
        records[0].pinned,
        Some(alacer_core::RangeBound::Min),
        "the clamp pinned the landing at the low wall (M50)"
    );
}

#[test]
fn m54_the_stories_without_a_unit_still_ride_the_absence() {
    let observer = Share::new();
    let (mut harness, handle, node) = mount_scrollable_list(&observer);

    // The wheel's coalesced run: a run is not an ask — the unit rides
    // its honest absence (M52's own words), and M53 gave the run its
    // own magnitude beside the unit's seat (the count, not the unit).
    clear_trail(&observer);
    let _ = wheel(&mut harness, node);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the wheel answered");
    assert_eq!(records[0].action, ScrollAction::Wheel);
    assert_eq!(records[0].unit, None, "the wheel rides no unit");
    assert_eq!(records[0].lines, Some(1.0), "the wheel prices its run");

    // The app's own door: a programmatic ask with no unit in it.
    clear_trail(&observer);
    handle.scroll_to(200.0);
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].action, ScrollAction::ScrollTo);
    assert_eq!(records[0].unit, None, "the app door rides no unit");

    // The content's own clamp (M51): a story with no unit either.
    clear_trail(&observer);
    handle.set_content_height(150.0);
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the clamp answered");
    assert_eq!(records[0].action, ScrollAction::ContentClamp);
    assert_eq!(records[0].unit, None, "the clamp rides no unit");
}

#[test]
fn m54_the_surface_rides_the_same_cut() {
    // The seam ships where the vocabulary lives: the door's fill, the
    // record's contract, the decision record itself.
    let list = read_workspace_file("crates/alacer-widgets/src/list.rs");
    for fragment in [
        "| ScrollAction::Increment",
        "| ScrollAction::Decrement",
        "Some(unit)",
    ] {
        assert!(
            list.contains(fragment),
            "the AT action door fills the step's page: {fragment}"
        );
    }
    let inspect = read_workspace_file("crates/alacer-core/src/inspect.rs");
    for fragment in ["ADR-0060", "the step's page beside the"] {
        assert!(
            inspect.contains(fragment),
            "the record's contract names the step's page cut: {fragment}"
        );
    }

    // The decision record itself documents the scope.
    let adr = read_workspace_file("docs/adr/ADR-0060-m54-scope.md");
    assert!(
        adr.contains("M54 scope") && adr.contains("page"),
        "ADR-0060 documents the step's page scope"
    );
}

#[test]
fn m54_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // features bullet carries the fact, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M61",
        "sixty-eight decision records",
        "Since M54 the value steps name their page too",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `step_pages` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("[`step_pages`](../crates/alacer/examples/step_pages.rs)"),
        "docs/examples.md carries the demonstration"
    );

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0060",
        "68 decision records",
        "the value step's page joins the scroll record",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The protocol document carries the wire vocabulary.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("the value step's page rides beside the step species"),
        "the protocol document carries the step's page"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M54 — the value step's page joins the scroll record (ADR-0060)**"),
        "the CHANGELOG carries the M54 entry"
    );

    // Every pin the M-count move touched moved together — the suites'
    // document pins ride the same cut, the chain cannot drift.
    for suite in [
        "m7", "m19", "m24", "m35", "m43", "m49", "m50", "m51", "m52", "m53",
    ] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("M0 through M61"),
            "the {suite} suite's README pin moved with the documents"
        );
    }
    for suite in [
        "m19", "m24", "m35", "m43", "m49", "m50", "m51", "m52", "m53",
    ] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("68 decision records"),
            "the {suite} suite's architecture pin moved with the register"
        );
    }
    for suite in ["m24", "m35", "m43", "m49", "m50", "m51", "m52", "m53"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("sixty-eight decision records"),
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
