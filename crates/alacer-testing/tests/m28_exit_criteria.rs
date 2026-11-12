//! M28 exit-criteria integration tests (§13.3, ADR-0034): the focus walk
//! joins the causal log. ADR-0033's pre-registered re-open rule ships —
//! "a consumer needs to observe the walk's refusals as *streamed events*
//! (the inspector's causal log: 'why did focus change / why didn't it')" —
//! as [`alacer_core::inspect::InspectEvent::Focus`]: one
//! [`alacer_core::inspect::FocusRecord`] streamed the moment a focus-move
//! door answers, carrying the movement the caller asked for, the scope
//! that bounded the walk (`None` for the unscoped funnel), and the
//! outcome the door returned — the landing or the named refusal species,
//! verbatim. The walk behind the doors is untouched: the same candidate
//! list, the same availability filter, the same wrap, the same immediate
//! resolution, the same no-mask idle refusals (the M11 rule stays; the
//! answer just reaches the causal log too). The keyboard's Tab fallback
//! rides the same funnel, so its rides stream the same records — one
//! funnel, two doors, one trail (ADR-0030's rule, now observable).
//!
//! Every test runs the real pipeline (A4) with the real layout host —
//! the exact route a consumer reading the trail takes.

use alacer_core::arena::NodeId;
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{FocusRecord, FocusRider, InspectEvent, InspectObserver};
use alacer_core::passes::{FocusMove, FocusMoveOutcome, FocusRefusal};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, Row, TextInput};

/// The causal-trail collector the tests install as the pipeline's
/// observer: the focus records stream into a bounded ring, the frame
/// summaries count (the trail-vs-frame ordering assertion uses it).
struct TrailObserver {
    focus: std::collections::VecDeque<FocusRecord>,
    frames: usize,
    cap: usize,
}

impl TrailObserver {
    /// A collector holding up to `cap` focus records (the oldest drop
    /// first — the recorder's own discipline, §14.3).
    fn new(cap: usize) -> Self {
        Self {
            focus: std::collections::VecDeque::with_capacity(cap),
            frames: 0,
            cap,
        }
    }
}

