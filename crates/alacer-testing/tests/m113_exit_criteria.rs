//! M113 exit-criteria integration tests (§10.3, ADR-0117): the level
//! tier — the nested member's depth, said by the tree. Through M112 the
//! snapshot said where a member sits in its grid (the M6 table slot),
//! what a control *does* (the description, ADR-0099), what an empty
//! field *hints* (the placeholder, ADR-0100), the span a value rides
//! (the range tier, ADR-0101), the form's ask and the form's rejection
//! (the required and invalid tiers, ADR-0110/ADR-0111), the collection's
//! pick, capacity and disclosure (ADR-0112/ADR-0113/ADR-0114), the
//! field's suggestion contract (the auto-complete tier, ADR-0115), the
//! sorted column's order (the sort direction tier, ADR-0116) — and the
//! word every nested-collection vocabulary speaks about how deep a
//! member stands ("this one is a level-2 row under the level-1 one")
//! had no tier: the AT announced the grid position, the name, the form
//! facts — and never the depth — the declared-but-unreported gap (the
//! M94 pattern), cashed at the level tier: the word grows beside the
//! collection words it depths — [`alacer_core::Widget::level`] beside
//! [`alacer_widgets::TableRow::level`], AccessKit's `Level` property
//! the sink maps in the same snapshot every other property rides.
//!
//! The tier prices the word and nothing else: a nested member's doors
//! answer exactly as they always did — the word names the depth, it
//! walls nothing, the member's routes the widget's own. The count is
//! zero-based — AccessKit's own contract names it ("this property is
//! zero-based", beside ARIA's one-based `aria-level` the property
//! maps) — so the mapping is one branch with no translation. `None` is
//! the honest absence spelled by `None` — a member that declared
//! nothing announces no depth, never a guessed `0`.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the word's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, ColumnHeader, TableRow};

/// The suite's corpus pin: the coordinated rule's count at the M113 cut —
/// the level route's case joining the sort route's pair (M112), the
/// auto-complete route's case (M111), the expanded route's pair (M110),
/// the multiselectable route's pair (M109), the selected route's pair
/// (M108), the invalid route's case (M107) and every tier's before them
/// on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 208;

