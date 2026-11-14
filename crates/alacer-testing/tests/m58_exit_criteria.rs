//! M58 exit-criteria integration tests (§13.3, ADR-0064): the typing
//! door's edits join the causal log. ADR-0062's first kill rule cashes
//! — a consumer needs the keyboard's and IME's edits on the log: today
//! the AT's and app's writes stream one
//! [`alacer_core::TextWriteRecord`] each and the journal streams its
//! depths, but the typing door's changes streamed no fact a trail
//! reader could price — "what did the typist change in this field"
//! was unanswerable from the log alone. The record vocabulary
//! ([`alacer_core::KeyEditRecord`]) and the key-edit trail name the
//! change's shape by lengths (never the payload — a field's value can
//! carry a secret, the clipboard trail's rule, ADR-0045), the landing
//! selection beside it, the driver who wrote it, and the journal's own
//! fold — a coalesced typed run landing as one record (the flood
//! rule), never a per-keystroke flood.
//!
//! Every test runs the real pipeline (A4) with a real input written
//! through the real typing door — the key route, the IME route — or
//! the real clipboard/app doors whose silence the door discipline
//! pins.

use alacer_core::events::{ImeEvent, Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{InspectEvent, InspectObserver};
use alacer_core::widget::{Role, TextSelection};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

/// The key-edit trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the edits' facts are the assertion source).
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

/// Empties the shared ring between beats: each assertion reads the
/// facts the edit under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// An input mounted through the real pipeline, focused and settled —
/// the typing door's own mount. Returns the harness, the input's
/// AccessKit-stable id (the funnel's stamp vocabulary) and its arena
/// node (the app door's mutate seam).
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

/// An Alt+Backspace press (the word chord; ADR-0019) — the exact event
/// the platform delivers when the user deletes word-wise.
fn alt_backspace() -> KeyEvent {
    KeyEvent {
        key: Key::Backspace,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            alt: true,
            ..Modifiers::default()
        },
        text: None,
    }
}

#[test]
fn m58_the_typed_run_lands_as_one_fact() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    clear_trail(&observer);
    harness.type_text("hey");
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "a typed run is one fact, not three");
    let edit = edits[0];
    assert_eq!(edit.action, alacer_core::KeyEditAction::Insert);
    assert_eq!(edit.driver, alacer_core::KeyEditDriver::Keyboard);
    assert_eq!(edit.replaced, 0, "a typed run consumed no span");
    assert_eq!(edit.inserted, 3, "the record's shape is the run's");
    assert!(
        edit.coalesced,
        "the journal folded the run — the record names the fold"
    );
    assert_eq!(
        edit.selection,
        TextSelection::new(3, 3),
        "the landing is the run's own caret"
    );
}

#[test]
fn m58_the_backspace_names_the_span_it_consumed() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    harness.type_text("ab");
    clear_trail(&observer);
    harness.press_key(Key::Backspace);
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "the delete is one fact");
    let edit = edits[0];
    assert_eq!(edit.action, alacer_core::KeyEditAction::Backspace);
    assert_eq!(edit.driver, alacer_core::KeyEditDriver::Keyboard);
    assert_eq!(edit.replaced, 1, "the unit the backspace consumed");
    assert_eq!(edit.inserted, 0, "a delete carries no payload");
    assert_eq!(
        edit.selection,
        TextSelection::new(1, 1),
        "the caret it left"
    );
}

#[test]
fn m58_the_word_chord_names_its_species() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    harness.type_text("hello brave");
    clear_trail(&observer);
    harness.key(&alt_backspace());
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "the chord is one fact");
    let edit = edits[0];
    assert_eq!(
        edit.action,
        alacer_core::KeyEditAction::WordDeleteBack,
        "the word chord is never a munged Backspace"
    );
    assert_eq!(
        edit.replaced, 5,
        "the word run the chord consumed (the separator rides the next chord)"
    );
    assert_eq!(edit.inserted, 0);
    assert_eq!(
        edit.selection,
        TextSelection::new(6, 6),
        "the caret it left"
    );
}

#[test]
fn m58_the_ime_commit_names_its_driver_and_keeps_its_own_record() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    harness.type_text("he");
    clear_trail(&observer);
    harness.ime(&ImeEvent::Preedit {
        text: "よ".into(),
        cursor_utf16: Some(1),
    });
    harness.ime(&ImeEvent::Commit {
        text: "よ!".into()
    });
    let edits = key_edits(&observer);
    assert_eq!(
        edits.len(),
        1,
        "the commit is one fact beside the typed run"
    );
    let edit = edits[0];
    assert_eq!(edit.action, alacer_core::KeyEditAction::Insert);
    assert_eq!(
        edit.driver,
        alacer_core::KeyEditDriver::Ime,
        "the composition's driver is the IME's, never the keyboard's"
    );
    assert_eq!(
        edit.inserted, 2,
        "the commit's UTF-16 shape — the surrogate's pair rides whole"
    );
    assert_eq!(
        edit.selection,
        TextSelection::new(4, 4),
        "the landing caret"
    );
    assert!(
        edit.coalesced,
        "the journal's own fold rides the record — the commit continued the tip's run"
    );
    // The trail never merges the commit into a typed run's record: the
    // drivers stay apart at the trail (the trail's own unit test pins
    // the no-settle ride beside a keyboard run).
}

