//! M44 exit-criteria integration tests (§13.3, ADR-0050): the value
//! record shows the clamp. ADR-0049's second re-open rule ships: a
//! consumer needs the *asked* value beside the landed one (a clamp or a
//! snap the trail could not show), so one additive
//! [`alacer_core::ValueRecord::requested`] rides every value fact — the
//! finite scalar the AT asked before the control's clamp-and-snap moved
//! it, the fact living where the knowledge lives (the widget's own door
//! computes the ask; the note site carries it, no stamp — nothing
//! outside the widget knows the ask). The absence is honest per
//! species: the unparsable payload, the closed door, the non-finite
//! wall (`None` — JSON cannot carry an infinite ask, the species names
//! it). The suite pins the suite the M44 cut owed: the clamp visible as
//! data through the real action tier, the honest absences per species,
//! and the step tier's arithmetic riding the same field.
//!
//! Every test runs the real pipeline (A4) with real widgets driven by
//! real AT actions — the exact route an inspector client reads.

use alacer_core::inspect::{InspectEvent, InspectObserver, ValueAction, ValueOutcome};
use alacer_core::widget::{A11yAction, EventResponse, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Slider};

/// The value-trail collector the tests install as the pipeline's
/// observer: the write facts stream into a bounded ring (the m43
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

/// One observed slider mounted through the real pipeline: the harness
/// beside the slider's own node (the focus route's vocabulary).
fn mount_slider(observer: &Share, step: f32) -> (TestHarness, alacer_core::NodeId) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .value(20.0)
            .step(step)
            .boxed(),
    );
    harness.settle();
    (harness, node)
}

/// One AT action routed to the named control through the real funnel.
fn act_on(harness: &mut TestHarness, action: A11yAction) -> bool {
    let id = harness.query(Role::Slider).named("Volume").first().id;
    harness.pipeline_mut().dispatch_a11y_action(id, action)
}

/// The focused slider's own door, driven by one routed key (the step
/// tier's route).
fn press(harness: &mut TestHarness, key: alacer_core::events::Key) -> EventResponse {
    harness
        .pipeline_mut()
        .dispatch_key(&alacer_core::events::KeyEvent {
            key,
            state: alacer_core::events::KeyState::Pressed,
            modifiers: alacer_core::events::Modifiers::default(),
            text: None,
        })
}

#[test]
fn m44_the_clamp_is_visible_as_one_records_story() {
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    let (mut harness, _node) = mount_slider(&observer, 1.0);

    // The ask beside the landing: "150" asked, the range's maximum
    // landed — the clamp the trail could not show is one record's
    // story now (the M44 demonstration's own words).
    clear_trail(&observer);
    let consumed = act_on(&mut harness, A11yAction::SetValue("150".to_string()));
    assert!(consumed, "the write answers as it always answered");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "one write, one fact");
    assert_eq!(records[0].action, ValueAction::SetValue);
    assert_eq!(records[0].outcome, ValueOutcome::Delivered);
    assert_eq!(
        records[0].requested,
        Some(150.0),
        "the ask rides beside the landing — the fact the widget's own door computed"
    );
    assert_eq!(
        records[0].value, 100.0,
        "the landing is the funnel's clamp-and-snap result"
    );
    assert_eq!(
        harness.query(Role::Slider).named("Volume").first().value,
        Some("100".to_string()),
        "the tree carries exactly what the record landed"
    );
}

#[test]
fn m44_the_snap_rides_the_same_field() {
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    let (mut harness, _node) = mount_slider(&observer, 5.0);

    // The snap: "42.7" asked on a step-5 range — the ask rides the
    // finite scalar the door computed, the landing the snapped value,
    // the pair tellable apart on one record.
    clear_trail(&observer);
    let consumed = act_on(&mut harness, A11yAction::SetValue("42.7".to_string()));
    assert!(consumed);
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "one write, one fact");
    assert_eq!(
        records[0].requested,
        Some(42.7),
        "the ask is the payload the door parsed, before the snap moved it"
    );
    let landed = harness
        .query(Role::Slider)
        .named("Volume")
        .first()
        .value
        .clone()
        .and_then(|v| v.parse::<f32>().ok())
        .expect("the tree's value reads as text");
    assert_eq!(
        records[0].value, landed,
        "the landing is the tree's own value — the same vocabulary the AT reads"
    );
    assert_ne!(
        records[0].requested,
        Some(records[0].value),
        "the snap moved the ask — the record shows both"
    );
}

