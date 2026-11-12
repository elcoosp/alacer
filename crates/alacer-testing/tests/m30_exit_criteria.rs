//! M30 exit-criteria integration tests (§13.3, ADR-0036): the causal trail
//! names the rider. The rule ADR-0035 pre-registered ships — "a consumer
//! needs the rides distinguished in the causal log → grow `FocusRecord`
//! beside its `movement` (a rider field), priced by a real reader — never
//! a second trail". One funnel, three riders: the keyboard's Tab fallback
//! (the M24 seam), the ring's directional keys (the M29 seam), and the
//! app's own door calls — until now the trail carried the movement, the
//! scope, and the outcome, but not *who* rode, so a status line, an
//! inspector client, or a test could not answer "did the user navigate,
//! or did the app?". The record grows the rider: a `FocusRider` the
//! funnel records when the door resolves, the wire carrying its name
//! beside the movement (the protocol's additive rule — new optional
//! fields are compatible, `PROTOCOL_VERSION` stays 1), and the walk
//! behind the doors is untouched.
//!
//! Every test runs the real pipeline (A4) with the real layout host —
//! the exact route a status line or a client's trail pane reads.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{FocusRecord, FocusRider, InspectEvent, InspectObserver};
use alacer_core::passes::{FocusMove, FocusMoveOutcome};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, Row};

/// The causal-trail collector the tests install as the pipeline's
/// observer: the focus records stream into a bounded ring (the m28
/// helper's shape — here the rider is the assertion source).
struct TrailObserver {
    focus: std::collections::VecDeque<FocusRecord>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            focus: std::collections::VecDeque::with_capacity(64),
        }
    }
}

impl InspectObserver for TrailObserver {
    fn observe(&mut self, event: InspectEvent) {
        if let InspectEvent::Focus(record) = event {
            if self.focus.len() >= 64 {
                self.focus.pop_front();
            }
            self.focus.push_back(record);
        }
    }
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the recorder's producer/consumer split, in the suite's own shape.
struct Share(std::rc::Rc<std::cell::RefCell<TrailObserver>>);

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

/// The two-by-two grid the M26–M29 suites pin: the pre-order candidate
/// order is `[top_left, top_right, bottom_left, bottom_right]`, the
/// geometry's reading disagrees with the tree's, and the arrows' rides
/// are the walk's own.
fn mount_grid(harness: &mut TestHarness) {
    let root = harness.mount(Column::new().gap(8.0).padding(8.0).boxed());
    let top = harness.add_child(root, Row::new().gap(8.0).boxed());
    let _ = harness.add_child(top, Button::new("Top left").on_click(|| {}).boxed());
    let _ = harness.add_child(top, Button::new("Top right").on_click(|| {}).boxed());
    let bottom = harness.add_child(root, Row::new().gap(8.0).boxed());
    let _ = harness.add_child(bottom, Button::new("Bottom left").on_click(|| {}).boxed());
    let _ = harness.add_child(bottom, Button::new("Bottom right").on_click(|| {}).boxed());
    let _ = harness.add_child(
        root,
        Label::new("The trail names who rode: the keyboard, the code, or the ring").boxed(),
    );
    harness.settle();
}

/// A plain Tab press: the keyboard's own shape.
fn tab() -> KeyEvent {
    KeyEvent {
        key: Key::Tab,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    }
}

/// A plain directional press: pressed state, no modifiers — the D-pad
/// button's own shape (the chords are the widgets' vocabulary).
fn dpad(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    }
}

#[test]
fn m30_the_code_doors_stream_the_consumer_rider() {
    let mut harness = TestHarness::new(360.0, 240.0);
    mount_grid(&mut harness);
    let observer = std::rc::Rc::new(std::cell::RefCell::new(TrailObserver::new()));
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer))));

    // The app's own doors: the record names the code as the rider.
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Next);
    assert!(outcome.moved(), "the door moved");
    let borrowed = observer.borrow();
    let first = borrowed.focus.front().expect("the door streamed");
    assert_eq!(
        first.rider,
        FocusRider::Consumer,
        "the code's ride names the consumer"
    );
    drop(borrowed);

    // The scoped door streams the same rider — the scope is named
    // beside it, the rider beside the scope.
    let root = harness.pipeline().arena().root().expect("the root");
    let outcome = harness.pipeline_mut().focus_move_in(root, FocusMove::Next);
    assert!(outcome.moved(), "the scoped door moved");
    let borrowed = observer.borrow();
    let last = borrowed.focus.back().expect("the scoped door streamed");
    assert_eq!(
        last.rider,
        FocusRider::Consumer,
        "the scoped ride is the code's"
    );
    assert!(
        last.scope.is_some(),
        "the scoped record still names its scope"
    );
}

#[test]
fn m30_the_tab_fallback_streams_the_keyboard_rider() {
    let mut harness = TestHarness::new(360.0, 240.0);
    mount_grid(&mut harness);
    let observer = std::rc::Rc::new(std::cell::RefCell::new(TrailObserver::new()));
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer))));

    // The keyboard's Tab fallback: the same funnel, a different rider —
    // the record distinguishes the user's navigation from the app's.
    let response = harness.pipeline_mut().dispatch_key(&tab());
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the entry Tab moved focus"
    );
    let borrowed = observer.borrow();
    let record = borrowed
        .focus
        .back()
        .expect("the funnel streamed Tab's ride");
    assert_eq!(
        record.rider,
        FocusRider::Keyboard,
        "the keyboard's ride names the keyboard"
    );
    assert_eq!(record.movement, FocusMove::Next, "Tab's movement is Next");
    assert!(
        record.outcome.moved(),
        "the outcome rides the record verbatim"
    );
}