/// The suite's own read door into the published corpus: the level
/// route's case — the 1,000-node row arena carrying the depth word —
/// priced against the same 1,000-node row arena bare
/// (`expanded/snapshot_1k_rows_bare`, the M110 case the tier reuses by
/// the M111 precedent; the M95 pricing rule: the word is a row word,
/// so the honest comparison is rows against rows, the M108/M110
/// denominator shape).
const LEVEL_CASE: &str = "level/snapshot_1k_level";
const ROWS_BARE_CASE: &str = "expanded/snapshot_1k_rows_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's level door: the mutate seam the app's code rides — the
/// word moved, `PAINT | A11Y` requested on the row, the frames the
/// re-announcement rides settled before the read. `None` clears the
/// word — the honest absence restored.
fn set_level(harness: &mut TestHarness, node_id: alacer_core::NodeId, level: Option<usize>) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(row) = guard.widget_mut().as_any_mut().downcast_mut::<TableRow>() {
        row.set_level(level);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m113_the_level_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 320.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The worded rows, mounted through the constructor word: the depth
    // the tree says from the first frame — the root-level row, the
    // level-1 row, the level-2 row — beside the plain row, the honest
    // absence the tier keeps honest (the flat row, never a guessed
    // `0`).
    let top = TableRow::new().level(0);
    let _ = harness.add_child(root, top.boxed());
    let group = TableRow::new().level(1);
    let _ = harness.add_child(root, group.boxed());
    let leaf = TableRow::new().level(2);
    let _ = harness.add_child(root, leaf.boxed());
    let plain = TableRow::new();
    let _ = harness.add_child(root, plain.boxed());
    // The header and the plain widget beside them: no depth to say —
    // the word is a member word, the header's order the sort tier's
    // own fact.
    let _ = harness.add_child(root, ColumnHeader::new("Price", 0).boxed());
    let _ = harness.add_child(root, Button::new("Expand all").boxed());
    harness.settle();

    let rows: Vec<TreeNode> = harness
        .pipeline()
        .a11y_snapshot()
        .map(|tree| {
            tree.nodes
                .iter()
                .filter(|n| n.role == Role::TableRow)
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(
        rows.len(),
        4,
        "the four rows ride the snapshot the way the harness mounted them"
    );
    assert_eq!(
        rows[0].level,
        Some(0),
        "the root-level row said — zero-based, AccessKit's own base"
    );
    assert_eq!(
        rows[1].level,
        Some(1),
        "the level-1 row said the depth the constructor named"
    );
    assert_eq!(
        rows[2].level,
        Some(2),
        "the level-2 row said the depth the constructor named"
    );
    assert!(
        rows[3].level.is_none(),
        "a row that declared nothing answers no depth — the flat row, never a guessed `0`"
    );
    let header = node(&harness, Role::ColumnHeader, "Price").expect("the header rides");
    assert!(
        header.level.is_none(),
        "a header role inherits the honest absence — the word is a member word"
    );
    let expand = node(&harness, Role::Button, "Expand all").expect("the button rides");
    assert!(
        expand.level.is_none(),
        "a non-member role inherits the honest absence"
    );

    // The word prices nothing but the word: the nested row's
    // neighbour's write door answers exactly as any button's.
    assert!(
        harness.act(Role::Button, "Expand all", A11yAction::DefaultPress),
        "the level word is not a wall — the rows' neighbour answers as ever"
    );
    harness.assert_all_named();
}

#[test]
fn m113_the_level_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 160.0);
    let root = harness.mount(Column::new().boxed());
    let row = TableRow::new();
    let row_id = harness.add_child(root, row.boxed());
    harness.settle();

    // The live door: the app's depth moved, the re-announcement the
    // mutate seam's request paid for — the zero-based count riding the
    // word, and the honest absence restored on the clear.
    set_level(&mut harness, row_id, Some(3));
    let nested = node(&harness, Role::TableRow, "").expect("the row survives");
    assert_eq!(
        nested.level,
        Some(3),
        "the app's door moves the word the tree says, zero-based"
    );
    set_level(&mut harness, row_id, Some(1));
    let regrouped = node(&harness, Role::TableRow, "").expect("the row survives");
    assert_eq!(
        regrouped.level,
        Some(1),
        "the same door moves the depth — the AT reads the new nesting"
    );
    set_level(&mut harness, row_id, None);
    let flattened = node(&harness, Role::TableRow, "").expect("the row survives");
    assert!(
        flattened.level.is_none(),
        "the same door carries the word away — the honest absence restored"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(row_id);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<TableRow>()
        .expect("the row survives")
        .set_level(None);
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m113_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the level route's case: the
    // 1,000-node row arena carrying the depth word — the word's cost
    // the measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(LEVEL_CASE),
        "the level route's case is published ({LEVEL_CASE})"
    );
    assert!(
        benches.contains_key(ROWS_BARE_CASE),
        "the tier's bare denominator is published ({ROWS_BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M113 cut"
    );

    // The contract set gates the case and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    let gate = gates
        .iter()
        .find(|gate| gate["baseline"].as_str() == Some(LEVEL_CASE))
        .unwrap_or_else(|| panic!("the case carries a regression gate ({LEVEL_CASE})"));
    assert_eq!(
        gate["scope"].as_str(),
        Some("same-host"),
        "the gate is same-host scoped like the tier's own"
    );
    let budgets = contracts["budgets"]
        .as_array()
        .expect("the contract set carries budgets");
    let budget = budgets
        .iter()
        .find(|budget| budget["id"].as_str() == Some("level_snapshot_overhead"))
        .expect("the level budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(LEVEL_CASE),
        "the budget prices the level route"
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
        "the m5 corpus pin re-agrees at the M113 cut"
    );
}
