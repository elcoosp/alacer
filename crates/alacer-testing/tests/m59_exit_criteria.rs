//! M59 exit-criteria integration tests (§13.3, ADR-0065): the IME's
//! boundary drops join the route-drop vocabulary. ADR-0063's first
//! kill rule cashes — a consumer prices the platform's other silences
//! on the log: an IME event no live field held (no field has focus,
//! or the focused node has gone) dropped with only a consumed
//! `Ignored` verdict — "my IME committed; where did the text go" was
//! unanswerable from the log alone. The route's own vocabulary grows
//! the species ([`alacer_core::RouteDropSpecies::NoFocusedField`]):
//! the event named by its stable tag (`ime_preedit`/`ime_commit`),
//! the aim riding its honest absence (the boundary had no aim to name
//! — the drop *is* the missing field).
//!
//! Every test runs the real pipeline (A4): the event routed the real
//! way — the drop noted where it resolves, the delivered commit
//! beside it streaming nothing (the honest absence, the M57 rule).

use alacer_core::events::ImeEvent;
use alacer_core::inspect::{InspectEvent, InspectObserver};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

/// The route-drop trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the drops' facts are the assertion source).
struct TrailObserver {
    events: std::collections::VecDeque<InspectEvent>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            events: std::collections::VecDeque::with_capacity(64),
        }
    }

    /// The streamed route-drop facts, oldest first.
    fn route_drops(&self) -> Vec<alacer_core::RouteDropRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::RouteDrop(record) => Some(*record),
                _ => None,
            })
            .collect()
    }
}

impl InspectObserver for TrailObserver {
    fn observe(&mut self, event: InspectEvent) {
        if self.events.len() >= 64 {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the m-suite's shape, exactly.
struct Share(std::rc::Rc<std::cell::RefCell<TrailObserver>>);

impl Share {
    fn new() -> Self {
        Self(std::rc::Rc::new(std::cell::RefCell::new(
            TrailObserver::new(),
        )))
    }
}

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

/// The streamed route-drop facts, oldest first.
fn route_drops(observer: &Share) -> Vec<alacer_core::RouteDropRecord> {
    observer.0.borrow().route_drops()
}

/// Empties the shared ring between beats: each assertion reads the
/// facts the drop under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// A harness with the observer riding and no field focused: the
/// route's silence is the assertion source. Returns the harness and
/// the field's arena node (the delivered beat's focus address).
fn mount_observer(observer: &Share) -> (TestHarness, alacer_core::NodeId) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let field = harness.add_child(root, TextInput::new("field").boxed());
    harness.settle();
    (harness, field)
}

#[test]
fn m59_the_commit_no_field_held_names_its_drop() {
    let observer = Share::new();
    let (mut harness, _field) = mount_observer(&observer);
    clear_trail(&observer);
    // No field holds focus: the committed composition drops before any
    // widget's door — the fact streams, the aim riding its honest
    // absence.
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Commit {
        text: "よ!".into()
    });
    harness.settle();
    let drops = route_drops(&observer);
    assert_eq!(drops.len(), 1, "the drop is a fact the log carries");
    assert_eq!(drops[0].action, "ime_commit", "the event's own tag");
    assert_eq!(
        drops[0].species,
        alacer_core::RouteDropSpecies::NoFocusedField,
        "the route's other silence, named"
    );
    assert_eq!(
        drops[0].target, None,
        "the boundary had no aim to name — the drop is the missing field"
    );
}

#[test]
fn m59_the_preedit_no_field_held_names_its_drop() {
    let observer = Share::new();
    let (mut harness, _field) = mount_observer(&observer);
    clear_trail(&observer);
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Preedit {
        text: "よ".into(),
        cursor_utf16: Some(1),
    });
    harness.settle();
    let drops = route_drops(&observer);
    assert_eq!(drops.len(), 1);
    assert_eq!(drops[0].action, "ime_preedit");
    assert_eq!(
        drops[0].species,
        alacer_core::RouteDropSpecies::NoFocusedField
    );
}

#[test]
fn m59_the_delivered_commit_streams_no_drop() {
    let observer = Share::new();
    let (mut harness, field) = mount_observer(&observer);
    // The field takes focus: the commit lands at its door — a
    // delivered event is not a drop (the honest absence, the M57
    // rule).
    harness.pipeline_mut().request_focus(field);
    harness.settle();
    clear_trail(&observer);
    let _ = harness
        .pipeline_mut()
        .dispatch_ime(&ImeEvent::Commit { text: "よ".into() });
    harness.settle();
    assert!(
        route_drops(&observer).is_empty(),
        "a delivered commit streams no drop — the honest absence"
    );
}
