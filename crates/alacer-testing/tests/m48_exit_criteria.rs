//! M48 exit-criteria integration tests (§13.3, ADR-0054): the drag's
//! wall joins the causal log as a named fact. ADR-0053's first kill
//! rule cashes: a consumer needs the bound jump the *pointer* asked
//! for — a drag that slams an edge — so the M46 gesture record's
//! vocabulary grows: the `Ended` record carries the bound the drag's
//! landing pinned ([`alacer_core::inspect::GestureRecord::bound`],
//! [`alacer_core::inspect::RangeBound::Min`]/[`alacer_core::inspect::RangeBound::Max`]),
//! never the AT tier's species — the drag tells its own story in the
//! gesture vocabulary it already owns, the wall named beside the
//! landing it pinned.
//!
//! Every test runs the real pipeline (A4) with a real slider driven by
//! real pointer gestures — the exact route a status line, an inspector
//! client, or a test reads.

use alacer_core::geom::Point;
use alacer_core::inspect::{GesturePhase, InspectEvent, InspectObserver, RangeBound};
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Slider};

/// The gesture-trail collector the tests install as the pipeline's
/// observer: the drag's facts stream into a bounded ring (the m46
/// suite's helper shape — here the bounds are the assertion source).
struct GestureObserver {
    gestures: std::collections::VecDeque<alacer_core::inspect::GestureRecord>,
}

impl GestureObserver {
    fn new() -> Self {
        Self {
            gestures: std::collections::VecDeque::with_capacity(64),
        }
    }
}

impl InspectObserver for GestureObserver {
    fn observe(&mut self, event: InspectEvent) {
        if let InspectEvent::Gesture(record) = event {
            if self.gestures.len() >= 64 {
                self.gestures.pop_front();
            }
            self.gestures.push_back(record);
        }
    }
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the m-suite's shape, exactly.
struct Share(std::rc::Rc<std::cell::RefCell<GestureObserver>>);

impl Share {
    fn new() -> Self {
        Self(std::rc::Rc::new(std::cell::RefCell::new(
            GestureObserver::new(),
        )))
    }
}

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

/// The streamed gesture facts, oldest first.
fn gesture_records(observer: &Share) -> Vec<alacer_core::inspect::GestureRecord> {
    observer.0.borrow().gestures.iter().copied().collect()
}

/// Empties the shared ring between beats: each assertion reads the facts
/// the gesture under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().gestures.clear();
}

/// One observed slider mounted through the real pipeline: the drag's
/// wall is the fact the record prices. Returns the harness, the
/// slider's node id, its center line, and its track's bounds.
fn mount_slider(
    observer: &Share,
) -> (
    TestHarness,
    alacer_core::NodeId,
    Point,
    alacer_core::geom::Rect,
) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .value(20.0)
            .step(1.0)
            .boxed(),
    );
    harness.settle();
    let bounds = harness.bounds(node);
    let center = Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + bounds.height() * 0.5,
    );
    (harness, node, center, bounds)
}

#[test]
fn m48_the_slam_names_its_wall() {
    let observer = Share::new();
    let (mut harness, _node, center, bounds) = mount_slider(&observer);

    // The drag slams the right edge: the press arms the gesture, the
    // move past the track's end clamps at the max (the range is a
    // wall), and the release lands there — the `Ended` record names
    // the bound the landing pinned, beside the landing itself.
    harness.pointer_press(center);
    clear_trail(&observer);
    harness.pointer_move(Point::new(bounds.max_x() + 50.0, center.y));
    harness.pointer_release(Point::new(bounds.max_x() + 50.0, center.y));
    let records = gesture_records(&observer);
    assert_eq!(records.len(), 1, "the release streams one fact");
    assert_eq!(records[0].phase, GesturePhase::Ended);
    assert_eq!(
        records[0].to,
        Some(100.0),
        "the drag landed at the range's maximum"
    );
    assert_eq!(
        records[0].bound,
        Some(RangeBound::Max),
        "the landing pinned the max — the wall is a named fact"
    );
    assert_eq!(harness.settle(), 0, "the trail queues no extra work");
}

#[test]
fn m48_the_started_record_carries_no_bound() {
    let observer = Share::new();
    let (mut harness, _node, center, _bounds) = mount_slider(&observer);

    // The beginning is not the wall's fact: a `Started` record carries
    // the value the drag found and the honest absence beside it — the
    // landing (and so the bound) is the gesture's answer, never its
    // beginning's.
    harness.pointer_press(center);
    let records = gesture_records(&observer);
    assert_eq!(records.len(), 1, "the press streams one fact");
    assert_eq!(records[0].phase, GesturePhase::Started);
    assert_eq!(
        records[0].bound, None,
        "the beginning carries no bound — the landing is the answer's fact"
    );
    assert_eq!(harness.settle(), 0, "the trail queues no extra work");
}

