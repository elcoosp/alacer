//! M49 exit-criteria integration tests (§13.3, ADR-0055): the funnel's
//! wall joins the value log as a named fact. ADR-0053's second kill
//! rule cashes: a consumer needs *which* bound the value pinned when a
//! write was turned back at the range's edge — so the value record
//! grows the pinned bound ([`alacer_core::inspect::ValueRecord::pinned`],
//! [`alacer_core::inspect::RangeBound`]) beside the action and outcome
//! species it already carries — the ask's own wall named beside the
//! landing it produced, never a second action species (the kill rule's
//! own words). The ask that wanted the wall (the M47 jumps) carries
//! the honest absence: the species already names the bound it asked
//! for.
//!
//! Every test runs the real pipeline (A4) with a real slider driven by
//! the real AT action route and the real key route — the exact paths a
//! status line, an inspector client, or a test reads.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{InspectEvent, InspectObserver, ValueAction, ValueOutcome};
use alacer_core::widget::{A11yAction, EventResponse, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Slider};

/// The value-trail collector the tests install as the pipeline's
/// observer: the write's facts stream into a bounded ring (the m-suite's
/// helper shape — here the pins are the assertion source).
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
/// the m-suite's shape, exactly.
struct Share(std::rc::Rc<std::cell::RefCell<ValueObserver>>);

impl Share {
    fn new() -> Self {
        Self(std::rc::Rc::new(std::cell::RefCell::new(
            ValueObserver::new(),
        )))
    }
}

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

/// The streamed value facts, oldest first.
fn value_records(observer: &Share) -> Vec<alacer_core::ValueRecord> {
    observer.0.borrow().values.iter().copied().collect()
}

/// Empties the shared ring between beats: each assertion reads the facts
/// the write under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().values.clear();
}

/// One observed, focused slider mounted through the real pipeline: the
/// funnel's wall is the fact the record prices (the m47 suite's mount,
/// exactly — the key route needs the focus, the action route the id).
fn mount_focused_slider(observer: &Share) -> (TestHarness, alacer_core::NodeId) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .value(50.0)
            .step(5.0)
            .boxed(),
    );
    harness.settle();
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    (harness, node)
}

/// One key routed through the real dispatch funnel: the response the
/// route gave.
fn press(harness: &mut TestHarness, key: Key) -> EventResponse {
    harness.pipeline_mut().dispatch_key(&KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    })
}

/// One AT action routed through the real dispatch funnel: the response
/// the route gave. The funnel routes by the AccessKit-stable id hash —
/// the same identity the stamp carries (the M43 vocabulary).
fn act(harness: &mut TestHarness, id: u64, action: A11yAction) -> bool {
    harness.pipeline_mut().dispatch_a11y_action(id, action)
}

#[test]
fn m49_the_past_ask_names_its_wall() {
    let observer = Share::new();
    let (mut harness, _node) = mount_focused_slider(&observer);
    let id = harness.query(Role::Slider).named("Volume").first().id;

    // The ask past the high wall: `SetValue("150")` on a 0..=100 slider
    // lands clamped at the maximum — the record names the bound the
    // funnel pinned the landing at, beside the ask it turned back (the
    // M44 ask already rode; the wall is the fact the pair could not
    // name).
    clear_trail(&observer);
    assert!(act(
        &mut harness,
        id,
        A11yAction::SetValue("150".to_string())
    ));
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "one write, one fact");
    assert_eq!(records[0].action, ValueAction::SetValue);
    assert_eq!(records[0].outcome, ValueOutcome::Delivered);
    assert_eq!(records[0].value, 100.0, "the range is a wall");
    assert_eq!(
        records[0].requested,
        Some(150.0),
        "the ask rides beside the landing (M44, ADR-0050)"
    );
    assert_eq!(
        records[0].pinned,
        Some(alacer_core::RangeBound::Max),
        "the funnel pinned the landing at the max — the wall is a named fact"
    );
    assert_eq!(records[0].target, Some(id), "the stamp rides (M43)");

    // The ask past the low wall: the other edge, the other species.
    clear_trail(&observer);
    assert!(act(
        &mut harness,
        id,
        A11yAction::SetValue("-5".to_string())
    ));
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the second write streams one fact");
    assert_eq!(records[0].value, 0.0, "the low wall held");
    assert_eq!(
        records[0].pinned,
        Some(alacer_core::RangeBound::Min),
        "the funnel pinned the landing at the min"
    );
    assert_eq!(
        harness.settle(),
        0,
        "the answered writes queue no extra work"
    );
}

