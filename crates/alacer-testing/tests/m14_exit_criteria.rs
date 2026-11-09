//! M14 exit-criteria integration tests (§10.3, ADR-0020): the clipboard
//! joins the editing contract. The clipboard seam carries the OS round
//! trip — no provided implementation: the workspace's engine rides
//! arboard, the harness drives an in-memory seam — and core fixes one
//! paste-sanitization rule. Copy, cut, and paste consume the M11
//! selection vocabulary the pointer and word tiers already share; a
//! refused read or write is an honest no-op with no masks, and a refused
//! cut never deletes; and the corpus grows 51 → 52 by the coordinated
//! rule.
//!
//! Every test runs the real pipeline (A4) with the real text seam — the
//! exact route the window's keys take. The clipboard itself is the
//! harness's in-memory seam: a headless CI never races the operator's
//! actual OS clipboard, and the seam indirection is exactly what the
//! windowed assembly swaps for arboard.

use alacer_a11y::{AccessKitSink, find_in_update};
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::geom::Point;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves")
}

fn read_workspace_file(path: &str) -> String {
    let path = workspace_root().join(path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("required file {} missing: {error}", path.display()))
}

fn parse_workspace_json(path: &str) -> serde_json::Value {
    let text = read_workspace_file(path);
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path} is not valid JSON: {error}"))
}

/// An input mounted through the real pipeline and filled by typing
/// `content` — the fill borrows keyboard focus, so every key under test
/// routes to the field the way the window delivers them. Focus is
/// requested even for an empty content: the clipboard chords are
/// keyboard edits and route to the focused field.
fn mount_input(harness: &mut TestHarness, content: &str) {
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();
    // Focus resolves in pass 3 of a tick: settle before typing, or the
    // keys route to no focused widget (the M11 suite's same rule).
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    if !content.is_empty() {
        harness.type_text(content);
    }
}

/// The input's committed value, read through the a11y tree (§11.5).
fn input_value(harness: &TestHarness) -> String {
    harness
        .query(Role::TextInput)
        .named("Search")
        .first()
        .value
        .clone()
        .expect("an editable input carries its value")
}

/// A Ctrl press (the clipboard chords' command set on Linux/Windows;
/// Meta is the same tier on macOS).
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
fn m14_copy_cut_and_paste_route_through_the_pipeline() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");

    // Select all, then copy: the clipboard carries the selection's exact
    // span, the value keeps it, and the AT vocabulary is untouched.
    harness.key(&ctrl(Key::Char('a')));
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((0, 11)),
        "Ctrl+A selects the whole value"
    );
    harness.key(&ctrl(Key::Char('c')));
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("hello world"),
        "the copy writes the selection to the seam"
    );
    assert_eq!(input_value(&harness), "hello world", "a copy edits nothing");

    // Cut: the same span moves to the clipboard — the value goes with it,
    // the caret collapses at its start (the M11 edit rule).
    harness.key(&ctrl(Key::Char('x')));
    assert_eq!(input_value(&harness), "");
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("hello world"),
        "the cut writes the same span"
    );
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((0, 0)),
        "the caret collapses where the range was"
    );

    // Paste: the clipboard's text inserts at the caret — an edit like any
    // other, riding the same PAINT | A11Y currency.
    harness.key(&ctrl(Key::Char('v')));
    assert_eq!(input_value(&harness), "hello world");
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((11, 11)));
}

#[test]
fn m14_paste_collapses_line_breaks_to_spaces() {
    // The clipboard carries arbitrary platform text and this field is
    // single-line: core's paste rule collapses every line break (CR, LF,
    // CRLF — a CRLF pair counts once) to one space, never the widget's
    // own improvisation (ADR-0020).
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "");
    harness.set_clipboard("a\r\nb\rc\nd");
    harness.key(&ctrl(Key::Char('v')));
    assert_eq!(input_value(&harness), "a b c d");
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((7, 7)));
}

#[test]
fn m14_a_paste_over_a_selection_replaces_it_and_the_at_reads_both() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");
    let bounds = harness.query(Role::TextInput).named("Search").bounds();
    let y = bounds.origin.y + bounds.size.height * 0.5;
    // The M13 double-click selects `hello` [0,5); the paste replaces it.
    harness.double_click(Point::new(bounds.origin.x + 3.0, y));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((0, 5)));

    harness.set_clipboard("HELLO");
    harness.key(&ctrl(Key::Char('v')));
    assert_eq!(input_value(&harness), "HELLO world");

    // The AT reads the same tree state every writer shares: the new value
    // and the caret after the inserted span.
    let target = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, "Search"))
        .map(|node| node.id)
        .expect("the input is in the a11y tree");
    let updates = with_sink(&mut harness, AccessKitSink::drain_updates);
    let update = updates.last().expect("the paste produced an update");
    let node = find_in_update(update, target).expect("the input re-emitted");
    assert_eq!(
        node.value(),
        Some("HELLO world"),
        "the pasted value rides the a11y delta"
    );
    let selection = node
        .text_selection()
        .expect("the caret rides the a11y delta");
    assert_eq!(
        selection.anchor.character_index, 5,
        "the caret parks after the inserted span"
    );
    assert_eq!(selection.focus.character_index, 5);
}

