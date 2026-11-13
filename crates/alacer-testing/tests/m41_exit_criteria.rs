//! M41 exit-criteria integration tests (§13.3, ADR-0047): the clipboard
//! engine names its platform-side facts. ADR-0045's own re-open rule
//! ships: a consumer needs the clipboard's *platform-side* facts the
//! widget's round trips cannot name (the engine's own refusal
//! diagnostics) → the seam's engine contract grows beside
//! [`ClipboardSeam`] — one *provided* `write_reported` door answering
//! [`ClipboardWriteFact`] (the M37 provided-method discipline: the
//! provided default maps the `bool` door and names its absence
//! honestly, a real engine names its species), and the round trip's
//! record carries the engine's species beside the `write_refused`
//! verdict — the additive field the wire's versioning rule admits at
//! version 1. The round trips themselves are untouched: a refused
//! write never deletes (ADR-0020), the copy writes, the cut deletes
//! only on an accepted write — the log is the log, the A7 idle
//! contract unchanged (one branch per chord while nobody listens).
//!
//! Every test runs the real pipeline (A4) with a real `TextInput`
//! driven by real key events — the exact route a status line, an
//! inspector client, or a test reads.

use alacer_core::clipboard::{ClipboardSeam, ClipboardWriteFact, ClipboardWriteRefusal, install};
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
/// answers only yes/no, the record honest about the absence (the m39
/// suite's own seam, riding the provided default now).
struct LockedClipboard;

impl ClipboardSeam for LockedClipboard {
    fn read_text(&self) -> Option<String> {
        None
    }

    fn write_text(&self, _text: &str) -> bool {
        false
    }
}

/// A seam whose engine names its own species: the kiosk's clipboard —
/// no subsystem at all, the write refused with the fact the OS answered
/// (the platform-side fact the widget's `bool` could not carry).
struct KioskClipboard;

impl ClipboardSeam for KioskClipboard {
    fn read_text(&self) -> Option<String> {
        None
    }

    fn write_text(&self, _text: &str) -> bool {
        false
    }

    fn write_reported(&self, _text: &str) -> ClipboardWriteFact {
        ClipboardWriteFact::Refused(ClipboardWriteRefusal::Unsupported)
    }
}

/// The clipboard-trail collector the tests install as the pipeline's
/// observer: the round-trip facts stream into a bounded ring (the m39
/// helper's shape — here the engine fact is the assertion source).
struct TrailObserver {
    clipboards: std::collections::VecDeque<ClipboardRecord>,
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
fn m41_a_bool_door_names_its_absence_honestly() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(TrailObserver {
        clipboards: std::collections::VecDeque::with_capacity(64),
    })));
    // The locked door: every write refused through the `bool` door
    // alone — the provided default answers, the record honest about the
    // engine that names nothing (the M37 `Option::None` discipline, as
    // a species).
    install(Rc::new(LockedClipboard) as Rc<dyn ClipboardSeam>);
    mount_input(&mut harness, &observer);
    harness.type_text("held");
    harness.settle();

    let _ = chord(&mut harness, Key::Char('a'));
    harness.settle();
    clear_trail(&observer);
    let response = chord(&mut harness, Key::Char('x'));
    assert!(
        matches!(response, EventResponse::Handled),
        "the field consumes its own cut chord even when the write refused"
    );
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the refused cut streams one fact");
    assert_eq!(records[0].action, ClipboardAction::Cut);
    assert_eq!(
        records[0].outcome,
        ClipboardOutcome::Refused(ClipboardRefusal::WriteRefused)
    );
    assert_eq!(
        records[0].engine,
        Some(ClipboardWriteRefusal::Unnamed),
        "the engine answered only its bool — the record honest about the absence"
    );
    assert_eq!(records[0].chars, 0, "the refusal moved nothing");
    assert_eq!(
        harness.query(Role::TextInput).named("Search").first().value,
        Some("held".to_string()),
        "the selection survives a clipboard that failed it"
    );
    assert_eq!(harness.settle(), 0, "a refused cut queues no masks");
}

