//! M36 exit-criteria integration tests (§13.3, ADR-0042): the keymap
//! rides the causal log. The M35 seam answered the funnel's consult
//! with a bare `bool` — a fired shortcut left no trace a trail reader
//! could route on, and the miss that let the framework's fallbacks
//! take the key looked identical to the window that never installed a
//! table at all. M36 cashes ADR-0041's pre-registered re-open rule:
//! the consult answers as data ([`KeymapConsult`]) and every funnel
//! consult streams one [`KeyRecord`] (the chord the funnel asked about
//! beside the outcome it got) the moment the consult resolves — before
//! the fallbacks' focus records — so a key event's trail reads keymap →
//! focus → input → frame. The funnel behind the record is untouched:
//! the M35 precedence answers exactly as before, and with no observer
//! installed the trail stays silent — one branch per consult, the A7
//! idle contract unchanged.
//!
//! Every test runs the real pipeline (A4) with real widgets driven by
//! real key events — the exact route a shortcut fires through.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{InspectEvent, InspectObserver, KeyRecord};
use alacer_core::keymap::{Chord, Keymap, KeymapConsult, KeymapSeam, consult};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, TextInput};

use std::cell::Cell;
use std::rc::Rc;

/// A pressed event for `key` carrying `modifiers`.
fn pressed(key: Key, modifiers: Modifiers) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers,
        text: None,
    }
}

/// The plain modifiers: nothing held.
fn none() -> Modifiers {
    Modifiers::default()
}

/// Control held (the command set on Linux/Windows).
fn ctrl() -> Modifiers {
    Modifiers {
        control: true,
        ..Modifiers::default()
    }
}

/// The keymap-trail collector the tests install as the pipeline's
/// observer: every [`InspectEvent`] streams in (the m33 helper's
/// shape), and the assertions read the key records beside the events
/// they ordered.
#[derive(Default)]
struct TrailCollector {
    events: std::collections::VecDeque<InspectEvent>,
}

impl TrailCollector {
    /// Every key record streamed so far, oldest first.
    fn key_records(&self) -> Vec<KeyRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::Key(record) => Some(record.clone()),
                _ => None,
            })
            .collect()
    }

    /// The streamed kinds in arrival order — the trail's shape as data.
    fn kinds(&self) -> Vec<&'static str> {
        self.events
            .iter()
            .map(|event| match event {
                InspectEvent::Edge(_) => "edge",
                InspectEvent::Write(_) => "write",
                InspectEvent::Input(_) => "input",
                InspectEvent::Focus(_) => "focus",
                InspectEvent::Edit(_) => "edit",
                InspectEvent::Announce(_) => "announce",
                InspectEvent::Key(_) => "key",
                InspectEvent::Clipboard(_) => "clipboard",
                InspectEvent::Value(_) => "value",
                InspectEvent::Gesture(_) => "gesture",
                InspectEvent::Frame(_) => "frame",
            })
            .collect()
    }
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the m30/m31/m33 suites' shape, exactly.
struct Share(Rc<std::cell::RefCell<TrailCollector>>);

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        let mut ring = self.0.borrow_mut();
        if ring.events.len() >= 256 {
            ring.events.pop_front();
        }
        ring.events.push_back(event);
    }
}

/// The ring's read half: the collector's current answer.
fn trail(observer: &Share) -> std::cell::Ref<'_, TrailCollector> {
    observer.0.borrow()
}

/// Empties the shared ring between beats: each assertion reads the facts
/// the beat under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// A mounted app with the observer riding from the `set_observer` call
/// on — the label gives the tree a focusable-free root the funnel can
/// fall through honestly.
fn mount_app(harness: &mut TestHarness, observer: &Share) {
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let _ = harness.add_child(root, Label::new("The shortcut trail").boxed());
    harness.settle();
    clear_trail(observer);
}

