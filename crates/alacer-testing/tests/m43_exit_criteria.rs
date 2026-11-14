//! M43 exit-criteria integration tests (§13.3, ADR-0049): the value
//! record names its control. ADR-0046's first re-open rule ships: a
//! consumer needs the *who* — the control's identity on the record —
//! → the record grows beside the action's routing (the dispatch funnel
//! knows the id the action arrived under; the widget does not), priced
//! by a real reader — never a node id guessed at the note site. One
//! additive field (`ValueRecord::target` — the AccessKit-stable id the
//! funnel routed, `None` when the write rode no routed action), the
//! stamp side living on the funnel (the AT action route holds the id
//! while the widget's own door answers), the additive wire field the
//! versioning rule admits at version 1, and the embedded client
//! rendering the control beside the record. The writes themselves are
//! untouched: the funnel's clamp-and-snap answers exactly as before,
//! a refusal streams as it always streamed — the log is the log, the
//! A7 idle contract unchanged (one branch per write while nobody
//! listens).
//!
//! Every test runs the real pipeline (A4) with real widgets driven by
//! real AT actions and real key events — the exact route a status
//! line, an inspector client, or a test reads.

use alacer_core::inspect::{InspectEvent, InspectObserver, ValueAction, ValueOutcome};
use alacer_core::widget::A11yAction;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Progress, Slider};

/// The value-trail collector the tests install as the pipeline's
/// observer: the write facts stream into a bounded ring (the m40
/// helper's shape — here the target is the assertion source).
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
/// the m40 suite's shape, exactly.
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
/// the beat under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().values.clear();
}

/// Two sliders mounted through the real pipeline and observed — the
/// multi-control form the attribution rule is priced by: two value
/// tiers in one tree, the facts a trail reader must tell apart.
fn mount_two_sliders(harness: &mut TestHarness, observer: &Share) {
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let _volume = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .value(20.0)
            .step(1.0)
            .boxed(),
    );
    let _brightness = harness.add_child(
        root,
        Slider::new("Brightness", 0.0, 100.0)
            .value(60.0)
            .step(1.0)
            .boxed(),
    );
    harness.settle();
}

/// One AT action routed to the named control through the real funnel:
/// the response the route gave.
fn act_on(harness: &mut TestHarness, name: &str, action: A11yAction) -> bool {
    let id = harness.query(Role::Slider).named(name).first().id;
    harness.pipeline_mut().dispatch_a11y_action(id, action)
}

#[test]
fn m43_the_routed_action_names_its_control() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    mount_two_sliders(&mut harness, &observer);

    // The volume's SetValue: one fact streams, carrying the id the
    // action arrived under — the identity the query answered, stamped
    // by the funnel that routed it (never a node id guessed at the
    // note site, the widget not knowing its own id).
    let volume_id = harness.query(Role::Slider).named("Volume").first().id;
    clear_trail(&observer);
    let consumed = act_on(
        &mut harness,
        "Volume",
        A11yAction::SetValue("42".to_string()),
    );
    assert!(consumed, "the slider consumes its own value-tier write");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "one write, one fact");
    assert_eq!(records[0].action, ValueAction::SetValue);
    assert_eq!(records[0].outcome, ValueOutcome::Delivered);
    assert_eq!(
        records[0].target,
        Some(volume_id),
        "the record names the control the action arrived under"
    );

    // The brightness's SetValue: the same funnel, a different control —
    // the facts a multi-slider form streams are now tellable apart.
    let brightness_id = harness.query(Role::Slider).named("Brightness").first().id;
    assert_ne!(
        volume_id, brightness_id,
        "the two controls carry distinct identities on the wire"
    );
    clear_trail(&observer);
    let consumed = act_on(
        &mut harness,
        "Brightness",
        A11yAction::SetValue("75".to_string()),
    );
    assert!(consumed, "the second slider consumes its own write");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the second write streams one fact");
    assert_eq!(
        records[0].value, 75.0,
        "the second slider holds its own value"
    );
    assert_eq!(
        records[0].target,
        Some(brightness_id),
        "the record names the control the action arrived under"
    );
    assert_eq!(
        harness.settle(),
        0,
        "the answered writes queue no extra work"
    );
}

#[test]
fn m43_the_refusals_name_which_control_refused() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    // A slider and a reporting bar in one tree: the bar's read-only
    // tier (M21, ADR-0027) refuses every AT write — and now the
    // refusal names which control refused it, beside the slider's
    // answers naming their own control.
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let _volume = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .value(20.0)
            .step(1.0)
            .boxed(),
    );
    let _batch = harness.add_child(root, Progress::new("Batch", 0.0, 100.0).boxed());
    harness.settle();

    // The bar's refused write: the species beside the control's id —
    // the log answering "which control refused it" without a second
    // observation channel.
    let batch_id = harness
        .query(Role::ProgressIndicator)
        .named("Batch")
        .first()
        .id;
    clear_trail(&observer);
    let consumed = act_on_progress(
        &mut harness,
        batch_id,
        A11yAction::SetValue("42".to_string()),
    );
    assert!(!consumed, "a reporting widget has no write door");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the refusal streams one fact");
    assert_eq!(
        records[0].outcome,
        ValueOutcome::Refused(alacer_core::ValueRefusal::NoWriteDoor)
    );
    assert_eq!(
        records[0].target,
        Some(batch_id),
        "the refusal names the control that has no write door"
    );

    // The slider's refused parse beside it: the two facts name their
    // controls apart — a multi-control form's trail is attributable.
    let volume_id = harness.query(Role::Slider).named("Volume").first().id;
    clear_trail(&observer);
    let consumed = act_on(
        &mut harness,
        "Volume",
        A11yAction::SetValue("loud".to_string()),
    );
    assert!(!consumed, "the parse refusal answers as it always answered");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the parse refusal streams one fact");
    assert_eq!(
        records[0].outcome,
        ValueOutcome::Refused(alacer_core::ValueRefusal::InvalidPayload)
    );
    assert_eq!(
        records[0].target,
        Some(volume_id),
        "the parse refusal names its own control"
    );
    assert_eq!(harness.settle(), 0, "the refused writes queue no masks");
}

