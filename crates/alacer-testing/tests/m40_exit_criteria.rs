//! M40 exit-criteria integration tests (§13.3, ADR-0046): the value tier
//! rides the causal log. The ride ADR-0010's additive rule and the
//! M31/M33/M36/M37/M39 rhythm pre-register cashed: the value tier's AT
//! actions (M20, ADR-0026 — the `Slider`; M21, ADR-0027 — the read-only
//! `Progress`) answered the widget's own `bool`, whose `false` collapsed
//! four different facts (the control the availability tier holds down,
//! the payload that parsed to nothing, the payload the funnel cannot
//! carry, the control that never had a write door) into one silence. Now
//! every value-tier action the AT asks — a `SetValue`, an `Increment`, a
//! `Decrement` — streams one [`ValueRecord`] into the thread's value
//! trail, and the dispatching pipeline drains it at the tick's start
//! into [`InspectEvent::Value`] — the clipboard trail's shape (push now,
//! the tick routes, one frame), the refusal tier's honesty (a refused
//! write streams too, the species naming why nothing moved). The writes
//! themselves are untouched: the funnel clamps and snaps exactly as
//! before, a no-op delivery queues no masks, the A7 idle contract holds
//! (one branch per action while nobody listens).
//!
//! Every test runs the real pipeline (A4) with a real `Slider` and a
//! real `Progress` driven by real a11y actions — the exact route the
//! platform's AccessKit adapter rides, the route a status line, an
//! inspector client, or a test reads.

use alacer_core::inspect::{
    InspectEvent, InspectObserver, ValueAction, ValueOutcome, ValueRecord, ValueRefusal,
};
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Progress, Slider};

/// The float equality the tests own (the widgets' own rule): `total_cmp`
/// ordering — `f32::eq` is a lint trap the whole workspace refuses.
fn same(a: f32, b: f32) -> bool {
    a.total_cmp(&b) == std::cmp::Ordering::Equal
}

/// The value-trail collector the tests install as the pipeline's
/// observer: the value facts stream into a bounded ring alongside the
/// arrival-order kind list (the m39 helper's shape — here the value
/// record is the assertion source).
struct ValueObserver {
    values: std::collections::VecDeque<ValueRecord>,
    kinds: std::collections::VecDeque<&'static str>,
}

impl ValueObserver {
    fn new() -> Self {
        Self {
            values: std::collections::VecDeque::with_capacity(64),
            kinds: std::collections::VecDeque::with_capacity(64),
        }
    }
}

impl InspectObserver for ValueObserver {
    fn observe(&mut self, event: InspectEvent) {
        match event {
            InspectEvent::Value(record) => {
                self.push("value");
                if self.values.len() >= 64 {
                    self.values.pop_front();
                }
                self.values.push_back(record);
            }
            InspectEvent::Edit(_) => self.push("edit"),
            InspectEvent::Focus(_) => self.push("focus"),
            InspectEvent::Input(_) => self.push("input"),
            InspectEvent::Frame(_) => self.push("frame"),
            _ => {}
        }
    }
}

impl ValueObserver {
    /// One arrival, whatever kind it carried — the trail's shape as data.
    fn push(&mut self, kind: &'static str) {
        if self.kinds.len() >= 64 {
            self.kinds.pop_front();
        }
        self.kinds.push_back(kind);
    }
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the m39 suite's shape, exactly.
struct Share(std::rc::Rc<std::cell::RefCell<ValueObserver>>);

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

/// The streamed value facts, oldest first.
fn value_records(observer: &Share) -> Vec<ValueRecord> {
    observer.0.borrow().values.iter().copied().collect()
}

/// Empties the shared ring between beats: each assertion reads the facts
/// the beat under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    let mut trail = observer.0.borrow_mut();
    trail.values.clear();
    trail.kinds.clear();
}

