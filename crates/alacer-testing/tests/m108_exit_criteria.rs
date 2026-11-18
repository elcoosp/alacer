//! M108 exit-criteria integration tests (§10.3, ADR-0112): the
//! selected tier — the collection's pick, said by the tree. Through
//! M107 the snapshot said where a member sits (the M6 table slot), the
//! form's ask (the required tier, ADR-0110) and the form's rejection
//! (the invalid tier, ADR-0111) — and the word every collection
//! vocabulary speaks about the member ("this one is the pick") had no
//! tree tier: the AT announced the position, the name, the form facts
//! — and never the pick. The declared-but-unreported gap (the M94
//! pattern) cashed at the selected tier: the word grows beside the
//! collection geometry the M6 canon already carries —
//! [`alacer_core::Widget::selected`] beside
//! [`alacer_widgets::TableRow::selected`] and
//! [`alacer_widgets::TableCell::selected`], AccessKit's `Selected`
//! property (ARIA's `aria-selected`) the sink maps in the same
//! snapshot every other property rides.
//!
//! The tier prices the word and nothing else: a selected member's
//! doors answer exactly as they always did — the word names state, it
//! walls nothing, the pick's press and focus routes the widget's own.
//! `None` is the honest absence (the ADR-0087 presence rule at the
//! selected tier): a member of no selection collection announces
//! nothing, never a guessed `false` — `Some(false)` the member
//! standing unpicked beside its picked siblings, `Some(true)` the pick.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the word's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{
    BoxedWidget as _, Button, Column, Label, Table, TableCell, TableRow, TextInput,
};

/// The suite's corpus pin: the coordinated rule's count at the M108 cut —
/// the selected route's pair joining the invalid route's case (M107),
/// the required route's case (M106), the reveal route's case (M102), the
/// scroll-position route's case (M101) and every tier's before them on
/// the published corpus.
const EXPECTED_BENCH_COUNT: usize = 135;

