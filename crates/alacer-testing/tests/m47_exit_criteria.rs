//! M47 exit-criteria integration tests (§13.3, ADR-0053): the keyboard's
//! jumps are named facts. ADR-0051's kill arm ships: a consumer needs
//! Home/End on the log — M45 priced the arrows' steps and left the
//! jumps off ("the record vocabulary has no bound action — a jump
//! munged into a `SetValue` would be the M11 rule's own munging"), so a
//! trail reader could not answer "did the jump land, and which edge"
//! without re-implementing the observation at the chord site. The price
//! the kill rule names: a *named* bound species beside `SetValue` —
//! `ValueAction::JumpToMin`/`ValueAction::JumpToMax`, the jump its own
//! action on the record, never a munged write. The slider's bounds'
//! door notes the record where the answer resolves (the bound the ask,
//! the landing beside it), the M45 key route stamps the focused
//! control, and the wire's `action` vocabulary grows the two tags
//! additively (the documented field values' rule at version 1).
//!
//! Every test runs the real pipeline (A4) with a real focused slider
//! driven by real key events — the exact route the dispatch funnel
//! rides.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{InspectEvent, InspectObserver, ValueAction, ValueOutcome};
use alacer_core::widget::EventResponse;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Slider};

/// The value-trail collector the tests install as the pipeline's
/// observer: the value facts stream into a bounded ring (the m43
/// helper's shape — here the action species is the assertion source).
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

/// Empties the shared ring between beats: each assertion reads the facts
/// the chord under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().values.clear();
}

/// One focused slider mounted through the real pipeline and observed —
/// the keyboard's route is focus-routed, so the tests focus the control
/// before every chord.
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

#[test]
fn m47_the_jumps_stream_their_own_named_species() {
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    let (mut harness, _node) = mount_focused_slider(&observer);
    let id = harness.query(Role::Slider).named("Volume").first().id;

    // End: the jump to the maximum streams the bound's own species —
    // never a munged `SetValue` (the M11 rule's own words at the value
    // tier), the landing the funnel's own arithmetic, the ask the bound
    // itself.
    clear_trail(&observer);
    let response = press(&mut harness, Key::End);
    assert!(
        matches!(response, EventResponse::Handled),
        "the focused slider consumes its own End"
    );
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "one jump, one fact");
    assert_eq!(records[0].action, ValueAction::JumpToMax);
    assert_eq!(records[0].outcome, ValueOutcome::Delivered);
    assert_eq!(records[0].value, 100.0, "the jump landed the maximum");
    assert_eq!(
        records[0].requested,
        Some(100.0),
        "the bound is the ask — the funnel's clamp-and-snap never moved it"
    );
    assert_eq!(
        records[0].target,
        Some(id),
        "the key route stamps the focused control (the M45 discipline)"
    );

    // Home beside it: the other edge, the other species — the two jumps
    // tellable apart on one log.
    clear_trail(&observer);
    let response = press(&mut harness, Key::Home);
    assert!(matches!(response, EventResponse::Handled));
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the second jump streams one fact");
    assert_eq!(records[0].action, ValueAction::JumpToMin);
    assert_eq!(records[0].value, 0.0, "the jump landed the minimum");
    assert_eq!(records[0].requested, Some(0.0));
    assert_eq!(records[0].target, Some(id));
    assert_eq!(
        harness.settle(),
        0,
        "the answered jumps queue no extra work"
    );
}