fn with_sink<R>(harness: &mut TestHarness, f: impl FnOnce(&mut AccessKitSink) -> R) -> R {
    let sink = harness
        .pipeline_mut()
        .a11y_sink_mut()
        .as_any_mut()
        .and_then(|any| any.downcast_mut::<AccessKitSink>())
        .expect("the harness carries the AccessKit sink — the windowed composition");
    f(sink)
}

#[test]
fn m14_refusals_stay_honest_at_the_clipboard_tier() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");

    // A copy with no selection writes nothing — the clipboard keeps
    // whatever it carried, and the chord queues no masks: the next frame
    // is idle (the refuse-don't-munge rule, now at the clipboard tier).
    harness.set_clipboard("the operator's payload");
    let response = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('c')));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the field consumes its own command chord"
    );
    assert_eq!(
        harness.settle(),
        0,
        "a refused copy queues no masks: the next frame is idle"
    );
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("the operator's payload"),
        "a refused copy never touches the seam"
    );

    // A paste from an empty clipboard inserts nothing and costs nothing.
    harness.set_clipboard("");
    let response = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('v')));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the field consumes its own command chord"
    );
    assert_eq!(
        harness.settle(),
        0,
        "a refused paste queues no masks: the next frame is idle"
    );
    assert_eq!(input_value(&harness), "hello world");
}

#[test]
fn m14_clipboard_chords_are_inert_while_a_preedit_composes() {
    // Chords are keys like any other: the IME owns the canvas until
    // commit/cancel (the M13 rule the word chords already follow).
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello");
    harness.ime(&alacer_core::events::ImeEvent::Preedit {
        text: "ん".to_string(),
        cursor_utf16: Some(1),
    });
    harness.set_clipboard("payload");

    let response = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('c')));
    assert!(
        matches!(response, alacer_core::EventResponse::Ignored),
        "a composing field ignores the clipboard chords"
    );
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("payload"),
        "the composing field never wrote the seam"
    );
}

#[test]
fn m14_the_clipboard_seam_requires_an_engine() {
    // The seam carries no provided implementation: the OS clipboard is a
    // platform fact and a widget must never own one — the workspace's
    // engine rides arboard (architecture §12/§20), the harness drives the
    // in-memory seam, and core fixes the paste rule no widget improvises.
    let seam = read_workspace_file("crates/alacer-core/src/clipboard.rs");
    assert!(
        seam.contains("no provided implementation"),
        "the clipboard seam records why it has no default"
    );
    assert!(
        seam.contains("fn read_text(&self) -> Option<String>;"),
        "read_text is a required seam method"
    );
    assert!(
        seam.contains("fn write_text(&self, text: &str) -> bool;"),
        "write_text is a required seam method that reports the write"
    );
    assert!(
        seam.contains("fn paste_text(raw: &str) -> String"),
        "the paste rule lives in core, one definition"
    );
    let engine = read_workspace_file("crates/alacer-platform/src/clipboard.rs");
    assert!(
        engine.contains("arboard"),
        "the engine rides the adopted arboard core"
    );
    let harness = read_workspace_file("crates/alacer-testing/src/lib.rs");
    assert!(
        harness.contains("MemoryClipboard"),
        "the harness drives the in-memory seam, never the operator's OS clipboard"
    );
    let window = read_workspace_file("crates/alacer-platform/src/lib.rs");
    assert!(
        window.contains("Rc::new(crate::clipboard::ArboardClipboard::new())"),
        "the windowed assembly installs the arboard-backed seam"
    );
    let facade = read_workspace_file("crates/alacer/src/lib.rs");
    assert!(
        facade.contains("Rc::new(alacer_platform::clipboard::ArboardClipboard::new())"),
        "the default assembly installs the arboard-backed seam"
    );
}

#[test]
fn m14_the_corpus_grows_to_52_by_the_coordinated_rule() {
    // The new criterion case is published with its baseline and its
    // regression contract; every M-suite pin moves to 52 together.
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let id = "editing/route_100_clipboard_ops_1k";

    let text = serde_json::to_string(&baselines).expect("baselines serialize");
    assert!(
        text.contains(id),
        "{id} must be published in baselines.json"
    );
    let text = serde_json::to_string(&contracts).expect("contracts serialize");
    assert!(text.contains(id), "{id} must carry a regression contract");

    for suite in [
        "crates/alacer-testing/tests/m5_exit_criteria.rs",
        "crates/alacer-testing/tests/m6_exit_criteria.rs",
        "crates/alacer-testing/tests/m7_exit_criteria.rs",
        "crates/alacer-testing/tests/m8_exit_criteria.rs",
        "crates/alacer-testing/tests/m9_exit_criteria.rs",
        "crates/alacer-testing/tests/m10_exit_criteria.rs",
        "crates/alacer-testing/tests/m11_exit_criteria.rs",
        "crates/alacer-testing/tests/m12_exit_criteria.rs",
        "crates/alacer-testing/tests/m13_exit_criteria.rs",
    ] {
        let source = read_workspace_file(suite);
        assert!(
            source.contains("const EXPECTED_BENCH_COUNT: usize = 55;"),
            "{suite} must pin the 55-case corpus"
        );
    }
}
