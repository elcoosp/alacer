//! M39 exit-criteria integration tests (§13.3, ADR-0045): the clipboard
//! rides the causal log. The ride ADR-0010's additive rule and the
//! M31/M33/M36 rhythm pre-register ships: every clipboard round trip the
//! editing contract answers — a copy, a cut, a paste, and every honest
//! refusal (no selection, an empty clipboard, a refused write) — streams
//! one [`ClipboardRecord`] into the thread's clipboard trail, and the
//! dispatching pipeline drains it at the tick's start into
//! [`InspectEvent::Clipboard`] — the edit trail's shape (push now, the
//! tick routes, one frame), the refusal tier's honesty (a refused round
//! trip streams too, the species naming why nothing moved). The round
//! trips themselves are untouched: the copy writes, the cut deletes only
//! on an accepted write, the paste sanitizes — the log is the log, the
//! A7 idle contract unchanged (one branch per chord while nobody
//! listens).
//!
//! Every test runs the real pipeline (A4) with a real `TextInput` driven
//! by real key events — the exact route a status line, an inspector
//! client, or a test reads.

use alacer_core::clipboard::{ClipboardSeam, install};
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{
    ClipboardAction, ClipboardOutcome, ClipboardRecord, ClipboardRefusal, InspectEvent,
    InspectObserver,
};
use alacer_core::widget::{EventResponse, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};
use std::rc::Rc;

/// A seam that answers every read with `None` and refuses every write:
/// the clipboard as a locked door — the permission/escape story the
/// M14 contract's refuse-don't-munge rule is written for (a cut whose
/// write was refused must not delete).
struct LockedClipboard;

impl ClipboardSeam for LockedClipboard {
    fn read_text(&self) -> Option<String> {
        None
    }

    fn write_text(&self, _text: &str) -> bool {
        false
    }
}

/// The clipboard-trail collector the tests install as the pipeline's
/// observer: the round-trip facts stream into a bounded ring alongside
/// the arrival-order kind list (the m36 helper's shape — here the
/// clipboard record is the assertion source).
struct TrailObserver {
    clipboards: std::collections::VecDeque<ClipboardRecord>,
    kinds: std::collections::VecDeque<&'static str>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            clipboards: std::collections::VecDeque::with_capacity(64),
            kinds: std::collections::VecDeque::with_capacity(64),
        }
    }
}

impl InspectObserver for TrailObserver {
    fn observe(&mut self, event: InspectEvent) {
        match event {
            InspectEvent::Clipboard(record) => {
                self.push("clipboard");
                if self.clipboards.len() >= 64 {
                    self.clipboards.pop_front();
                }
                self.clipboards.push_back(record);
            }
            InspectEvent::Edit(_) => self.push("edit"),
            InspectEvent::Focus(_) => self.push("focus"),
            InspectEvent::Input(_) => self.push("input"),
            InspectEvent::Frame(_) => self.push("frame"),
            _ => {}
        }
    }
}

impl TrailObserver {
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
/// the m30 suite's shape, exactly.
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

/// The streamed kinds in arrival order — the trail's shape as data.
fn kinds(observer: &Share) -> Vec<&'static str> {
    observer.0.borrow().kinds.iter().copied().collect()
}

/// Empties the shared ring between beats: each assertion reads the facts
/// the beat under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    let mut trail = observer.0.borrow_mut();
    trail.clipboards.clear();
    trail.kinds.clear();
}

