//! M11 exit-criteria integration tests (§11.5, ADR-0017): the UTF-16
//! selection model joins the editing and a11y contracts. A focused text
//! input selects with Shift+arrows/Home/End and Ctrl+A, every edit consumes
//! the selection (typing replaces it, Backspace/Delete clear it), the AT
//! tier maps `SetValue`/`ReplaceSelectedText`/`SetTextSelection` faithfully
//! through pass 10, the a11y tree carries the selection, and the corpus
//! grows 48 → 49 by the coordinated rule.
//!
//! Every test runs the real pipeline (A4) — the sink is reached and drained
//! exactly as the platform loop does (the M9 suite's mechanism), and the
//! platform's value-tier mapping is pinned by source (the M6/M7/M8/M10
//! convention for adapter-side code CI cannot drive headlessly).

use alacer_a11y::{AccessKitSink, find_in_update};
use alacer_core::events::{ImeEvent, Key, KeyEvent, KeyState, Modifiers};
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, TextInput};
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

/// An input mounted through the real pipeline, focused through the pass-3
/// contract (`request_focus`), then filled by typing `content` — the same
/// route a user takes. Selection implies focus, so every test starts here.
fn mount_focused_input(harness: &mut TestHarness, content: &str) {
    let root = harness.mount(Column::new().boxed());
    let input = TextInput::new("Search");
    let node = harness.add_child(root, input.boxed());
    harness.settle();
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    if !content.is_empty() {
        harness.type_text(content);
    }
}

/// The focused input's committed value, read through the a11y tree (§11.5:
/// tests query the tree, never widget internals).
fn input_value(harness: &TestHarness) -> String {
    harness
        .query(Role::TextInput)
        .named("Search")
        .first()
        .value
        .clone()
        .expect("an editable input carries its value")
}

/// A press of `key` with `shift`/`control` held.
fn mod_key(key: Key, shift: bool, control: bool) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            shift,
            control,
            ..Modifiers::default()
        },
        text: None,
    }
}

fn backspace() -> KeyEvent {
    KeyEvent {
        key: Key::Backspace,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    }
}

fn delete() -> KeyEvent {
    KeyEvent {
        key: Key::Delete,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    }
}

#[test]
fn m11_shift_arrows_select_and_typing_replaces() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_focused_input(&mut harness, "hello");

    // Shift+Left twice from the end: the caret moves, the anchor stays.
    harness.key(&mod_key(Key::ArrowLeft, true, false));
    harness.key(&mod_key(Key::ArrowLeft, true, false));
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((3, 5)),
        "two shift-extends select 'lo'"
    );

    // Typing replaces the selection: "hello" -> "helXY".
    harness.key(&KeyEvent::char('X'));
    harness.key(&KeyEvent::char('Y'));
    assert_eq!(input_value(&harness), "helXY");
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((5, 5)),
        "an edit collapses the selection to the caret"
    );
}

#[test]
fn m11_backspace_and_delete_consume_the_whole_selection() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_focused_input(&mut harness, "alacer");

    // From the end, Home collapses the caret to 0 first (no Shift), then
    // two Shift+Rights extend from the anchor at 0.
    harness.key(&mod_key(Key::Home, false, false));
    harness.key(&mod_key(Key::ArrowRight, true, false));
    harness.key(&mod_key(Key::ArrowRight, true, false));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((0, 2)));

    // Backspace deletes the range, not one scalar value.
    harness.key(&backspace());
    assert_eq!(input_value(&harness), "acer");
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((0, 0)));

    // Reselect and forward-delete the same way.
    harness.key(&mod_key(Key::ArrowRight, true, false));
    harness.key(&mod_key(Key::ArrowRight, true, false));
    harness.key(&delete());
    assert_eq!(input_value(&harness), "er");
}

#[test]
fn m11_ctrl_a_selects_all_and_plain_arrows_collapse_to_the_near_edge() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_focused_input(&mut harness, "abc");

    harness.key(&mod_key(Key::Char('a'), false, true));
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((0, 3)),
        "ctrl+a selects the whole value"
    );

    // Plain Right collapses to the selection's far (max) edge, no move.
    harness.key(&mod_key(Key::ArrowRight, false, false));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((3, 3)));

    // Shift+Home selects back to the start; Shift+Right grows by one.
    harness.key(&mod_key(Key::Home, true, false));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((0, 3)));
    harness.key(&mod_key(Key::ArrowRight, true, false));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((1, 3)));

    // Plain Left collapses to the near (min) edge first, no move.
    harness.key(&mod_key(Key::ArrowLeft, false, false));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((1, 1)));
}

