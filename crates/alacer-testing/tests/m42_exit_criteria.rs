//! M42 exit-criteria integration tests (§13.3, ADR-0048): the clipboard
//! read names its facts. ADR-0047's third re-open rule ships: a consumer
//! needs the read side's facts the paste's `None` collapsed (the empty
//! clipboard, the non-text payload, the unopenable clipboard) → the
//! seam's read door grows beside the write door (the same provided-method
//! discipline): one *provided* `ClipboardSeam::read_reported` answering
//! [`ClipboardReadFact`] (the provided default maps the `Option` door and
//! names its absence honestly, a real engine names its species), and the
//! round trip's record carries the read's species beside the paste's
//! verdict — the additive field the wire's versioning rule admits at
//! version 1. The M39 collapse splits at last: a paste that refused for
//! the *read* names the engine's species; a paste that refused for the
//! *field* (a payload that sanitized to nothing) carries the read's
//! absence — one verdict, two facts, no second log. The reads themselves
//! are untouched: `read_text` stays the contract of record, a refused
//! read inserts nothing and queues no masks, and the A7 idle contract
//! holds (one branch per chord while nobody listens).
//!
//! Every test runs the real pipeline (A4) with a real `TextInput`
//! driven by real key events — the exact route a status line, an
//! inspector client, or a test reads.

use alacer_core::clipboard::{
    ClipboardReadFact, ClipboardReadRefusal, ClipboardSeam, ClipboardWriteFact,
    ClipboardWriteRefusal, install,
};
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{
    ClipboardAction, ClipboardOutcome, ClipboardRecord, ClipboardRefusal, InspectEvent,
    InspectObserver,
};
use alacer_core::widget::{EventResponse, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};
use std::rc::Rc;

/// A seam that answers every read with `None` and refuses every write
/// through the `bool` door alone — overriding nothing: the engine that
/// answers only yes/no and some/nothing, the record honest about the
/// read's absence (the m41 suite's own seam, the provided default
/// answering the read side too).
struct LockedClipboard;

impl ClipboardSeam for LockedClipboard {
    fn read_text(&self) -> Option<String> {
        None
    }

    fn write_text(&self, _text: &str) -> bool {
        false
    }
}

/// A seam whose engine names its read species: the sandboxed session's
/// clipboard — the read refused with the fact the engine itself
/// answered (the platform-side fact the `Option` door cannot carry).
struct KioskClipboard;

impl ClipboardSeam for KioskClipboard {
    fn read_text(&self) -> Option<String> {
        None
    }

    fn write_text(&self, _text: &str) -> bool {
        false
    }

    fn read_reported(&self) -> ClipboardReadFact {
        ClipboardReadFact::Refused(ClipboardReadRefusal::Unsupported)
    }

    fn write_reported(&self, _text: &str) -> ClipboardWriteFact {
        ClipboardWriteFact::Refused(ClipboardWriteRefusal::Unsupported)
    }
}

/// A seam whose engine names the non-text payload: the screenshot the
/// clipboard carries — the read refused with the species the engine
/// answered (a paste never invents content from an image).
struct ImageOnlyClipboard;

impl ClipboardSeam for ImageOnlyClipboard {
    fn read_text(&self) -> Option<String> {
        None
    }

    fn write_text(&self, _text: &str) -> bool {
        false
    }

    fn read_reported(&self) -> ClipboardReadFact {
        ClipboardReadFact::Refused(ClipboardReadRefusal::NonText)
    }
}

/// The clipboard-trail collector the tests install as the pipeline's
/// observer: the round-trip facts stream into a bounded ring (the m39
/// helper's shape — here the read fact is the assertion source).
struct TrailObserver {
    clipboards: std::collections::VecDeque<ClipboardRecord>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            clipboards: std::collections::VecDeque::with_capacity(64),
        }
    }
}