#[test]
fn m36_the_fired_chord_streams_one_record_with_its_chord() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(std::cell::RefCell::new(TrailCollector::default())));
    let fired = Rc::new(Cell::new(0_u32));

    let mut keymap = Keymap::new();
    let counter = Rc::clone(&fired);
    keymap.bind(Chord::new(Key::Char('s')).control(), "save", move || {
        counter.set(counter.get() + 1);
    });
    alacer_core::keymap::install(Rc::new(keymap));

    mount_app(&mut harness, &observer);
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Char('s'), ctrl()));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the keymap answered a chord the label's root ignored"
    );
    assert_eq!(fired.get(), 1, "the bound action ran exactly once");
    harness.settle();

    let records = trail(&observer).key_records();
    assert_eq!(records.len(), 1, "one consult, one record");
    let record = &records[0];
    assert_eq!(record.outcome, KeymapConsult::Fired, "the firing streams");
    assert_eq!(
        *record.chord.key(),
        Key::Char('s'),
        "the record names the chord the funnel asked about"
    );
    assert_eq!(
        record.chord.modifiers(),
        ctrl(),
        "the chord's modifier shape rides the record"
    );
    assert!(
        record.chord.matches(&pressed(Key::Char('s'), ctrl())),
        "the record's chord is the event's own shape"
    );

    // The trail reads keymap → input → frame: the consult record
    // streams before the input record the dispatch forwards and the
    // frame summary the tick builds.
    let kinds = trail(&observer).kinds();
    let key_at = kinds
        .iter()
        .position(|kind| *kind == "key")
        .expect("a key record");
    let input_at = kinds
        .iter()
        .position(|kind| *kind == "input")
        .expect("an input record");
    let frame_at = kinds
        .iter()
        .position(|kind| *kind == "frame")
        .expect("a frame record");
    assert!(
        key_at < input_at && input_at < frame_at,
        "the trail reads keymap → input → frame, got {kinds:?}"
    );
}

#[test]
fn m36_the_miss_streams_the_fall_through() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(std::cell::RefCell::new(TrailCollector::default())));

    // A table whose chords are not the key under test: the consult
    // runs, names no binding, and the funnel falls through to the
    // framework's fallbacks — the refusal streams too, that is exactly
    // the fact a trail reader wants ("why didn't my shortcut fire").
    let mut keymap = Keymap::new();
    keymap.bind(Chord::new(Key::Char('q')), "quit", || {});
    alacer_core::keymap::install(Rc::new(keymap));

    mount_app(&mut harness, &observer);
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Char('x'), none()));
    assert!(
        matches!(response, alacer_core::EventResponse::Ignored),
        "an unbound chord is nobody's key: the funnel falls through"
    );
    harness.settle();

    let records = trail(&observer).key_records();
    assert_eq!(records.len(), 1, "the miss streams one record");
    assert_eq!(
        records[0].outcome,
        KeymapConsult::FellThrough,
        "the seam's silence is named, not collapsed into the uninstalled window"
    );
    assert_eq!(*records[0].chord.key(), Key::Char('x'), "the asked chord");
}

#[test]
fn m36_the_uninstalled_window_streams_its_honest_answer() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(std::cell::RefCell::new(TrailCollector::default())));
    // No `install` on this test's thread: the window carries no
    // shortcuts at all — a legitimate application (the M35 rule), and
    // the trail says which answer the funnel got.
    assert!(
        !alacer_core::keymap::is_installed(),
        "a fresh test thread carries no keymap"
    );

    mount_app(&mut harness, &observer);
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Char('x'), none()));
    assert!(
        matches!(response, alacer_core::EventResponse::Ignored),
        "the uninstalled window answers exactly as today"
    );
    harness.settle();

    let records = trail(&observer).key_records();
    assert_eq!(records.len(), 1, "the uninstalled consult streams");
    assert_eq!(
        records[0].outcome,
        KeymapConsult::Uninstalled,
        "no seam is a named answer, never a silence"
    );
}

