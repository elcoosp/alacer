//! M46 exit-criteria integration tests (§13.3, ADR-0052): the drag's
//! story joins the causal log as one per-gesture record. ADR-0046's
//! second kill rule and ADR-0051's kill arm cash together: a consumer
//! needs the drag on the log — the value tier's trail answers the AT's
//! writes and the keyboard's steps, but the drag queued nothing, so a
//! trail reader could not answer "why did the slider's value move while
//! the hand dragged it" (the moves flooded the input ring, the value
//! ring stayed silent). The price the kill rule names: a per-*gesture*
//! record, never per-move — one `GestureRecord` when the drag starts
//! (the value it found) and one when it ends (the value it left the
//! control holding), the phases the record's own vocabulary names, the
//! control stamped by the dispatch funnel's pointer route (the M19
//! capture answers the *who* exactly the M43 action route does), and
//! every per-pointer-move fact staying off the log (the flood rule).
//!
//! Every test runs the real pipeline (A4) with a real slider driven by
//! real pointer gestures — the exact route a status line, an inspector
//! client, or a test reads.

use alacer_core::geom::Point;
use alacer_core::inspect::{GesturePhase, InspectEvent, InspectObserver};
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Slider};

/// The gesture-trail collector the tests install as the pipeline's
/// observer: the drag's facts stream into a bounded ring (the value
/// suite's helper shape — here the phases are the assertion source).
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
/// the m43 suite's shape, exactly.
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

/// One observed slider mounted through the real pipeline: the value
/// tier's drag is the gesture the record prices. Returns the harness,
/// the slider's node, and its track's center line.
fn mount_slider(observer: &Share) -> (TestHarness, alacer_core::NodeId, Point) {
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
    (harness, node, center)
}

/// The slider's id, as the funnel's stamp and the a11y tree both name
/// it — the identity the records must carry (the M43 vocabulary).
fn slider_id(harness: &TestHarness) -> u64 {
    harness.query(Role::Slider).named("Volume").first().id
}

#[test]
fn m46_the_drag_streams_two_records_start_and_end() {
    let observer = Share::new();
    let (mut harness, node, center) = mount_slider(&observer);
    let id = slider_id(&harness);

    // The press arms the gesture: one `Started` record streams the
    // moment the slider's door answers — the value it found (20.0, the
    // app's own), stamped with the control the hit test named.
    clear_trail(&observer);
    harness.pointer_press(center);
    let records = gesture_records(&observer);
    assert_eq!(records.len(), 1, "the press streams one fact");
    assert_eq!(records[0].phase, GesturePhase::Started);
    assert_eq!(records[0].from, 20.0, "the drag found the app's value");
    assert_eq!(
        records[0].to, None,
        "the landing is not the beginning's fact"
    );
    assert_eq!(
        records[0].target,
        Some(id),
        "the funnel's pointer route stamps the control the gesture rides"
    );

    // The release ends it: one `Ended` record — the value it left the
    // control holding, the beginning remembered beside it (each record
    // self-contained: a trail reader answers the drag's whole story from
    // one line).
    clear_trail(&observer);
    let right = Point::new(center.x + 40.0, center.y);
    harness.pointer_release(right);
    let records = gesture_records(&observer);
    assert_eq!(records.len(), 1, "the release streams one fact");
    assert_eq!(records[0].phase, GesturePhase::Ended);
    assert_eq!(records[0].from, 20.0, "the drag's beginning rides its end");
    let landed = harness
        .query(Role::Slider)
        .named("Volume")
        .first()
        .value
        .clone()
        .and_then(|v| v.parse::<f32>().ok())
        .expect("the tree's value reads as text");
    assert_eq!(
        records[0].to,
        Some(landed),
        "the landing is the tree's own value — the same vocabulary the AT reads"
    );
    assert_eq!(records[0].target, Some(id), "the end is attributed too");
    let _ = node;
    assert_eq!(
        harness.settle(),
        0,
        "the gesture's facts queue no work beyond the drag's own masks"
    );
}

#[test]
fn m46_the_flood_rule_holds_one_gesture_two_records() {
    let observer = Share::new();
    let (mut harness, _node, center) = mount_slider(&observer);

    // One hundred dispatched moves inside one pressed gesture: the
    // per-move facts stay off the log — exactly two records stream, the
    // phases the record prices (the kill rule's own words: a per-gesture
    // record first, never per-move).
    harness.pointer_press(center);
    clear_trail(&observer);
    for step in 0..100_u16 {
        let position = Point::new(center.x + f32::from(step) * 0.4, center.y);
        harness.pointer_move(position);
    }
    assert!(
        gesture_records(&observer).is_empty(),
        "the moves stream nothing — the flood rule"
    );
    harness.pointer_release(center);
    let records = gesture_records(&observer);
    assert_eq!(
        records.len(),
        1,
        "the gesture's end is the one fact the moves priced"
    );
    assert_eq!(records[0].phase, GesturePhase::Ended);
    assert_eq!(harness.settle(), 0, "the trail queues no extra work");
}