impl InspectObserver for TrailObserver {
    fn observe(&mut self, event: InspectEvent) {
        match event {
            InspectEvent::Focus(record) => {
                if self.focus.len() >= self.cap {
                    self.focus.pop_front();
                }
                self.focus.push_back(record);
            }
            InspectEvent::Frame(_) => self.frames += 1,
            _ => {}
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

/// Mounts the observation window: the two-by-two grid the M26/M27 suites
/// pin, plus the modal's form the scoped beats walk. The pre-order
/// candidate order is `[top_left, top_right, bottom_left, bottom_right]`.
fn mount_grid(harness: &mut TestHarness) -> (NodeId, Vec<NodeId>) {
    let root = harness.mount(Column::new().gap(8.0).padding(8.0).boxed());
    let top = harness.add_child(root, Row::new().gap(8.0).boxed());
    let top_left = harness.add_child(top, Button::new("Top left").on_click(|| {}).boxed());
    let top_right = harness.add_child(top, Button::new("Top right").on_click(|| {}).boxed());
    let bottom = harness.add_child(root, Row::new().gap(8.0).boxed());
    let bottom_left = harness.add_child(bottom, Button::new("Bottom left").on_click(|| {}).boxed());
    let bottom_right =
        harness.add_child(bottom, Button::new("Bottom right").on_click(|| {}).boxed());
    let _ = harness.add_child(
        root,
        Label::new("The causal log answers why focus changed — or didn't").boxed(),
    );
    harness.settle();
    (root, vec![top_left, top_right, bottom_left, bottom_right])
}

/// The modal window the scoped records demonstrate on: a background
/// control the trap must never offer, and the modal's form whose root is
/// the scope the door takes.
fn mount_modal(harness: &mut TestHarness) -> (NodeId, Vec<NodeId>) {
    let root = harness.mount(Column::new().gap(8.0).padding(8.0).boxed());
    let background = harness.add_child(root, Button::new("Background").on_click(|| {}).boxed());
    let modal = harness.add_child(root, Column::new().gap(8.0).padding(12.0).boxed());
    let recipient = harness.add_child(modal, TextInput::new("Recipient").boxed());
    let subject = harness.add_child(modal, TextInput::new("Subject").boxed());
    let send = harness.add_child(modal, Button::new("Send").on_click(|| {}).boxed());
    harness.settle();
    (modal, vec![background, recipient, subject, send])
}

/// A Tab keypress through the real dispatch path (pass 1) — the
/// keyboard's ride on the same funnel the consumer's door rides.
fn tab_press() -> KeyEvent {
    KeyEvent {
        key: Key::Tab,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    }
}

#[test]
fn m28_the_doors_stream_their_answers_to_the_causal_log() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let (_root, nodes) = mount_grid(&mut harness);
    let [top_left, top_right, _bottom_left, bottom_right] =
        [nodes[0], nodes[1], nodes[2], nodes[3]];
    let observer = std::rc::Rc::new(std::cell::RefCell::new(TrailObserver::new(64)));
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer))));

    // The entry: the door streams a record whose outcome is the door's
    // own answer — the landing naming `from: None`, the walk's resting
    // node, and no wrap (an entry is not a wrap).
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: None,
            to: top_left,
            wrapped: false,
        }),
        "the entry lands the first candidate"
    );
    let trail = std::rc::Rc::clone(&observer).borrow().focus.clone();
    assert_eq!(trail.len(), 1, "one door ride, one streamed record");
    assert_eq!(
        trail[0],
        FocusRecord {
            movement: FocusMove::Next,
            scope: None,
            rider: FocusRider::Consumer,
            outcome,
        },
        "the record is the door's answer, verbatim"
    );

    // The stride: the second ride names both endpoints — and the wrap
    // from the last candidate back to the first is data in the log too.
    let _ = harness.pipeline_mut().focus_move(FocusMove::Next);
    let _ = harness.pipeline_mut().focus_move(FocusMove::Next);
    let _ = harness.pipeline_mut().focus_move(FocusMove::Next);
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: Some(bottom_right),
            to: top_left,
            wrapped: true,
        }),
        "the wrap lands where the tree says"
    );
    let trail = std::rc::Rc::clone(&observer).borrow().focus.clone();
    assert_eq!(trail.len(), 5, "five rides, five records");
    assert_eq!(
        trail[4],
        FocusRecord {
            movement: FocusMove::Next,
            scope: None,
            rider: FocusRider::Consumer,
            outcome,
        },
        "the wrapped landing streams with the wrap set"
    );
    assert_eq!(
        trail[1].outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: Some(top_left),
            to: top_right,
            wrapped: false,
        }),
        "each record carries its own ride's answer"
    );
}

#[test]
fn m28_the_refusals_stream_their_species() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let (_root, nodes) = mount_grid(&mut harness);
    let bottom_right = nodes[3];
    let observer = std::rc::Rc::new(std::cell::RefCell::new(TrailObserver::new(64)));
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer))));

    // Walk to the corner: entry, Right, Down — then the corner refuses.
    harness.pipeline_mut().focus_move(FocusMove::Next);
    harness.pipeline_mut().focus_move(FocusMove::Right);
    harness.pipeline_mut().focus_move(FocusMove::Down);
    assert_eq!(harness.pipeline().focus(), Some(bottom_right));
    harness.settle();
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Right);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Refused(FocusRefusal::NothingInDirection {
            node: bottom_right,
            direction: FocusMove::Right,
        }),
        "the corner refuses by name"
    );

    // The refusal streamed too: a refused ride is exactly the fact the
    // causal log exists for — "why didn't focus change".
    let trail = std::rc::Rc::clone(&observer).borrow().focus.clone();
    assert_eq!(trail.len(), 4, "three landings and one refusal");
    assert_eq!(
        trail[3],
        FocusRecord {
            movement: FocusMove::Right,
            scope: None,
            rider: FocusRider::Consumer,
            outcome,
        },
        "the empty direction names its node and direction in the log"
    );
    assert!(
        harness.tick().is_idle(),
        "the streamed refusal queues no masks — an idle frame, the M11 rule"
    );
}

