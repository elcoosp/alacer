//! The `.alc` record/replay round-trip (§13.4, ADR-0012): user interactions
//! recorded to `.alc` files and replayed in `cargo test` — bug reports that
//! are also regression tests.
//!
//! The m6 exit suite replays the committed todo recording; this suite
//! proves the *round-trip* on a canon trio the todo suite does not drive:
//! a focused `TextInput`, a `Button` whose press drives a bound `Label` —
//! record, serialize to the versioned `.alc` JSON, parse back, replay on a
//! fresh harness, and require the final UI state to match the original
//! session exactly. A recording that cannot survive save → load → replay
//! is not a bug report; it is prose.

use alacer_core::flux::RenderBinding;
use alacer_core::widget::Role;
use alacer_testing::Recording;
use alacer_testing::Session;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, TextInput};

/// The viewport both sessions run at (replay refuses a mismatched one).
const VIEWPORT: (f32, f32) = (320.0, 240.0);

/// Mounts the stateful trio the session drives: a `TextInput` (focused),
/// a `Button` that increments a flux signal, and a `Label` bound to it —
/// the smallest canon app whose final state names both interactions.
fn mount_counter_app(harness: &mut TestHarness) -> alacer_core::arena::NodeId {
    use reactive_graph::traits::{GetUntracked as _, Set as _};

    let root = harness.mount(Column::new().gap(8.0).padding(12.0).boxed());
    let field = harness.add_child(root, TextInput::new("Search").boxed());
    let count = harness.pipeline().flux().signal(0_i32);
    let label_binding = RenderBinding::new("Clicked 0 times".to_string());
    let label = harness.add_child(root, Label::bound(label_binding.clone()).boxed());
    let _button = harness.add_child(
        root,
        Button::new("Increment")
            .on_click(move || {
                // One write batch per click: the ADR-6 contract at the seam.
                alacer_core::flux::Flux::batch_of(|| {
                    let current = count.try_get_untracked().unwrap_or_default();
                    count.set(current + 1);
                });
            })
            .boxed(),
    );
    harness.pipeline_mut().flux_mut().bind_value(
        count,
        label,
        &label_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Clicked {n} times"),
    );
    // Focus resolves in pass 3 of a tick: settle before the keys route, or
    // the typed steps land on no focused widget (the editing contract's
    // rule) — the session and the replay ride the same discipline.
    harness.pipeline_mut().request_focus(field);
    harness.settle();
    field
}

/// The live session's end state, read through the a11y tree: the committed
/// text and the counter the press drove.
fn assert_counter_state(harness: &TestHarness, text: &str, clicks: usize) {
    assert_eq!(
        harness
            .query(Role::TextInput)
            .named("Search")
            .first()
            .value
            .as_deref(),
        Some(text),
        "the field's committed text does not match the session"
    );
    assert!(
        harness
            .query(Role::Label)
            .named(format!("Clicked {clicks} times"))
            .exists(),
        "the bound label does not reflect {clicks} recorded presses"
    );
}

#[test]
fn a_recorded_session_round_trips_through_an_alc_string_and_replays() {
    // (1) Mount: the focused field, the button, the bound label.
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let _field = mount_counter_app(&mut harness);

    // (2) Record: type "hello" into the focused field, press the button.
    let mut session = Session::begin(&mut harness);
    session.type_text("hello");
    session.press(Role::Button, "Increment");
    let recording = session.finish();

    // The live session's own answer first: the recording describes reality.
    assert_eq!(recording.steps().len(), 2, "both interactions recorded");
    assert_counter_state(&harness, "hello", 1);

    // (3) Serialize to the `.alc` payload — versioned, semantic, and
    // deterministic: the same recording always writes the same bytes.
    let alc = serde_json::to_string(&recording).expect("the .alc payload serializes");
    assert_eq!(
        serde_json::to_string(&recording).expect("the re-serialization agrees"),
        alc,
        "the .alc payload is not deterministic"
    );
    assert!(
        alc.contains(r#""schema":1"#),
        "the format version rides the payload: {alc}"
    );
    assert!(
        alc.contains(r#""kind":"type_text""#) && alc.contains(r#""kind":"press""#),
        "the semantic steps ride the payload: {alc}"
    );

    // (4) Parse the payload back into a `Recording` — exact, schema-checked.
    let parsed: Recording = serde_json::from_str(&alc).expect("the .alc payload parses back");
    assert_eq!(parsed, recording, "the string round-trip is not exact");
    assert_eq!(parsed.schema, alacer_testing::RECORDING_SCHEMA);

    // (5) Replay the parsed steps onto a fresh harness: same canon, fresh
    // state, the real input pipeline (each step settles).
    let mut replay = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let _replayed_field = mount_counter_app(&mut replay);
    parsed.replay(&mut replay);

    // (6) The replayed final state matches the original session's — the
    // recording *is* the regression test, end to end.
    assert_counter_state(&replay, "hello", 1);
}

#[test]
fn a_recorded_session_survives_the_alc_file_round_trip() {
    // The product sentence (§13.4): the recording is *saved as a file* and
    // loaded back anywhere. Same trio, same session, one temp `.alc` file.
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let _field = mount_counter_app(&mut harness);
    let mut session = Session::begin(&mut harness);
    session.type_text("hello");
    session.press(Role::Button, "Increment");
    let recording = session.finish();
    assert_counter_state(&harness, "hello", 1);

    let path = std::env::temp_dir().join("alacer_counter_roundtrip.alc");
    recording.save(&path).expect("the .alc file saves");
    let loaded = Recording::load(&path).expect("the .alc file loads");
    let _ = std::fs::remove_file(&path);
    assert_eq!(loaded, recording, "the file round-trip is not exact");
    assert_eq!(loaded.viewport, [VIEWPORT.0, VIEWPORT.1]);

    let mut replay = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let _replayed_field = mount_counter_app(&mut replay);
    loaded.replay(&mut replay);
    assert_counter_state(&replay, "hello", 1);
}

#[test]
fn a_replayed_recording_against_a_drifted_ui_fails_loudly() {
    // The regression-test sentence, other half: a recording that no longer
    // describes reality *fails*, it does not silently pass. The replay
    // harness renames the button — the recorded press has no target — and
    // the replay must panic (the harness press's own refusal).
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let _field = mount_counter_app(&mut harness);
    let mut session = Session::begin(&mut harness);
    session.press(Role::Button, "Increment");
    let recording = session.finish();
    assert_eq!(recording.steps().len(), 1);

    let mut drifted = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let root = drifted.mount(Column::new().gap(8.0).padding(12.0).boxed());
    let _renamed = drifted.add_child(root, Button::new("Renamed").boxed());
    drifted.settle();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        recording.replay(&mut drifted);
    }));
    assert!(
        result.is_err(),
        "a drifted UI replayed a recorded press that no longer has a target"
    );
}
