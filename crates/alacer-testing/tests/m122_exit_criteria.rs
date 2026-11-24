//! M122 exit-criteria integration tests (§10.3, ADR-0126): the multiline
//! tier — the field's line capacity, said by the tree. Through M121 the
//! snapshot said what an empty field *hints* (ADR-0100), the form's ask
//! and the form's rejection (ADR-0110/ADR-0111), the field's suggestion
//! contract (ADR-0115), the field's secret entry and its reveal
//! (ADR-0106), the wall the read-only field holds (ADR-0109) — and the
//! word every form-and-composer vocabulary speaks about *how much the
//! field takes* ("this field wraps onto many lines") had no tier: the AT
//! announced the hint, the ask, the rejection, the suggestion contract —
//! and never the capacity — the declared-but-unreported gap (the M94
//! pattern), cashed at the multiline tier: the word grows beside the
//! field vocabulary it completes — [`alacer_core::Widget::multiline`]
//! beside [`alacer_widgets::TextInput::multiline`], AccessKit's
//! `Multiline` property (ARIA's `aria-multiline`) the sink maps in the
//! same snapshot every other property rides.
//!
//! The tier prices the word and nothing else: a worded field's doors
//! answer exactly as they always did — the word names the capacity, it
//! walls nothing, the field's typing, caret, selection and submit routes
//! the widget's own (a tree that refused on the word's behalf would
//! price a contract it does not own — the word says the field wraps, it
//! never changes what the field takes). `false` is the honest absence
//! (the ADR-0087 presence rule at the tier, the read-only shape): a
//! field that declared nothing announces no capacity, never a guessed
//! "multiline" — the single-line field the AT read before the tier.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the word's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M122 cut —
/// the multiline route's case joining the placeholder route's honest pair
/// (M96), the shortcut route's case (M121), the live route's case (M120),
/// the set route's pair (M119), the busy route's pair (M117) and every
/// tier's before them on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 204;

/// The suite's own read door into the published corpus: the multiline
/// route's case — the 1,000-node input arena carrying the constructor's
/// capacity word — against the placeholder route's published bare arena
/// (`placeholder/snapshot_1k_bare`, the M96 case the tier reuses by the
/// M111 precedent; the M95 pricing rule — the word is a field word, so
/// the honest comparison is fields against fields, the M108/M121
/// denominator shape).
const MULTILINE_CASE: &str = "multiline/snapshot_1k_fields_multiline";
const FIELDS_BARE_CASE: &str = "placeholder/snapshot_1k_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's multiline door: the mutate seam the app's code rides — the
/// capacity moved, `PAINT | A11Y` requested on the control, the frames
/// the re-announcement rides settled before the read.
fn set_field_multiline(harness: &mut TestHarness, node_id: alacer_core::NodeId, multiline: bool) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(field) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        field.set_multiline(multiline);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m122_the_multiline_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The worded field, mounted through the constructor word: the fact
    // the tree says from the first frame — the composer's capacity the
    // AT reads beside the hint — beside the plain field and the plain
    // label, the honest absences the tier keeps honest.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(root, TextInput::new("Notes").multiline().boxed());
    let _ = harness.add_child(root, TextInput::new("Title").boxed());
    let _ = harness.add_child(root, Label::new("A plain label").boxed());
    harness.settle();

    let notes = node(&harness, Role::TextInput, "Notes").expect("the worded field rides");
    assert!(
        notes.multiline,
        "the constructor's word is the tree's truth — the capacity said beside the name"
    );
    let title = node(&harness, Role::TextInput, "Title").expect("the plain field rides");
    assert!(
        !title.multiline,
        "a field that declared no capacity announces none — the honest absence"
    );
    let label = node(&harness, Role::Label, "A plain label").expect("the plain label rides");
    assert!(
        !label.multiline,
        "the word is a field word: the plain roles answer nothing — the inherited default"
    );
    // The word prices nothing but the word: the worded field's doors
    // answer exactly as they always did — the capacity names a fact, it
    // walls nothing, the AT's write door lands where it always landed.
    assert!(
        harness.act(
            Role::TextInput,
            "Notes",
            A11yAction::SetValue("line one\nline two".to_owned()),
        ),
        "the worded field's write door answers unchanged beside the word"
    );
    harness.assert_all_named();
}

#[test]
fn m122_the_multiline_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let summary = TextInput::new("Summary");
    let summary_id = harness.add_child(root, summary.boxed());
    harness.settle();

    // The multiline door: the app's capacity moved, the re-announcement
    // the mutate seam's request paid for — and the honest absence
    // restored on the clear (the capacity withdrawn, the field the
    // single-line widget it always was).
    set_field_multiline(&mut harness, summary_id, true);
    let worded = node(&harness, Role::TextInput, "Summary").expect("the field survives");
    assert!(
        worded.multiline,
        "the app's door moves the word the tree says"
    );
    set_field_multiline(&mut harness, summary_id, false);
    let withdrawn = node(&harness, Role::TextInput, "Summary").expect("the field survives");
    assert!(
        !withdrawn.multiline,
        "the same door carries the word away — the honest absence restored"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(summary_id);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<TextInput>()
        .expect("the field survives")
        .set_multiline(false);
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m122_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the multiline route's case: the
    // 1,000-node input arena carrying the constructor's capacity word —
    // the word's cost the measurement, priced against the placeholder
    // route's published bare arena (the same fields, the word absent).
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(MULTILINE_CASE) && benches.contains_key(FIELDS_BARE_CASE),
        "the multiline route's case is published beside its reused denominator ({MULTILINE_CASE}, {FIELDS_BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M122 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [MULTILINE_CASE, FIELDS_BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("multiline_snapshot_overhead"))
        .expect("the multiline budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(MULTILINE_CASE),
        "the budget prices the multiline route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(FIELDS_BARE_CASE),
        "the budget prices it against the same field arena, bare"
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
        "the m5 corpus pin re-agrees at the M122 cut"
    );
}