impl InspectObserver for TrailObserver {
    fn observe(&mut self, event: InspectEvent) {
        if let InspectEvent::Clipboard(record) = event {
            if self.clipboards.len() >= 64 {
                self.clipboards.pop_front();
            }
            self.clipboards.push_back(record);
        }
    }
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the m39 suite's shape, exactly.
struct Share(std::rc::Rc<std::cell::RefCell<TrailObserver>>);

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

/// The streamed clipboard facts, oldest first.
fn clipboard_records(observer: &Share) -> Vec<ClipboardRecord> {
    observer.0.borrow().clipboards.iter().copied().collect()
}

/// Empties the shared ring between beats: each assertion reads the facts
/// the beat under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().clipboards.clear();
}

/// An input mounted through the real pipeline, focused so every chord
/// under test routes to the field, and observed — the clipboard trail
/// rides from the `set_observer` call on (the m39 mount, plus the
/// observer).
fn mount_input(harness: &mut TestHarness, observer: &Share) {
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();
    // Focus resolves in pass 3 of a tick: settle before the chords, or
    // they route to no focused widget (the M11 suite's same rule).
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

/// One chord through the real funnel: the response the route gave.
fn chord(harness: &mut TestHarness, key: Key) -> EventResponse {
    harness.pipeline_mut().dispatch_key(&ctrl(key))
}

#[test]
fn m42_an_option_door_names_its_absence_honestly() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        TrailObserver::new(),
    )));
    // The locked door: every read answered through the `Option` door
    // alone — the provided default answers, the record honest about the
    // engine that names nothing (the M37 `Option::None` discipline, as
    // a species, the write side's own rule carried across).
    install(Rc::new(LockedClipboard) as Rc<dyn ClipboardSeam>);
    mount_input(&mut harness, &observer);

    let response = chord(&mut harness, Key::Char('v'));
    assert!(
        matches!(response, EventResponse::Handled),
        "the field consumes its own paste chord even when the read refused"
    );
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the refused paste streams one fact");
    assert_eq!(records[0].action, ClipboardAction::Paste);
    assert_eq!(
        records[0].outcome,
        ClipboardOutcome::Refused(ClipboardRefusal::EmptyClipboard)
    );
    assert_eq!(
        records[0].read,
        Some(ClipboardReadRefusal::Unnamed),
        "the engine answered only its Option — the record honest about the absence"
    );
    assert_eq!(records[0].chars, 0, "the refusal moved nothing");
    assert_eq!(harness.settle(), 0, "a refused paste queues no masks");
}