#[test]
fn m36_the_focused_widget_answers_before_any_consult_streams() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(std::cell::RefCell::new(TrailCollector::default())));

    let mut keymap = Keymap::new();
    keymap.bind(Chord::new(Key::Char('x')), "stamp", || {});
    alacer_core::keymap::install(Rc::new(keymap));

    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let input = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();
    harness.pipeline_mut().request_focus(input);
    harness.settle();
    clear_trail(&observer);

    // The input owns the plain 'x': the focused widget's vocabulary
    // answers first (the M29/M35 rule) and the keymap is never asked —
    // no consult, no record. The input record the dispatch forwards
    // still streams (the routed event's own trail).
    let response = harness.pipeline_mut().dispatch_key(&KeyEvent::char('x'));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the input answered its own key"
    );
    harness.settle();

    assert!(
        trail(&observer).key_records().is_empty(),
        "the funnel never consulted the keymap: nothing streams"
    );
    let kinds = trail(&observer).kinds();
    assert!(
        kinds.contains(&"input"),
        "the routed key's own input record streams"
    );
}

#[test]
fn m36_the_consult_answers_as_data_beside_the_total_read() {
    // A fresh thread: the consult's three species answer without a
    // pipeline — the funnel's read as data, the M27 pattern at the
    // keymap seam (ADR-0033's rule, cashed by ADR-0042).
    std::thread::spawn(|| {
        let event = pressed(Key::Char('s'), ctrl());

        // Uninstalled: the total read and the consult agree.
        assert!(!alacer_core::keymap::is_installed());
        assert_eq!(consult(&event), KeymapConsult::Uninstalled);
        assert!(
            !consult(&event).fired(),
            "the uninstalled consult never fires"
        );
        assert_eq!(
            alacer_core::keymap::dispatch_installed(&event),
            consult(&event).fired(),
            "the M35 read is the consult's fired verdict — byte-for-byte today's contract"
        );

        // Fired: the bound chord runs exactly once per consult.
        let ran = Rc::new(Cell::new(0_u32));
        let mut keymap = Keymap::new();
        let counter = Rc::clone(&ran);
        keymap.bind(Chord::new(Key::Char('s')).control(), "save", move || {
            counter.set(counter.get() + 1);
        });
        alacer_core::keymap::install(Rc::new(keymap));
        assert_eq!(consult(&event), KeymapConsult::Fired);
        assert_eq!(ran.get(), 1, "the consult ran the binding");
        assert!(alacer_core::keymap::dispatch_installed(&event));

        // Fell through: a seam answered, no binding named the chord.
        let miss = pressed(Key::Char('x'), none());
        assert_eq!(consult(&miss), KeymapConsult::FellThrough);
        assert!(!consult(&miss).fired());
        assert!(!alacer_core::keymap::dispatch_installed(&miss));

        // The species name themselves on the wire's vocabulary.
        assert_eq!(KeymapConsult::Fired.name(), "fired");
        assert_eq!(KeymapConsult::FellThrough.name(), "fell_through");
        assert_eq!(KeymapConsult::Uninstalled.name(), "uninstalled");
    })
    .join()
    .expect("the thread-local story is thread-shaped");
}

/// The m35 example's wrapper (the m35 suite's own shape): the trait is
/// the funnel's only contract, so a live-rebinding table rides it —
/// module-level because items read from the start of a scope.
struct SharedKeymap(Rc<std::cell::RefCell<Keymap>>);

impl KeymapSeam for SharedKeymap {
    fn dispatch(&self, event: &KeyEvent) -> bool {
        self.0.borrow().dispatch(event)
    }
}