/// A slider mounted through the real pipeline and observed — the value
/// trail rides from the `set_observer` call on (the m39 mount's shape).
fn mount_slider(harness: &mut TestHarness, observer: &Share) {
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
    harness.settle();
}

/// A progress bar mounted through the real pipeline and observed — the
/// reporting widget's read-only tier under the same observer.
fn mount_progress(harness: &mut TestHarness, observer: &Share) {
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let _batch = harness.add_child(root, Progress::new("Batch", 0.0, 100.0).boxed());
    harness.settle();
}

/// One value-tier action through the real funnel: the response the
/// route gave.
fn act(harness: &mut TestHarness, action: A11yAction) -> bool {
    let id = harness.query(Role::Slider).named("Volume").first().id;
    harness.pipeline_mut().dispatch_a11y_action(id, action)
}

#[test]
fn m40_the_value_tier_streams_one_fact_per_write() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    mount_slider(&mut harness, &observer);

    // The AT's SetValue: the funnel parses, clamps, and snaps exactly as
    // before — one fact streams, the landed value beside it, the tree
    // carrying what the record says.
    clear_trail(&observer);
    let consumed = act(&mut harness, A11yAction::SetValue("42".to_string()));
    assert!(consumed, "the slider consumes its own value-tier write");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "one write, one fact");
    assert_eq!(records[0].action, ValueAction::SetValue);
    assert_eq!(records[0].outcome, ValueOutcome::Delivered);
    assert!(
        same(records[0].value, 42.0),
        "the record carries the landed value, got {}",
        records[0].value
    );
    assert_eq!(
        harness.query(Role::Slider).named("Volume").first().value,
        Some("42".to_string()),
        "the tree carries exactly what the record streams"
    );

    // The AT's step tier: an Increment and a Decrement ride the same
    // funnel, one fact each, the landed value beside it.
    clear_trail(&observer);
    let consumed = act(&mut harness, A11yAction::Increment);
    assert!(consumed, "the slider consumes its own step");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the increment streams one fact");
    assert_eq!(records[0].action, ValueAction::Increment);
    assert_eq!(records[0].outcome, ValueOutcome::Delivered);
    assert!(same(records[0].value, 43.0), "the step landed");

    clear_trail(&observer);
    let consumed = act(&mut harness, A11yAction::Decrement);
    assert!(consumed, "the slider consumes its own step");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the decrement streams one fact");
    assert_eq!(records[0].action, ValueAction::Decrement);
    assert!(same(records[0].value, 42.0), "the step back landed");
    assert_eq!(harness.settle(), 0, "the value tier queues no extra masks");
}

