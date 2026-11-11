//! M31 exit-criteria integration tests (§13.3, ADR-0037): the journal
//! rides the causal log. The hook the journal's own docs named — the
//! depth "the inspector may someday surface" — ships: every
//! [`EditHistory`] action streams one [`EditRecord`] (the action, the
//! depths it left, the refusal it named) into the thread's edit trail,
//! and the dispatching pipeline drains it at the tick's start into
//! [`InspectEvent::Edit`] — the announcement drain's shape (push now,
//! the tick routes, one frame), the focus trail's honesty (a refused
//! chord streams too). The walk behind the journal is untouched: the
//! coalescing, reset, and bounded rules of ADR-0021 answer exactly as
//! before, and with no observer installed the trail stays silent — one
//! branch per action, the A7 idle contract unchanged.
//!
//! Every test runs the real pipeline (A4) with a real `TextInput` driven
//! by real key events — the exact route a status line, an inspector
//! client, or a test reads.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::history::EditAction;
use alacer_core::inspect::{EditRecord, InspectEvent, InspectObserver};
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

/// The edit-trail collector the tests install as the pipeline's
/// observer: the journal facts stream into a bounded ring (the m28/m30
/// helper's shape — here the edit record is the assertion source).
struct EditObserver {
    edits: std::collections::VecDeque<EditRecord>,
}

impl EditObserver {
    fn new() -> Self {
        Self {
            edits: std::collections::VecDeque::with_capacity(64),
        }
    }
}

impl InspectObserver for EditObserver {
    fn observe(&mut self, event: InspectEvent) {
        if let InspectEvent::Edit(record) = event {
            if self.edits.len() >= 64 {
                self.edits.pop_front();
            }
            self.edits.push_back(record);
        }
    }
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the m30 suite's shape, exactly.
struct Share(std::rc::Rc<std::cell::RefCell<EditObserver>>);

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

/// The ring's read half: every streamed fact, oldest first.
fn edits(observer: &Share) -> Vec<EditRecord> {
    observer.0.borrow().edits.iter().copied().collect()
}

/// Empties the shared ring between beats: each assertion reads the facts
/// the beat under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().edits.clear();
}

/// An input mounted through the real pipeline, focused so every key
/// under test routes to the field, and observed — the edit trail rides
/// from the `set_observer` call on (the m15 mount, plus the observer).
fn mount_input(harness: &mut TestHarness, observer: &Share) {
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();
    // Focus resolves in pass 3 of a tick: settle before typing, or the
    // keys route to no focused widget (the M11 suite's same rule).
    harness.pipeline_mut().request_focus(node);
    harness.settle();
}

/// A Ctrl press (the command set on Linux/Windows; Meta is the same tier
/// on macOS).
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
fn m31_the_typed_run_streams_one_coalesced_fact_per_keystroke() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        EditObserver::new(),
    )));
    mount_input(&mut harness, &observer);
    harness.settle();
    clear_trail(&observer);

    // Three keystrokes ride the widget's edit funnel: every one notes a
    // `record` fact — the first opens the step, the rest coalesce into
    // the tip (the pure-append rule, visible in the log as depth that
    // never moves).
    harness.type_text("hey");
    harness.settle();

    let trail = edits(&observer);
    assert_eq!(trail.len(), 3, "every keystroke's journal action streamed");
    for (index, record) in trail.iter().enumerate() {
        assert_eq!(
            record.action,
            EditAction::Record,
            "fact {index} is a record"
        );
        assert!(!record.refused, "a typed keystroke never refuses");
        assert_eq!(record.undo_depth, 1, "the typed run is one step deep");
        assert_eq!(record.redo_depth, 0, "an edit clears the redo branch");
    }
    assert!(
        !trail[0].coalesced,
        "the first keystroke opened the journal's step"
    );
    assert!(
        trail[1].coalesced && trail[2].coalesced,
        "the continued keystrokes folded into the tip"
    );
    assert_eq!(
        harness.query(Role::TextInput).named("Search").first().value,
        Some("hey".to_string()),
        "the journal walked exactly as before — the log is the log"
    );
}