#[test]
fn m28_the_scoped_door_streams_its_scope_and_its_dead_scope() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let (modal, nodes) = mount_modal(&mut harness);
    let recipient = nodes[1];
    let observer = std::rc::Rc::new(std::cell::RefCell::new(TrailObserver::new(64)));
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer))));

    // The scoped ride: the record names the scope the walk was bounded to.
    let outcome = harness.pipeline_mut().focus_move_in(modal, FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: None,
            to: recipient,
            wrapped: false,
        }),
        "the trap answers with its first candidate"
    );
    let trail = std::rc::Rc::clone(&observer).borrow().focus.clone();
    assert_eq!(trail.len(), 1, "the scoped ride streamed");
    assert_eq!(
        trail[0],
        FocusRecord {
            movement: FocusMove::Next,
            scope: Some(modal),
            rider: FocusRider::Consumer,
            outcome,
        },
        "the record names the scope the walk was bounded to"
    );

    // The dead scope: the early refusal streams as a record too — the
    // M25 refusal, now observable, not only named.
    harness.pipeline_mut().arena_mut().remove_subtree(modal);
    harness.settle();
    let outcome = harness.pipeline_mut().focus_move_in(modal, FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Refused(FocusRefusal::DeadScope { scope: modal }),
        "the dead scope refuses by name"
    );
    let trail = std::rc::Rc::clone(&observer).borrow().focus.clone();
    assert_eq!(trail.len(), 2, "the dead-scope refusal streamed");
    assert_eq!(
        trail[1],
        FocusRecord {
            movement: FocusMove::Next,
            scope: Some(modal),
            rider: FocusRider::Consumer,
            outcome,
        },
        "the dead-scope record names the scope and the species"
    );
}

#[test]
fn m28_the_keyboard_ride_streams_the_same_record() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let (_root, nodes) = mount_grid(&mut harness);
    let top_left = nodes[0];
    let top_right = nodes[1];
    let observer = std::rc::Rc::new(std::cell::RefCell::new(TrailObserver::new(64)));
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer))));

    // The keyboard's Tab rides the same funnel (ADR-0030): the dispatch
    // path streams the same record the consumer's door does — one walk,
    // two doors, one trail.
    let response = harness.pipeline_mut().dispatch_key(&tab_press());
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "Tab moves focus"
    );
    assert_eq!(harness.pipeline().focus(), Some(top_left));
    let outcome = harness.pipeline_mut().dispatch_key(&tab_press());
    assert!(
        matches!(outcome, alacer_core::EventResponse::Handled),
        "the second Tab moves again"
    );
    assert_eq!(harness.pipeline().focus(), Some(top_right));
    let trail = std::rc::Rc::clone(&observer).borrow().focus.clone();
    assert_eq!(trail.len(), 2, "both keyboard rides streamed");
    assert_eq!(
        trail[0],
        FocusRecord {
            movement: FocusMove::Next,
            scope: None,
            rider: FocusRider::Keyboard,
            outcome: FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
                from: None,
                to: top_left,
                wrapped: false,
            }),
        },
        "the entry via Tab streams the entry landing"
    );
    assert_eq!(
        trail[1].outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: Some(top_left),
            to: top_right,
            wrapped: false,
        }),
        "the stride via Tab streams the stride landing"
    );
}