#[test]
fn m40_the_refused_writes_stream_too() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    // The availability tier (M22, ADR-0028) holding the control down:
    // every door refuses before the request is even parsed — and the
    // refusal is a fact the log carries, the species naming why.
    let root = harness.mount(Column::new().boxed());
    let _locked = harness.add_child(
        root,
        Slider::new("Locked", 0.0, 100.0).enabled(false).boxed(),
    );
    // The live control the parse beats ride: the availability tier
    // refuses first (an unavailable control answers nothing), so the
    // funnel's own species need a slider that is actually on.
    let _live = harness.add_child(root, Slider::new("Live", 0.0, 100.0).value(7.0).boxed());
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    harness.settle();
    let id = harness.query(Role::Slider).named("Locked").first().id;
    let live_id = harness.query(Role::Slider).named("Live").first().id;

    clear_trail(&observer);
    let consumed = harness
        .pipeline_mut()
        .dispatch_a11y_action(id, A11yAction::SetValue("10".to_string()));
    assert!(!consumed, "an unavailable slider refuses the write");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the refusal is a fact the log carries");
    assert_eq!(records[0].action, ValueAction::SetValue);
    assert_eq!(
        records[0].outcome,
        ValueOutcome::Refused(ValueRefusal::Unavailable)
    );
    assert!(
        same(records[0].value, 0.0),
        "the tree keeps the value the app wrote"
    );
    assert_eq!(harness.settle(), 0, "a refused write queues no masks");

    // A payload that parses to nothing: the AT's string was not a
    // number — the funnel refused before the range ever saw it.
    clear_trail(&observer);
    let consumed = harness
        .pipeline_mut()
        .dispatch_a11y_action(live_id, A11yAction::SetValue("loud".to_string()));
    assert!(!consumed, "a malformed payload is refused");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the parse refusal streams");
    assert_eq!(
        records[0].outcome,
        ValueOutcome::Refused(ValueRefusal::InvalidPayload),
        "the malformed payload is its own species, not the unavailable door's"
    );
    assert!(
        same(records[0].value, 7.0),
        "the live slider keeps the value the app wrote"
    );

    // A payload that parses but is not a finite scalar: "inf" parses
    // fine — the funnel's own wall is the fact, a different species than
    // the parse's (the m20 suite's two refusals, now named apart).
    clear_trail(&observer);
    let consumed = harness
        .pipeline_mut()
        .dispatch_a11y_action(live_id, A11yAction::SetValue("inf".to_string()));
    assert!(!consumed, "a non-finite payload is refused");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the funnel's wall streams");
    assert_eq!(
        records[0].outcome,
        ValueOutcome::Refused(ValueRefusal::NonFiniteValue),
        "the funnel's refusal is its own species, not the parse's"
    );

    // A question outside the value tier: a DefaultPress is not a value
    // control's contract — refused, never munged (the M11 rule), and no
    // value fact streams (the clipboard trail's own scope: the record
    // documents the tier the AT asked about).
    clear_trail(&observer);
    let consumed = harness
        .pipeline_mut()
        .dispatch_a11y_action(live_id, A11yAction::DefaultPress);
    assert!(!consumed, "a press is not a value control's contract");
    harness.settle();
    assert!(
        value_records(&observer).is_empty(),
        "a non-value question is not a value fact — the record's scope holds"
    );
}

#[test]
fn m40_the_reporting_widget_names_its_read_only_tier() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    mount_progress(&mut harness, &observer);
    let id = harness
        .query(Role::ProgressIndicator)
        .named("Batch")
        .first()
        .id;

    // The read-only tier (M21, ADR-0027): every value-tier action is
    // refused — and now the refusal names the species: a reporting
    // widget has no write door. The tree keeps the value the app wrote.
    clear_trail(&observer);
    let consumed = harness
        .pipeline_mut()
        .dispatch_a11y_action(id, A11yAction::SetValue("50".to_string()));
    assert!(!consumed, "a reporting widget has no write door");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the read-only tier's refusal streams");
    assert_eq!(records[0].action, ValueAction::SetValue);
    assert_eq!(
        records[0].outcome,
        ValueOutcome::Refused(ValueRefusal::NoWriteDoor)
    );
    assert!(
        same(records[0].value, 0.0),
        "the tree keeps the value the app wrote"
    );

    clear_trail(&observer);
    let consumed = harness
        .pipeline_mut()
        .dispatch_a11y_action(id, A11yAction::Increment);
    assert!(!consumed, "no step tier either");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(
        records[0].outcome,
        ValueOutcome::Refused(ValueRefusal::NoWriteDoor),
        "the step tier refuses the same honest species"
    );
    assert_eq!(harness.settle(), 0, "a refused act queues no masks");
}

#[test]
fn m40_a_noop_write_is_an_answered_request() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    mount_slider(&mut harness, &observer);

    // The write that lands where the value already is (M20's idle-frame
    // property): an answered request, not a refusal — the record says
    // `delivered` at the value the control already carried, and the
    // frame queues nothing.
    let _ = act(&mut harness, A11yAction::SetValue("42".to_string()));
    harness.settle();
    clear_trail(&observer);
    let consumed = act(&mut harness, A11yAction::SetValue("42".to_string()));
    assert!(consumed, "the funnel answers the request it can carry");
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the no-op is still a fact");
    assert_eq!(records[0].outcome, ValueOutcome::Delivered);
    assert!(same(records[0].value, 42.0));
    assert_eq!(
        harness.settle(),
        0,
        "a no-op delivery queues no masks — the idle frame the log names"
    );
}