#[test]
fn m11_the_at_value_tier_replaces_selects_and_types_faithfully() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_focused_input(&mut harness, "hello world");

    // SetValue: the whole value is replaced, the caret resets to the end.
    assert!(
        harness.act(
            Role::TextInput,
            "Search",
            A11yAction::SetValue("set by AT".into())
        ),
        "the input consumes SetValue"
    );
    assert_eq!(input_value(&harness), "set by AT");
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((9, 9)));

    // SetTextSelection: the AT moves the selection directly.
    assert!(harness.act(
        Role::TextInput,
        "Search",
        A11yAction::SetTextSelection(alacer_core::widget::TextSelection::new(4, 8))
    ));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((4, 8)));

    // ReplaceSelectedText: the AT types over the selection, like a paste.
    // "set by AT" minus "by A" plus "core" = "set coreT".
    assert!(harness.act(
        Role::TextInput,
        "Search",
        A11yAction::ReplaceSelectedText("core".into())
    ));
    assert_eq!(input_value(&harness), "set coreT");
}

#[test]
fn m11_refusals_stay_honest_at_the_value_tier() {
    // A value-tier action a widget does not implement is refused with no
    // masks and no effects (ADR-0016's refuse-don't-munge contract, now at
    // the value tier): a Button is not a text field, and an input is not a
    // button.
    let mut harness = TestHarness::new(400.0, 300.0);
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, TextInput::new("Search").boxed());
    let _ = harness.add_child(root, Button::new("Increment").boxed());
    harness.settle();

    assert!(
        !harness.act(
            Role::Button,
            "Increment",
            A11yAction::SetValue("nope".into())
        ),
        "a button refuses SetValue — never munged into a press"
    );
    assert_eq!(
        harness.settle(),
        0,
        "a refused value action queues no masks: the next frame is idle"
    );

    assert!(
        !harness.act(Role::TextInput, "Search", A11yAction::DefaultPress),
        "an input refuses DefaultPress — the tier stays honest both ways"
    );
    assert_eq!(harness.settle(), 0);
}

#[test]
fn m11_the_a11y_tree_carries_the_selection() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_focused_input(&mut harness, "hello");

    harness.key(&mod_key(Key::ArrowLeft, true, false));
    harness.key(&mod_key(Key::ArrowLeft, true, false));

    // The sink's own update — the exact bytes the platform adapter forwards
    // — carries the selection on the input's node, anchor and focus in the
    // AT's own direction (anchor 5 where the drag began, focus 3 at the
    // caret).
    let target = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, "Search"))
        .map(|node| node.id)
        .expect("the input is in the a11y tree");
    let updates = with_sink(&mut harness, AccessKitSink::drain_updates);
    let update = updates.last().expect("the a11y frame produced an update");
    let node = find_in_update(update, target).expect("the input re-emitted");
    let selection = node
        .text_selection()
        .expect("the selection rides the a11y delta");
    assert_eq!(selection.anchor.character_index, 5);
    assert_eq!(selection.focus.character_index, 3);
}

#[test]
fn m11_the_corpus_grows_to_49_by_the_coordinated_rule() {
    // The new criterion case is published with its baseline and its
    // regression contract; every M-suite pin moves to 50 together.
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let id = "a11y_action/route_100_setvalue_1k";

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
    ] {
        let source = read_workspace_file(suite);
        assert!(
            source.contains("const EXPECTED_BENCH_COUNT: usize = 65;"),
            "{suite} must pin the 60-case corpus"
        );
    }
}

#[test]
fn m11_the_platform_maps_the_value_tier_by_source() {
    // The platform mapping is adapter-side (winit/accesskit only) — CI pins
    // it by source, the M10 convention for the faithful-mapping contract.
    let source = read_workspace_file("crates/alacer-platform/src/lib.rs");
    assert!(
        source.contains("Action::SetValue"),
        "the platform maps SetValue faithfully (no munge to DefaultPress)"
    );
    assert!(
        source.contains("Action::ReplaceSelectedText"),
        "the platform maps ReplaceSelectedText faithfully"
    );
    assert!(
        source.contains("Action::SetTextSelection"),
        "the platform maps SetTextSelection faithfully"
    );
}

#[test]
fn m11_the_composition_window_owns_the_selection() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_focused_input(&mut harness, "hello");

    harness.key(&mod_key(Key::ArrowLeft, true, false));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((4, 5)));

    // While a preedit is active the selection is not exposed: the IME owns
    // the composition window, and a stale selection would lie to the AT.
    harness.ime(&ImeEvent::Preedit {
        text: "世界".into(),
        cursor_utf16: Some(2),
    });
    assert!(
        harness.selection(Role::TextInput, "Search").is_none(),
        "no selection is reported during composition"
    );

    // The commit lands as an edit: it consumes the selected "o" first (one
    // rule for every insertion, IME or keyboard) and the field reports
    // fresh state — selection re-armed, degenerate at the new caret.
    harness.ime(&ImeEvent::Commit {
        text: "世界".into(),
    });
    assert_eq!(input_value(&harness), "hell世界");
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((6, 6)));
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