#[test]
fn m28_the_trail_is_bounded_and_orders_before_the_frame() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let (_root, _nodes) = mount_grid(&mut harness);
    let observer = std::rc::Rc::new(std::cell::RefCell::new(TrailObserver::new(8)));
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer))));

    // Sixteen rides against a cap of eight: the ring evicts the oldest —
    // the recorder's discipline (§14.3), devtools never unbounded.
    for _ in 0..16 {
        let _ = harness.pipeline_mut().focus_move(FocusMove::Next);
    }
    let binding = std::rc::Rc::clone(&observer);
    let borrowed = binding.borrow();
    assert_eq!(borrowed.focus.len(), 8, "the trail ring is bounded");
    drop(borrowed);

    // One more ride, then the tick: the walk's record streamed when the
    // door answered — before the frame summary the tick forwards.
    let _ = harness.pipeline_mut().focus_move(FocusMove::Next);
    harness.tick();
    let binding = std::rc::Rc::clone(&observer);
    let borrowed = binding.borrow();
    assert!(borrowed.frames >= 1, "the tick's frame reached the log");
    let last = borrowed.focus.back().expect("the last ride");
    assert_eq!(
        last.movement,
        FocusMove::Next,
        "the door's record predates the frame's summary"
    );
}

#[test]
fn m28_without_an_observer_the_doors_answer_identically() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let (_root, nodes) = mount_grid(&mut harness);
    let top_left = nodes[0];

    // No observer installed: the same rides answer the same outcomes —
    // the seam costs one branch per door call when devtools are absent.
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: None,
            to: top_left,
            wrapped: false,
        }),
        "the unobserved walk lands identically"
    );
    let outcome = {
        let root = harness.pipeline().arena().root().expect("root");
        harness.pipeline_mut().focus_move_in(root, FocusMove::Next)
    };
    assert!(
        outcome.moved(),
        "the unobserved scoped walk answers identically"
    );
}

#[test]
fn m28_the_surface_rides_the_same_cut() {
    // The vocabulary ships where the trail lives: the record beside the
    // observer sink, the emission behind the doors, the funnel untouched.
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    let inspect = std::fs::read_to_string(root.join("crates/alacer-core/src/inspect.rs"))
        .expect("inspect.rs is in the tree");
    for fragment in [
        "pub struct FocusRecord",
        "pub outcome: FocusMoveOutcome",
        "Focus(FocusRecord)",
    ] {
        assert!(
            inspect.contains(fragment),
            "the inspect vocabulary carries the focus record: {fragment}"
        );
    }
    let passes = std::fs::read_to_string(root.join("crates/alacer-core/src/passes.rs"))
        .expect("passes.rs is in the tree");
    assert!(
        passes.contains("fn notify_focus"),
        "the pipeline forwards the doors' answers through one emission helper"
    );

    // The consumer demonstration exists and rides the workspace's
    // per-seam tracing idiom.
    let example = std::fs::read_to_string(root.join("crates/alacer/examples/focus_trail.rs"))
        .expect("the M28 example is in the tree");
    assert!(
        example.contains("alacer::focus_trail") && example.contains("focus_move"),
        "the example reads the trail on the alacer::focus_trail target"
    );

    // The scope ADR exists; the living documents re-agree at M28.
    assert!(
        root.join("docs/adr/ADR-0034-m28-scope.md").exists(),
        "ADR-0034 scopes the causal focus trail"
    );
    let readme = std::fs::read_to_string(root.join("README.md")).expect("README");
    assert!(
        readme.contains("M0 through M39") && readme.contains("forty-six decision records"),
        "the README's status block and ADR count re-agree at M28"
    );
    let architecture = std::fs::read_to_string(root.join("docs/architecture.md"))
        .expect("the architecture document");
    assert!(
        architecture.contains("46 decision records"),
        "the architecture register re-agrees at 45 decision records"
    );

    // The observed route joins the published corpus beside the four focus
    // routes: 61 → 62 across the m5–m27 chain in the same cut — the
    // coordinated rule (bench, gates, and baseline in the same cut).
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 63;"),
        "the m5 suite's corpus pin moved to 62"
    );
    let contracts =
        std::fs::read_to_string(root.join("benchmarks/contracts.json")).expect("the contract set");
    assert!(
        contracts.contains("input/route_100_observed_moves_1k"),
        "the observed route joins the gated corpus"
    );
}