#[test]
fn m31_the_chords_stream_and_the_refused_chord_streams_too() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        EditObserver::new(),
    )));
    mount_input(&mut harness, &observer);

    // The refused undo: an empty journal, the chord refused with no
    // masks (the M15 rule) — and the trail carries the refusal, the
    // depths naming the journal it did not move (the M28 honesty, the
    // edit tier's own).
    clear_trail(&observer);
    let response = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('z')));
    assert!(
        matches!(response, alacer_core::widget::EventResponse::Handled),
        "the field consumes its own chord even when the journal refuses"
    );
    harness.settle();
    let trail = edits(&observer);
    assert_eq!(trail.len(), 1, "the refusal is a fact the log carries");
    assert_eq!(trail[0].action, EditAction::Undo);
    assert!(trail[0].refused, "the refusal is named, never munged");
    assert_eq!((trail[0].undo_depth, trail[0].redo_depth), (0, 0));
    assert_eq!(harness.settle(), 0, "a refused chord queues no masks");

    // A typed run, then the applied undo: the fact names the depth it
    // left, the redo branch it opened.
    harness.type_text("hi");
    harness.settle();
    clear_trail(&observer);
    harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('z')));
    harness.settle();
    let trail = edits(&observer);
    assert_eq!(trail.len(), 1, "the applied undo streamed");
    assert_eq!(trail[0].action, EditAction::Undo);
    assert!(!trail[0].refused, "the chord was answered, not refused");
    assert_eq!(trail[0].undo_depth, 0, "the undo emptied the branch");
    assert_eq!(trail[0].redo_depth, 1, "the undone step is replayable");

    // The redo re-pushes the step: the fact mirrors it.
    clear_trail(&observer);
    harness.pipeline_mut().dispatch_key(&ctrl_shift_redo());
    harness.settle();
    let trail = edits(&observer);
    assert_eq!(trail.len(), 1, "the applied redo streamed");
    assert_eq!(trail[0].action, EditAction::Redo);
    assert!(!trail[0].refused);
    assert_eq!(trail[0].undo_depth, 1, "the redo re-pushed the step");
    assert_eq!(trail[0].redo_depth, 0, "the redo branch drained");
}

#[test]
fn m31_the_app_write_reset_streams_and_a_fact_streams_once() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        EditObserver::new(),
    )));
    mount_input(&mut harness, &observer);
    harness.type_text("typed");
    harness.settle();

    // The AT's `SetValue` is the app's write: the journal resets (the
    // M15 rule) — and the reset is a fact the log carries.
    clear_trail(&observer);
    assert!(
        harness.act(
            Role::TextInput,
            "Search",
            A11yAction::SetValue("replaced by the app".into())
        ),
        "the AT's value tier writes the field"
    );
    harness.settle();
    let trail = edits(&observer);
    assert_eq!(trail.len(), 1, "the reset streamed once");
    assert_eq!(trail[0].action, EditAction::Reset);
    assert!(!trail[0].refused, "a reset never refuses");
    assert_eq!(
        (trail[0].undo_depth, trail[0].redo_depth),
        (0, 0),
        "the reset named the emptied journal"
    );

    // A fact streams exactly once: the next tick with no journal work
    // streams nothing (the drain clears, the announcement drain's rule).
    clear_trail(&observer);
    harness.settle();
    assert!(
        edits(&observer).is_empty(),
        "an idle journal queues nothing"
    );
    assert_eq!(harness.settle(), 0, "the trail ride costs no masks");
}

#[test]
fn m31_no_observer_no_trail_the_idle_contract_holds() {
    // The default path: no observer, no enabled trail — the journal
    // walks exactly as before and the frame costs what it always cost.
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    harness.type_text("unheard");
    assert_eq!(
        harness.query(Role::TextInput).named("Search").first().value,
        Some("unheard".to_string()),
        "the journal answers without a listener"
    );
    assert_eq!(harness.settle(), 0, "the disabled trail queues no work");
}

/// A Shift+Ctrl press (the redo chord — the M15 route).
fn ctrl_shift_redo() -> KeyEvent {
    KeyEvent {
        key: Key::Char('z'),
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            shift: true,
            ..Modifiers::default()
        },
        text: None,
    }
}
