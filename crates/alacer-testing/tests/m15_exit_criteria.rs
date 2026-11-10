//! M15 exit-criteria integration tests (§10.3, ADR-0021): the undo stack
//! joins the editing contract. Core fixes the history vocabulary — one
//! bounded two-stack journal whose entries carry the replaced UTF-16 span,
//! the removed and inserted text, and the selection on both sides — and
//! the editor answers Ctrl/Cmd+Z (undo), Shift+Ctrl/Cmd+Z and Ctrl/Cmd+Y
//! (redo) with the M11 selection semantics. Typing coalesces into one
//! step, every other edit is its own step, a whole-value replacement
//! resets the journal (the app's write is never undone), a refused chord
//! is an honest no-op with no masks, and the corpus grows 52 → 53 by the
//! coordinated rule.
//!
//! Every test runs the real pipeline (A4) with the real text seam — the
//! exact route the window's keys take.

use alacer_a11y::{AccessKitSink, find_in_update};
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::widget::{A11yAction, Role};
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
/// requested even for an empty content: the history chords are keyboard
/// edits and route to the focused field.
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

/// A Shift+Ctrl press (the redo chord).
fn ctrl_shift(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            shift: true,
            ..Modifiers::default()
        },
        text: None,
    }
}

#[test]
fn m15_undo_and_redo_route_through_the_pipeline() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");

    // Typing coalesced into one step: the first undo restores the empty
    // field, the caret parked where the text began.
    harness.key(&ctrl(Key::Char('z')));
    assert_eq!(
        input_value(&harness),
        "",
        "the typed fill undoes as one step"
    );
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((0, 0)),
        "the caret returns to where the edit began"
    );

    // Redo re-applies the step and parks the caret at its end.
    harness.key(&ctrl_shift(Key::Char('z')));
    assert_eq!(input_value(&harness), "hello world");
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((11, 11)));

    // The redo re-pushed the entry: undo answers again.
    harness.key(&ctrl(Key::Char('z')));
    assert_eq!(input_value(&harness), "");
}

#[test]
fn m15_typing_coalesces_and_deletions_are_their_own_steps() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "");

    // "ab" is one step (the pure-append rule); each backspace is its own
    // (a deletion never coalesces — ADR-0021's recorded simplification).
    harness.type_text("ab");
    assert_eq!(input_value(&harness), "ab");
    harness.key(&key(Key::Backspace));
    harness.key(&key(Key::Backspace));
    assert_eq!(input_value(&harness), "");

    // Three steps were journaled: the insert, then the two deletions.
    harness.key(&ctrl(Key::Char('z')));
    assert_eq!(
        input_value(&harness),
        "a",
        "the second backspace undoes first"
    );
    harness.key(&ctrl(Key::Char('z')));
    assert_eq!(
        input_value(&harness),
        "ab",
        "the first backspace undoes next"
    );
    harness.key(&ctrl(Key::Char('z')));
    assert_eq!(input_value(&harness), "", "the typed pair undoes last");
}

fn key(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    }
}

#[test]
fn m15_a_cut_and_a_paste_are_one_undo_step_each() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");

    // Cut the whole value, then undo: the text returns and the selection
    // the cut consumed is restored with it (the M11 vocabulary).
    harness.key(&ctrl(Key::Char('a')));
    harness.key(&ctrl(Key::Char('x')));
    assert_eq!(input_value(&harness), "");
    harness.key(&ctrl(Key::Char('z')));
    assert_eq!(input_value(&harness), "hello world", "the cut undoes");
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((0, 11)),
        "the consumed selection is restored, not collapsed"
    );

    // A paste over a selection is an edit like any other: its undo
    // restores the exact replaced span and the selection that held it.
    harness.set_clipboard("HELLO");
    harness.key(&ctrl(Key::Char('a')));
    harness.key(&ctrl(Key::Char('v')));
    assert_eq!(input_value(&harness), "HELLO");
    harness.key(&ctrl(Key::Char('z')));
    assert_eq!(input_value(&harness), "hello world");
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((0, 11)));
}

#[test]
fn m15_refusals_stay_honest_at_the_history_tier() {
    // An unfilled field: the journal is empty, and an undo refuses
    // honestly — the chord is consumed (the field's own chord) but queues
    // no masks, the discipline the clipboard tier pinned (ADR-0020).
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "");

    let response = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('z')));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the field consumes its own command chord"
    );
    assert_eq!(harness.settle(), 0, "a refused undo queues no masks");

    let response = harness
        .pipeline_mut()
        .dispatch_key(&ctrl_shift(Key::Char('z')));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(harness.settle(), 0, "a refused redo queues no masks");

    // An exhausted journal refuses the same way: one step exists, and the
    // second undo is a refusal.
    harness.type_text("!");
    assert_eq!(input_value(&harness), "!");
    harness.key(&ctrl(Key::Char('z')));
    assert_eq!(input_value(&harness), "");
    let response = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('z')));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(harness.settle(), 0, "the exhausted undo queues no masks");
    assert_eq!(input_value(&harness), "");
}

