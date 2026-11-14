//! M60 exit-criteria integration tests (§13.3, ADR-0066): the clipboard
//! chords' field changes join the key-edit vocabulary. ADR-0064's third
//! kill rule cashes — a consumer prices the chords' edits *at the field*
//! on the log: the paste's shape (how much the insertion consumed and
//! carried, priced by lengths, never the payload — a field's value can
//! carry a secret, ADR-0045), the cut's span (the range the deletion
//! took when the value moved out) — today the chords stream the
//! clipboard round trip and the journal's depths, but the field's own
//! change streamed no typing-door fact: "what did the paste change in
//! this field" was answerable only by re-pricing the round trip's
//! counts against the journal's. The chords join the vocabulary behind
//! their own actions ([`alacer_core::KeyEditAction::Paste`],
//! [`alacer_core::KeyEditAction::Cut`]) and driver
//! ([`alacer_core::KeyEditDriver::Clipboard`]) — never munged onto the
//! inserts and deletes they mirror, and never a double-priced edit: one
//! fact per trail, the round trip the clipboard trail's, the field's
//! change the typing door's.
//!
//! Every test runs the real pipeline (A4) with the real clipboard
//! chords routed the way the platform's keys arrive — the key route,
//! the funnel stamp riding.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{InspectEvent, InspectObserver};
use alacer_core::widget::{Role, TextSelection};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

/// The key-edit trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the chords' facts are the assertion source).
struct TrailObserver {
    events: std::collections::VecDeque<InspectEvent>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            events: std::collections::VecDeque::with_capacity(64),
        }
    }

    /// The streamed key-edit facts, oldest first.
    fn key_edits(&self) -> Vec<alacer_core::KeyEditRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::KeyEdit(record) => Some(*record),
                _ => None,
            })
            .collect()
    }

    /// The streamed clipboard round trips, oldest first.
    fn clipboards(&self) -> Vec<alacer_core::ClipboardRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::Clipboard(record) => Some(*record),
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

/// The streamed key-edit facts, oldest first.
fn key_edits(observer: &Share) -> Vec<alacer_core::KeyEditRecord> {
    observer.0.borrow().key_edits()
}

/// The streamed clipboard round trips, oldest first.
fn clipboards(observer: &Share) -> Vec<alacer_core::ClipboardRecord> {
    observer.0.borrow().clipboards()
}

/// Empties the shared ring between beats: each assertion reads the
/// facts the chord under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// An input mounted through the real pipeline, focused and settled —
/// the typing door's own mount. Returns the harness, the input's
/// AccessKit-stable id (the funnel's stamp vocabulary) and its arena
/// node.
fn mount_input(observer: &Share) -> (TestHarness, u64, alacer_core::NodeId) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(root, TextInput::new("field").boxed());
    harness.settle();
    let id = harness.query(Role::TextInput).named("field").first().id;
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    (harness, id, node)
}

/// A Ctrl press (the clipboard chords' command set; ADR-0020) — the
/// exact event the platform delivers when the user pastes or cuts.
fn ctrl(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            ..Modifiers::default()
        },
        text: None,
    }
}

#[test]
fn m60_the_paste_names_its_shape_and_its_driver() {
    let observer = Share::new();
    let (mut harness, id, _node) = mount_input(&observer);
    harness.set_clipboard("seed");
    clear_trail(&observer);
    harness.key(&ctrl(Key::Char('v')));
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "the paste's field change is one fact");
    let edit = edits[0];
    assert_eq!(
        edit.action,
        alacer_core::KeyEditAction::Paste,
        "the chord is never a munged Insert (the M11 vocabulary's own word)"
    );
    assert_eq!(
        edit.driver,
        alacer_core::KeyEditDriver::Clipboard,
        "the driver names the chord's own door, never the keyboard's"
    );
    assert_eq!(edit.replaced, 0, "a paste at the caret consumed no span");
    assert_eq!(
        edit.inserted, 4,
        "the payload's UTF-16 length — the shape, never the payload"
    );
    assert_eq!(
        edit.selection,
        TextSelection::new(4, 4),
        "the landing caret, anchor collapsed onto it"
    );
    assert_eq!(
        edit.target,
        Some(id),
        "the key route's stamp: the fact names the control it rode"
    );
}

