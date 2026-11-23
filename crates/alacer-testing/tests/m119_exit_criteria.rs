//! M119 exit-criteria integration tests (§10.3, ADR-0123): the set
//! tier — the member's place and the collection's size, said by the
//! tree. Through M118 the snapshot said what a control is read as (the
//! role-description tier, ADR-0120), the subtree's updating fact (the
//! busy tier, ADR-0121), what it opens (ADR-0119), the set's current
//! item (ADR-0118), the nested member's depth (ADR-0117), the sorted
//! column's order (ADR-0116), the field's suggestion contract
//! (ADR-0115), the member's disclosure (ADR-0114), the container's
//! capacity (ADR-0113), the member's pick (ADR-0112), the form's
//! rejection (ADR-0111), the form's ask (ADR-0110) — and the numbers
//! every collection vocabulary speaks about its members and its span
//! ("row 3 of 500") had no tree tier: the AT announced the grid
//! position the M6 slot carried, the name, the form facts — and never
//! the place or the size — the declared-but-unreported gap (the M94
//! pattern), cashed at the set tier: the two words grow beside the
//! collection machinery they name — [`alacer_core::Widget::position_in_set`]
//! and [`alacer_core::Widget::size_of_set`] beside
//! [`alacer_widgets::TableRow::position_in_set`] and
//! [`alacer_widgets::Table::size_of_set`], AccessKit's
//! `PositionInSet` property (ARIA's `aria-posinset`) and
//! `SizeOfSet` property (ARIA's `aria-setsize`) the sink maps in the
//! same snapshot every other property rides.
//!
//! The tier prices the words and nothing else: a collection's doors
//! answer exactly as they always did — the words name a place and a
//! span, they wall nothing, the rows' and the table's routes the
//! widgets' own. `None` is the honest absence (the ADR-0087 presence
//! rule at the tier, the level shape): a row that declared no place
//! and a table that declared no size announce neither — never a
//! guessed `0`.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's doors ride the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the words' nodes.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Table, TableCell, TableRow, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M119 cut —
/// the set route's pair joining the busy route's pair (M117), the
/// role-description route's case (M116), the has-popup route's case
/// (M115), the current route's pair (M114), the level route's case
/// (M113), the sort route's pair (M112), the auto-complete route's case
/// (M111), the expanded route's pair (M110), the multiselectable
/// route's pair (M109) and every tier's before them on the published
/// corpus.
const EXPECTED_BENCH_COUNT: usize = 157;

/// The suite's own read door into the published corpus: the set
/// route's pair — the 1,000-node row arena carrying the two words (the
/// place cycling down the rows, the size declared on the table) against
/// the same arena bare. The ratio's honest denominator is the same
/// shapes with the words absent (the M95 pricing rule: the same corpus
/// measuring what the tier adds; the words are row-and-container
/// words, so the honest comparison is rows against rows — the M117
/// denominator shape).
const SET_CASE: &str = "set/snapshot_1k_rows_set";
const ROWS_BARE_CASE: &str = "set/snapshot_1k_rows_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// A container's child nodes in tree order — the walk the nameless
/// members (rows read through their cells) answer through, the m108
/// suite's query shape.
fn children_of(harness: &TestHarness, container: &TreeNode) -> Vec<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .map(|tree| {
            container
                .children
                .iter()
                .filter_map(|id| tree.by_id(*id).cloned())
                .collect()
        })
        .unwrap_or_default()
}

/// The app's set doors: the mutate seam the app's code rides — a word
/// moved, `PAINT | A11Y` requested on the row or the table, the frames
/// the re-announcement rides settled before the read.
fn move_set_words(
    harness: &mut TestHarness,
    row_id: Option<alacer_core::NodeId>,
    table_id: Option<alacer_core::NodeId>,
    place: Option<usize>,
    size: Option<usize>,
) {
    if let Some(row_id) = row_id {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(row_id);
        if let Some(row) = guard.widget_mut().as_any_mut().downcast_mut::<TableRow>() {
            row.set_position_in_set(place);
            guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
        }
        drop(guard);
    }
    if let Some(table_id) = table_id {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(table_id);
        if let Some(table) = guard.widget_mut().as_any_mut().downcast_mut::<Table>() {
            table.set_size_of_set(size);
            guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
        }
        drop(guard);
    }
    harness.settle();
}