#[test]
fn m36_the_record_rides_the_existing_keymap_door_for_the_reader() {
    // The settings story with the trail on: a rebind moves in place,
    // and each consult's record answers with the chord that *is* the
    // binding's identity — one binding per chord, so the record's chord
    // names the fired shortcut without a second trail.
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(std::cell::RefCell::new(TrailCollector::default())));
    let table = Rc::new(std::cell::RefCell::new(Keymap::new()));
    table
        .borrow_mut()
        .bind(Chord::new(Key::ArrowUp).control(), "increment", || {});
    alacer_core::keymap::install(Rc::new(SharedKeymap(Rc::clone(&table))));

    mount_app(&mut harness, &observer);
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::ArrowUp, ctrl()));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    harness.settle();

    // The settings story: the same chord rebinds to `Home`; the old
    // action never fires again and the record's chord moves with it.
    table
        .borrow_mut()
        .unbind(&Chord::new(Key::ArrowUp).control());
    table
        .borrow_mut()
        .bind(Chord::new(Key::Home), "reset", || {});
    clear_trail(&observer);

    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::ArrowUp, ctrl()));
    assert!(
        matches!(response, alacer_core::EventResponse::Ignored),
        "the unbound chord is nobody's key now"
    );
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Home, none()));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    harness.settle();

    let records = trail(&observer).key_records();
    assert_eq!(records.len(), 2, "both consults streamed");
    assert_eq!(records[0].outcome, KeymapConsult::FellThrough);
    assert_eq!(records[1].outcome, KeymapConsult::Fired);
    assert_eq!(*records[1].chord.key(), Key::Home, "the rebind's chord");
    assert!(
        !records
            .iter()
            .any(|record| record.chord == Chord::new(Key::ArrowUp).control()
                && record.outcome == KeymapConsult::Fired),
        "the displaced action never fires again — the record agrees"
    );
}

#[test]
fn m36_with_no_observer_the_funnel_answers_byte_for_byte() {
    // The A7 idle contract: without an observer the trail is silent,
    // the consult still runs the binding, and the response is today's.
    let mut harness = TestHarness::new(400.0, 200.0);
    let fired = Rc::new(Cell::new(0_u32));
    let mut keymap = Keymap::new();
    let counter = Rc::clone(&fired);
    keymap.bind(Chord::new(Key::Char('s')).control(), "save", move || {
        counter.set(counter.get() + 1);
    });
    alacer_core::keymap::install(Rc::new(keymap));

    let root = harness.mount(Column::new().gap(8.0).boxed());
    let button = harness.add_child(root, Button::new("Idle").boxed());
    harness.settle();

    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Char('s'), ctrl()));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(fired.get(), 1, "the binding ran with no observer riding");
    let _ = button;
}

// ---------------------------------------------------------------------------
// The living documents re-agree at M36 (the m-suite's document pins).
// ---------------------------------------------------------------------------

#[test]
fn m36_the_living_documents_re_agree() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");

    // The scope ADR exists; the register and the roadmap name M36.
    assert!(
        root.join("docs/adr/ADR-0042-m36-scope.md").exists(),
        "ADR-0042 scopes the keymap's causal record"
    );
    let architecture = std::fs::read_to_string(root.join("docs/architecture.md"))
        .expect("the architecture document");
    assert!(
        architecture.contains("56 decision records"),
        "the architecture register re-agrees at 45 decision records"
    );
    assert!(
        architecture.contains("ADR-0042"),
        "the register carries the M36 row"
    );

    // The README's status block, count, and ADR line re-agree.
    let readme = std::fs::read_to_string(root.join("README.md")).expect("README");
    assert!(
        readme.contains("M0 through M49") && readme.contains("fifty-six decision records"),
        "the README's status block and ADR count re-agree at M36"
    );

    // The example joins the catalog (the m5 suite's own rule pins the
    // catalog against the directory; the README's table carries the
    // demonstration too).
    let example = std::fs::read_to_string(root.join("crates/alacer/examples/keymap_trail.rs"))
        .expect("the keymap_trail example");
    assert!(
        example.contains("alacer::keymap_trail") && example.contains("KeyRecord"),
        "the example reads the consults on the alacer::keymap_trail target"
    );

    // The corpus stands at 63: the record rides the published keymap
    // route's consult — no hot loop the corpus does not already price
    // (the coordinated rule's no-growth arm, the M31/M33 rule).
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 64;"),
        "the m5 suite's corpus pin stands at 63"
    );

    // The protocol document carries the key event; the changelog
    // stages the milestone.
    let protocol =
        std::fs::read_to_string(root.join("docs/inspector-protocol.md")).expect("the protocol doc");
    assert!(
        protocol.contains("`key` — a keymap consult answered"),
        "the protocol document documents the key event"
    );
    let changelog = std::fs::read_to_string(root.join("CHANGELOG.md")).expect("CHANGELOG");
    assert!(
        changelog.contains("M36"),
        "the changelog stages the keymap's causal record"
    );
}
