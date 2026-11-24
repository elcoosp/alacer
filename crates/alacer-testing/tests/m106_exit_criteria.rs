//! M106 exit-criteria integration tests (§10.3, ADR-0110): the
//! required tier — the declared word behind the form contract. Since
//! M95 (ADR-0099) the tree says what a control *does* (the
//! description), since M96 what an empty field *hints* (the
//! placeholder), since M97 the span a value rides — and the word every
//! form vocabulary speaks before the submit fires ("this one must be
//! filled") had no tree tier: the AT announced the field, the hint,
//! the span — and never the ask. The word grows the required tier
//! beside the read-only one's — [`alacer_core::Widget::required`]
//! beside [`alacer_widgets::TextInput::required`],
//! [`alacer_widgets::Slider::required`] and
//! [`alacer_widgets::Checkbox::required`] — AccessKit's `Required`
//! flag (ARIA's `aria-required`) the sink maps in the same snapshot
//! every other property rides.
//!
//! The tier prices the word and nothing else: a required field's doors
//! answer exactly as they always did (required is not a wall — the
//! field stays editable, the empty submit the app's own contract), the
//! declaration the AT reads before it asks. `false` is the honest
//! absence (the ADR-0087 presence rule at the tier): a control that
//! declared nothing announces nothing, never a guessed default.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the field.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Checkbox, Column, Slider, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M106 cut —
/// the required route's case joining the read-only route's case
/// (M105), the reveal route's case (M102), the scroll position route's
/// case (M101), the secure route's case (M100), the numeric route's
/// case (M99), the orientation route's case (M98), the range route's
/// case (M97), the placeholder route's pair (M96), the description
/// route's case (M95), the toggle route's pair (M94), the derive
/// route's pair (M92), and the recording route's pair (M93) — and
/// every case M103 and M104 published — on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 204;

/// The suite's own read door into the published corpus: the required
/// route's case — the 1,000-node required field arena (the coordinated
/// rule's bench half, ADR-0110's exit criteria). The ratio's honest
/// denominator is the same 1,000-node input arena bare
/// (`placeholder/snapshot_1k_bare`, the M96 case), the M95 pricing
/// rule: the same corpus measuring what the tier adds.
const REQUIRED_CASE: &str = "required/snapshot_1k_required";
const BARE_CASE: &str = "placeholder/snapshot_1k_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's required door: the mutate seam the app's code rides —
/// the word moved, `PAINT | A11Y` requested on the field, the frames
/// the re-announcement rides settled before the read.
fn set_required(harness: &mut TestHarness, node: alacer_core::NodeId, required: bool) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_required(required);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m106_the_required_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 320.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The required field, mounted through the constructor word: the
    // declaration the tree says from the first frame.
    let card = TextInput::new("Card").required();
    let _ = harness.add_child(root, card.boxed());
    // The value-tier controls beside it: the tier's trait door answers
    // them without a seam change (the re-open rule's shape).
    let amount = Slider::new("Amount", 0.0, 100.0).required();
    let _ = harness.add_child(root, amount.boxed());
    let agree = Checkbox::new("Agree").required();
    let _ = harness.add_child(root, agree.boxed());
    // The plain field: no required word to say.
    let _ = harness.add_child(root, TextInput::new("Note").boxed());
    let _ = harness.add_child(root, Button::new("Save").boxed());
    harness.settle();

    let card = node(&harness, Role::TextInput, "Card").expect("the required field rides");
    assert!(card.required, "the constructor's word is the tree's truth");
    let amount = node(&harness, Role::Slider, "Amount").expect("the required slider rides");
    assert!(
        amount.required,
        "the value tier's controls carry the word without a seam change"
    );
    let agree = node(&harness, Role::CheckBox, "Agree").expect("the required checkbox rides");
    assert!(
        agree.required,
        "the toggle tier carries the word beside its own"
    );
    let note = node(&harness, Role::TextInput, "Note").expect("the plain field rides");
    assert!(
        !note.required,
        "a field that declared nothing answers no required word"
    );
    let button = node(&harness, Role::Button, "Save").expect("the button rides");
    assert!(
        !button.required,
        "a non-input role inherits the honest absence"
    );

    // The word prices nothing but the word: the required field's write
    // door answers exactly as any editable's.
    assert!(
        harness.act(Role::TextInput, "Card", A11yAction::SetValue("4242".into())),
        "required is not a wall — the field stays editable"
    );
    let filled = node(&harness, Role::TextInput, "Card").expect("the field rides");
    assert_eq!(
        filled.value.as_deref(),
        Some("4242"),
        "the AT's write lands on the required field as ever"
    );
    harness.assert_all_named();
}

#[test]
fn m106_the_required_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 160.0);
    let root = harness.mount(Column::new().boxed());
    let note = TextInput::new("Note");
    let note_id = harness.add_child(root, note.boxed());
    harness.settle();

    // The live door: the app's word moves, the re-announcement the
    // mutate seam's request paid for — and moves back.
    set_required(&mut harness, note_id, true);
    let marked = node(&harness, Role::TextInput, "Note").expect("the field survives");
    assert!(
        marked.required,
        "the app's door moves the word the tree says"
    );
    set_required(&mut harness, note_id, false);
    let cleared = node(&harness, Role::TextInput, "Note").expect("the field survives");
    assert!(
        !cleared.required,
        "the same door carries the word both ways — the honest absence restored"
    );
    harness.assert_all_named();
}

#[test]
fn m106_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the required route's case: the
    // 1,000-node required field arena — the word's cost the
    // measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(REQUIRED_CASE) && benches.contains_key(BARE_CASE),
        "the required route's case and its denominator are published ({REQUIRED_CASE}, {BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M106 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [REQUIRED_CASE, BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("required_snapshot_overhead"))
        .expect("the required budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(REQUIRED_CASE),
        "the budget prices the required route"
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
        "the m5 corpus pin re-agrees at the M106 cut"
    );
}