#[test]
fn m119_the_set_words_land_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The collection facts, mounted through the constructor words: the
    // place the tree says for every row from the first frame, the size
    // the tree says on the table — beside the words' honest absences,
    // the bare rows and the plain widgets the tier keeps honest.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let ledger = Table::new("Ledger", 2).size_of_set(500);
    let ledger_id = harness.add_child(root, ledger.boxed());
    let placed = harness.add_child(ledger_id, TableRow::new().position_in_set(2).boxed());
    let _ = harness.add_child(placed, TableCell::new("Entry 1", 0, 0).boxed());
    let bare_row_id = harness.add_child(ledger_id, TableRow::new().boxed());
    let _ = harness.add_child(bare_row_id, TableCell::new("Entry 2", 1, 0).boxed());
    let bare_table = Table::new("Archive", 2);
    let bare_table_id = harness.add_child(root, bare_table.boxed());
    let bare_table_row = harness.add_child(bare_table_id, TableRow::new().boxed());
    let _ = harness.add_child(bare_table_row, TableCell::new("Entry 3", 0, 0).boxed());
    // The plain widgets beside them: no place and no size to say.
    let _ = harness.add_child(root, Button::new("Save").boxed());
    let _ = harness.add_child(root, TextInput::new("Note").boxed());
    harness.settle();

    let ledger_node = node(&harness, Role::Table, "Ledger").expect("the sized table rides");
    assert_eq!(
        ledger_node.size_of_set,
        Some(500),
        "the constructor's word is the tree's truth — the span said"
    );
    let ledger_rows = children_of(&harness, &ledger_node);
    assert_eq!(
        ledger_rows.first().and_then(|row| row.position_in_set),
        Some(2),
        "the row's place lands in the same snapshot — the tier's member word"
    );
    assert_eq!(
        ledger_rows.get(1).and_then(|row| row.position_in_set),
        None,
        "a row that declared no place announces none — the honest absence"
    );
    let bare_table = node(&harness, Role::Table, "Archive").expect("the bare table rides");
    assert_eq!(
        bare_table.size_of_set, None,
        "a table that declared no size announces none — the honest absence"
    );
    let bare_table_rows = children_of(&harness, &bare_table);
    assert_eq!(
        bare_table_rows.first().and_then(|row| row.position_in_set),
        None,
        "the bare table's row answers the absence beside it"
    );
    let save = node(&harness, Role::Button, "Save").expect("the button rides");
    assert_eq!(
        save.position_in_set, None,
        "a non-collection role inherits the honest absence"
    );
    let note = node(&harness, Role::TextInput, "Note").expect("the plain field rides");
    assert_eq!(
        note.position_in_set, None,
        "a field that declared nothing answers no place word"
    );
    // The words price nothing but the words: the worded collection's
    // neighbours answer exactly as they always did — a place and a span
    // name facts, they wall nothing.
    assert!(
        harness.act(Role::Button, "Save", A11yAction::DefaultPress),
        "the press route answers unchanged beside the collection's words"
    );
    harness.assert_all_named();
}

#[test]
fn m119_the_set_doors_move_the_words_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let table = Table::new("Ledger", 2);
    let table_id = harness.add_child(root, table.boxed());
    let row_id = harness.add_child(table_id, TableRow::new().boxed());
    let _ = harness.add_child(row_id, TableCell::new("Entry 1", 0, 0).boxed());
    harness.settle();

    // The live doors: the app's collection facts moved, the
    // re-announcements the mutate seam's request paid for — and the
    // honest absences restored on the clears (the collection shrank,
    // the row left the set).
    move_set_words(
        &mut harness,
        Some(row_id),
        Some(table_id),
        Some(2),
        Some(500),
    );
    let sized = node(&harness, Role::Table, "Ledger").expect("the table survives");
    assert_eq!(
        sized.size_of_set,
        Some(500),
        "the app's door moves the size the tree says"
    );
    let placed = children_of(&harness, &sized)
        .first()
        .and_then(|row| row.position_in_set)
        .expect("the placed row rides");
    assert_eq!(placed, 2, "the app's door moves the place the tree says");
    move_set_words(&mut harness, Some(row_id), Some(table_id), None, None);
    let cleared_table = node(&harness, Role::Table, "Ledger").expect("the table survives");
    assert_eq!(
        cleared_table.size_of_set, None,
        "the same door carries the size away — the honest absence restored"
    );
    let cleared_row = children_of(&harness, &cleared_table)
        .first()
        .and_then(|row| row.position_in_set);
    assert_eq!(
        cleared_row, None,
        "the same door carries the place away — the honest absence restored"
    );

    // The writes that land where the words already are queue nothing:
    // the doors answer `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(table_id);
    let size_moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Table>()
        .expect("the table survives")
        .set_size_of_set(None);
    drop(guard);
    let mut guard = harness.pipeline_mut().arena_mut().mutate(row_id);
    let place_moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<TableRow>()
        .expect("the row survives")
        .set_position_in_set(None);
    drop(guard);
    harness.settle();
    assert!(
        !size_moved && !place_moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m119_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the set route's pair: the
    // 1,000-node row arena carrying the two words (the place cycling
    // down the rows, the size on the table) — the words' cost the
    // measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(SET_CASE) && benches.contains_key(ROWS_BARE_CASE),
        "the set route's pair is published ({SET_CASE}, {ROWS_BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M119 cut"
    );

    // The contract set gates both cases and prices the words' budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [SET_CASE, ROWS_BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("set_snapshot_overhead"))
        .expect("the set budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(SET_CASE),
        "the budget prices the set route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(ROWS_BARE_CASE),
        "the budget prices it against the same row arena, bare"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the words must not outprice the snapshot they ride"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M119 cut"
    );
}
