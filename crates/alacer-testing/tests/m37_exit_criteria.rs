//! M37 exit-criteria integration tests (§13.3, ADR-0043): the binding
//! names itself in the record. The M36 consult's record carried the
//! chord beside the species — and the chord is the fired binding's
//! identity (one binding per chord, ADR-0041) — but a trail reader that
//! wants the shortcut's *prose* name (a settings UI reading the log, an
//! inspector row a human labels) had to keep its own chord → name table
//! beside the log. M37 cashes ADR-0042's pre-registered re-open rule:
//! the seam grows a provided `fired_name` beside `dispatch` (the
//! provided [`Keymap`] table answers it, custom implementations default
//! to `None`), the funnel captures the name before the consult's
//! dispatch — the name the consult resolves to, never what a mid-flight
//! rebind leaves behind — and only when an observer rides (the A7 idle
//! contract: the unobserved funnel is byte-for-byte, one extra
//! thread-local read never runs), and the record grows the name beside
//! chord and outcome. One consult channel still: the name leg is a pure
//! read that runs no action, the consult's dispatch stays the only
//! door.
//!
//! Every test runs the real pipeline (A4) with real widgets driven by
//! real key events — the exact route a shortcut fires through.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{InspectEvent, InspectObserver, KeyRecord};
use alacer_core::keymap::{Chord, Keymap, KeymapConsult, KeymapSeam, consult_name};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label};

use std::cell::{Cell, RefCell};
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
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the m30/m31/m33/m36 suites' shape, exactly.
struct Share(Rc<RefCell<TrailCollector>>);

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
    let _ = harness.add_child(root, Label::new("The named shortcut trail").boxed());
    harness.settle();
    clear_trail(observer);
}

#[test]
fn m37_the_fired_record_carries_the_binding_s_prose_name() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(RefCell::new(TrailCollector::default())));
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
        record.name.as_deref(),
        Some("save"),
        "the record names the binding the consult fired"
    );
    assert_eq!(
        *record.chord.key(),
        Key::Char('s'),
        "the chord rides beside the name — the M36 record's shape, grown"
    );
}

#[test]
fn m37_the_miss_and_the_uninstalled_stream_no_name() {
    // The uninstalled window FIRST — the thread carries no keymap yet:
    // the funnel consults, nothing answers, and the record names the
    // absence honestly, no binding to name.
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(RefCell::new(TrailCollector::default())));
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
        "an unbound chord is nobody's key: the funnel falls through"
    );
    harness.settle();
    let records = trail(&observer).key_records();
    assert_eq!(records.len(), 1, "the uninstalled consult streams");
    assert_eq!(records[0].outcome, KeymapConsult::Uninstalled);
    assert_eq!(records[0].name, None, "no seam, no name");

    // Then the miss: a table whose chords are not the key under test —
    // the consult streams its fall-through and *no* name (a miss has no
    // binding to name). A fresh harness mounts the second app (one
    // mount per harness).
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(RefCell::new(TrailCollector::default())));
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
    assert_eq!(records[0].outcome, KeymapConsult::FellThrough);
    assert_eq!(
        records[0].name, None,
        "a fall-through names no binding — there is none"
    );
}

/// A custom seam that carries no name: the trait's provided method
/// answers `None` and the implementation overrides nothing — the
/// M35/M36 custom seams keep compiling and streaming unnamed.
struct OnlyEscape;

impl KeymapSeam for OnlyEscape {
    fn dispatch(&self, event: &KeyEvent) -> bool {
        event.presses(&Key::Escape)
    }
}

/// A custom seam that names its own answer: the provided method is the
/// contract, and an implementation whose shortcuts come from somewhere
/// else — a settings file, a menu table — answers it beside `dispatch`.
struct NamedEscape(Rc<Cell<u32>>);

impl KeymapSeam for NamedEscape {
    fn dispatch(&self, _event: &KeyEvent) -> bool {
        self.0.set(self.0.get() + 1);
        true
    }

    fn fired_name(&self, _event: &KeyEvent) -> Option<String> {
        Some("dismiss".to_string())
    }
}

