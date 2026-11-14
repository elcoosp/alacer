//! M38 exit-criteria integration tests (§10.3, ADR-0044): the primary
//! chord. The exact `Modifiers` shape a [`Chord`] names could not
//! express the app's most common intent — "the platform's primary
//! accelerator" — because the shape is *physical*: `Ctrl+S` on
//! Linux/Windows is `Cmd+S` on macOS, and an app that wants one chord
//! to mean "the primary modifier plus S" everywhere had to fork its
//! table per platform (or reach for stringly-typed chords the funnel
//! cannot compare — the rule ADR-0041 refused). M38 cashes ADR-0043's
//! pre-registered re-open rule: the vocabulary grows beside `Chord` —
//! one `Chord::primary()` builder that resolves at build time the way
//! `Modifiers::primary` reads (Command on Apple platforms, Control
//! elsewhere) — and the resolution is the chord's identity: the
//! exact-shape match (the M35 invariant) is untouched, a `primary()`
//! chord matches the event the platform actually sends, never the
//! other platform's flag, and off Apple platforms a `primary()` chord
//! *is* the `control()` chord (a rebind moves in place, never
//! shadows).
//!
//! Every test runs the real pipeline (A4) with real widgets driven by
//! real key events — the exact route a shortcut fires through.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{InspectEvent, InspectObserver, KeyRecord};
use alacer_core::keymap::{Chord, Keymap, KeymapConsult, KeymapSeam};
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

/// Control held (the platform's primary modifier off Apple platforms).
fn ctrl() -> Modifiers {
    Modifiers {
        control: true,
        ..Modifiers::default()
    }
}

/// Command held (the platform's primary modifier on Apple platforms).
fn cmd() -> Modifiers {
    Modifiers {
        meta: true,
        ..Modifiers::default()
    }
}

/// The keymap-trail collector the tests install as the pipeline's
/// observer (the m33 helper's shape).
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

/// The shared-ring wrapper the pipeline owns (the m30/m31/m33/m36/m37
/// suites' shape, exactly).
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

/// Empties the shared ring between beats.
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
    let _ = harness.add_child(root, Label::new("The primary chord").boxed());
    harness.settle();
    clear_trail(observer);
}

/// The live-rebinding wrapper (the m35 suite's shape, both legs — the
/// m37 lesson): the trait is the funnel's only contract, so a wrapper
/// around a naming table delegates `dispatch` beside `fired_name`.
struct SharedKeymap(Rc<RefCell<Keymap>>);

impl KeymapSeam for SharedKeymap {
    fn dispatch(&self, event: &KeyEvent) -> bool {
        self.0.borrow().dispatch(event)
    }

    fn fired_name(&self, event: &KeyEvent) -> Option<String> {
        self.0.borrow().fired_name(event)
    }
}

#[test]
fn m38_the_primary_chord_fires_through_the_real_funnel() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(RefCell::new(TrailCollector::default())));
    let fired = Rc::new(Cell::new(0_u32));

    let mut keymap = Keymap::new();
    let counter = Rc::clone(&fired);
    keymap.bind(Chord::new(Key::Char('s')).primary(), "save", move || {
        counter.set(counter.get() + 1);
    });
    alacer_core::keymap::install(Rc::new(keymap));

    mount_app(&mut harness, &observer);

    // The event the platform sends for "primary+S": Control held off
    // Apple platforms, Command held on them. The same chord matches
    // the platform's own shape — the vocabulary's whole point.
    #[cfg(not(target_os = "macos"))]
    let primary_event = pressed(Key::Char('s'), ctrl());
    #[cfg(target_os = "macos")]
    let primary_event = pressed(Key::Char('s'), cmd());
    let response = harness.pipeline_mut().dispatch_key(&primary_event);
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the primary chord answered the platform's own shape"
    );
    assert_eq!(fired.get(), 1, "the bound action ran exactly once");
    harness.settle();

    let records = trail(&observer).key_records();
    assert_eq!(records.len(), 1, "one consult, one record");
    assert_eq!(records[0].outcome, KeymapConsult::Fired);
    assert_eq!(
        records[0].name.as_deref(),
        Some("save"),
        "the M37 name rides the primary chord's record too"
    );
    assert!(
        records[0].chord.matches(&primary_event),
        "the record's chord is the event's own shape — the resolved physical flags"
    );

    // The other platform's flag is a different chord: off Apple
    // platforms a Command event never aliases the primary chord (the
    // exact shape, the M35 rule — the fall-through streams the asked
    // chord, honestly unnamed).
    #[cfg(not(target_os = "macos"))]
    {
        clear_trail(&observer);
        let response = harness
            .pipeline_mut()
            .dispatch_key(&pressed(Key::Char('s'), cmd()));
        assert!(
            matches!(response, alacer_core::EventResponse::Ignored),
            "a Command event is a different shape off Apple platforms"
        );
        assert_eq!(fired.get(), 1, "the aliasing never fired");
        harness.settle();
        let records = trail(&observer).key_records();
        assert_eq!(records.len(), 1, "the fall-through streams");
        assert_eq!(records[0].outcome, KeymapConsult::FellThrough);
        assert_eq!(
            records[0].chord.modifiers(),
            cmd(),
            "the record carries the chord the funnel asked about — the meta shape"
        );
        assert_eq!(records[0].name, None, "the miss names nothing");
    }
}

