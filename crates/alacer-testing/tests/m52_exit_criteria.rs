//! M52 exit-criteria integration tests (§13.3, ADR-0058): the asker's
//! unit joins the scroll record. ADR-0056's first kill rule cashes: a
//! consumer needs the asker's unit named — one item or one page — on
//! the AT's unit scrolls, so the record grows the unit beside the
//! action species ("the distance is derivable from the ask and the
//! landing today; the growth waits for a consumer that cannot derive
//! it" — the consumer the M50 demonstration priced). The unit rides
//! the honest absence everywhere else: the wheel's notch is a coalesced
//! run, not a unit ask; the value step's page is the step, not a unit
//! scroll; the app's own door and the content's own clamp (M51) are
//! stories with no unit at all.
//!
//! Every test runs the real pipeline (A4) with a real list driven by
//! the real AT action route and the real pointer route — the exact
//! paths a screen reader's scroll command and a status line ride.

use alacer_core::geom::{Point, Vector};
use alacer_core::inspect::{InspectEvent, InspectObserver, ScrollAction, ScrollOutcome};
use alacer_core::widget::{A11yAction, EventResponse, Role, ScrollUnit};
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};

/// The scroll-trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the unit facts are the assertion source).
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
fn m52_the_unit_scroll_names_its_unit() {
    let observer = Share::new();
    let (mut harness, _handle, _node) = mount_scrollable_list(&observer);

    // The AT's item scroll through the real action route: the record
    // names the asker's unit beside the action species — one item, the
    // line the ask asked by (the kill rule's own words).
    clear_trail(&observer);
    assert!(at_ask(
        &mut harness,
        A11yAction::ScrollDown(ScrollUnit::Item)
    ));
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "one ask, one fact");
    assert_eq!(records[0].action, ScrollAction::ScrollDown);
    assert_eq!(records[0].outcome, ScrollOutcome::Delivered);
    assert_eq!(records[0].offset, 40.0, "one item = one line = 40 px");
    assert_eq!(
        records[0].unit,
        Some(ScrollUnit::Item),
        "the asker's unit rides beside the action (ADR-0056's first kill rule)"
    );

    // The page scroll: the other unit, the same vocabulary — one page,
    // the viewport the ask asked by.
    clear_trail(&observer);
    assert!(at_ask(
        &mut harness,
        A11yAction::ScrollDown(ScrollUnit::Page)
    ));
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].offset, 140.0, "40 + one page (100 px)");
    assert_eq!(
        records[0].unit,
        Some(ScrollUnit::Page),
        "one page = the viewport, the unit named beside the action"
    );

    // The up direction: the unit rides the reverse ask too.
    clear_trail(&observer);
    assert!(at_ask(&mut harness, A11yAction::ScrollUp(ScrollUnit::Item)));
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].action, ScrollAction::ScrollUp);
    assert_eq!(records[0].offset, 100.0);
    assert_eq!(records[0].unit, Some(ScrollUnit::Item));
}

#[test]
fn m52_the_stories_without_a_unit_ride_the_absence() {
    let observer = Share::new();
    let (mut harness, handle, node) = mount_scrollable_list(&observer);

    // The value step (M10: the scrollbar's value *is* the offset): a
    // page-wide step. M52's cut rode the absence — the kill rule's
    // own words holding the door open — and M54 (ADR-0060) cashed it:
    // the step's page is a unit-like distance, attributed the same
    // way, riding beside the species now.
    clear_trail(&observer);
    assert!(at_ask(&mut harness, A11yAction::Increment));
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the step answered");
    assert_eq!(records[0].action, ScrollAction::Increment);
    assert_eq!(
        records[0].unit,
        Some(ScrollUnit::Page),
        "the step's page rides beside the species (M54, ADR-0060 — the kill rule cashed)"
    );

    // The wheel: a coalesced run the pointer asked, never a unit ask —
    // the absence again.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the wheel answered");
    assert_eq!(records[0].action, ScrollAction::Wheel);
    assert_eq!(records[0].unit, None, "the wheel rides no unit");

    // The app's own door: a programmatic ask with no unit in it.
    clear_trail(&observer);
    handle.scroll_to(200.0);
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].action, ScrollAction::ScrollTo);
    assert_eq!(records[0].unit, None, "the app door rides no unit");

    // The content's own clamp (M51): a story with no ask has no unit
    // either.
    clear_trail(&observer);
    handle.set_content_height(150.0);
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the clamp answered");
    assert_eq!(records[0].action, ScrollAction::ContentClamp);
    assert_eq!(records[0].unit, None, "the clamp rides no unit");
}

#[test]
fn m52_the_surface_rides_the_same_cut() {
    // The seam ships where the vocabulary lives: the unit's wire tag on
    // the widget enum, the record field in core, the additive wire
    // field, the demonstration in the catalog.
    let widget = read_workspace_file("crates/alacer-core/src/widget.rs");
    for fragment in ["Self::Item => \"item\"", "Self::Page => \"page\""] {
        assert!(
            widget.contains(fragment),
            "the unit's wire tag ships on the vocabulary: {fragment}"
        );
    }
    let inspect = read_workspace_file("crates/alacer-core/src/inspect.rs");
    assert!(
        inspect.contains("pub unit: Option<crate::widget::ScrollUnit>"),
        "the record grows the unit beside the action"
    );
    let wire = read_workspace_file("crates/alacer-inspect/src/wire.rs");
    assert!(
        wire.contains("unit: Option<String>"),
        "the wire carries the additive unit field"
    );
    let list = read_workspace_file("crates/alacer-widgets/src/list.rs");
    for fragment in ["unit: if matches!(", "Some(unit)"] {
        assert!(
            list.contains(fragment),
            "the unit-scroll doors fill the asker's unit: {fragment}"
        );
    }

    // The decision record itself documents the scope.
    let adr = read_workspace_file("docs/adr/ADR-0058-m52-scope.md");
    assert!(
        adr.contains("M52 scope") && adr.contains("unit"),
        "ADR-0058 documents the unit scope"
    );
}

#[test]
fn m52_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // features bullet carries the fact, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M54",
        "sixty-one decision records",
        "Since M52 the AT's unit scrolls name their unit too",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `scroll_units` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("[`scroll_units`](../crates/alacer/examples/scroll_units.rs)"),
        "docs/examples.md carries the demonstration"
    );

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0058",
        "61 decision records",
        "the asker's unit joins the scroll record",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The protocol document carries the wire vocabulary.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`unit`") && protocol.contains("`item`") && protocol.contains("`page`"),
        "the protocol document carries the unit field"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M52 — the asker's unit joins the scroll record (ADR-0058)**"),
        "the CHANGELOG carries the M52 entry"
    );

    // Every pin the M-count move touched moved together — the suites'
    // document pins ride the same cut, the chain cannot drift.
    for suite in ["m7", "m19", "m24", "m35", "m43", "m49", "m50", "m51"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("M0 through M54"),
            "the {suite} suite's README pin moved with the documents"
        );
    }
    for suite in ["m19", "m24", "m35", "m43", "m49", "m50", "m51"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("61 decision records"),
            "the {suite} suite's architecture pin moved with the register"
        );
    }
    for suite in ["m24", "m35", "m43", "m49", "m50", "m51"] {
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