#[test]
fn m47_no_munge_and_no_silence_the_vocabulary_holds() {
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    let (mut harness, _node) = mount_focused_slider(&observer);

    // The munge never happens: a jump's record carries the bound's own
    // species, and no `set_value`-tagged fact rides a Home/End chord —
    // ADR-0051's kill rule answered by the vocabulary, not by silence.
    clear_trail(&observer);
    let _ = press(&mut harness, Key::Home);
    harness.settle();
    let records = value_records(&observer);
    assert!(
        records
            .iter()
            .all(|record| record.action != ValueAction::SetValue),
        "a jump is never munged into a SetValue"
    );
    assert_eq!(records.len(), 1, "the jump is a named fact, not a silence");

    // The no-op delivery answers like the landed write it is (the M20
    // idle-frame property, the log naming it): Home at the minimum
    // already lands where the value is, and the funnel's own answer —
    // the jump happened — streams.
    clear_trail(&observer);
    let _ = press(&mut harness, Key::Home);
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(
        records.len(),
        1,
        "a no-op jump answers like the landed write it is"
    );
    assert_eq!(records[0].action, ValueAction::JumpToMin);
    assert_eq!(records[0].outcome, ValueOutcome::Delivered);
    assert_eq!(harness.settle(), 0, "the no-op jump queued no masks");
}

#[test]
fn m47_the_steps_and_the_jumps_tell_apart_on_one_log() {
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    let (mut harness, _node) = mount_focused_slider(&observer);

    // The arrows beside the bounds: the same key route, the species
    // telling the rides apart — the step's `Increment` beside the
    // jump's `JumpToMax`, one funnel, four named facts.
    clear_trail(&observer);
    let _ = press(&mut harness, Key::ArrowRight);
    let _ = press(&mut harness, Key::End);
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 2, "two chords, two facts, one log");
    assert_eq!(records[0].action, ValueAction::Increment);
    assert_eq!(records[1].action, ValueAction::JumpToMax);
    assert_eq!(records[0].requested, Some(55.0), "the step's ask");
    assert_eq!(records[1].requested, Some(100.0), "the bound's ask");
    assert_eq!(
        harness.settle(),
        0,
        "the answered chords queue no extra work"
    );
}

#[test]
fn m47_the_species_names_ride_the_wire_vocabulary() {
    // The wire's `action` field is the documented vocabulary's rule:
    // the new tags are the records' own names, `snake_case`, stable —
    // the additive rule the versioning discipline admits at version 1
    // (clients that ignore unknown field values are unaffected).
    assert_eq!(ValueAction::JumpToMin.name(), "jump_to_min");
    assert_eq!(ValueAction::JumpToMax.name(), "jump_to_max");
    assert_eq!(ValueAction::SetValue.name(), "set_value");
    assert_eq!(ValueAction::Increment.name(), "increment");
    assert_eq!(ValueAction::Decrement.name(), "decrement");
}

#[test]
fn m47_no_observer_no_trail_the_idle_contract_holds() {
    // The default path: no observer, no enabled trail — the jumps land
    // exactly as before and the frame costs what it always cost.
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
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    let response = press(&mut harness, Key::End);
    assert!(matches!(response, EventResponse::Handled));
    harness.settle();
    assert_eq!(
        harness.query(Role::Slider).named("Volume").first().value,
        Some("100".to_string()),
        "the jump landed exactly as before"
    );
    assert_eq!(harness.settle(), 0, "the disabled trail queues no work");
}

#[test]
fn m47_the_living_documents_re_agree_at_the_cut() {
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
        readme.contains("M0 through M50") && readme.contains("fifty-seven decision records"),
        "the README's status block and count re-agree at the cut"
    );
    assert!(
        architecture.contains("57 decision records"),
        "the architecture register re-agrees at 57 decision records"
    );
    // The protocol document carries the bound species (the wire's
    // documented vocabulary).
    assert!(
        protocol.contains("jump_to_min") && protocol.contains("jump_to_max"),
        "the protocol document documents the bound species"
    );
    // The catalog lists the demonstration.
    assert!(
        catalog.contains("bound_jumps"),
        "the examples catalog lists the bound jumps demo"
    );
    // The changelog stages the milestone.
    assert!(
        changelog.contains("M47 — the keyboard's jumps are named facts"),
        "the changelog stages M47"
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