#[test]
fn m30_the_arrows_stream_the_ring_rider() {
    let mut harness = TestHarness::new(360.0, 240.0);
    mount_grid(&mut harness);
    let observer = std::rc::Rc::new(std::cell::RefCell::new(TrailObserver::new()));
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer))));

    // The ring's arrows (M29's seam): the same funnel, the ring's own
    // rider — a trail reader can now tell the remote from the app.
    let response = harness.pipeline_mut().dispatch_key(&dpad(Key::ArrowRight));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the entry arrow moved focus"
    );
    let borrowed = observer.borrow();
    let record = borrowed.focus.back().expect("the funnel streamed the ride");
    assert_eq!(
        record.rider,
        FocusRider::Ring,
        "the arrows' ride names the ring"
    );
    assert_eq!(record.movement, FocusMove::Right, "the arrow's movement");
}

#[test]
fn m30_three_riders_one_funnel_readable_as_data() {
    let mut harness = TestHarness::new(360.0, 240.0);
    mount_grid(&mut harness);
    let observer = std::rc::Rc::new(std::cell::RefCell::new(TrailObserver::new()));
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer))));

    // One funnel, three riders, one trail — the reader's view is the
    // attribution, in order, without re-implementing the observation.
    let _ = harness.pipeline_mut().focus_move(FocusMove::Next);
    let _ = harness.pipeline_mut().dispatch_key(&tab());
    let _ = harness.pipeline_mut().dispatch_key(&dpad(Key::ArrowDown));
    let riders: Vec<FocusRider> = observer
        .borrow()
        .focus
        .iter()
        .map(|record| record.rider)
        .collect();
    assert_eq!(
        riders,
        vec![FocusRider::Consumer, FocusRider::Keyboard, FocusRider::Ring],
        "the trail reads as an attributed history"
    );
}

#[test]
fn m30_the_wire_carries_the_rider_beside_the_movement() {
    // The wire's focus shape grows the rider beside the movement — the
    // protocol's additive rule (new optional fields are compatible, the
    // version stays 1), the name vocabulary snake_case like every tag.
    let names = [
        (FocusRider::Keyboard, "keyboard"),
        (FocusRider::Ring, "ring"),
        (FocusRider::Consumer, "consumer"),
    ];
    for (rider, name) in names {
        assert_eq!(rider.name(), name, "the wire name is stable: {name}");
    }

    // The serialized shape carries the field: the recorder's broadcast
    // is the wire-shaped truth the server drains.
    let recorder = alacer_inspect::CausalRecorder::new("m30");
    let mut handle = recorder.clone_handle();
    handle.observe(InspectEvent::Focus(FocusRecord {
        rider: FocusRider::Ring,
        movement: FocusMove::Right,
        scope: None,
        outcome: FocusMoveOutcome::Refused(alacer_core::passes::FocusRefusal::NothingInDirection {
            node: alacer_core::NodeId::allocate(),
            direction: FocusMove::Right,
        }),
    }));
    let drained = recorder.take_broadcast();
    let last = drained.last().expect("the focus broadcast");
    let json = serde_json::to_string(last).expect("the wire event serializes");
    assert!(
        json.contains(r#""rider":"ring""#),
        "the wire carries the rider beside the movement: {json}"
    );
}

#[test]
fn m30_the_surface_rides_the_same_cut() {
    // The rider ships where the trail lives: the vocabulary beside the
    // record, the attribution behind the doors, the funnel untouched.
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    let inspect = std::fs::read_to_string(root.join("crates/alacer-core/src/inspect.rs"))
        .expect("inspect.rs is in the tree");
    for fragment in ["pub enum FocusRider", "pub rider: FocusRider"] {
        assert!(
            inspect.contains(fragment),
            "the inspect vocabulary carries the rider: {fragment}"
        );
    }
    let passes = std::fs::read_to_string(root.join("crates/alacer-core/src/passes.rs"))
        .expect("passes.rs is in the tree");
    for fragment in [
        "FocusRider::Keyboard",
        "FocusRider::Ring",
        "FocusRider::Consumer",
    ] {
        assert!(
            passes.contains(fragment),
            "the funnel attributes its riders: {fragment}"
        );
    }

    // The consumer demonstration exists and rides the workspace's
    // per-seam tracing idiom.
    let example = std::fs::read_to_string(root.join("crates/alacer/examples/focus_riders.rs"))
        .expect("the M30 example is in the tree");
    assert!(
        example.contains("alacer::focus_riders") && example.contains("FocusRider"),
        "the example reads the riders on the alacer::focus_riders target"
    );

    // The scope ADR exists; the living documents re-agree at M30.
    assert!(
        root.join("docs/adr/ADR-0036-m30-scope.md").exists(),
        "ADR-0036 scopes the trail's rider"
    );
    let readme = std::fs::read_to_string(root.join("README.md")).expect("README");
    assert!(
        readme.contains("M0 through M38") && readme.contains("forty-five decision records"),
        "the README's status block and ADR count re-agree at M30"
    );
    let architecture = std::fs::read_to_string(root.join("docs/architecture.md"))
        .expect("the architecture document");
    assert!(
        architecture.contains("45 decision records"),
        "the architecture register re-agrees at 45 decision records"
    );

    // The corpus stands at 62: the riders ride the published routes —
    // the record grew a field, the walk's cost is what it always was
    // (the coordinated rule's no-growth arm, the M29 precedent).
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 63;"),
        "the m5 suite's corpus pin stands at 62"
    );

    // The protocol document names the rider among the focus fields.
    let protocol = std::fs::read_to_string(root.join("docs/inspector-protocol.md"))
        .expect("the protocol document");
    assert!(
        protocol.contains("`rider`"),
        "the protocol documents the rider field"
    );
}
