//! M45 exit-criteria integration tests (§13.3, ADR-0051): the
//! keyboard's ride joins the value log, attributed. ADR-0049's first
//! kill rule ships: a consumer needs the keyboard's value rides on the
//! log *attributed*, so the focused slider's arrows stream the same
//! [`alacer_core::ValueRecord`] the AT tier streams through the
//! keyboard's own door — the dispatch funnel's key route holds the
//! stamp the action route holds (the focused control's AccessKit-stable
//! id, one `Cell` read while nobody listens — the A7 idle contract),
//! the record carrying the step's own action species, the landed value,
//! and M44's ask beside it. The suite pins the suite the M45 cut owed:
//! the arrows' ride attributed through the real dispatch, the two
//! routes' stamps answering one vocabulary, the drag staying off the
//! value log (the flood rule — the M46 gesture record carries the
//! drag's story instead), and the jumps never munged into a write (the
//! M47 named species beside `SetValue`).
//!
//! Every test runs the real pipeline (A4) with a real focused slider
//! driven by real keys and real pointer gestures.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::geom::Point;
use alacer_core::inspect::{InspectEvent, InspectObserver, ValueAction, ValueOutcome};
use alacer_core::widget::{A11yAction, EventResponse, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Slider};

/// The value-trail collector the tests install as the pipeline's
/// observer: the value facts stream into a bounded ring (the m43
/// helper's shape).
struct ValueObserver {
    values: std::collections::VecDeque<alacer_core::ValueRecord>,
}

impl ValueObserver {
    fn new() -> Self {
        Self {
            values: std::collections::VecDeque::with_capacity(64),
        }
    }
}

impl InspectObserver for ValueObserver {
    fn observe(&mut self, event: InspectEvent) {
        if let InspectEvent::Value(record) = event {
            if self.values.len() >= 64 {
                self.values.pop_front();
            }
            self.values.push_back(record);
        }
    }
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the m43 suite's shape, exactly.
struct Share(std::rc::Rc<std::cell::RefCell<ValueObserver>>);

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

/// The streamed value facts, oldest first.
fn value_records(observer: &Share) -> Vec<alacer_core::ValueRecord> {
    observer.0.borrow().values.iter().copied().collect()
}

/// Empties the shared ring between beats.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().values.clear();
}

/// One observed, focused slider mounted through the real pipeline: the
/// harness, the slider's node, and its track center (the gesture's
/// route).
fn mount_focused_slider(observer: &Share) -> (TestHarness, alacer_core::NodeId, Point) {
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
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    let bounds = harness.bounds(node);
    let center = Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + bounds.height() * 0.5,
    );
    (harness, node, center)
}

/// One key routed through the real dispatch funnel.
fn press(harness: &mut TestHarness, key: Key) -> EventResponse {
    harness.pipeline_mut().dispatch_key(&KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    })
}

#[test]
fn m45_the_arrows_ride_attributed() {
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    let (mut harness, _node, _center) = mount_focused_slider(&observer);
    let id = harness.query(Role::Slider).named("Volume").first().id;

    // The keyboard's step: one fact streams the moment the door
    // resolves — the step's own action species, the landed value, the
    // focused control's id (the key route's stamp, the action route's
    // own discipline).
    clear_trail(&observer);
    let response = press(&mut harness, Key::ArrowRight);
    assert!(
        matches!(response, EventResponse::Handled),
        "the focused slider consumes its own arrow"
    );
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the keyboard's ride streams one fact");
    assert_eq!(records[0].action, ValueAction::Increment);
    assert_eq!(records[0].outcome, ValueOutcome::Delivered);
    assert_eq!(
        records[0].value, 21.0,
        "the step landed exactly as it landed"
    );
    assert_eq!(
        records[0].requested,
        Some(21.0),
        "the step's ask rides beside the landing (M44, ADR-0050)"
    );
    assert_eq!(
        records[0].target,
        Some(id),
        "the ride carries the focused control's id — the M45 attribution"
    );

    // The backward step beside it: the other species, the same stamp —
    // the rides tellable apart on one log.
    clear_trail(&observer);
    let response = press(&mut harness, Key::ArrowLeft);
    assert!(matches!(response, EventResponse::Handled));
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the second ride streams one fact");
    assert_eq!(records[0].action, ValueAction::Decrement);
    assert_eq!(records[0].target, Some(id));
    assert_eq!(
        harness.settle(),
        0,
        "the answered rides queue no extra work"
    );
}