#[test]
fn m58_the_funnel_stamp_names_the_control() {
    let observer = Share::new();
    let (mut harness, id, _node) = mount_input(&observer);
    clear_trail(&observer);
    harness.type_text("a");
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1);
    assert_eq!(
        edits[0].target,
        Some(id),
        "the key route's stamp: the fact names the control it rode"
    );
}

#[test]
fn m58_the_noop_edit_rides_the_honest_absence() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    clear_trail(&observer);
    // Backspace on an empty field consumes nothing — no edit landed,
    // no fact queues (the journal recorded nothing either).
    harness.press_key(Key::Backspace);
    harness.press_key(Key::Delete);
    assert!(
        key_edits(&observer).is_empty(),
        "a no-op door answers no fact"
    );
}

#[test]
fn m58_the_clipboard_chords_price_their_field_changes() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    harness.type_text("copy me");
    harness.set_clipboard("seed");
    clear_trail(&observer);
    // Ctrl+A selects whole; Ctrl+C copies — the round trip's facts ride
    // the clipboard trail and the journal's, never the typing door's
    // (a copy edits nothing: no journal step, no fact).
    harness.key(&KeyEvent {
        key: Key::Char('a'),
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            ..Modifiers::default()
        },
        text: None,
    });
    harness.key(&KeyEvent {
        key: Key::Char('c'),
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            ..Modifiers::default()
        },
        text: None,
    });
    assert!(
        key_edits(&observer).is_empty(),
        "the copy edits nothing — the typing door streams none"
    );
    // Since M60 (ADR-0066) the paste names its field change beside the
    // round trip: the chord's own action and driver, the shape priced
    // by lengths (one fact per trail, never a double-priced edit — the
    // pin re-agreed from the M58 absence the kill rule re-opened).
    harness.key(&KeyEvent {
        key: Key::Char('v'),
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            ..Modifiers::default()
        },
        text: None,
    });
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "the paste's field change is one fact");
    assert_eq!(edits[0].action, alacer_core::KeyEditAction::Paste);
    assert_eq!(edits[0].driver, alacer_core::KeyEditDriver::Clipboard);
    assert_eq!(
        edits[0].replaced, 7,
        "the selection the paste consumed (the copy's own payload rides back)"
    );
    assert_eq!(
        edits[0].inserted, 7,
        "the payload's length, never the payload"
    );
}

#[test]
fn m58_the_app_seam_rides_no_typing_fact() {
    let observer = Share::new();
    let (mut harness, _id, node) = mount_input(&observer);
    clear_trail(&observer);
    // The app door's whole-value write through the mutate seam: the
    // text-write trail names it, the typing door streams none — the
    // doors tellable apart on the log.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_value("written by the app");
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
    assert!(
        key_edits(&observer).is_empty(),
        "the app door's write is the text-write trail's fact, never the typing door's"
    );
    let wrote: Vec<_> = observer
        .0
        .borrow()
        .events
        .iter()
        .filter_map(|event| match event {
            InspectEvent::TextWrite(record) => Some(*record),
            _ => None,
        })
        .collect();
    assert_eq!(wrote.len(), 1, "the app door's fact rides its own trail");
    assert_eq!(
        wrote[0].source,
        alacer_core::TextWriteSource::AppDoor,
        "the door named where the record is built"
    );
}

#[test]
fn m58_the_disabled_field_answers_no_key() {
    let observer = Share::new();
    let (mut harness, _id, node) = mount_input(&observer);
    // The availability tier (M22, ADR-0028): an unavailable input
    // answers no key — no edit lands, no fact queues.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_enabled(false);
    }
    drop(guard);
    harness.settle();
    clear_trail(&observer);
    harness.type_text("a");
    harness.press_key(Key::Backspace);
    assert!(
        key_edits(&observer).is_empty(),
        "the closed door's silence is the availability tier's own answer"
    );
    // Re-agreed at M62 (ADR-0068): the silence this pin prices is the
    // *evicted frame's* — the settle above runs the availability tier's
    // eviction pass, so these keystrokes are the boundary's own (the
    // key route's fallbacks, the field's door hearing no ask). In the
    // frame the control becomes unavailable — before the eviction runs
    // — the held door answers the routed edit key with the wall's own
    // fact (`KeyEditOutcome::Refused`, the value tier's vocabulary):
    // the m62 suite prices that window.
}