/// The suite's own read door into the published corpus: the selected
/// route's pair — the 1,000-node row arena carrying the pick word,
/// against the same arena bare. The ratio's honest denominator is the
/// same row arena with the word absent (the M95 pricing rule: the same
/// corpus measuring what the tier adds).
const SELECTED_CASE: &str = "selected/snapshot_1k_selected";
const ROWS_BARE_CASE: &str = "selected/snapshot_1k_rows_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's selected door: the mutate seam the app's code rides —
/// the word moved, `PAINT | A11Y` requested on the row, the frames
/// the re-announcement rides settled before the read.
fn set_row_selected(
    harness: &mut TestHarness,
    node_id: alacer_core::NodeId,
    selected: Option<bool>,
) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(row) = guard.widget_mut().as_any_mut().downcast_mut::<TableRow>() {
        row.set_selected(selected);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

/// The app's selected door on a cell: the same mutate seam, the same
/// request, the cell's word the re-announcement rides.
fn set_cell_selected(
    harness: &mut TestHarness,
    node_id: alacer_core::NodeId,
    selected: Option<bool>,
) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(cell) = guard.widget_mut().as_any_mut().downcast_mut::<TableCell>() {
        cell.set_selected(selected);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m108_the_selected_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The selectable table, mounted through the constructor word: the
    // pick the tree says from the first frame — row 0 the pick, row 1
    // standing unpicked beside it (its first cell explicitly unpicked
    // at the cell tier), row 2 belonging to no selection.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let table = Table::new("Leads", 2);
    let table_id = harness.add_child(root, table.boxed());
    let picked = harness.add_child(table_id, TableRow::new().selected(true).boxed());
    let _ = harness.add_child(picked, TableCell::new("Ada", 0, 0).boxed());
    let _ = harness.add_child(picked, TableCell::new("640", 0, 1).boxed());
    let unpicked = harness.add_child(table_id, TableRow::new().selected(false).boxed());
    let bo = harness.add_child(unpicked, TableCell::new("Bo", 1, 0).selected(false).boxed());
    let _ = harness.add_child(bo, TableCell::new("128", 1, 1).boxed());
    let _ = harness.add_child(table_id, TableRow::new().boxed());
    // The plain widgets beside it: no collection, no word to say.
    let _ = harness.add_child(root, TextInput::new("Note").boxed());
    let _ = harness.add_child(root, Label::new("Plain").boxed());
    let _ = harness.add_child(root, Button::new("Approve").boxed());
    harness.settle();

    let snapshot = harness
        .pipeline()
        .a11y_snapshot()
        .expect("the snapshot rides");
    let rows: Vec<&TreeNode> = snapshot
        .walk(None)
        .into_iter()
        .filter(|n| n.role == Role::TableRow)
        .collect();
    assert_eq!(
        rows.len(),
        3,
        "the three rows ride, the wordless one beside its worded siblings"
    );
    assert_eq!(
        rows[0].selected,
        Some(true),
        "the constructor's word is the tree's truth — the pick said"
    );
    assert_eq!(
        rows[1].selected,
        Some(false),
        "the member standing unpicked beside its picked siblings says false, never a guessed absence"
    );
    assert_eq!(
        rows[2].selected, None,
        "a member of no selection collection announces nothing — the honest absence"
    );
    // The word is per-member state, never an inherited one: the pick
    // row's own cells carry no word of their own unless they declare
    // one — the cell tier beside the row tier (the re-open rule's
    // shape, both producers).
    let ada = node(&harness, Role::TableCell, "Ada").expect("the pick's cell rides");
    assert_eq!(
        ada.selected, None,
        "a cell that declared no word of its own announces nothing — the word never inherits"
    );
    let bo = node(&harness, Role::TableCell, "Bo").expect("the unpicked cell rides");
    assert_eq!(
        bo.selected,
        Some(false),
        "the cell tier's constructor answers beside the row tier's without a seam change"
    );
    // The word is a member word, not a widget word: the plain widgets
    // beside the collection inherit the absence.
    let note = node(&harness, Role::TextInput, "Note").expect("the plain field rides");
    assert!(
        note.selected.is_none(),
        "a field that declared nothing answers no selected word"
    );
    let approve = node(&harness, Role::Button, "Approve").expect("the button rides");
    assert!(
        approve.selected.is_none(),
        "a non-collection role inherits the honest absence"
    );

    // The word prices nothing but the word: the worded collection's
    // neighbours answer exactly as they always did — the pick names
    // state, it walls nothing.
    assert!(
        harness.act(Role::Button, "Approve", A11yAction::DefaultPress),
        "the press route answers unchanged beside the collection's word"
    );
    harness.assert_all_named();
}

#[test]
fn m108_the_selected_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let table = Table::new("Queues", 1);
    let table_id = harness.add_child(root, table.boxed());
    let row = harness.add_child(table_id, TableRow::new().boxed());
    let cell_id = harness.add_child(row, TableCell::new("Kai", 0, 0).boxed());
    harness.settle();

    // The live door: the app's pick moves, the re-announcement the
    // mutate seam's request paid for — and the honest absence restored
    // on the clear.
    set_row_selected(&mut harness, row, Some(true));
    let picked = harness
        .pipeline()
        .a11y_snapshot()
        .expect("the snapshot rides")
        .walk(None)
        .into_iter()
        .find(|n| n.role == Role::TableRow)
        .expect("the row survives");
    assert_eq!(
        picked.selected,
        Some(true),
        "the app's door moves the word the tree says"
    );
    set_row_selected(&mut harness, row, Some(false));
    let unpicked = harness
        .pipeline()
        .a11y_snapshot()
        .expect("the snapshot rides")
        .walk(None)
        .into_iter()
        .find(|n| n.role == Role::TableRow)
        .expect("the row survives");
    assert_eq!(
        unpicked.selected,
        Some(false),
        "the same door unpicks — the member standing beside the pick"
    );
    set_row_selected(&mut harness, row, None);
    let cleared = harness
        .pipeline()
        .a11y_snapshot()
        .expect("the snapshot rides")
        .walk(None)
        .into_iter()
        .find(|n| n.role == Role::TableRow)
        .expect("the row survives");
    assert!(
        cleared.selected.is_none(),
        "the same door carries the word away — the honest absence restored"
    );

    // The cell's door beside the row's: the tier's trait door already
    // answers both producers (the re-open rule's shape).
    set_cell_selected(&mut harness, cell_id, Some(true));
    let kai = node(&harness, Role::TableCell, "Kai").expect("the cell survives");
    assert_eq!(
        kai.selected,
        Some(true),
        "the cell's live door moves its word without a seam change"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(cell_id);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<TableCell>()
        .expect("the cell survives")
        .set_selected(Some(true));
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m108_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the selected route's pair: the
    // 1,000-node row arena carrying the pick word — the word's cost the
    // measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(SELECTED_CASE) && benches.contains_key(ROWS_BARE_CASE),
        "the selected route's pair is published ({SELECTED_CASE}, {ROWS_BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M108 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [SELECTED_CASE, ROWS_BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("selected_snapshot_overhead"))
        .expect("the selected budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(SELECTED_CASE),
        "the budget prices the selected route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(ROWS_BARE_CASE),
        "the budget prices it against the same row arena, bare"
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
        "the m5 corpus pin re-agrees at the M108 cut"
    );
}