#[test]
fn m45_the_two_routes_answer_one_stamp_vocabulary() {
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    let (mut harness, _node, _center) = mount_focused_slider(&observer);
    let id = harness.query(Role::Slider).named("Volume").first().id;

    // The AT's step beside the keyboard's: one funnel, two routes, one
    // stamp vocabulary — the records tellable apart by nothing but the
    // route that routed them, the attribution identical.
    clear_trail(&observer);
    let response = press(&mut harness, Key::ArrowRight);
    assert!(matches!(response, EventResponse::Handled));
    let consumed = {
        let volume_id = harness.query(Role::Slider).named("Volume").first().id;
        harness
            .pipeline_mut()
            .dispatch_a11y_action(volume_id, A11yAction::Increment)
    };
    assert!(consumed, "the AT's step tier answers as it always answered");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 2, "two rides, two facts, one log");
    assert_eq!(
        records[0].target,
        Some(id),
        "the keyboard's ride is attributed (the key route's stamp)"
    );
    assert_eq!(
        records[1].target,
        Some(id),
        "the AT's ride is attributed (the action route's stamp)"
    );
    assert_eq!(records[0].action, ValueAction::Increment);
    assert_eq!(records[1].action, ValueAction::Increment);
}

#[test]
fn m45_the_drag_stays_off_the_value_log() {
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    let (mut harness, _node, center) = mount_focused_slider(&observer);

    // The flood rule (ADR-0046's second kill rule, ADR-0051's own arm):
    // a drag moves the value without streaming one value fact — the
    // per-pointer-move flood the kill rule refuses. The drag's story
    // rides the M46 gesture record instead (the kill rule's priced
    // answer); the value log stays the AT tier's and the step tier's.
    clear_trail(&observer);
    harness.pointer_press(center);
    for step in 0..40_u16 {
        harness.pointer_move(Point::new(center.x + f32::from(step), center.y));
    }
    harness.pointer_release(center);
    harness.settle();
    let moved = harness
        .query(Role::Slider)
        .named("Volume")
        .first()
        .value
        .clone()
        .and_then(|v| v.parse::<f32>().ok())
        .expect("the tree's value reads as text");
    assert!(
        moved > 20.0,
        "the drag wrote the value exactly as it always wrote it: {moved}"
    );
    assert!(
        value_records(&observer).is_empty(),
        "the drag streamed no value fact — the flood rule"
    );
}

#[test]
fn m45_home_and_end_never_munge_into_a_write() {
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    let (mut harness, _node, _center) = mount_focused_slider(&observer);

    // The no-munging rule (ADR-0051's own words: a jump munged into a
    // `SetValue` would be the M11 rule's own munging): Home/End stream
    // no `set_value`-tagged fact — the kill rule's priced answer is the
    // named bound species (M47, ADR-0053), never a collapsed write.
    clear_trail(&observer);
    let response = press(&mut harness, Key::Home);
    assert!(
        matches!(response, EventResponse::Handled),
        "the focused slider consumes its own Home"
    );
    let response = press(&mut harness, Key::End);
    assert!(matches!(response, EventResponse::Handled));
    harness.settle();
    let records = value_records(&observer);
    assert!(
        records
            .iter()
            .all(|record| record.action != ValueAction::SetValue),
        "a jump is never munged into a SetValue"
    );
    assert!(
        records.iter().all(|record| !matches!(
            record.action,
            ValueAction::Increment | ValueAction::Decrement
        )),
        "a jump is never munged into a step either — the named species is the priced answer"
    );
}

#[test]
fn m45_no_observer_no_trail_the_idle_contract_holds() {
    // The default path: no observer, no enabled trail — the stamp is
    // one `Cell` read per routed key, the arrows land exactly as before,
    // and the frame costs what it always cost.
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
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    let response = press(&mut harness, Key::ArrowRight);
    assert!(matches!(response, EventResponse::Handled));
    harness.settle();
    assert_eq!(
        harness.query(Role::Slider).named("Volume").first().value,
        Some("21".to_string()),
        "the step landed exactly as before"
    );
    assert_eq!(harness.settle(), 0, "the disabled trail queues no work");
}

#[test]
fn m45_the_living_documents_carry_the_attribution() {
    let root = workspace_root();
    let changelog = std::fs::read_to_string(root.join("CHANGELOG.md")).expect("changelog");
    // The changelog stages the milestone (the entry the cut staged).
    assert!(
        changelog.contains("M45 — the keyboard's ride joins the value log"),
        "the changelog stages M45"
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