#[test]
fn m38_the_settings_story_rebinds_the_primary_chord_in_place() {
    // The resolution is the chord's identity: off Apple platforms a
    // `primary()` chord and a `control()` chord are the same chord, so
    // binding the physical shape over the primary one moves in place —
    // the displaced action never fires again (the M35 invariant,
    // visible at the vocabulary's own resolution).
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(Rc::new(RefCell::new(TrailCollector::default())));

    let table = Rc::new(RefCell::new(Keymap::new()));
    table
        .borrow_mut()
        .bind(Chord::new(Key::Char('s')).primary(), "save", || {});
    alacer_core::keymap::install(Rc::new(SharedKeymap(Rc::clone(&table))));

    mount_app(&mut harness, &observer);
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Char('s'), ctrl()));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    harness.settle();
    let records = trail(&observer).key_records();
    assert_eq!(
        records[0].name.as_deref(),
        Some("save"),
        "the primary chord's binding names itself"
    );

    // The settings story: the physical Control shape binds over the
    // primary one — off Apple platforms that is the SAME chord, the
    // rebind moves in place.
    #[cfg(not(target_os = "macos"))]
    {
        table
            .borrow_mut()
            .bind(Chord::new(Key::Char('s')).control(), "save-as", || {});
        clear_trail(&observer);

        let response = harness
            .pipeline_mut()
            .dispatch_key(&pressed(Key::Char('s'), ctrl()));
        assert!(matches!(response, alacer_core::EventResponse::Handled));
        harness.settle();
        let records = trail(&observer).key_records();
        assert_eq!(
            records[0].name.as_deref(),
            Some("save-as"),
            "the rebind moved in place — the primary chord and the Control chord are one chord off Apple platforms"
        );
    }
}

#[test]
fn m38_with_no_observer_the_funnel_answers_byte_for_byte() {
    // The A7 idle contract: without an observer the trail is silent
    // and the primary chord's consult answers exactly as a `control()`
    // chord's would — the resolution is build-time, the funnel's read
    // untouched.
    let mut harness = TestHarness::new(400.0, 200.0);
    let fired = Rc::new(Cell::new(0_u32));
    let mut keymap = Keymap::new();
    let counter = Rc::clone(&fired);
    keymap.bind(Chord::new(Key::Char('s')).primary(), "save", move || {
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

// ---------------------------------------------------------------------------
// The living documents re-agree at M38 (the m-suite's document pins).
// ---------------------------------------------------------------------------

#[test]
fn m38_the_living_documents_re_agree() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");

    // The scope ADR exists; the register and the roadmap name M38.
    assert!(
        root.join("docs/adr/ADR-0044-m38-scope.md").exists(),
        "ADR-0044 scopes the primary chord"
    );
    let architecture = std::fs::read_to_string(root.join("docs/architecture.md"))
        .expect("the architecture document");
    assert!(
        architecture.contains("64 decision records"),
        "the architecture register re-agrees at 45 decision records"
    );
    assert!(
        architecture.contains("ADR-0044"),
        "the register carries the M38 row"
    );

    // The README's status block, count, and ADR line re-agree.
    let readme = std::fs::read_to_string(root.join("README.md")).expect("README");
    assert!(
        readme.contains("M0 through M57") && readme.contains("sixty-four decision records"),
        "the README's status block and ADR count re-agree at M38"
    );

    // The example joins the catalog (the m5 suite's own rule pins the
    // catalog against the directory; the README's table carries the
    // demonstration too).
    let example = std::fs::read_to_string(root.join("crates/alacer/examples/primary_chords.rs"))
        .expect("the primary_chords example");
    assert!(
        example.contains("alacer::primary_chords") && example.contains("Chord::primary"),
        "the example reads the primary chords on the alacer::primary_chords target"
    );

    // The corpus stands at 63: the primary chord resolves at build
    // time — no new hot loop the corpus does not already price (the
    // coordinated rule's no-growth arm, the M31/M33/M36/M37 rule).
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 65;"),
        "the m5 suite's corpus pin stands at 63"
    );

    // The protocol document names the resolved physical shape the
    // `modifiers` field carries; the changelog stages the milestone.
    let protocol =
        std::fs::read_to_string(root.join("docs/inspector-protocol.md")).expect("the protocol doc");
    assert!(
        protocol.contains("resolved physical shape"),
        "the protocol document documents the primary chord's resolution"
    );
    let changelog = std::fs::read_to_string(root.join("CHANGELOG.md")).expect("CHANGELOG");
    assert!(
        changelog.contains("M38"),
        "the changelog stages the primary chord"
    );
}