#[test]
fn m37_a_custom_seam_defaults_to_no_name_and_answers_its_own() {
    // The default: a seam that overrides nothing still streams —
    // unnamed. The provided method keeps the custom implementation's
    // contract exactly where M35/M36 left it.
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(RefCell::new(TrailCollector::default())));
    alacer_core::keymap::install(Rc::new(OnlyEscape));
    mount_app(&mut harness, &observer);
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Escape, none()));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    harness.settle();
    let records = trail(&observer).key_records();
    assert_eq!(records.len(), 1, "the custom seam's consult streams");
    assert_eq!(
        records[0].name, None,
        "the default answers None — the record stays honest about it"
    );

    // The override: a seam that names its binding streams the name —
    // the trait grew beside `dispatch`, priced by a real reader. A
    // fresh harness mounts the second app (one mount per harness).
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(RefCell::new(TrailCollector::default())));
    let ran = Rc::new(Cell::new(0_u32));
    alacer_core::keymap::install(Rc::new(NamedEscape(Rc::clone(&ran))));
    mount_app(&mut harness, &observer);
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Escape, none()));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(
        ran.get(),
        1,
        "dispatch ran exactly once — the name leg never dispatches"
    );
    harness.settle();
    let records = trail(&observer).key_records();
    assert_eq!(
        records[0].name.as_deref(),
        Some("dismiss"),
        "the custom seam's own name rides its record"
    );
}

/// The live-rebinding wrapper the m35 example carries: the trait is the
/// funnel's only contract, so the table moves in place under it. The
/// wrapper delegates *both* legs — dispatch beside `fired_name` —
/// because a seam that wraps a naming table and names nothing would
/// stream unnamed records: the name leg is the consult's read, not the
/// table's private fact.
struct SharedKeymap(Rc<RefCell<Keymap>>);

impl KeymapSeam for SharedKeymap {
    fn dispatch(&self, event: &KeyEvent) -> bool {
        self.0.borrow().dispatch(event)
    }

    fn fired_name(&self, event: &KeyEvent) -> Option<String> {
        self.0.borrow().fired_name(event)
    }
}

/// A seam whose action swaps the thread's table mid-flight — the core
/// suite's `SelfReplacing` shape, carrying a name: the funnel's capture
/// runs before the consult's dispatch, so the record names the binding
/// the consult resolved to, never what the mid-flight swap left behind.
struct SelfRenaming(Rc<Cell<bool>>);

impl KeymapSeam for SelfRenaming {
    fn dispatch(&self, _event: &KeyEvent) -> bool {
        if !self.0.get() {
            self.0.set(true);
            let mut replacement = Keymap::new();
            replacement.bind(Chord::new(Key::Char('s')).control(), "renamed", || {});
            alacer_core::keymap::install(Rc::new(replacement));
        }
        true
    }

    fn fired_name(&self, _event: &KeyEvent) -> Option<String> {
        Some("save".to_string())
    }
}

#[test]
fn m37_the_captured_name_is_the_binding_the_consult_resolved_to() {
    // The capture runs before the consult's dispatch: the fired action
    // replaces the thread's table mid-flight, and the record still
    // names the binding the consult resolved to — a name read *after*
    // dispatch would answer the replacement ("renamed"), not the
    // binding that answered ("save").
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(RefCell::new(TrailCollector::default())));

    let swapped = Rc::new(Cell::new(false));
    alacer_core::keymap::install(Rc::new(SelfRenaming(Rc::clone(&swapped))));

    mount_app(&mut harness, &observer);
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Char('s'), ctrl()));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert!(swapped.get(), "the action swapped the table mid-flight");
    harness.settle();

    let records = trail(&observer).key_records();
    assert_eq!(records.len(), 1, "the consult streamed");
    assert_eq!(records[0].outcome, KeymapConsult::Fired);
    assert_eq!(
        records[0].name.as_deref(),
        Some("save"),
        "the record names the binding the consult resolved to — never what the mid-flight swap left behind"
    );

    // The next consult answers the replacement: the table's present
    // truth, its own record.
    clear_trail(&observer);
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Char('s'), ctrl()));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    harness.settle();
    let records = trail(&observer).key_records();
    assert_eq!(
        records[0].name.as_deref(),
        Some("renamed"),
        "the next consult names the table's present answer"
    );
}

#[test]
fn m37_the_rebind_moves_the_name_with_the_chord() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(RefCell::new(TrailCollector::default())));

    let table = Rc::new(RefCell::new(Keymap::new()));
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
    let records = trail(&observer).key_records();
    assert_eq!(
        records[0].name.as_deref(),
        Some("increment"),
        "the fired binding's name rides its record"
    );

    // The settings story: the chord rebinds to `Home` under the name
    // the new action carries; the old chord falls through unnamed.
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
    assert!(matches!(response, alacer_core::EventResponse::Ignored));
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Home, none()));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    harness.settle();

    let records = trail(&observer).key_records();
    assert_eq!(records.len(), 2, "both consults streamed");
    assert_eq!(records[0].outcome, KeymapConsult::FellThrough);
    assert_eq!(
        records[0].name, None,
        "the displaced chord names nothing now"
    );
    assert_eq!(records[1].outcome, KeymapConsult::Fired);
    assert_eq!(
        records[1].name.as_deref(),
        Some("reset"),
        "the name moved with the rebind — the settings story, read from the log"
    );
}

