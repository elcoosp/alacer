//! M35 exit-criteria integration tests (§13.3, ADR-0041): the app's
//! shortcuts ride the dispatch contract. The keyboard's other half ships —
//! a window-level [`KeymapSeam`] the assembly site installs (the
//! clipboard seam's thread-local shape, ADR-0020), consulted by
//! [`PassPipeline::dispatch_key`] when the focused widget did not answer
//! and *before* the framework's own fallbacks (Tab cycling, the M24 seam;
//! the directional ring, the M29 seam). The precedence ADR-0041 fixes —
//! the focused widget's vocabulary, the app's shortcuts, the framework's
//! fallbacks — keeps every rule the funnel already carries: a focused
//! input's Space types, an explicit `Tab` binding beats the framework's
//! cycle, an unbound `Shift+Tab` still rides the walk backward, and a
//! window with no shortcuts installed answers exactly as today (the M11
//! rule at the dispatch seam — an optional seam has no panic door).
//!
//! Every test runs the real pipeline (A4) with real widgets driven by
//! real key events — the exact route a shortcut fires through.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::keymap::{Chord, Keymap, KeymapSeam, install, is_installed};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, TextInput};

use std::cell::Cell;
use std::rc::Rc;

/// The two-button form the focus assertions walk: the pre-order candidate
/// order is `[first, second]` (the m30 grid's shape, minus the geometry).
fn mount_two_buttons(harness: &mut TestHarness) {
    let root = harness.mount(Column::new().gap(8.0).padding(8.0).boxed());
    let _ = harness.add_child(root, Button::new("First").boxed());
    let _ = harness.add_child(root, Button::new("Second").boxed());
    harness.settle();
}

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

#[test]
fn m35_the_bound_chord_fires_the_app_vocabulary() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let saved = Rc::new(Cell::new(0_u32));
    let mut keymap = Keymap::new();
    let counter = Rc::clone(&saved);
    keymap.bind(Chord::new(Key::Char('s')).control(), "save", move || {
        counter.set(counter.get() + 1);
    });
    install(Rc::new(keymap));
    assert!(is_installed(), "the assembly installed the seam");

    // A focused input holds the field: the shortcut still answers —
    // window-level vocabulary answers wherever the focus sits (the
    // seam's reason to exist).
    let root = harness.mount(Column::new().boxed());
    let input = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();
    harness.pipeline_mut().request_focus(input);
    harness.settle();
    let before = harness.pipeline().focus().expect("the input holds focus");

    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Char('s'), ctrl()));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the keymap answered a chord the focused input ignored"
    );
    assert_eq!(saved.get(), 1, "the bound action ran exactly once");
    assert_eq!(
        harness.pipeline().focus(),
        Some(before),
        "a shortcut is focus-neutral: the input still holds the field"
    );
}

#[test]
fn m35_the_focused_widget_answers_before_the_keymap() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let stamped = Rc::new(Cell::new(0_u32));
    let mut keymap = Keymap::new();
    let counter = Rc::clone(&stamped);
    keymap.bind(Chord::new(Key::Char('x')), "stamp", move || {
        counter.set(counter.get() + 1);
    });
    install(Rc::new(keymap));

    // The input owns the plain 'x': its editing vocabulary (M3) answers
    // first, and the bound action never fires — the focused widget's
    // answer always took precedence (the M29 rule, now pinned at the
    // keymap tier too).
    let root = harness.mount(Column::new().boxed());
    let input = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();
    harness.pipeline_mut().request_focus(input);
    harness.settle();

    let response = harness.pipeline_mut().dispatch_key(&KeyEvent::char('x'));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the input answered its own key"
    );
    assert_eq!(
        stamped.get(),
        0,
        "the keymap never fired behind a widget's own answer"
    );
}

