//! M12 exit-criteria integration tests (§10.3, ADR-0018): the pointer
//! joins the editing contract. A click places the caret at the hit point,
//! a press-drag extends the selection from the anchor (the M11 anchor
//! becomes the drag anchor), a release ends the gesture and keeps the
//! range, the drag's selection rides the a11y delta in the drag's own
//! direction, a click during composition maps the committed text, and the
//! corpus grows 49 → 50 by the coordinated rule.
//!
//! Every test runs the real pipeline (A4) with the real text seam — the
//! exact route the window's pointer events take. Exact-offset mapping is
//! pinned at the widget level against a deterministic seam stub
//! (`alacer-widgets/src/text_input.rs`); the integration tests assert the
//! contract's relationships (edges, ordering, the AT direction) so the
//! suite does not re-implement the font math it is testing.

use alacer_a11y::{AccessKitSink, find_in_update};
use alacer_core::events::{ImeEvent, Key, KeyEvent, KeyState, Modifiers};
use alacer_core::geom::Point;
use alacer_core::widget::{A11yAction, Role, TextSelection};
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
/// `content` — the fill borrows keyboard focus, and every pointer gesture
/// under test re-arms the seam the way a mouse user drives it: press,
/// drag, release, each routed through pass 1 with the field's resolved
/// frame.
fn mount_input(harness: &mut TestHarness, content: &str) {
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();
    if !content.is_empty() {
        // Focus resolves in pass 3 of a tick: settle before typing, or the
        // keys route to no focused widget (the M11 suite's same rule).
        harness.pipeline_mut().request_focus(node);
        harness.settle();
        harness.type_text(content);
    }
}

/// The input's a11y bounds — where the clicks land (§11.5: tests target
/// through the tree, never widget internals).
fn input_bounds(harness: &TestHarness) -> alacer_core::geom::Rect {
    harness.query(Role::TextInput).named("Search").bounds()
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

/// A Ctrl+A press (the M11 select-all chord).
fn ctrl_a() -> KeyEvent {
    KeyEvent {
        key: Key::Char('a'),
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            ..Modifiers::default()
        },
        text: None,
    }
}

/// A Backspace press.
fn backspace() -> KeyEvent {
    KeyEvent {
        key: Key::Backspace,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    }
}

#[test]
fn m12_a_click_places_the_caret_at_the_hit_point() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");
    let bounds = input_bounds(&harness);
    let y = bounds.origin.y + bounds.size.height * 0.5;

    // Click near the field's right edge: beyond the run's width, so the
    // caret lands on the value's end — the clamped hit.
    let right = Point::new(bounds.origin.x + bounds.size.width - 3.0, y);
    harness.pointer_press(right);
    harness.pointer_release(right);
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((11, 11)),
        "a right-edge click parks the caret at the end"
    );

    // Click at the field's left edge: before the first glyph's advance, so
    // the caret lands on the value's start — the clamped hit again.
    let left = Point::new(bounds.origin.x + 3.0, y);
    harness.pointer_press(left);
    harness.pointer_release(left);
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((0, 0)),
        "a left-edge click parks the caret at the start"
    );
}

#[test]
fn m12_a_fresh_click_focuses_and_collapses_a_selection() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");

    // Select all, then click: the fresh click is a caret gesture — the
    // M11 range collapses to the hit point (degenerate), never lingers.
    harness.key(&ctrl_a());
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((0, 11)));
    let bounds = input_bounds(&harness);
    let middle = Point::new(
        bounds.origin.x + bounds.size.width * 0.5,
        bounds.origin.y + bounds.size.height * 0.5,
    );
    harness.pointer_press(middle);
    harness.pointer_release(middle);
    let selection = harness
        .selection(Role::TextInput, "Search")
        .expect("a click focuses the input, so the selection is reported");
    assert_eq!(
        selection.0, selection.1,
        "a fresh click collapses the selection to the caret"
    );
    assert!(
        (0..=11).contains(&selection.0),
        "the collapsed caret sits inside the value, got {selection:?}"
    );
}

#[test]
fn m12_a_drag_selects_the_range_and_backspace_consumes_it() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");
    let bounds = input_bounds(&harness);
    let y = bounds.origin.y + bounds.size.height * 0.5;

    // Press at the far left, drag to the far right, release: the anchor
    // stays where the press landed, the caret rides the drag — the whole
    // value selects. The M11 keyboard anchor is now the pointer's too.
    let left = Point::new(bounds.origin.x + 3.0, y);
    let right = Point::new(bounds.origin.x + bounds.size.width - 3.0, y);
    harness.drag(left, right);
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((0, 11)),
        "a full-width drag selects the whole value"
    );

    // The release ended the gesture but the range survives (a selection
    // is a state, not a gesture) — Backspace consumes exactly it.
    harness.key(&backspace());
    assert_eq!(input_value(&harness), "", "backspace consumed the range");
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((0, 0)),
        "the edit collapses the selection to the caret"
    );
}