#[test]
fn m37_with_no_observer_the_funnel_answers_byte_for_byte() {
    // The A7 idle contract: without an observer the name leg never
    // runs — no capture read, no record — and the consult's answer is
    // today's.
    let mut harness = TestHarness::new(400.0, 200.0);
    let fired = Rc::new(Cell::new(0_u32));
    let mut keymap = Keymap::new();
    let counter = Rc::clone(&fired);
    keymap.bind(Chord::new(Key::Char('s')).control(), "save", move || {
        counter.set(counter.get() + 1);
    });
    alacer_core::keymap::install(Rc::new(keymap));

    let root = harness.mount(Column::new().gap(8.0).boxed());
    let _ = harness.add_child(root, Label::new("Idle").boxed());
    harness.settle();

    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Char('s'), ctrl()));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(fired.get(), 1, "the binding ran with no observer riding");
}

#[test]
fn m37_the_name_leg_answers_beside_the_consult_without_dispatching() {
    // The read as data, without a pipeline: `consult_name` answers the
    // installed seam's name for the chord — a pure read, the action
    // count unmoved (the consult's dispatch is the only door).
    std::thread::spawn(|| {
        let event = pressed(Key::Char('s'), ctrl());
        assert!(
            !alacer_core::keymap::is_installed(),
            "a fresh test thread carries no keymap"
        );
        assert_eq!(consult_name(&event), None, "no seam, no name");

        let ran = Rc::new(Cell::new(0_u32));
        let mut keymap = Keymap::new();
        let counter = Rc::clone(&ran);
        keymap.bind(Chord::new(Key::Char('s')).control(), "save", move || {
            counter.set(counter.get() + 1);
        });
        alacer_core::keymap::install(Rc::new(keymap));

        assert_eq!(
            consult_name(&event),
            Some("save".to_string()),
            "the table names the chord's binding"
        );
        assert_eq!(ran.get(), 0, "the name leg ran no action");
        assert_eq!(
            consult_name(&pressed(Key::Char('x'), none())),
            None,
            "the miss names nothing"
        );
    })
    .join()
    .expect("the thread-local story is thread-shaped");
}

// ---------------------------------------------------------------------------
// The living documents re-agree at M37 (the m-suite's document pins).
// ---------------------------------------------------------------------------

#[test]
fn m37_the_living_documents_re_agree() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");

    // The scope ADR exists; the register and the roadmap name M37.
    assert!(
        root.join("docs/adr/ADR-0043-m37-scope.md").exists(),
        "ADR-0043 scopes the binding's name in the record"
    );
    let architecture = std::fs::read_to_string(root.join("docs/architecture.md"))
        .expect("the architecture document");
    assert!(
        architecture.contains("52 decision records"),
        "the architecture register re-agrees at 45 decision records"
    );
    assert!(
        architecture.contains("ADR-0043"),
        "the register carries the M37 row"
    );

    // The README's status block, count, and ADR line re-agree.
    let readme = std::fs::read_to_string(root.join("README.md")).expect("README");
    assert!(
        readme.contains("M0 through M45") && readme.contains("fifty-two decision records"),
        "the README's status block and ADR count re-agree at M37"
    );

    // The example joins the catalog (the m5 suite's own rule pins the
    // catalog against the directory; the README's table carries the
    // demonstration too).
    let example = std::fs::read_to_string(root.join("crates/alacer/examples/shortcut_names.rs"))
        .expect("the shortcut_names example");
    assert!(
        example.contains("alacer::shortcut_names") && example.contains("KeyRecord"),
        "the example reads the named consults on the alacer::shortcut_names target"
    );

    // The corpus stands at 63: the name leg costs the unobserved
    // funnel nothing and rides the published keymap route's consult —
    // no hot loop the corpus does not already price (the coordinated
    // rule's no-growth arm, the M31/M33/M36 rule).
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 63;"),
        "the m5 suite's corpus pin stands at 63"
    );

    // The protocol document carries the key event's name field; the
    // changelog stages the milestone.
    let protocol =
        std::fs::read_to_string(root.join("docs/inspector-protocol.md")).expect("the protocol doc");
    assert!(
        protocol.contains("`name` — the fired binding's prose name"),
        "the protocol document documents the key event's name field"
    );
    let changelog = std::fs::read_to_string(root.join("CHANGELOG.md")).expect("CHANGELOG");
    assert!(
        changelog.contains("M37"),
        "the changelog stages the binding's name in the record"
    );
}
