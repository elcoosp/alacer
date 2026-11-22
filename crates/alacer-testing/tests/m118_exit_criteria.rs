//! M118 exit-criteria integration tests (§10.3, ADR-0122): the
//! routing-outcome doors — the harness's keyboard and IME doors answer
//! what the pipeline answered, as data. Through M117 the harness's
//! typing doors dispatched and settled — and dropped the verdict on
//! the floor (`let _ = self.pipeline.dispatch_key(event)`): a test
//! that wanted to know whether the field took the key, the wall
//! refused it, or the funnel fell through had to smuggle the answer
//! out through effects — the value that moved, the trail that streamed
//! — the one routing question the harness could not ask directly. The
//! pipeline always answered as data (the M27 rule at the dispatch
//! seam): [`alacer_core::widget::EventResponse`] — `Handled` the
//! widget (or the framework's own fallback) consumed the key,
//! `Ignored` it bubbled on. The doors now carry that answer to the
//! test: [`alacer_testing::TestHarness::key`],
//! [`alacer_testing::TestHarness::press_key`] and
//! [`alacer_testing::TestHarness::ime`] answer the pipeline's own
//! verdict; [`alacer_testing::TestHarness::type_text`] answers one
//! verdict per dispatched character, in order.
//!
//! The doors price nothing but the answer: the dispatch they rode is
//! byte-for-byte the dispatch they always ran (the settle still
//! follows, the `# Panics` contract stands) — the verdict is the
//! return value the funnel already computed, never a second route.
//! Nothing in the frame pays for the tree's truth here: the corpus
//! keeps its count at the M117 cut, the door prices no frame work.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the value the typing landed is read from the
//! snapshot the currency paid for.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{EventResponse, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M118
/// cut — the count the M117 suite pinned, kept: the routing-outcome
/// doors price no frame work (the verdict is the return value the
/// dispatch already computed), so no case joins and no case leaves.
const EXPECTED_BENCH_COUNT: usize = 151;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// A field mounted through the real pipeline, focused and settled —
/// the typing door's own mount (the m58 shape).
fn mount_field(name: &str, read_only: bool) -> TestHarness {
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let mut field = TextInput::new(name);
    if read_only {
        field = field.read_only();
    }
    let node_id = harness.add_child(root, field.boxed());
    harness.settle();
    harness.pipeline_mut().request_focus(node_id);
    harness.settle();
    harness
}

#[test]
fn m118_the_doors_answer_the_routing_the_pipeline_answered() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(root, Button::new("Save").boxed());
    let _ = harness.add_child(root, Button::new("Cancel").boxed());
    harness.settle();

    // A stray key with nothing focused: the funnel falls through
    // honestly — no widget heard it, no fallback took it, the verdict
    // is the `Ignored` the dispatch computed.
    let stray = harness.press_key(alacer_core::events::Key::Char('x'));
    assert_eq!(
        stray,
        EventResponse::Ignored,
        "a key no widget holds and no fallback takes answers the routing as data — the honest fall-through"
    );

    // The framework's own fallback answers as the framework's: the
    // button ignored Tab, the focus cycling took it, the verdict is
    // `Handled` — the same answer dispatch_key computed, now carried.
    let tab = harness.press_key(alacer_core::events::Key::Tab);
    assert_eq!(
        tab,
        EventResponse::Handled,
        "the cycling that took the ignored Tab answers Handled — the fallback's verdict carried"
    );
    // The second walk answers again: one Tab, one verdict, every time.
    let next = harness.press_key(alacer_core::events::Key::Tab);
    assert_eq!(
        next,
        EventResponse::Handled,
        "the walk to the next candidate answers its own Handled"
    );
    harness.assert_all_named();
}

#[test]
fn m118_the_typing_door_answers_per_character() {
    let _guard = alacer_core::flux::batch_test_lock();

    // The open field: every character lands, every verdict `Handled`.
    let mut harness = mount_field("field", false);
    let outcomes = harness.type_text("ab");
    assert_eq!(
        outcomes,
        vec![EventResponse::Handled, EventResponse::Handled],
        "the typing door answers one verdict per dispatched character, in order"
    );
    let field = node(&harness, Role::TextInput, "field").expect("the field rides");
    assert_eq!(
        field.value.as_deref(),
        Some("ab"),
        "the verdicts name what the value shows — the characters the Handled verdicts carried"
    );

    // The plain button: a widget that takes no characters, the
    // fallbacks taking none either — both verdicts `Ignored`, the
    // contrast the door exists to pin.
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let save = harness.add_child(root, Button::new("Save").boxed());
    harness.settle();
    harness.pipeline_mut().request_focus(save);
    harness.settle();
    let outcomes = harness.type_text("ab");
    assert_eq!(
        outcomes,
        vec![EventResponse::Ignored, EventResponse::Ignored],
        "a focused button that takes no characters answers the fall-through per character"
    );

    // The read-only wall (M105, ADR-0109): the field heard its own
    // keys, the wall answered them — the verdicts `Handled` (the
    // consumed keys the wall's honesty pins), the value untouched.
    let mut harness = mount_field("wall", true);
    let outcomes = harness.type_text("ab");
    assert_eq!(
        outcomes,
        vec![EventResponse::Handled, EventResponse::Handled],
        "a refused edit is a consumed key — the wall answered, the verdict names the consumption"
    );
    let wall = node(&harness, Role::TextInput, "wall").expect("the wall rides");
    assert!(
        wall.value.as_deref().unwrap_or("").is_empty(),
        "the wall consumed the keys and landed nothing — the refusal costs an idle frame, never a munged edit"
    );
    harness.assert_all_named();
}

#[test]
fn m118_the_ime_door_answers_as_data() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = mount_field("field", false);

    // The committed composition: the insertion landed, the verdict
    // `Handled` — the same answer dispatch_ime computed, now carried.
    let commit = harness.ime(&alacer_core::events::ImeEvent::Commit {
        text: "你好".to_owned(),
    });
    assert_eq!(
        commit,
        EventResponse::Handled,
        "the committed composition the field inserted answers Handled — the IME route's verdict carried"
    );
    let field = node(&harness, Role::TextInput, "field").expect("the field rides");
    assert_eq!(
        field.value.as_deref(),
        Some("你好"),
        "the verdict names what the value shows — the committed text the Handled verdict carried"
    );
    harness.assert_all_named();
}

#[test]
fn m118_the_corpus_pin_holds_at_the_cut() {
    let _guard = alacer_core::flux::batch_test_lock();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let baselines = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("benchmarks/baselines/baselines.json"))
            .expect("the published baseline is readable"),
    )
    .expect("the published baseline is valid JSON");

    // The corpus keeps its count: the routing-outcome doors price no
    // frame work (the verdict is the return value the dispatch already
    // computed), so no case joins and no case leaves — the pin the
    // M117 suite set re-agrees unchanged.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin holds at the M118 cut — the doors price nothing"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M118 cut"
    );
}