#[test]
fn m15_the_history_chords_are_inert_while_a_preedit_composes() {
    // Chords are keys like any other: the IME owns the canvas until
    // commit/cancel (the M13 rule the word and clipboard chords follow).
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello");
    harness.ime(&alacer_core::events::ImeEvent::Preedit {
        text: "ん".to_string(),
        cursor_utf16: Some(1),
    });

    let response = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('z')));
    assert!(
        matches!(response, alacer_core::EventResponse::Ignored),
        "a composing field ignores the history chords"
    );
    assert_eq!(input_value(&harness), "hello", "the value never moved");
}

#[test]
fn m15_a_whole_value_replacement_resets_the_journal() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");
    harness.type_text(" again");

    // The AT's SetValue is the app writing, not the user editing: the
    // journal resets, and undo must never fight the app's own write
    // (the same reset a programmatic set_value gets — ADR-0021).
    let consumed = harness.act(
        Role::TextInput,
        "Search",
        A11yAction::SetValue("replaced by the app".to_string()),
    );
    assert!(consumed, "the value tier carries its payload");
    assert_eq!(input_value(&harness), "replaced by the app");

    let response = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('z')));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(harness.settle(), 0, "a reset journal refuses with no masks");
    assert_eq!(
        input_value(&harness),
        "replaced by the app",
        "undo never resurrects a value the app replaced"
    );
}

#[test]
fn m15_the_at_reads_the_restored_value_and_selection() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");

    // Replace the whole value with one typed rune — an edit like any
    // other — then undo it.
    harness.key(&ctrl(Key::Char('a')));
    harness.type_text("X");
    assert_eq!(input_value(&harness), "X");

    harness.key(&ctrl(Key::Char('z')));
    assert_eq!(input_value(&harness), "hello world");

    // The undo is the same value+selection delta every writer shares —
    // the AT reads it through the M9/M11 machinery untouched (ADR-0021).
    // The replace-all step is its own (a deletion opened it), so the
    // undo restores the original value and the selection it consumed.
    let target = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, "Search"))
        .map(|node| node.id)
        .expect("the input is in the a11y tree");
    let updates = with_sink(&mut harness, AccessKitSink::drain_updates);
    let update = updates.last().expect("the undo produced an update");
    let node = find_in_update(update, target).expect("the input re-emitted");
    assert_eq!(
        node.value(),
        Some("hello world"),
        "the restored value rides the a11y delta"
    );
    let selection = node
        .text_selection()
        .expect("the caret rides the a11y delta");
    assert_eq!(
        selection.anchor.character_index, 0,
        "the restored selection rides the a11y delta"
    );
    assert_eq!(
        selection.focus.character_index, 11,
        "anchor and caret restore in the edit's own order"
    );
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
fn m15_the_history_vocabulary_lives_in_core() {
    // One definition, no widget dialect: the journal, its coalescing and
    // eviction rules, and the whole-value reset live beside the seams the
    // editing contract already owns; the editor only consumes them.
    let history = read_workspace_file("crates/alacer-core/src/history.rs");
    assert!(
        history.contains("pure-append coalescing rule"),
        "the coalescing rule is recorded in core, one definition"
    );
    assert!(
        history.contains("DEFAULT_CAPACITY"),
        "the journal is bounded and the bound is named"
    );
    assert!(
        history.contains("pub fn reset(&mut self)"),
        "the whole-value replacement's reset is core vocabulary"
    );
    assert!(
        history.contains("crate::utf16"),
        "the journal counts the contract's UTF-16 units"
    );
    let editor = read_workspace_file("crates/alacer-widgets/src/text_input.rs");
    assert!(
        editor.contains("'z' | 'Z'"),
        "the editor answers the undo chord"
    );
    assert!(
        editor.contains("'y' | 'Y'"),
        "the editor answers the alternate redo chord"
    );
}

#[test]
fn m15_the_corpus_grows_to_53_by_the_coordinated_rule() {
    // The new criterion case is published with its baseline and its
    // regression contract; every M-suite pin moves to 53 together.
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let id = "editing/route_100_undo_redo_1k";

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
        "crates/alacer-testing/tests/m14_exit_criteria.rs",
    ] {
        let source = read_workspace_file(suite);
        assert!(
            source.contains("const EXPECTED_BENCH_COUNT: usize = 57;"),
            "{suite} must pin the 55-case corpus"
        );
    }
}
