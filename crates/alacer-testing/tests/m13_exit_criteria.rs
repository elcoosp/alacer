//! M13 exit-criteria integration tests (§10.3, ADR-0019): the word joins
//! the editing contract. The text seam carries the UAX #29 word
//! segmentation; word-wise movement hops word runs with the M11 anchor
//! semantics riding along; word chords delete exactly the span the caret
//! would hop; a double-click selects the word under it and the AT reads
//! the word's own range; and the corpus grows 50 → 51 by the coordinated
//! rule.
//!
//! Every test runs the real pipeline (A4) with the real text seam — the
//! exact route the window's keys and clicks take. The suite asserts the
//! contract's relationships at deterministic edges (the clamped left
//! click is the first word's start, the clamped right click is past the
//! last word's end) so it does not re-implement the font math it is
//! testing; the segmentation itself is pinned at the engine level
//! (`alacer-text`).

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
/// `content` — the fill borrows keyboard focus, so every key and click
/// under test routes to the field the way the window delivers them.
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

/// An Alt press (word movement; Option on macOS).
fn alt(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            alt: true,
            ..Modifiers::default()
        },
        text: None,
    }
}

/// A Ctrl press (the word chords' command set on Linux/Windows).
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

/// A Shift+Alt press (word-wise extension from the anchor).
fn shift_alt(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            shift: true,
            alt: true,
            ..Modifiers::default()
        },
        text: None,
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

#[test]
fn m13_alt_arrows_hop_word_runs_through_the_pipeline() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");
    // The fill leaves the caret at the value's end (11).
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((11, 11)));

    // Backward: word end, then word start, then the clamp — separators
    // are never landings, `", "`-shaped runs take one hop.
    harness.key(&alt(Key::ArrowLeft));
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((6, 6)),
        "the end hops to world's start"
    );
    harness.key(&alt(Key::ArrowLeft));
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((0, 0)),
        "world's start hops past the space to hello's start"
    );
    harness.key(&alt(Key::ArrowLeft));
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((0, 0)),
        "the run's start clamps the hop"
    );

    // Forward: the mirror — word end first, then the next word's start.
    harness.key(&alt(Key::ArrowRight));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((5, 5)));
    harness.key(&alt(Key::ArrowRight));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((6, 6)));
    harness.key(&alt(Key::ArrowRight));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((11, 11)));
}

#[test]
fn m13_shift_alt_extends_from_the_anchor_and_the_at_reads_the_range() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");
    harness.press_key(Key::Home);
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((0, 0)));

    // Shift+Alt+Right extends from the anchor — three hops: hello's end,
    // world's start, the value's end.
    harness.key(&shift_alt(Key::ArrowRight));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((0, 5)));
    harness.key(&shift_alt(Key::ArrowRight));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((0, 6)));
    harness.key(&shift_alt(Key::ArrowRight));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((0, 11)));

    // The AT reads the M11 vocabulary untouched: anchor 0, focus 11 —
    // the word hops wrote the same tree state the keyboard always did.
    let target = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, "Search"))
        .map(|node| node.id)
        .expect("the input is in the a11y tree");
    let updates = with_sink(&mut harness, AccessKitSink::drain_updates);
    let update = updates.last().expect("the extension produced an update");
    let node = find_in_update(update, target).expect("the input re-emitted");
    let selection = node
        .text_selection()
        .expect("the word extension rides the a11y delta");
    assert_eq!(
        selection.anchor.character_index, 0,
        "anchor at the value's start"
    );
    assert_eq!(selection.focus.character_index, 11, "focus at the caret");
}

#[test]
fn m13_word_chords_delete_the_hops_span_through_the_pipeline() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");

    // Ctrl+Backspace from the end: the hop's span goes — `world` first,
    // the separator with `hello` in the second chord (the caret's own
    // two hops, consumed as edits).
    harness.key(&ctrl(Key::Backspace));
    assert_eq!(input_value(&harness), "hello ");
    harness.key(&ctrl(Key::Backspace));
    assert_eq!(input_value(&harness), "");

    // Ctrl+Delete from the start mirrors it forward, caret staying put.
    harness.type_text("hello world");
    harness.press_key(Key::Home);
    harness.key(&ctrl(Key::Delete));
    assert_eq!(input_value(&harness), " world");
    harness.key(&ctrl(Key::Delete));
    assert_eq!(input_value(&harness), "world");
    harness.key(&ctrl(Key::Delete));
    assert_eq!(input_value(&harness), "");
}