/// An input mounted through the real pipeline, focused so every chord
/// under test routes to the field, and observed — the clipboard trail
/// rides from the `set_observer` call on (the m31 mount, plus the
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
fn m39_the_chords_stream_one_fact_per_round_trip() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        TrailObserver::new(),
    )));
    mount_input(&mut harness, &observer);
    harness.type_text("hello world");
    harness.settle();

    // The copy: the selected span's exact round trip — one fact, the
    // payload's scalar count beside it, the clipboard carrying what the
    // record says moved.
    let _ = chord(&mut harness, Key::Char('a'));
    harness.settle();
    clear_trail(&observer);
    let response = chord(&mut harness, Key::Char('c'));
    assert!(
        matches!(response, EventResponse::Handled),
        "the field consumes its own copy chord"
    );
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "one round trip, one fact");
    assert_eq!(records[0].action, ClipboardAction::Copy);
    assert_eq!(records[0].outcome, ClipboardOutcome::Delivered);
    assert_eq!(records[0].chars, 11, "the record counts the span it moved");
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("hello world"),
        "the round trip answered exactly as the record streams"
    );
    assert_eq!(harness.settle(), 0, "a copy queues no masks");

    // The cut: the write lands, the deletion follows — the fact names the
    // span that moved out before the journal's own step streams.
    let _ = chord(&mut harness, Key::Char('a'));
    harness.settle();
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('x'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the cut streams one fact");
    assert_eq!(records[0].action, ClipboardAction::Cut);
    assert_eq!(records[0].outcome, ClipboardOutcome::Delivered);
    assert_eq!(records[0].chars, 11);
    assert_eq!(
        harness.query(Role::TextInput).named("Search").first().value,
        Some(String::new()),
        "the cut deleted the selection"
    );

    // The paste: the sanitized insert — the record counts the text that
    // landed, the field carrying it.
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('v'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the paste streams one fact");
    assert_eq!(records[0].action, ClipboardAction::Paste);
    assert_eq!(records[0].outcome, ClipboardOutcome::Delivered);
    assert_eq!(records[0].chars, 11, "the paste counts what landed");
    assert_eq!(
        harness.query(Role::TextInput).named("Search").first().value,
        Some("hello world".to_string()),
        "the round trip restored the field"
    );
}

#[test]
fn m39_the_refused_ops_stream_too() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        TrailObserver::new(),
    )));
    mount_input(&mut harness, &observer);

    // The copy without a selection: refused, the species named — the
    // fact a trail reader wants is exactly the round trip that never
    // moved anything (the M28 honesty, the clipboard tier's own).
    clear_trail(&observer);
    let response = chord(&mut harness, Key::Char('c'));
    assert!(
        matches!(response, EventResponse::Handled),
        "the field consumes its own chord even when it refuses"
    );
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the refusal is a fact the log carries");
    assert_eq!(records[0].action, ClipboardAction::Copy);
    assert_eq!(
        records[0].outcome,
        ClipboardOutcome::Refused(ClipboardRefusal::NoSelection)
    );
    assert_eq!(records[0].chars, 0, "a refusal moved nothing");
    assert_eq!(harness.settle(), 0, "a refused chord queues no masks");

    // The paste on an empty clipboard: refused the same way — the read
    // answered `None`, nothing inserted, the species naming why.
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('v'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the empty read streams");
    assert_eq!(records[0].action, ClipboardAction::Paste);
    assert_eq!(
        records[0].outcome,
        ClipboardOutcome::Refused(ClipboardRefusal::EmptyClipboard)
    );

    // A payload that sanitizes to nothing pastes nothing — the clipboard
    // carried text but the field cannot carry it (the single-line rule's
    // edge, ADR-0020): the same species, honestly.
    harness.set_clipboard("");
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('v'));
    harness.settle();
    let records = clipboard_records(&observer);
    assert_eq!(records.len(), 1, "the nothing-pasteable read streams");
    assert_eq!(
        records[0].outcome,
        ClipboardOutcome::Refused(ClipboardRefusal::EmptyClipboard),
        "an empty payload is the empty clipboard's honest answer"
    );
}