#[test]
fn m42_an_engine_that_names_its_read_species_names_it_on_the_record() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        TrailObserver::new(),
    )));
    // The kiosk's clipboard: the read refused with the species the
    // engine itself answered — the sandboxed session's honest answer,
    // the fact the `Option` door collapsed into `None`.
    install(Rc::new(KioskClipboard) as Rc<dyn ClipboardSeam>);
    mount_input(&mut harness, &observer);

    let _ = chord(&mut harness, Key::Char('v'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the kiosk's refused read streams");
    assert_eq!(
        records[0].outcome,
        ClipboardOutcome::Refused(ClipboardRefusal::EmptyClipboard)
    );
    assert_eq!(
        records[0].read,
        Some(ClipboardReadRefusal::Unsupported),
        "the engine's own species rides beside the round trip's verdict"
    );

    // The screenshot's clipboard: the payload is not text — the engine
    // names the species, the paste never invents content from an image.
    install(Rc::new(ImageOnlyClipboard) as Rc<dyn ClipboardSeam>);
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('v'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the non-text read streams");
    assert_eq!(
        records[0].read,
        Some(ClipboardReadRefusal::NonText),
        "the non-text payload names itself"
    );

    // The honest store with nothing in it: the engine names the empty
    // clipboard — the fact the `None` carried, now spoken.
    install(Rc::new(alacer_core::clipboard::MemoryClipboard::default()) as Rc<dyn ClipboardSeam>);
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('v'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the empty read streams");
    assert_eq!(
        records[0].read,
        Some(ClipboardReadRefusal::Empty),
        "the empty clipboard names itself"
    );
    assert_eq!(harness.settle(), 0, "the refused reads queue no masks");
}

#[test]
fn m42_the_field_refusal_and_the_read_refusal_name_apart() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        TrailObserver::new(),
    )));
    // The honest store: the same `EmptyClipboard` verdict M39 pinned,
    // the read fact now splitting the two pastes that shared it.
    install(Rc::new(alacer_core::clipboard::MemoryClipboard::default()) as Rc<dyn ClipboardSeam>);
    mount_input(&mut harness, &observer);

    // The read-refused paste: the clipboard carried nothing — the read's
    // species rides beside the verdict.
    let _ = chord(&mut harness, Key::Char('v'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the read-refused paste streams");
    assert_eq!(
        records[0].outcome,
        ClipboardOutcome::Refused(ClipboardRefusal::EmptyClipboard)
    );
    assert_eq!(
        records[0].read,
        Some(ClipboardReadRefusal::Empty),
        "the read refused — the species says which clipboard answered"
    );

    // The field-refused paste: the clipboard delivered a payload the
    // field cannot carry (it sanitizes to nothing — the single-line
    // rule's edge, ADR-0020) — the same verdict, the read's absence:
    // the read delivered, the field refused. The fact M39 collapsed
    // into one silence, named apart at last.
    let clipboard = Rc::new(alacer_core::clipboard::MemoryClipboard::default());
    clipboard.set("");
    install(Rc::clone(&clipboard) as Rc<dyn ClipboardSeam>);
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('v'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the field-refused paste streams");
    assert_eq!(
        records[0].outcome,
        ClipboardOutcome::Refused(ClipboardRefusal::EmptyClipboard)
    );
    assert_eq!(
        records[0].read, None,
        "the read delivered — the field's own refusal, the absence honest"
    );
    assert_eq!(harness.settle(), 0, "both refusals queue no masks");
}

#[test]
fn m42_a_delivered_paste_and_the_write_sides_carry_no_read_fact() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        TrailObserver::new(),
    )));
    // The honest store, seeded: the delivered paste, the delivered copy,
    // the delivered cut — none of them asked a read-refusal question,
    // the additive field's null riding every one.
    let clipboard = Rc::new(alacer_core::clipboard::MemoryClipboard::default());
    clipboard.set("held text");
    install(Rc::clone(&clipboard) as Rc<dyn ClipboardSeam>);
    mount_input(&mut harness, &observer);

    let _ = chord(&mut harness, Key::Char('v'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the paste streams one fact");
    assert_eq!(records[0].outcome, ClipboardOutcome::Delivered);
    assert_eq!(records[0].read, None, "a delivery asked no read question");
    assert_eq!(records[0].chars, 9, "the paste counts what landed");
    assert_eq!(
        harness.query(Role::TextInput).named("Search").first().value,
        Some("held text".to_string()),
        "the paste landed what the store holds"
    );

    // The copy side: a write-side round trip — no read fact rides.
    let _ = chord(&mut harness, Key::Char('a'));
    harness.settle();
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('c'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the copy streams one fact");
    assert_eq!(records[0].action, ClipboardAction::Copy);
    assert_eq!(records[0].outcome, ClipboardOutcome::Delivered);
    assert_eq!(records[0].read, None, "a copy never asked a read question");
    assert_eq!(
        records[0].engine, None,
        "an accepted write asked no engine question either"
    );

    // The read-refused copy (a locked door): the write fact rides, the
    // read fact does not — the round trip never read.
    install(Rc::new(LockedClipboard) as Rc<dyn ClipboardSeam>);
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('x'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the refused cut streams one fact");
    assert_eq!(
        records[0].outcome,
        ClipboardOutcome::Refused(ClipboardRefusal::WriteRefused)
    );
    assert_eq!(
        records[0].engine,
        Some(ClipboardWriteRefusal::Unnamed),
        "the write fact rides beside the write refusal"
    );
    assert_eq!(records[0].read, None, "a cut never asked a read question");
    assert_eq!(
        harness.query(Role::TextInput).named("Search").first().value,
        Some("held text".to_string()),
        "the selection survives the clipboard that failed the write"
    );
}

#[test]
fn m42_the_option_door_stays_the_contract_of_record() {
    // The `read_text` door answers exactly as before — the richer door
    // grows beside it, never over it (the M35 `dispatch_installed`
    // rule): a custom engine overriding nothing compiles and answers,
    // and the memory engine's round trip is untouched.
    let memory = alacer_core::clipboard::MemoryClipboard::default();
    assert_eq!(
        memory.read_text(),
        None,
        "the Option door answers as it always answered"
    );
    assert_eq!(
        memory.read_reported(),
        ClipboardReadFact::Refused(ClipboardReadRefusal::Empty),
        "the memory engine names its own emptiness"
    );
    memory.set("held");
    assert_eq!(
        memory.read_reported(),
        ClipboardReadFact::Delivered("held".to_string()),
        "the memory engine delivers the payload the store holds"
    );
    let locked = LockedClipboard;
    assert_eq!(locked.read_text(), None, "the Option door refuses");
    assert_eq!(
        locked.read_reported(),
        ClipboardReadFact::Refused(ClipboardReadRefusal::Unnamed),
        "the provided default maps the absence it cannot name"
    );
    // The write contract of record is untouched too (the M41 rule
    // stands byte-for-byte).
    assert!(!locked.write_text("held"), "the bool door refuses");
    assert_eq!(
        locked.write_reported("held"),
        ClipboardWriteFact::Refused(ClipboardWriteRefusal::Unnamed)
    );
}

#[test]
fn m42_the_read_species_carry_their_wire_names() {
    // The protocol's stable, documented `snake_case` tags — the wire
    // vocabulary the client renders.
    assert_eq!(ClipboardReadRefusal::Unnamed.name(), "unnamed");
    assert_eq!(ClipboardReadRefusal::Unsupported.name(), "unsupported");
    assert_eq!(ClipboardReadRefusal::Empty.name(), "empty");
    assert_eq!(ClipboardReadRefusal::NonText.name(), "non_text");
    assert_eq!(ClipboardReadRefusal::OsError.name(), "os_error");
    assert_eq!(
        ClipboardReadFact::Delivered("held".to_string()).name(),
        "delivered"
    );
    assert_eq!(
        ClipboardReadFact::Refused(ClipboardReadRefusal::NonText).name(),
        "refused"
    );
}

#[test]
fn m42_no_observer_no_trail_the_idle_contract_holds() {
    // The default path: no observer, no enabled trail — the reads
    // answer exactly as before and the frame costs what it always cost.
    let mut harness = TestHarness::new(400.0, 200.0);
    install(Rc::new(LockedClipboard) as Rc<dyn ClipboardSeam>);
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    harness.type_text("held");
    harness.settle();
    let _ = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('v')));
    harness.settle();
    assert_eq!(
        harness.query(Role::TextInput).named("Search").first().value,
        Some("held".to_string()),
        "the refused read inserts nothing, without a listener either"
    );
    assert_eq!(harness.settle(), 0, "the disabled trail queues no work");
}

#[test]
fn m42_the_living_documents_re_agree_at_m42() {
    let root = workspace_root();
    let readme = std::fs::read_to_string(root.join("README.md")).expect("readme");
    let architecture =
        std::fs::read_to_string(root.join("docs/architecture.md")).expect("architecture");
    let protocol = std::fs::read_to_string(root.join("docs/inspector-protocol.md"))
        .expect("protocol document");
    let catalog = std::fs::read_to_string(root.join("docs/examples.md")).expect("catalog");
    let changelog = std::fs::read_to_string(root.join("CHANGELOG.md")).expect("changelog");

    // The status block and the ADR count re-agree at the cut (the
    // m-suite pins moving M41 → M43 and 48 → 50 across the chain —
    // M42 and M43 landing in one cut, the workspace's one-milestone-
    // per-minor rule counting them as the minor's two entries).
    assert!(
        readme.contains("M0 through M43") && readme.contains("fifty decision records"),
        "the README's status block and count re-agree at M43"
    );
    assert!(
        architecture.contains("50 decision records"),
        "the architecture register re-agrees at 50 decision records"
    );
    // The protocol document carries the read fact (the wire's
    // documented shape).
    assert!(
        protocol.contains("`non_text`") && protocol.contains("`read`"),
        "the protocol document documents the read fact"
    );
    // The catalog lists the demonstration.
    assert!(
        catalog.contains("clipboard_read"),
        "the examples catalog lists the clipboard read demo"
    );
    // The changelog stages the milestone.
    assert!(
        changelog.contains("M42 — the clipboard read names its facts"),
        "the changelog stages M42"
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