#[test]
fn m60_the_paste_over_a_selection_prices_both_spans() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    harness.type_text("copy me");
    harness.set_clipboard("new");
    clear_trail(&observer);
    harness.key(&ctrl(Key::Char('a')));
    harness.key(&ctrl(Key::Char('v')));
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "select-all rides no edit; the paste is one");
    let edit = edits[0];
    assert_eq!(edit.action, alacer_core::KeyEditAction::Paste);
    assert_eq!(edit.replaced, 7, "the selection the paste consumed, priced");
    assert_eq!(edit.inserted, 3, "the payload the paste carried, priced");
    assert_eq!(
        edit.selection,
        TextSelection::new(3, 3),
        "the landing caret"
    );
}

#[test]
fn m60_the_cut_names_the_span_it_moved() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    harness.type_text("moved out");
    clear_trail(&observer);
    harness.key(&ctrl(Key::Char('a')));
    harness.key(&ctrl(Key::Char('x')));
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "the cut's field change is one fact");
    let edit = edits[0];
    assert_eq!(
        edit.action,
        alacer_core::KeyEditAction::Cut,
        "the cut is never a munged Delete (the chord's own species)"
    );
    assert_eq!(edit.driver, alacer_core::KeyEditDriver::Clipboard);
    assert_eq!(
        edit.replaced, 9,
        "the span the cut moved out, priced by lengths"
    );
    assert_eq!(edit.inserted, 0, "a cut carries no payload into the field");
    assert_eq!(
        edit.selection,
        TextSelection::new(0, 0),
        "the caret the emptied field left"
    );
}

#[test]
fn m60_the_refused_paste_rides_the_honest_absence() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    // The harness's store was never seeded: the read refuses, the field
    // inserts nothing — the journal recorded nothing, the trail queues
    // none (the clipboard trail names the refusal; the typing door
    // answers no fact).
    clear_trail(&observer);
    harness.key(&ctrl(Key::Char('v')));
    assert!(
        key_edits(&observer).is_empty(),
        "a refused paste answers no typing-door fact"
    );
    let trips = clipboards(&observer);
    assert_eq!(trips.len(), 1, "the refusal is the clipboard trail's fact");
    assert!(
        matches!(trips[0].outcome, alacer_core::ClipboardOutcome::Refused(_)),
        "the round trip names why it refused"
    );
}

#[test]
fn m60_the_copy_edits_nothing_and_streams_none() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    harness.type_text("copy me");
    clear_trail(&observer);
    harness.key(&ctrl(Key::Char('a')));
    harness.key(&ctrl(Key::Char('c')));
    assert!(
        key_edits(&observer).is_empty(),
        "a copy edits nothing — no journal step, no typing-door fact"
    );
    assert_eq!(
        clipboards(&observer).len(),
        1,
        "the round trip is the clipboard trail's own fact"
    );
}

#[test]
fn m60_the_chord_fact_never_merges_the_typed_run() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    harness.set_clipboard("y");
    clear_trail(&observer);
    // The typed run and the chord routed inside one tick window (raw
    // dispatches — the harness's typed doors settle between calls, and
    // a settle is a drain): the paste the journal folded into the
    // run's tip lands beside the run's record — the drivers' records
    // never merge even when the journal's tip folded the shape.
    for ch in "he".chars() {
        let _ = harness.pipeline_mut().dispatch_key(&KeyEvent::char(ch));
    }
    let _ = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('v')));
    harness.settle();
    let edits = key_edits(&observer);
    assert_eq!(
        edits.len(),
        2,
        "the paste lands beside the typed run — the drivers' records never merge even when the journal's tip folded the shape"
    );
    assert_eq!(edits[0].driver, alacer_core::KeyEditDriver::Keyboard);
    assert_eq!(edits[0].inserted, 2, "the run's own shape");
    assert_eq!(edits[1].driver, alacer_core::KeyEditDriver::Clipboard);
    assert_eq!(
        edits[1].action,
        alacer_core::KeyEditAction::Paste,
        "a following chord is never superseded into the run's record"
    );
    assert_eq!(edits[1].inserted, 1, "the chord's own shape");
    assert!(
        edits[1].coalesced,
        "the journal folded the shape — the trail kept the records apart"
    );
}

#[test]
fn m60_the_round_trip_and_the_field_change_stream_beside() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    harness.set_clipboard("pair");
    clear_trail(&observer);
    harness.key(&ctrl(Key::Char('v')));
    // One chord, two trails, one fact each — tellable apart on one log:
    // the round trip (the clipboard trail's) and the field change (the
    // typing door's). Never a double-priced edit: neither trail prices
    // the other tier's fact.
    assert_eq!(
        clipboards(&observer).len(),
        1,
        "the round trip's fact, exactly one"
    );
    assert_eq!(
        key_edits(&observer).len(),
        1,
        "the field change's fact, exactly one"
    );
}