#[test]
fn m39_a_refused_write_never_deletes_and_names_itself() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        TrailObserver::new(),
    )));
    // The locked door: every write refused — the permission/escape story
    // the refuse-don't-munge rule is written for.
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
    assert_eq!(records.len(), 1, "the refused cut streams");
    assert_eq!(records[0].action, ClipboardAction::Cut);
    assert_eq!(
        records[0].outcome,
        ClipboardOutcome::Refused(ClipboardRefusal::WriteRefused)
    );
    assert_eq!(records[0].chars, 0, "the refusal moved nothing");
    assert_eq!(
        harness.query(Role::TextInput).named("Search").first().value,
        Some("held".to_string()),
        "the selection survives a clipboard that failed it"
    );
    assert!(harness.clipboard_text().is_none(), "the write never landed");
    assert_eq!(harness.settle(), 0, "a refused cut queues no masks");
}

#[test]
fn m39_the_trail_reads_clipboard_then_edit_and_a_fact_streams_once() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        TrailObserver::new(),
    )));
    mount_input(&mut harness, &observer);
    harness.type_text("hey");
    harness.settle();

    // The cut resolves in order: the clipboard's fact (what moved out)
    // precedes the journal's own (the deletion it drove) — the trail
    // reads clipboard → edit, the causal order the round trip ran.
    let _ = chord(&mut harness, Key::Char('a'));
    harness.settle();
    clear_trail(&observer);
    let _ = chord(&mut harness, Key::Char('x'));
    harness.settle();
    let streamed = kinds(&observer);
    let clipboard_at = streamed
        .iter()
        .position(|kind| *kind == "clipboard")
        .expect("the cut streamed a clipboard fact");
    let edit_at = streamed
        .iter()
        .position(|kind| *kind == "edit")
        .expect("the cut journaled its deletion");
    assert!(
        clipboard_at < edit_at,
        "the trail reads clipboard → edit, got {streamed:?}"
    );

    // A fact streams exactly once: the next tick with no clipboard work
    // streams nothing (the drain clears, the edit trail's rule).
    clear_trail(&observer);
    harness.settle();
    assert!(
        clipboard_records(&observer).is_empty(),
        "an idle clipboard queues nothing"
    );
    assert_eq!(harness.settle(), 0, "the trail ride costs no masks");
}

#[test]
fn m39_no_observer_no_trail_the_idle_contract_holds() {
    // The default path: no observer, no enabled trail — the round trips
    // answer exactly as before and the frame costs what it always cost.
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    harness.type_text("held");
    harness.settle();
    let _ = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('a')));
    let _ = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('c')));
    harness.settle();
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("held"),
        "the round trip answers without a listener"
    );
    assert_eq!(harness.settle(), 0, "the disabled trail queues no work");
}

#[test]
fn m39_the_living_documents_re_agree_at_m39() {
    let root = workspace_root();
    let readme = std::fs::read_to_string(root.join("README.md")).expect("readme");
    let architecture =
        std::fs::read_to_string(root.join("docs/architecture.md")).expect("architecture");
    let protocol = std::fs::read_to_string(root.join("docs/inspector-protocol.md"))
        .expect("protocol document");
    let catalog = std::fs::read_to_string(root.join("docs/examples.md")).expect("catalog");
    let changelog = std::fs::read_to_string(root.join("CHANGELOG.md")).expect("changelog");

    // The status block and the ADR count re-agree at M39 (the m-suite
    // pins moving M41 → M43 and 48 → 50 across the chain — M42 and
    // M43 landing in one cut).
    assert!(
        readme.contains("M0 through M52") && readme.contains("fifty-nine decision records"),
        "the README's status block and count re-agree at M39"
    );
    assert!(
        architecture.contains("59 decision records"),
        "the architecture register re-agrees at 46 decision records"
    );
    // The protocol document carries the clipboard event (the wire's
    // documented shape).
    assert!(
        protocol.contains("\"kind\":\"clipboard\"") && protocol.contains("`write_refused`"),
        "the protocol document documents the clipboard event"
    );
    // The catalog lists the demonstration.
    assert!(
        catalog.contains("clipboard_trail"),
        "the examples catalog lists the clipboard trail demo"
    );
    // The changelog stages the milestone.
    assert!(
        changelog.contains("M39 — the clipboard rides the causal log"),
        "the changelog stages M39"
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