#[test]
fn m35_the_explicit_binding_beats_the_framework_fallback() {
    let mut harness = TestHarness::new(360.0, 240.0);
    mount_two_buttons(&mut harness);
    // Enter focus: from no focus, the first Tab takes the first candidate.
    harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Tab, none()));
    let started = harness.pipeline().focus().expect("the first button");
    harness.settle();

    let pane_moved = Rc::new(Cell::new(0_u32));
    let mut keymap = Keymap::new();
    let counter = Rc::clone(&pane_moved);
    keymap.bind(Chord::new(Key::Tab), "next pane", move || {
        counter.set(counter.get() + 1);
    });
    install(Rc::new(keymap));

    // The app's explicit `Tab` beats the framework's cycle: the action
    // fires and focus stays — the explicit ordering the implicit cannot
    // overrule.
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Tab, none()));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(pane_moved.get(), 1, "the explicit binding fired");
    assert_eq!(
        harness.pipeline().focus(),
        Some(started),
        "the framework's cycle did not move focus behind the binding"
    );

    // The chord the app did not bind still rides the framework: a plain
    // chord's shape is exact, so Shift+Tab (a different chord — another
    // modifier, free for another action) walks the candidates backward.
    let response = harness.pipeline_mut().dispatch_key(&pressed(
        Key::Tab,
        Modifiers {
            shift: true,
            ..none()
        },
    ));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the unbound Shift+Tab is still the framework's backward walk"
    );
    assert_ne!(
        harness.pipeline().focus(),
        Some(started),
        "the walk moved where the app stayed silent"
    );
    assert_eq!(
        pane_moved.get(),
        1,
        "the shifted chord never fired the plain chord's action"
    );
}

#[test]
fn m35_a_window_without_shortcuts_answers_unchanged() {
    // The uninstalled window is a legitimate shape, not a wiring bug:
    // the read is total, there is no panic door (the clipboard's rule,
    // inverted for an optional seam). This test installs nothing, so it
    // runs on its own thread's fresh thread-local.
    assert!(!is_installed(), "no seam was installed on this thread");

    let mut harness = TestHarness::new(360.0, 240.0);
    mount_two_buttons(&mut harness);

    // The framework's fallbacks stand exactly where the app stayed
    // silent: Tab cycles, the unknown chord is nobody's.
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Tab, none()));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "Tab cycles as it always has"
    );
    assert!(harness.pipeline().focus().is_some(), "the cycle landed");

    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Char('s'), ctrl()));
    assert!(
        matches!(response, alacer_core::EventResponse::Ignored),
        "an unbound chord with no seam is nobody's key"
    );
}

#[test]
fn m35_rebind_and_unbind_are_the_settings_story() {
    let mut harness = TestHarness::new(360.0, 240.0);
    mount_two_buttons(&mut harness);
    harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Tab, none()));
    let started = harness.pipeline().focus().expect("the first button");
    harness.settle();

    let first = Rc::new(Cell::new(0_u32));
    let second = Rc::new(Cell::new(0_u32));
    let mut keymap = Keymap::new();
    let one = Rc::clone(&first);
    keymap.bind(Chord::new(Key::Tab), "cycle", move || {
        one.set(one.get() + 1);
    });
    let table = Rc::new(std::cell::RefCell::new(keymap));
    install(Rc::new(Settings(table.clone())));

    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Tab, none()));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(first.get(), 1, "the original binding fired");
    assert_eq!(
        harness.pipeline().focus(),
        Some(started),
        "the binding held focus where it was"
    );

    table.borrow_mut().bind(Chord::new(Key::Tab), "next pane", {
        let two = Rc::clone(&second);
        move || two.set(two.get() + 1)
    });
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Tab, none()));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(first.get(), 1, "the displaced action never ran again");
    assert_eq!(second.get(), 1, "the replacement ran");

    assert!(
        table.borrow_mut().unbind(&Chord::new(Key::Tab)),
        "the unbind moved one"
    );
    let response = harness
        .pipeline_mut()
        .dispatch_key(&pressed(Key::Tab, none()));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the framework's cycle answered the unbound chord"
    );
    assert_ne!(
        harness.pipeline().focus(),
        Some(started),
        "focus cycles again where the app unbound"
    );
}

