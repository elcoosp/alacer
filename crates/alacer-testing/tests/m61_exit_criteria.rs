//! M61 exit-criteria integration tests (§13.3, ADR-0067): the field's
//! composing state joins the causal log. ADR-0064's first kill rule
//! cashes — a consumer prices the *preedit's own trail*, the
//! composition's inline shape — today a committed composition streamed
//! its [`alacer_core::KeyEditRecord`] (the M58 record, the IME driver
//! named), but the composing state itself — the session the field
//! showed while the IME owned the canvas — streamed nothing: "how large
//! did the composition grow", "what is the field showing now" were
//! unanswerable from the log alone. The IME tier grows its own record
//! beside the commit's ([`alacer_core::PreeditRecord`]) — the preedit
//! is not an edit; it lands nowhere, the journal never hears of it —
//! named by lengths (never the payload: a field's value can carry a
//! secret, ADR-0011's rule at the composition tier), the platform's
//! cursor inside the preedit beside it, and the funnel's stamp (the
//! M43 rule at the composing tier). One live session is one record:
//! N composition frames supersede the session's record in place (the
//! flood rule at the composition tier), never a default-on echo of
//! every composition frame.
//!
//! Every test runs the real pipeline (A4) with the real IME route —
//! the preedit the platform delivers, the commit that lands beside it.

use alacer_core::events::ImeEvent;
use alacer_core::inspect::{InspectEvent, InspectObserver};
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

/// The preedit trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the composition's facts are the assertion source).
struct TrailObserver {
    events: std::collections::VecDeque<InspectEvent>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            events: std::collections::VecDeque::with_capacity(64),
        }
    }

    /// The streamed preedit facts, oldest first.
    fn preedits(&self) -> Vec<alacer_core::PreeditRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::Preedit(record) => Some(*record),
                _ => None,
            })
            .collect()
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

/// The streamed preedit facts, oldest first.
fn preedits(observer: &Share) -> Vec<alacer_core::PreeditRecord> {
    observer.0.borrow().preedits()
}

/// The streamed key-edit facts, oldest first.
fn key_edits(observer: &Share) -> Vec<alacer_core::KeyEditRecord> {
    observer.0.borrow().key_edits()
}

/// Empties the shared ring between beats: each assertion reads the
/// facts the composition under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// An input mounted through the real pipeline, focused and settled —
/// the composing door's own mount. Returns the harness and the input's
/// AccessKit-stable id (the funnel's stamp vocabulary).
fn mount_input(observer: &Share) -> (TestHarness, u64) {
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
    (harness, id)
}

#[test]
fn m61_the_composing_session_lands_one_growing_record() {
    let observer = Share::new();
    let (mut harness, _id) = mount_input(&observer);
    clear_trail(&observer);
    // Three composition frames routed inside one tick window (raw
    // dispatches — the harness's ime door settles between calls, and a
    // settle is a drain): the session's record supersedes in place —
    // one live session, one record whose shape is the composition's.
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Preedit {
        text: "あ".into(),
        cursor_utf16: Some(1),
    });
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Preedit {
        text: "あい".into(),
        cursor_utf16: Some(2),
    });
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Preedit {
        text: "あいう".into(),
        cursor_utf16: Some(3),
    });
    harness.settle();
    let facts = preedits(&observer);
    assert_eq!(
        facts.len(),
        1,
        "N composition frames are one record — never an echo per frame"
    );
    let fact = facts[0];
    assert_eq!(
        fact.action,
        alacer_core::PreeditAction::Composing,
        "the live session's own phase"
    );
    assert_eq!(
        fact.shape, 3,
        "the record's shape is the session's — the composition's UTF-16 length"
    );
    assert_eq!(
        fact.cursor,
        Some(3),
        "the platform's cursor inside the preedit, as the last frame carried it"
    );
}

#[test]
fn m61_the_shape_is_named_by_lengths_never_the_payload() {
    let observer = Share::new();
    let (mut harness, _id) = mount_input(&observer);
    clear_trail(&observer);
    // A surrogate pair composes: the pair rides whole in the shape's
    // count — the payload itself never streams (a field's value can
    // carry a secret, ADR-0011's rule at the composition tier; the
    // secret-entry field suppresses the preedit echo at the platform).
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Preedit {
        text: "😀".into(),
        cursor_utf16: Some(2),
    });
    harness.settle();
    let facts = preedits(&observer);
    assert_eq!(facts.len(), 1);
    assert_eq!(
        facts[0].shape, 2,
        "the surrogate pair counts whole, in UTF-16 units"
    );
}

#[test]
fn m61_the_cleared_composition_names_its_end() {
    let observer = Share::new();
    let (mut harness, _id) = mount_input(&observer);
    clear_trail(&observer);
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Preedit {
        text: "あ".into(),
        cursor_utf16: Some(1),
    });
    harness.settle();
    clear_trail(&observer);
    // The composition ended with no landing — the empty preedit the
    // platform delivers on a cancel: the session's end names itself
    // (the commit would have named its own landing on the key-edit
    // trail instead).
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Preedit {
        text: String::new(),
        cursor_utf16: None,
    });
    harness.settle();
    let facts = preedits(&observer);
    assert_eq!(facts.len(), 1, "the cleared session answers one fact");
    assert_eq!(
        facts[0].action,
        alacer_core::PreeditAction::Cleared,
        "the composing state's end is its own phase, never a munged commit"
    );
    assert_eq!(facts[0].shape, 0, "the field shows no composing text");
}

#[test]
fn m61_the_spurious_clear_rides_no_fact() {
    let observer = Share::new();
    let (mut harness, _id) = mount_input(&observer);
    clear_trail(&observer);
    // An empty preedit with no live session behind it is a no-op door:
    // the field had no composing state to end, no fact queues.
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Preedit {
        text: String::new(),
        cursor_utf16: None,
    });
    harness.settle();
    assert!(
        preedits(&observer).is_empty(),
        "a cleared state with no session answers no fact"
    );
}

#[test]
fn m61_the_commit_rides_beside_the_session_not_over_it() {
    let observer = Share::new();
    let (mut harness, _id) = mount_input(&observer);
    clear_trail(&observer);
    // The session composes and the commit lands: the composition's
    // shape is the preedit trail's fact, the landing is the key-edit
    // trail's (M58's record, the IME driver named) — one fact per
    // trail, tellable apart on one log.
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Preedit {
        text: "よ".into(),
        cursor_utf16: Some(1),
    });
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Commit {
        text: "よ!".into()
    });
    harness.settle();
    let facts = preedits(&observer);
    assert_eq!(
        facts.len(),
        1,
        "the session's shape is one preedit fact (the commit lands nowhere)"
    );
    assert_eq!(facts[0].action, alacer_core::PreeditAction::Composing);
    assert_eq!(facts[0].shape, 1);
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "the commit's landing is one key-edit fact");
    assert_eq!(edits[0].driver, alacer_core::KeyEditDriver::Ime);
    assert_eq!(
        edits[0].inserted, 2,
        "the commit's UTF-16 shape rides its own record"
    );
}

#[test]
fn m61_the_stamp_names_the_control() {
    let observer = Share::new();
    let (mut harness, id) = mount_input(&observer);
    clear_trail(&observer);
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Preedit {
        text: "あ".into(),
        cursor_utf16: Some(1),
    });
    harness.settle();
    let facts = preedits(&observer);
    assert_eq!(facts.len(), 1);
    assert_eq!(
        facts[0].target,
        Some(id),
        "the IME route's stamp: the fact names the control it rode"
    );
}