#[test]
fn m40_the_drain_streams_once_a_frame_late() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        ValueObserver::new(),
    )));
    mount_slider(&mut harness, &observer);

    // Push now, the tick routes (the clipboard trail's shape): the act
    // queues the fact on the thread trail, and the tick's start drain
    // streams it — one frame after the write resolved.
    let id = harness.query(Role::Slider).named("Volume").first().id;
    clear_trail(&observer);
    let consumed = harness
        .pipeline_mut()
        .dispatch_a11y_action(id, A11yAction::SetValue("55".to_string()));
    assert!(consumed);
    assert!(
        value_records(&observer).is_empty(),
        "the fact queues where the write resolves, before any tick"
    );
    harness.settle();
    let records = value_records(&observer);
    assert_eq!(records.len(), 1, "the drain streams at the tick's start");
    assert!(same(records[0].value, 55.0));

    // A fact streams exactly once: the next tick with no value work
    // streams nothing (the drain clears, the clipboard trail's rule).
    clear_trail(&observer);
    harness.settle();
    assert!(
        value_records(&observer).is_empty(),
        "an idle value tier queues nothing"
    );
    assert_eq!(harness.settle(), 0, "the trail ride costs no masks");
}

#[test]
fn m40_no_observer_no_trail_the_idle_contract_holds() {
    // The default path: no observer, no enabled trail — the writes
    // answer exactly as before and the frame costs what it always cost.
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let _volume = harness.add_child(root, Slider::new("Volume", 0.0, 100.0).boxed());
    harness.settle();
    let id = harness.query(Role::Slider).named("Volume").first().id;
    let consumed = harness
        .pipeline_mut()
        .dispatch_a11y_action(id, A11yAction::SetValue("30".to_string()));
    assert!(consumed, "the write answers without a listener");
    harness.settle();
    assert_eq!(
        harness.query(Role::Slider).named("Volume").first().value,
        Some("30".to_string()),
        "the round trip lands exactly as before"
    );
    assert_eq!(harness.settle(), 0, "the disabled trail queues no work");
}

#[test]
fn m40_the_living_documents_re_agree_at_m40() {
    let root = workspace_root();
    let readme = std::fs::read_to_string(root.join("README.md")).expect("readme");
    let architecture =
        std::fs::read_to_string(root.join("docs/architecture.md")).expect("architecture");
    let protocol = std::fs::read_to_string(root.join("docs/inspector-protocol.md"))
        .expect("protocol document");
    let catalog = std::fs::read_to_string(root.join("docs/examples.md")).expect("catalog");
    let changelog = std::fs::read_to_string(root.join("CHANGELOG.md")).expect("changelog");

    // The status block and the ADR count re-agree at M40 (the m-suite
    // pins moving M41 → M43 and 48 → 50 across the chain — M42 and
    // M43 landing in one cut).
    assert!(
        readme.contains("M0 through M45") && readme.contains("fifty-two decision records"),
        "the README's status block and count re-agree at M40"
    );
    assert!(
        architecture.contains("52 decision records"),
        "the architecture register re-agrees at 47 decision records"
    );
    // The protocol document carries the value event (the wire's
    // documented shape).
    assert!(
        protocol.contains("\"kind\":\"value\"") && protocol.contains("`no_write_door`"),
        "the protocol document documents the value event"
    );
    // The catalog lists the demonstration.
    assert!(
        catalog.contains("value_trail"),
        "the examples catalog lists the value trail demo"
    );
    // The changelog stages the milestone.
    assert!(
        changelog.contains("M40 — the value tier rides the causal log"),
        "the changelog stages M40"
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