#[test]
fn m46_the_leave_ends_the_gesture_honestly() {
    let observer = Share::new();
    let (mut harness, _node, center) = mount_slider(&observer);

    // The press captures; the pointer leaving the window releases the
    // capture and the drag ends — the slider's own honest answer (the
    // widget's Leave ends the drag) streams the `Ended` phase, the
    // landing the last value the funnel committed.
    harness.pointer_press(center);
    clear_trail(&observer);
    harness
        .pipeline_mut()
        .dispatch_pointer(&alacer_core::events::PointerEvent::Leave);
    harness.settle();
    let records = gesture_records(&observer);
    assert_eq!(records.len(), 1, "the leave ends the gesture as data");
    assert_eq!(records[0].phase, GesturePhase::Ended);
    assert!(
        records[0].to.is_some(),
        "the landing is the value the drag left the control holding"
    );

    // The next press is a new gesture: the trail reads Started again,
    // the phases never conflating two gestures into one record.
    clear_trail(&observer);
    harness.pointer_press(center);
    let records = gesture_records(&observer);
    assert_eq!(records.len(), 1, "the second press is a new gesture");
    assert_eq!(records[0].phase, GesturePhase::Started);
}

#[test]
fn m46_the_wall_arms_nothing_and_the_hover_notes_nothing() {
    let observer = Share::new();
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .value(20.0)
            .enabled(false)
            .boxed(),
    );
    harness.settle();
    let bounds = harness.bounds(node);
    let center = Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + bounds.height() * 0.5,
    );

    // The availability tier's wall (M22, ADR-0028): the press is
    // consumed but arms nothing — no capture, no write, and now no
    // gesture fact either. A gesture that never was streams nothing;
    // the trail answers "why did nothing move" with silence that is
    // the honest absence, the wall's own.
    harness.pointer_press(center);
    let right = Point::new(center.x + 40.0, center.y);
    harness.pointer_move(right);
    harness.pointer_release(right);
    assert!(
        gesture_records(&observer).is_empty(),
        "the wall arms no gesture — no Started, no Ended"
    );
    assert_eq!(
        harness.query(Role::Slider).named("Volume").first().value,
        Some("20".to_string()),
        "the tree keeps the value the app wrote"
    );

    // The hover and the wheel are not value gestures: without a press
    // in flight the moves and the wheel stream nothing either.
    let observer2 = Share::new();
    let (mut harness2, _node2, center2) = mount_slider(&observer2);
    harness2.pointer_move(Point::new(center2.x + 50.0, center2.y));
    harness2.wheel(center2, (0.0, 12.0));
    assert!(
        gesture_records(&observer2).is_empty(),
        "the hover and the wheel are not gestures"
    );
}

#[test]
fn m46_no_observer_no_trail_the_idle_contract_holds() {
    // The default path: no observer, no enabled trail — the drag answers
    // exactly as before and the frame costs what it always cost (the
    // A7 idle contract: the stamp is one `Cell` read per routed event).
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
    harness.drag(center, Point::new(center.x + 60.0, center.y));
    let landed = harness
        .query(Role::Slider)
        .named("Volume")
        .first()
        .value
        .clone()
        .and_then(|v| v.parse::<f32>().ok())
        .expect("the value landed as text");
    assert!(
        landed > 20.0,
        "the drag wrote the value exactly as it always wrote it: {landed}"
    );
    assert_eq!(harness.settle(), 0, "the disabled trail queues no work");
}

#[test]
fn m46_the_records_ride_the_one_sink_the_trails_share() {
    // The gesture facts ride the observer's own stream — the same
    // `InspectEvent` the other trails ride, drained at the tick's start
    // after the value drain (a gesture explains the value the pointer
    // moved; it streams beside it) — never a second observation channel.
    let observer = Share::new();
    let (mut harness, _node, center) = mount_slider(&observer);
    harness.pointer_press(center);
    harness.pointer_release(center);
    harness.settle();
    let records = gesture_records(&observer);
    assert_eq!(
        records.len(),
        2,
        "press and release each streamed one fact through the one sink"
    );
    assert_eq!(records[0].phase, GesturePhase::Started);
    assert_eq!(records[1].phase, GesturePhase::Ended);
}

#[test]
fn m46_the_living_documents_re_agree_at_the_cut() {
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
        readme.contains("M0 through M57") && readme.contains("sixty-four decision records"),
        "the README's status block and count re-agree at the cut"
    );
    assert!(
        architecture.contains("64 decision records"),
        "the architecture register re-agrees at 64 decision records"
    );
    // The protocol document carries the gesture event (the wire's
    // documented shape).
    assert!(
        protocol.contains("`gesture`") && protocol.contains("per-gesture"),
        "the protocol document documents the gesture event"
    );
    // The catalog lists the demonstration.
    assert!(
        catalog.contains("drag_gesture"),
        "the examples catalog lists the drag trail demo"
    );
    // The changelog stages the milestone.
    assert!(
        changelog.contains("M46 — the drag's story joins the log"),
        "the changelog stages M46"
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
