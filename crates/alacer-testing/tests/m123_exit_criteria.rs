//! M123 exit-criteria integration tests (§10.3, ADR-0127): the language
//! tier — the language the text is written in, said by the tree. Through
//! M122 the snapshot said what a control *does* (the description,
//! ADR-0099), what an empty field *hints* (ADR-0100), the field's line
//! capacity (ADR-0126), the form facts, the collection facts, the
//! declared live region, the chord the control is invoked with — and the
//! word every multilingual vocabulary speaks about *what language the
//! text speaks* ("this label is French, that one Japanese") had no tier:
//! the AT announced the text in whatever voice the platform guessed —
//! the declared-but-unreported gap (the M94 pattern), cashed at the
//! language tier: the word grows beside the text it names —
//! [`alacer_core::Widget::language`] beside
//! [`alacer_widgets::Label::language`], AccessKit's `Language` property
//! (the BCP 47 tag the platform AT voices the text with) the sink maps
//! in the same snapshot every other property rides.
//!
//! The tier prices the word and nothing else: a worded label's doors
//! answer exactly as they always did — the word names the language, it
//! walls nothing, the label's read routes the widget's own (a tree that
//! refused on the word's behalf would price a contract it does not own
//! — the word says the text's language, it never translates the text).
//! `None` is the honest absence (the ADR-0087 presence rule at the
//! tier): a label that declared no language announces none — the text
//! the AT voiced before the tier, never a guessed tag.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the word's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Label};

/// The suite's corpus pin: the coordinated rule's count at the M123 cut —
/// the language route's honest pair (the tier's own: bare beside worded,
/// the M109/M119 pair shape) joining the multiline route's case (M122),
/// the shortcut route's case (M121), the live route's case (M120) and
/// every tier's before them on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 157;

/// The suite's own read doors into the published corpus: the language
/// route's pair — the 1,000-node label arena bare beside the same arena
/// carrying the constructor's language word (the tier's own honest pair,
/// the M119 shape; the M95 pricing rule — the word is a label word, so
/// the honest comparison is labels against labels).
const LABELS_BARE_CASE: &str = "language/snapshot_1k_labels_bare";
const LABELS_LANGUAGE_CASE: &str = "language/snapshot_1k_labels_language";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's language door: the mutate seam the app's code rides — the
/// tag moved, `PAINT | A11Y` requested on the control, the frames the
/// re-announcement rides settled before the read.
fn set_label_language(
    harness: &mut TestHarness,
    node_id: alacer_core::NodeId,
    language: Option<&str>,
) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(label) = guard.widget_mut().as_any_mut().downcast_mut::<Label>() {
        label.set_language(language.map(str::to_owned));
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m123_the_language_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The worded label, mounted through the constructor word: the fact
    // the tree says from the first frame — the tag the AT voices the
    // text with — beside the plain label and the plain button, the
    // honest absences the tier keeps honest.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(root, Label::new("Bienvenue").language("fr-FR").boxed());
    let _ = harness.add_child(root, Label::new("Welcome").boxed());
    let _ = harness.add_child(root, Button::new("Continue").boxed());
    harness.settle();

    let french = node(&harness, Role::Label, "Bienvenue").expect("the worded label rides");
    assert_eq!(
        french.language.as_deref(),
        Some("fr-FR"),
        "the constructor's word is the tree's truth — the tag said beside the text"
    );
    let plain = node(&harness, Role::Label, "Welcome").expect("the plain label rides");
    assert_eq!(
        plain.language, None,
        "a label that declared no language announces none — the honest absence"
    );
    let button = node(&harness, Role::Button, "Continue").expect("the plain button rides");
    assert_eq!(
        button.language, None,
        "the word is a text word: the plain roles answer nothing — the inherited default"
    );
    // The word prices nothing but the word: the worded label's doors
    // answer exactly as they always did — the tag names a fact, it
    // walls nothing, the text rides the widget's own.
    assert_eq!(
        french.label, "Bienvenue",
        "the text the tree carries is the text the app wrote — the word never translates"
    );
    harness.assert_all_named();
}

#[test]
fn m123_the_language_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let greeting = Label::new("Hello");
    let greeting_id = harness.add_child(root, greeting.boxed());
    harness.settle();

    // The language door: the app's tag moved, the re-announcement the
    // mutate seam's request paid for — and the honest absence restored
    // on the clear (the tag withdrawn, the label the widget it always
    // was).
    set_label_language(&mut harness, greeting_id, Some("ja-JP"));
    let worded = node(&harness, Role::Label, "Hello").expect("the label survives");
    assert_eq!(
        worded.language.as_deref(),
        Some("ja-JP"),
        "the app's door moves the word the tree says"
    );
    set_label_language(&mut harness, greeting_id, None);
    let withdrawn = node(&harness, Role::Label, "Hello").expect("the label survives");
    assert_eq!(
        withdrawn.language, None,
        "the same door carries the word away — the honest absence restored"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(greeting_id);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Label>()
        .expect("the label survives")
        .set_language(None);
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m123_the_corpus_grew_by_the_coordinated_rule() {
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
    let contracts = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("benchmarks/contracts.json"))
            .expect("the contract set is readable"),
    )
    .expect("the contract set is valid JSON");

    // The published corpus carries the language route's pair: the
    // 1,000-node label arena bare beside the same arena carrying the
    // constructor's word — the pair the tier's own budget prices (the
    // M119 shape, labels against labels).
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(LABELS_BARE_CASE) && benches.contains_key(LABELS_LANGUAGE_CASE),
        "the language route's pair is published beside itself ({LABELS_BARE_CASE}, {LABELS_LANGUAGE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M123 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [LABELS_BARE_CASE, LABELS_LANGUAGE_CASE] {
        let gate = gates
            .iter()
            .find(|gate| gate["baseline"].as_str() == Some(case))
            .unwrap_or_else(|| panic!("the case carries a regression gate ({case})"));
        assert_eq!(
            gate["scope"].as_str(),
            Some("same-host"),
            "the gate is same-host scoped like the tier's own"
        );
    }
    let budgets = contracts["budgets"]
        .as_array()
        .expect("the contract set carries budgets");
    let budget = budgets
        .iter()
        .find(|budget| budget["id"].as_str() == Some("language_snapshot_overhead"))
        .expect("the language budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(LABELS_LANGUAGE_CASE),
        "the budget prices the worded route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(LABELS_BARE_CASE),
        "the budget prices it against the same label arena, bare"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the word must not outprice the snapshot it rides"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M123 cut"
    );
}