#[test]
fn m44_the_absences_are_honest_per_species() {
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    let (mut harness, _node) = mount_slider(&observer, 1.0);

    // The unparsable payload: the ask never became a finite scalar —
    // `None` rides the refusal (the honest absence).
    clear_trail(&observer);
    let consumed = act_on(&mut harness, A11yAction::SetValue("loud".to_string()));
    assert!(!consumed, "the parse refusal answers as it always answered");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the refusal streams one fact");
    assert_eq!(
        records[0].outcome,
        ValueOutcome::Refused(alacer_core::ValueRefusal::InvalidPayload)
    );
    assert_eq!(
        records[0].requested, None,
        "no finite ask ever was — the absence rides it"
    );

    // The non-finite wall: "inf" parses fine, the funnel refuses — and
    // the ask is not a finite scalar either (JSON cannot carry an
    // infinite ask honestly; the species names the wall).
    clear_trail(&observer);
    let consumed = act_on(&mut harness, A11yAction::SetValue("inf".to_string()));
    assert!(!consumed, "the funnel's wall refuses as it always refused");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the wall streams one fact");
    assert_eq!(
        records[0].outcome,
        ValueOutcome::Refused(alacer_core::ValueRefusal::NonFiniteValue)
    );
    assert_eq!(
        records[0].requested, None,
        "the infinite ask is not a finite scalar — the absence is honest"
    );
    assert_eq!(harness.settle(), 0, "the refusals queue no masks");
}

#[test]
fn m44_the_step_tier_rides_the_ask_too() {
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    let (mut harness, node) = mount_slider(&observer, 5.0);

    // The AT's step: the ask is the arithmetic before the funnel —
    // value + direction * step, the same field the payload carries.
    clear_trail(&observer);
    let consumed = act_on(&mut harness, A11yAction::Increment);
    assert!(consumed, "the step tier answers as it always answered");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "one step, one fact");
    assert_eq!(records[0].action, ValueAction::Increment);
    assert_eq!(
        records[0].requested,
        Some(25.0),
        "the step's ask is the arithmetic the door computed (20 + 5)"
    );
    assert_eq!(records[0].value, 25.0, "the landing is the funnel's answer");

    // The keyboard's step (the M45 ride): the same field, the same
    // door's arithmetic — the ask beside the landing on every route.
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    clear_trail(&observer);
    let response = press(&mut harness, alacer_core::events::Key::ArrowRight);
    assert!(
        matches!(response, EventResponse::Handled),
        "the focused slider consumes its own arrow"
    );
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the keyboard's ride streams one fact");
    assert_eq!(
        records[0].requested,
        Some(30.0),
        "the keyboard's step carries the ask beside its landing too"
    );
    assert_eq!(
        harness.settle(),
        0,
        "the answered steps queue no extra work"
    );
}

#[test]
fn m44_the_living_documents_carry_the_ask() {
    let root = workspace_root();
    let protocol = std::fs::read_to_string(root.join("docs/inspector-protocol.md"))
        .expect("protocol document");
    let changelog = std::fs::read_to_string(root.join("CHANGELOG.md")).expect("changelog");
    // The wire's documented shape: the ask beside the landing, the
    // absence honest — the field the m44 cut owed the protocol document.
    assert!(
        protocol.contains("`requested`") && protocol.contains("asked"),
        "the protocol document documents the ask field"
    );
    // The changelog stages the milestone (the entry the cut staged).
    assert!(
        changelog.contains("M44 — the value record shows the clamp"),
        "the changelog stages M44"
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