/// The settings story's own shape: the app keeps its table behind
/// interior mutability and wraps it in the seam — the trait is the
/// funnel's only contract, so a live-rebinding table rides unchanged.
struct Settings(Rc<std::cell::RefCell<Keymap>>);

impl KeymapSeam for Settings {
    fn dispatch(&self, event: &KeyEvent) -> bool {
        self.0.borrow().dispatch(event)
    }
}
#[test]
fn m35_the_surface_rides_the_same_cut() {
    // The seam ships where the funnel consults it: the vocabulary beside
    // the clipboard and text seams, the consult behind the dispatch
    // contract.
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    let keymap = std::fs::read_to_string(root.join("crates/alacer-core/src/keymap.rs"))
        .expect("keymap.rs is in the tree");
    for fragment in [
        "pub trait KeymapSeam",
        "pub struct Chord",
        "pub fn dispatch_installed",
        "pub fn install",
        "alacer::keymap",
    ] {
        assert!(
            keymap.contains(fragment),
            "the keymap vocabulary carries its own seam: {fragment}"
        );
    }
    let passes = std::fs::read_to_string(root.join("crates/alacer-core/src/passes.rs"))
        .expect("passes.rs is in the tree");
    for fragment in [
        // Since M36 (ADR-0042) the funnel consults through the read as
        // data — `dispatch_installed` is the consult's fired verdict at
        // the keymap seam, the M35 contract byte-for-byte.
        "crate::keymap::consult",
        "The app's shortcuts (M35, ADR-0041)",
    ] {
        assert!(
            passes.contains(fragment),
            "the dispatch funnel consults the installed keymap: {fragment}"
        );
    }
    let core = std::fs::read_to_string(root.join("crates/alacer-core/src/lib.rs"))
        .expect("lib.rs is in the tree");
    assert!(
        // Since M36 (ADR-0042) the vocabulary carries the consult's
        // answer beside the seam: the re-export grew, the seam stays.
        core.contains("pub use keymap::{Chord, Keymap, KeymapConsult, KeymapSeam};"),
        "the crate's public vocabulary carries the seam"
    );

    // The consumer demonstration exists and rides the workspace's
    // per-seam tracing idiom.
    let example = std::fs::read_to_string(root.join("crates/alacer/examples/shortcut_map.rs"))
        .expect("the M35 example is in the tree");
    assert!(
        example.contains("alacer::shortcut_map") && example.contains("Keymap"),
        "the example reads the shortcuts on the alacer::shortcut_map target"
    );

    // The scope ADR exists; the living documents re-agree at M35.
    assert!(
        root.join("docs/adr/ADR-0041-m35-scope.md").exists(),
        "ADR-0041 scopes the keymap seam"
    );
    let readme = std::fs::read_to_string(root.join("README.md")).expect("README");
    assert!(
        readme.contains("M0 through M39") && readme.contains("forty-six decision records"),
        "the README's status block and ADR count re-agree at M35"
    );
    let architecture = std::fs::read_to_string(root.join("docs/architecture.md"))
        .expect("the architecture document");
    assert!(
        architecture.contains("46 decision records"),
        "the architecture register re-agrees at 45 decision records"
    );

    // The corpus stands at 63: the keymap's mixed walk joins the
    // published routes at this cut (the coordinated bench +
    // contracts.json + baseline rule).
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 63;"),
        "the m5 suite's corpus pin stands at 63"
    );

    // The changelog stages the milestone.
    let changelog = std::fs::read_to_string(root.join("CHANGELOG.md")).expect("CHANGELOG");
    assert!(
        changelog.contains("M35"),
        "the changelog stages the keymap seam"
    );
}