/// One value-tier action routed to an explicit id (the progress bar's
/// route — the m40 suite's `act`, generalized over the named control).
fn act_on_progress(harness: &mut TestHarness, id: u64, action: A11yAction) -> bool {
    harness.pipeline_mut().dispatch_a11y_action(id, action)
}

#[test]
fn m43_the_widget_doors_rides_re_agree_at_m45() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    // One slider, focused: the keyboard's step rode the widget's own
    // door, which M43's scope kept off the value log ("the input tier's
    // records carry those rides until a consumer prices them",
    // ADR-0046's second kill rule). M45 priced them: ADR-0049's first
    // kill rule cashed (ADR-0051) — the arrows' rides stream the same
    // record the AT tier streams, the funnel's key route stamping the
    // focused control, the ride attributed like every other fact. The
    // drag stays off (the per-pointer-move flood the kill rule
    // refuses), and the attribution stays the funnel's own answer.
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

    clear_trail(&observer);
    let response = harness
        .pipeline_mut()
        .dispatch_key(&alacer_core::events::KeyEvent {
            key: alacer_core::events::Key::ArrowRight,
            state: alacer_core::events::KeyState::Pressed,
            modifiers: alacer_core::events::Modifiers::default(),
            text: None,
        });
    assert!(
        matches!(response, alacer_core::widget::EventResponse::Handled),
        "the focused slider consumes its own arrow"
    );
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(
        records.len(),
        1,
        "the keyboard's ride streams one fact — the M45 attribution"
    );
    assert_eq!(
        records[0].action,
        alacer_core::inspect::ValueAction::Increment,
        "the ride carries the step tier's own action species"
    );
    assert_eq!(
        records[0].value, 21.0,
        "the step landed exactly as it always landed"
    );

    // The AT write beside it: the same funnel's two doors, both
    // attributable — one log, two routes, two names.
    let volume_id = harness.query(Role::Slider).named("Volume").first().id;
    clear_trail(&observer);
    let consumed = act_on(&mut harness, "Volume", A11yAction::Increment);
    assert!(consumed, "the AT's step tier answers as it always answered");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the AT's step streams one fact");
    assert_eq!(
        records[0].target,
        Some(volume_id),
        "the routed ride names its control"
    );
    assert_eq!(
        harness.settle(),
        0,
        "the answered rides queue no extra work"
    );
}

#[test]
fn m43_no_observer_no_trail_the_idle_contract_holds() {
    // The default path: no observer, no enabled trail — the AT writes
    // answer exactly as before and the frame costs what it always cost.
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let _volume = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .value(20.0)
            .step(1.0)
            .boxed(),
    );
    harness.settle();
    let id = harness.query(Role::Slider).named("Volume").first().id;
    let consumed = harness
        .pipeline_mut()
        .dispatch_a11y_action(id, A11yAction::SetValue("42".to_string()));
    assert!(consumed, "the write answers without a listener either");
    harness.settle();
    assert_eq!(
        harness.query(Role::Slider).named("Volume").first().value,
        Some("42".to_string()),
        "the value landed exactly as before"
    );
    assert_eq!(harness.settle(), 0, "the disabled trail queues no work");
}

#[test]
fn m43_the_living_documents_re_agree_at_m43() {
    let root = workspace_root();
    let readme = std::fs::read_to_string(root.join("README.md")).expect("readme");
    let architecture =
        std::fs::read_to_string(root.join("docs/architecture.md")).expect("architecture");
    let protocol = std::fs::read_to_string(root.join("docs/inspector-protocol.md"))
        .expect("protocol document");
    let catalog = std::fs::read_to_string(root.join("docs/examples.md")).expect("catalog");
    let changelog = std::fs::read_to_string(root.join("CHANGELOG.md")).expect("changelog");

    // The status block and the ADR count re-agree at M43 (the m-suite
    // pins moving M41 → M43 and 48 → 50 across the chain — M42 and
    // M43 landing in one cut, the workspace's one-milestone-per-minor
    // rule counting them as the minor's two entries).
    assert!(
        readme.contains("M0 through M57") && readme.contains("sixty-four decision records"),
        "the README's status block and count re-agree at M43"
    );
    assert!(
        architecture.contains("64 decision records"),
        "the architecture register re-agrees at 64 decision records"
    );
    // The protocol document carries the target field (the wire's
    // documented shape).
    assert!(
        protocol.contains("`target`") && protocol.contains("which control"),
        "the protocol document documents the target field"
    );
    // The catalog lists the demonstration.
    assert!(
        catalog.contains("value_targets"),
        "the examples catalog lists the value attribution demo"
    );
    // The changelog stages the milestone.
    assert!(
        changelog.contains("M43 — the value record names its control"),
        "the changelog stages M43"
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