#[test]
fn m41_an_engine_that_names_its_species_names_it_on_the_record() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(TrailObserver {
        clipboards: std::collections::VecDeque::with_capacity(64),
    })));
    // The kiosk's clipboard: no subsystem at all — the engine answers
    // its own species, the platform-side fact the widget's round trip
    // could not name before (ADR-0045's re-open rule, priced by a real
    // engine shape).
    install(Rc::new(KioskClipboard) as Rc<dyn ClipboardSeam>);
    mount_input(&mut harness, &observer);
    harness.type_text("held");
    harness.settle();

    // The cut through the kiosk: refused, the species named — the
    // selection surviving the clipboard that failed it, the log now
    // answering "which clipboard failed it".
    let _ = chord(&mut harness, Key::Char('a'));
    harness.settle();
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('x'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the kiosk's refusal streams one fact");
    assert_eq!(
        records[0].outcome,
        ClipboardOutcome::Refused(ClipboardRefusal::WriteRefused)
    );
    assert_eq!(
        records[0].engine,
        Some(ClipboardWriteRefusal::Unsupported),
        "the engine's own species rides beside the round trip's verdict"
    );

    // The copy through the kiosk: refused the same way, the species the
    // engine answered — a copy never invents content either way.
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('a'));
    harness.settle();
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('c'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the kiosk's refused copy streams");
    assert_eq!(records[0].action, ClipboardAction::Copy);
    assert_eq!(
        records[0].engine,
        Some(ClipboardWriteRefusal::Unsupported),
        "the copy's refusal carries the same engine fact"
    );
    assert_eq!(harness.settle(), 0, "the refused rides queue no masks");

    // The honest store: the write accepted, the round trip delivered,
    // and the engine fact rides as its absence — an accepted write
    // asked no refusal question (the additive field's null, the wire's
    // documented shape).
    install(Rc::new(alacer_core::clipboard::MemoryClipboard::default()) as Rc<dyn ClipboardSeam>);
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('a'));
    harness.settle();
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('c'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the delivery streams one fact");
    assert_eq!(records[0].outcome, ClipboardOutcome::Delivered);
    assert_eq!(
        records[0].engine, None,
        "an accepted write carries no engine refusal"
    );
    assert_eq!(records[0].chars, 4, "the record counts the span it moved");

    // The paste side reads; the engine fact is a write-side fact — a
    // paste's round trip carries the absence, whatever the engine.
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('v'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the paste streams one fact");
    assert_eq!(records[0].outcome, ClipboardOutcome::Delivered);
    assert_eq!(records[0].engine, None, "a read asked no write question");
}

#[test]
fn m41_the_bool_door_stays_the_contract_of_record() {
    // The `write_text` door answers exactly as before — the richer door
    // grows beside it, never over it (the M35 `dispatch_installed`
    // rule): a custom engine overriding nothing compiles and answers,
    // and the memory engine's round trip is untouched.
    let memory = alacer_core::clipboard::MemoryClipboard::default();
    assert!(
        memory.write_text("held"),
        "the bool door answers as it always answered"
    );
    assert_eq!(memory.read_text().as_deref(), Some("held"));
    assert_eq!(
        memory.write_reported("again"),
        ClipboardWriteFact::Accepted,
        "the provided default maps the accepted write"
    );
    let locked = LockedClipboard;
    assert!(!locked.write_text("held"), "the bool door refuses");
    assert_eq!(
        locked.write_reported("held"),
        ClipboardWriteFact::Refused(ClipboardWriteRefusal::Unnamed),
        "the provided default maps the refusal it cannot name"
    );
}

#[test]
fn m41_no_observer_no_trail_the_idle_contract_holds() {
    // The default path: no observer, no enabled trail — the round trips
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
    let _ = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('a')));
    let _ = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('x')));
    harness.settle();
    assert_eq!(
        harness.query(Role::TextInput).named("Search").first().value,
        Some("held".to_string()),
        "the refused write never deletes, without a listener either"
    );
    assert_eq!(harness.settle(), 0, "the disabled trail queues no work");
}

#[test]
fn m41_the_living_documents_re_agree_at_m41() {
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
    // M43 landing in one cut).
    assert!(
        readme.contains("M0 through M49") && readme.contains("fifty-six decision records"),
        "the README's status block and count re-agree at M43"
    );
    assert!(
        architecture.contains("56 decision records"),
        "the architecture register re-agrees at 56 decision records"
    );
    // The protocol document carries the engine fact (the wire's
    // documented shape).
    assert!(
        protocol.contains("\"engine\":null") && protocol.contains("`os_error`"),
        "the protocol document documents the engine fact"
    );
    // The catalog lists the demonstration.
    assert!(
        catalog.contains("clipboard_engine"),
        "the examples catalog lists the clipboard engine demo"
    );
    // The changelog stages the milestone.
    assert!(
        changelog.contains("M41 — the clipboard engine names its facts"),
        "the changelog stages M41"
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