#[test]
fn m12_the_drag_selection_rides_the_a11y_delta_both_ways() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");
    let bounds = input_bounds(&harness);
    let y = bounds.origin.y + bounds.size.height * 0.5;
    let left = Point::new(bounds.origin.x + 3.0, y);
    let right = Point::new(bounds.origin.x + bounds.size.width - 3.0, y);

    // Drag left → right: the anchor is where the press landed (0), the
    // focus is where the drag ended (11) — the AT's own direction.
    harness.drag(left, right);
    let target = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, "Search"))
        .map(|node| node.id)
        .expect("the input is in the a11y tree");
    let updates = with_sink(&mut harness, AccessKitSink::drain_updates);
    let update = updates.last().expect("the drag frame produced an update");
    let node = find_in_update(update, target).expect("the input re-emitted");
    let selection = node
        .text_selection()
        .expect("the drag's selection rides the a11y delta");
    assert_eq!(selection.anchor.character_index, 0, "anchor at the press");
    assert_eq!(selection.focus.character_index, 11, "focus at the drag end");

    // Drag right → left: the direction is the AT's information, not noise
    // to normalize away — anchor 11 (the press), focus 0 (the drag end).
    harness.drag(right, left);
    let updates = with_sink(&mut harness, AccessKitSink::drain_updates);
    let update = updates
        .last()
        .expect("the reversed drag produced an update");
    let node = find_in_update(update, target).expect("the input re-emitted");
    let selection = node
        .text_selection()
        .expect("the reversed drag's selection rides the delta too");
    assert_eq!(selection.anchor.character_index, 11);
    assert_eq!(selection.focus.character_index, 0);
}

#[test]
fn m12_a_click_during_composition_maps_the_committed_text() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");

    // Start a composition; the IME owns the canvas — the selection report
    // drops (M11), but the committed text under it stays pointer-addressable:
    // the preedit is overlay spliced at the caret, not a target.
    harness.ime(&ImeEvent::Preedit {
        text: "世界".into(),
        cursor_utf16: Some(2),
    });
    assert!(harness.selection(Role::TextInput, "Search").is_none());

    let bounds = input_bounds(&harness);
    let left = Point::new(
        bounds.origin.x + 3.0,
        bounds.origin.y + bounds.size.height * 0.5,
    );
    harness.pointer_press(left);
    harness.pointer_release(left);
    assert!(
        harness.selection(Role::TextInput, "Search").is_none(),
        "still composing: the report stays dropped through the click"
    );

    // The commit splices at the caret the click placed — offset 0, the
    // clamped left-edge hit — so the committed value grows in front.
    harness.ime(&ImeEvent::Commit {
        text: "世界".into(),
    });
    assert_eq!(input_value(&harness), "世界hello world");
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((2, 2)),
        "the caret sits after the committed units"
    );
}

#[test]
fn m12_moves_without_a_press_never_extend_the_selection() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");
    let bounds = input_bounds(&harness);
    let y = bounds.origin.y + bounds.size.height * 0.5;

    // A plain move (no press held) is hover, not selection: the field's
    // caret must not ride it — the drag arm is press-gated.
    let left = Point::new(bounds.origin.x + 3.0, y);
    harness.pointer_press(left);
    harness.pointer_release(left);
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((0, 0)));

    let right = Point::new(bounds.origin.x + bounds.size.width - 3.0, y);
    harness.pointer_move(right);
    harness.settle();
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((0, 0)),
        "an unpressed move never moves the caret"
    );
}

#[test]
fn m12_clicks_outside_the_field_never_focus_it() {
    let mut harness = TestHarness::new(400.0, 300.0);
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();

    // Click beside the input (inside the bare column, outside the field's
    // frame): no focus, no selection report — the frame the router hands
    // the widget is the contract's boundary.
    let bounds = input_bounds(&harness);
    let beside = Point::new(
        bounds.origin.x + bounds.size.width + 40.0,
        bounds.origin.y + bounds.size.height * 0.5,
    );
    harness.pointer_press(beside);
    harness.pointer_release(beside);
    harness.settle();
    assert!(
        harness.selection(Role::TextInput, "Search").is_none(),
        "an outside click never focuses the input"
    );
}

#[test]
fn m12_the_router_hands_the_resolved_frame_to_pointer_handlers() {
    // The core growth (ADR-0018): the event contract carries the node's
    // resolved layout frame, and the pass-1 router fills it from the arena
    // before the capture walk. Pinned by source — the wiring is a
    // constructor call CI cannot drive headlessly from outside the crate.
    let events = read_workspace_file("crates/alacer-core/src/events.rs");
    assert!(
        events.contains("pub const fn bounds(&self)"),
        "EventCtx exposes the resolved frame to pointer handlers"
    );
    assert!(
        events.contains("with_phase_and_bounds"),
        "EventCtx carries the frame from construction"
    );
    let passes = read_workspace_file("crates/alacer-core/src/passes.rs");
    assert!(
        passes.contains("with_phase_and_bounds"),
        "the pass-1 router feeds the arena's resolved bounds into the event context"
    );
}

#[test]
fn m12_the_a11y_set_text_selection_tier_agrees_with_the_pointer() {
    // The value tier (M11) and the pointer (M12) write the same anchor+
    // caret state — an AT that sets a range and a drag that selects one
    // are interchangeable to every consumer downstream.
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");
    assert!(harness.act(
        Role::TextInput,
        "Search",
        A11yAction::SetTextSelection(TextSelection::new(4, 8))
    ));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((4, 8)));

    // The pointer sees the AT's range as its own: a fresh click collapses
    // it, exactly like it collapses a keyboard selection.
    let bounds = input_bounds(&harness);
    let middle = Point::new(
        bounds.origin.x + bounds.size.width * 0.5,
        bounds.origin.y + bounds.size.height * 0.5,
    );
    harness.pointer_press(middle);
    harness.pointer_release(middle);
    let selection = harness
        .selection(Role::TextInput, "Search")
        .expect("focused");
    assert_eq!(
        selection.0, selection.1,
        "the click collapsed the AT's range"
    );
}

#[test]
fn m12_the_corpus_grows_to_50_by_the_coordinated_rule() {
    // The new criterion case is published with its baseline and its
    // regression contract; every M-suite pin moves to 50 together.
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let id = "editing/route_100_caret_clicks_1k";

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
    ] {
        let source = read_workspace_file(suite);
        assert!(
            source.contains("const EXPECTED_BENCH_COUNT: usize = 50;"),
            "{suite} must pin the 50-case corpus"
        );
    }
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