#[test]
fn m49_the_held_wall_names_itself() {
    let observer = Share::new();
    let (mut harness, _node) = mount_focused_slider(&observer);
    let id = harness.query(Role::Slider).named("Volume").first().id;

    // The wall held: the value already at the maximum, a past ask
    // turned back again — the no-op delivery answers like the landed
    // write it is (the M20 idle-frame property) and the log names the
    // bound that held, never one silence.
    assert!(act(
        &mut harness,
        id,
        A11yAction::SetValue("100".to_string())
    ));
    harness.settle();
    clear_trail(&observer);
    assert!(act(
        &mut harness,
        id,
        A11yAction::SetValue("150".to_string())
    ));
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the no-op streams its fact");
    assert_eq!(records[0].outcome, ValueOutcome::Delivered);
    assert_eq!(records[0].value, 100.0, "the wall kept the value");
    assert_eq!(
        records[0].pinned,
        Some(alacer_core::RangeBound::Max),
        "the wall that held the write names itself"
    );
    assert_eq!(records[0].target, Some(id));
    assert_eq!(harness.settle(), 0, "the no-op answered without work");
}

#[test]
fn m49_the_ask_that_wanted_the_wall_carries_no_pin() {
    let observer = Share::new();
    let (mut harness, _node) = mount_focused_slider(&observer);

    // The jumps (M47, ADR-0053) asked for the wall exactly — the ask
    // *is* the bound, the species already names it, and the pin rides
    // the honest absence: a pin would say the funnel moved an ask that
    // it never moved.
    clear_trail(&observer);
    assert!(matches!(
        press(&mut harness, Key::End),
        EventResponse::Handled
    ));
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "one jump, one fact");
    assert_eq!(records[0].action, ValueAction::JumpToMax);
    assert_eq!(records[0].value, 100.0);
    assert_eq!(
        records[0].pinned, None,
        "the ask wanted the wall — the species names it, the pin is absent"
    );

    // Home beside it: the other jump, the same honest absence.
    clear_trail(&observer);
    assert!(matches!(
        press(&mut harness, Key::Home),
        EventResponse::Handled
    ));
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records[0].action, ValueAction::JumpToMin);
    assert_eq!(records[0].value, 0.0);
    assert_eq!(records[0].pinned, None, "the ask wanted the min");
    assert_eq!(
        harness.settle(),
        0,
        "the answered jumps queue no extra work"
    );
}

#[test]
fn m49_the_steps_name_the_wall_and_the_absence_holds() {
    let observer = Share::new();
    let (mut harness, _node) = mount_focused_slider(&observer);

    // A mid-range step pinned nothing: the honest absence rides — the
    // bound names a landing at the range's own edge, never a value
    // that merely sits nearby.
    clear_trail(&observer);
    assert!(matches!(
        press(&mut harness, Key::ArrowRight),
        EventResponse::Handled
    ));
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records[0].action, ValueAction::Increment);
    assert_eq!(records[0].value, 55.0);
    assert_eq!(records[0].pinned, None, "a mid-range step pinned no wall");

    // The step at the wall: the arrows' ask past the maximum turns
    // back — the wall held the step, and the log names it (the M45
    // key route's stamp riding beside, the M47 suite's own route).
    clear_trail(&observer);
    let _ = press(&mut harness, Key::End);
    harness.settle();
    clear_trail(&observer);
    assert!(matches!(
        press(&mut harness, Key::ArrowRight),
        EventResponse::Handled
    ));
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "one step, one fact");
    assert_eq!(records[0].action, ValueAction::Increment);
    assert_eq!(records[0].value, 100.0, "the wall kept the step");
    assert_eq!(
        records[0].pinned,
        Some(alacer_core::RangeBound::Max),
        "the step's ask turned back at the max — the wall named"
    );
    assert_eq!(
        harness.settle(),
        0,
        "the answered steps queue no extra work"
    );
}

#[test]
fn m49_no_observer_no_trail_the_idle_contract_holds() {
    // No observer, no trail: the pinned write lands (the wall still
    // holds the value), nothing queues, and the frame stays idle — the
    // A7 contract the trails ride (the m47 suite's own pin).
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .value(50.0)
            .step(5.0)
            .boxed(),
    );
    harness.settle();
    let id = harness.query(Role::Slider).named("Volume").first().id;
    assert!(act(
        &mut harness,
        id,
        A11yAction::SetValue("150".to_string())
    ));
    harness.settle();
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
    let _ = node;
    assert_eq!(harness.settle(), 0, "no observer, no queued facts");
}

#[test]
fn m49_the_living_documents_re_agree_at_the_cut() {
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
        readme.contains("M0 through M54") && readme.contains("sixty-one decision records"),
        "the README's status block and count re-agree at the cut"
    );
    assert!(
        architecture.contains("61 decision records"),
        "the architecture register re-agrees at 61 decision records"
    );
    // The protocol document carries the pinned vocabulary (the wire's
    // documented field).
    assert!(
        protocol.contains("\"pinned\""),
        "the protocol document documents the value's pinned field"
    );
    // The catalog lists the demonstration.
    assert!(
        catalog.contains("pinned_writes"),
        "the examples catalog lists the pinned writes demo"
    );
    // The changelog stages the milestone.
    assert!(
        changelog.contains("M49 — the funnel's wall is a named fact"),
        "the changelog stages M49"
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