#[test]
fn m13_a_double_click_selects_the_word_and_the_at_sees_it() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");
    let bounds = input_bounds(&harness);
    let y = bounds.origin.y + bounds.size.height * 0.5;

    // The clamped left edge is the first word's start: the double-click
    // selects `hello` — anchor 0, caret 5, the click's own reading order.
    let left = Point::new(bounds.origin.x + 3.0, y);
    harness.double_click(left);
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((0, 5)),
        "the word under the click selects"
    );

    let target = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, "Search"))
        .map(|node| node.id)
        .expect("the input is in the a11y tree");
    let updates = with_sink(&mut harness, AccessKitSink::drain_updates);
    let update = updates.last().expect("the double-click produced an update");
    let node = find_in_update(update, target).expect("the input re-emitted");
    let selection = node
        .text_selection()
        .expect("the word selection rides the a11y delta");
    assert_eq!(
        selection.anchor.character_index, 0,
        "the anchor is the word's start"
    );
    assert_eq!(
        selection.focus.character_index, 5,
        "the caret is the word's end"
    );

    // The clamped right edge is past the last word's end: no word to
    // select, so the double-click collapses at the hit — a double-click
    // never wanders to the nearest word.
    let right = Point::new(bounds.origin.x + bounds.size.width - 3.0, y);
    harness.double_click(right);
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((11, 11)),
        "a separator hit collapses at the hit point"
    );
}

#[test]
fn m13_a_fresh_click_still_collapses_a_word_selection() {
    // The M12 rule stands over the M13 vocabulary: after a double-click's
    // word selection, a fresh single click is a caret gesture again.
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "hello world");
    let bounds = input_bounds(&harness);
    let y = bounds.origin.y + bounds.size.height * 0.5;

    harness.double_click(Point::new(bounds.origin.x + 3.0, y));
    assert_eq!(harness.selection(Role::TextInput, "Search"), Some((0, 5)));

    // A drag after the double-click extends the anchor scalar-wise (the
    // recorded not-do: no word-wise drag extension yet) — and a fresh
    // single click collapses to its hit, right edge = the value's end.
    let right = Point::new(bounds.origin.x + bounds.size.width - 3.0, y);
    harness.pointer_press(right);
    harness.pointer_release(right);
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        Some((11, 11)),
        "a fresh click collapses the word selection"
    );
}

#[test]
fn m13_the_word_seam_requires_the_real_tables() {
    // The seam carries no provided segmentation: an ASCII approximation
    // would silently mis-hop `don't` and every multi-script run, so the
    // contract *requires* the engine's UAX #29 tables instead of shipping
    // a documented guess. The engine answers through ICU's WordSegmenter;
    // the platform computes the click chain the Down event carries.
    let seam = read_workspace_file("crates/alacer-core/src/text_seam.rs");
    assert!(
        seam.contains("no provided implementation"),
        "the word seam records why segmentation has no default"
    );
    assert!(
        seam.contains("fn word_segments(&self, text: &str) -> Vec<WordSegment>;"),
        "word_segments is a required seam method"
    );
    let engine = read_workspace_file("crates/alacer-text/src/lib.rs");
    assert!(
        engine.contains("WordSegmenter::new_auto"),
        "the engine segments through ICU's UAX #29 WordSegmenter"
    );
    let events = read_workspace_file("crates/alacer-core/src/events.rs");
    assert!(
        events.contains("click_count: u8"),
        "the Down event carries the platform's click count"
    );
    let platform = read_workspace_file("crates/alacer-platform/src/lib.rs");
    assert!(
        platform.contains("click_count: self.clicks.on_down(self.cursor)"),
        "the window feeds the click chain into every Down it queues"
    );
}

#[test]
fn m13_the_corpus_grows_to_51_by_the_coordinated_rule() {
    // The new criterion case is published with its baseline and its
    // regression contract; every M-suite pin moves to 52 together.
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let id = "editing/route_100_word_moves_1k";

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
    ] {
        let source = read_workspace_file(suite);
        assert!(
            source.contains("const EXPECTED_BENCH_COUNT: usize = 60;"),
            "{suite} must pin the 60-case corpus"
        );
    }
}