#[test]
fn m48_the_mid_landing_carries_no_bound() {
    let observer = Share::new();
    let (mut harness, _node, center, _bounds) = mount_slider(&observer);

    // A drag that lands mid-track pinned no wall: the `Ended` record
    // carries the honest absence — the bound names only a landing at
    // the range's own edge, never a value that merely sits nearby.
    harness.pointer_press(center);
    clear_trail(&observer);
    harness.pointer_move(Point::new(center.x + 8.0, center.y));
    harness.pointer_release(Point::new(center.x + 8.0, center.y));
    let records = gesture_records(&observer);
    assert_eq!(records.len(), 1, "the release streams one fact");
    assert_eq!(records[0].phase, GesturePhase::Ended);
    assert_eq!(records[0].bound, None, "a mid-track landing pinned no wall");
    assert!(records[0].to.is_some(), "the landing still rides");
    assert_eq!(harness.settle(), 0, "the trail queues no extra work");
}

#[test]
fn m48_the_leave_at_the_wall_names_it() {
    let observer = Share::new();
    let (mut harness, _node, center, bounds) = mount_slider(&observer);

    // The leave ends the drag honestly (the capture released) — and a
    // drag the leave resolves at the range's minimum names the wall
    // exactly the release's answer does: one gesture vocabulary, both
    // doors, one fact.
    harness.pointer_press(center);
    harness.pointer_move(Point::new(bounds.min_x() - 50.0, center.y));
    clear_trail(&observer);
    harness
        .pipeline_mut()
        .dispatch_pointer(&alacer_core::events::PointerEvent::Leave);
    harness.settle();
    let records = gesture_records(&observer);
    assert_eq!(records.len(), 1, "the leave ends the gesture as data");
    assert_eq!(records[0].phase, GesturePhase::Ended);
    assert_eq!(
        records[0].to,
        Some(0.0),
        "the drag landed at the range's minimum"
    );
    assert_eq!(
        records[0].bound,
        Some(RangeBound::Min),
        "the leave's landing pinned the min — the same named wall"
    );
    assert_eq!(harness.settle(), 0, "the trail queues no extra work");
}

#[test]
fn m48_the_species_names_ride_the_wire_vocabulary() {
    // The wire's `snake_case` tags (the protocol's stable, documented
    // field values): the bound names itself the way the phases and the
    // refusal species do.
    assert_eq!(RangeBound::Min.name(), "min");
    assert_eq!(RangeBound::Max.name(), "max");
}

#[test]
fn m48_no_observer_no_trail_the_idle_contract_holds() {
    // No observer, no trail: the drag works (the wall still holds the
    // value), nothing queues, and the frame stays idle — the A7
    // contract the trails ride (the m47 suite's own pin).
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .value(20.0)
            .step(1.0)
            .boxed(),
    );
    harness.settle();
    let bounds = harness.bounds(node);
    let center = Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + bounds.height() * 0.5,
    );
    harness.pointer_press(center);
    harness.pointer_move(Point::new(bounds.max_x() + 50.0, center.y));
    harness.pointer_release(Point::new(bounds.max_x() + 50.0, center.y));
    let value = harness
        .query(Role::Slider)
        .named("Volume")
        .first()
        .value
        .clone()
        .and_then(|v| v.parse::<f32>().ok())
        .expect("the tree's value reads as text");
    assert!(
        (value - 100.0).abs() < f32::EPSILON,
        "the wall held the write"
    );
    assert_eq!(harness.settle(), 0, "no observer, no queued facts");
}

#[test]
fn m48_the_living_documents_re_agree_at_the_cut() {
    let root = workspace_root();
    let readme = std::fs::read_to_string(root.join("README.md")).expect("readme");
    let architecture =
        std::fs::read_to_string(root.join("docs/architecture.md")).expect("architecture");
    let protocol = std::fs::read_to_string(root.join("docs/inspector-protocol.md"))
        .expect("protocol document");
    let catalog = std::fs::read_to_string(root.join("docs/examples.md")).expect("catalog");
    let changelog = std::fs::read_to_string(root.join("CHANGELOG.md")).expect("changelog");

    // The status block and the ADR count re-agree at the cut (the
    // m-suite pins move across the chain together — the workspace's
    // one-milestone-per-minor rule counting the cut's entries).
    assert!(
        readme.contains("M0 through M49") && readme.contains("fifty-six decision records"),
        "the README's status block and count re-agree at the cut"
    );
    assert!(
        architecture.contains("56 decision records"),
        "the architecture register re-agrees at 56 decision records"
    );
    // The protocol document carries the bound vocabulary (the wire's
    // documented field).
    assert!(
        protocol.contains("\"bound\""),
        "the protocol document documents the gesture's bound field"
    );
    // The catalog lists the demonstration.
    assert!(
        catalog.contains("drag_bounds"),
        "the examples catalog lists the drag bounds demo"
    );
    // The changelog stages the milestone.
    assert!(
        changelog.contains("M48 — the drag's wall is a named fact"),
        "the changelog stages M48"
    );
}

/// The workspace root, walked up from this test file's own directory —
/// the m-suite's `CARGO_MANIFEST_DIR` route.
fn workspace_root() -> std::path::PathBuf {
    let manifest = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest)
        .ancestors()
        .nth(2)
        .expect("the testing crate sits two levels below the workspace root")
        .to_path_buf()
}
