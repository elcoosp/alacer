//! M107 exit-criteria integration tests (§10.3, ADR-0111): the
//! invalid tier — the declared word behind the form's rejection.
//! Since M95 (ADR-0099) the tree says what a control *does* (the
//! description), since M96 what an empty field *hints* (the
//! placeholder), since M97 the span a value rides, since M106 the
//! form's ask ("this one must be filled") — and the word every form
//! vocabulary speaks when the value lands wrong ("this one is not
//! valid as given") had no tree tier: the AT announced the ask, the
//! hint, the span — and never the rejection. The word grows the
//! invalid tier beside the required one's —
//! [`alacer_core::Widget::invalid`] beside
//! [`alacer_widgets::TextInput::invalid`],
//! [`alacer_widgets::Slider::invalid`] and
//! [`alacer_widgets::Checkbox::invalid`] — AccessKit's `Invalid`
//! property (ARIA's `aria-invalid`) the sink maps in the same snapshot
//! every other property rides, the kind (`true` / `grammar` /
//! `spelling`) riding beside the word.
//!
//! The tier prices the word and nothing else: an invalid control's
//! doors answer exactly as they always did (invalid is not a wall —
//! the value stays editable, the re-submit the app's own contract),
//! the rejection the AT reads *while* the user edits. `None` is the
//! honest absence (the ADR-0087 presence rule at the tier): a control
//! that declared nothing announces nothing, never a guessed default.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the field.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, InvalidKind, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Checkbox, Column, Slider, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M107 cut —
/// the invalid route's case joining the required route's case (M106),
/// the reveal route's case (M102), the scroll-position route's case
/// (M101) and every tier's before them on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 157;

/// The suite's own read door into the published corpus: the invalid
/// route's case — the 1,000-node invalid field arena, the rejection
/// word's snapshot price. The ratio's honest denominator is the same
/// 1,000-node input arena bare (`placeholder/snapshot_1k_bare`, the
/// M96 case), the M95 pricing rule: the same corpus measuring what the
/// tier adds.
const INVALID_CASE: &str = "invalid/snapshot_1k_invalid";
const BARE_CASE: &str = "placeholder/snapshot_1k_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's invalid door: the mutate seam the app's code rides —
/// the word moved, `PAINT | A11Y` requested on the field, the frames
/// the re-announcement rides settled before the read.
fn set_invalid(
    harness: &mut TestHarness,
    node_id: alacer_core::NodeId,
    invalid: Option<InvalidKind>,
) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_invalid(invalid);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m107_the_invalid_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 320.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The invalid field, mounted through the constructor word: the
    // rejection the tree says from the first frame, the kind beside it
    // (the prose-error kind the AT reads natively).
    let card = TextInput::new("Card").invalid(InvalidKind::Spelling);
    let _ = harness.add_child(root, card.boxed());
    // The value-tier controls beside it: the tier's trait door answers
    // them without a seam change (the re-open rule's shape).
    let amount = Slider::new("Amount", 0.0, 100.0).invalid(InvalidKind::True);
    let _ = harness.add_child(root, amount.boxed());
    let agree = Checkbox::new("Agree").invalid(InvalidKind::Grammar);
    let _ = harness.add_child(root, agree.boxed());
    // The plain field: no invalid word to say.
    let _ = harness.add_child(root, TextInput::new("Note").boxed());
    let _ = harness.add_child(root, Button::new("Save").boxed());
    harness.settle();

    let card = node(&harness, Role::TextInput, "Card").expect("the invalid field rides");
    assert_eq!(
        card.invalid,
        Some(InvalidKind::Spelling),
        "the constructor's word and kind are the tree's truth"
    );
    let amount = node(&harness, Role::Slider, "Amount").expect("the invalid slider rides");
    assert_eq!(
        amount.invalid,
        Some(InvalidKind::True),
        "the value tier's controls carry the word without a seam change"
    );
    let agree = node(&harness, Role::CheckBox, "Agree").expect("the invalid checkbox rides");
    assert_eq!(
        agree.invalid,
        Some(InvalidKind::Grammar),
        "the toggle tier carries the word beside its own"
    );
    let note = node(&harness, Role::TextInput, "Note").expect("the plain field rides");
    assert!(
        note.invalid.is_none(),
        "a field that declared nothing answers no invalid word"
    );
    let button = node(&harness, Role::Button, "Save").expect("the button rides");
    assert!(
        button.invalid.is_none(),
        "a non-input role inherits the honest absence"
    );

    // The word prices nothing but the word: the invalid field's write
    // door answers exactly as any editable's.
    assert!(
        harness.act(Role::TextInput, "Card", A11yAction::SetValue("4242".into())),
        "invalid is not a wall — the field stays editable"
    );
    let filled = node(&harness, Role::TextInput, "Card").expect("the field rides");
    assert_eq!(
        filled.value.as_deref(),
        Some("4242"),
        "the AT's write lands on the invalid field as ever"
    );
    harness.assert_all_named();
}

#[test]
fn m107_the_invalid_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 160.0);
    let root = harness.mount(Column::new().boxed());
    let note = TextInput::new("Note");
    let note_id = harness.add_child(root, note.boxed());
    harness.settle();

    // The live door: the app's rejection moves, the re-announcement the
    // mutate seam's request paid for — the kind riding beside the word,
    // and the honest absence restored on the clear.
    set_invalid(&mut harness, note_id, Some(InvalidKind::True));
    let marked = node(&harness, Role::TextInput, "Note").expect("the field survives");
    assert_eq!(
        marked.invalid,
        Some(InvalidKind::True),
        "the app's door moves the word the tree says, kind beside it"
    );
    set_invalid(&mut harness, note_id, Some(InvalidKind::Spelling));
    let retyped = node(&harness, Role::TextInput, "Note").expect("the field survives");
    assert_eq!(
        retyped.invalid,
        Some(InvalidKind::Spelling),
        "the same door moves the kind — the AT reads the new rejection"
    );
    set_invalid(&mut harness, note_id, None);
    let cleared = node(&harness, Role::TextInput, "Note").expect("the field survives");
    assert!(
        cleared.invalid.is_none(),
        "the same door carries the word away — the honest absence restored"
    );
    harness.assert_all_named();
}

#[test]
fn m107_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the invalid route's case: the
    // 1,000-node invalid field arena — the word's cost the
    // measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(INVALID_CASE) && benches.contains_key(BARE_CASE),
        "the invalid route's case and its denominator are published ({INVALID_CASE}, {BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M107 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [INVALID_CASE, BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("invalid_snapshot_overhead"))
        .expect("the invalid budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(INVALID_CASE),
        "the budget prices the invalid route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(BARE_CASE),
        "the budget prices it against the same arena, bare"
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
        "the m5 corpus pin re-agrees at the M107 cut"
    );
}
