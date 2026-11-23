//! M112 exit-criteria integration tests (§10.3, ADR-0116): the sort
//! direction tier — the sorted column's order, said by the tree.
//! Through M111 the snapshot said where a member sits (the M6 table
//! slot), what a control *does* (the description, ADR-0099), what an
//! empty field *hints* (the placeholder, ADR-0100), the span a value
//! rides (the range tier, ADR-0101), the form's ask and the form's
//! rejection (the required and invalid tiers, ADR-0110/ADR-0111), the
//! collection's pick, capacity and disclosure (ADR-0112/ADR-0113/
//! ADR-0114), the field's suggestion contract (the auto-complete tier,
//! ADR-0115) — and the word every sorted-column vocabulary speaks
//! about the rows it orders ("this one sorts ascending, that one
//! descending") had no tier: the AT announced the position, the name,
//! the form facts — and never the sort — the declared-but-unreported
//! gap (the M94 pattern), cashed at the sort direction tier: the word
//! grows beside the collection words it orders —
//! [`alacer_core::Widget::sort_direction`] beside
//! [`alacer_widgets::ColumnHeader::sort`], AccessKit's `SortDirection`
//! property (ARIA's `aria-sort`) the sink maps in the same snapshot
//! every other property rides.
//!
//! The tier prices the word and nothing else: a sorted column's doors
//! answer exactly as they always did — the word names the order, it
//! walls nothing, the column's routes the widget's own. ARIA's `none`
//! value is the honest absence spelled by `None` — a header that
//! declared nothing announces no order, never a guessed direction; the
//! kind the word carries (`Ascending` / `Descending` / `Other`)
//! mirrors AccessKit's own enum values, so the mapping is one branch
//! with no translation.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the word's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role, SortDirection};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, ColumnHeader, TableRow};

/// The suite's corpus pin: the coordinated rule's count at the M112 cut —
/// the sort route's pair joining the auto-complete route's case (M111),
/// the expanded route's pair (M110), the multiselectable route's pair
/// (M109), the selected route's pair (M108), the invalid route's case
/// (M107) and every tier's before them on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 159;

/// The suite's own read door into the published corpus: the sort
/// route's pair — the 1,000-node header arena carrying the sort word —
/// priced against the same 1,000-node header arena bare (the M95
/// pricing rule: the word is a header word, so the honest comparison
/// is headers against headers, the M108/M110 denominator shape).
const SORT_CASE: &str = "sort/snapshot_1k_sort";
const HEADERS_BARE_CASE: &str = "sort/snapshot_1k_headers_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's sort door: the mutate seam the app's code rides — the
/// word moved, `PAINT | A11Y` requested on the header, the frames the
/// re-announcement rides settled before the read. `None` clears the
/// word — the honest absence restored.
fn set_sort(
    harness: &mut TestHarness,
    node_id: alacer_core::NodeId,
    sort_direction: Option<SortDirection>,
) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(header) = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<ColumnHeader>()
    {
        header.set_sort(sort_direction);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m112_the_sort_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 320.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The worded headers, mounted through the constructor word: the
    // order the tree says from the first frame — the column that sorts
    // ascending, the column that sorts descending, the column that
    // sorts by an order ARIA names no word for — beside the plain
    // header, the honest absence the tier keeps honest (ARIA's `none`,
    // never a guessed direction).
    let price = ColumnHeader::new("Price", 0).sort(SortDirection::Ascending);
    let _ = harness.add_child(root, price.boxed());
    let name = ColumnHeader::new("Name", 1).sort(SortDirection::Descending);
    let _ = harness.add_child(root, name.boxed());
    let added = ColumnHeader::new("Added", 2).sort(SortDirection::Other);
    let _ = harness.add_child(root, added.boxed());
    let plain = ColumnHeader::new("Notes", 3);
    let _ = harness.add_child(root, plain.boxed());
    // The collection member and the plain widgets beside them: no
    // order to say — the word is a header word, the row's position the
    // M6 slot's own fact.
    let _ = harness.add_child(root, TableRow::new().boxed());
    let _ = harness.add_child(root, Button::new("Sort").boxed());
    harness.settle();

    let price = node(&harness, Role::ColumnHeader, "Price").expect("the ascending header rides");
    assert_eq!(
        price.sort_direction,
        Some(SortDirection::Ascending),
        "the constructor's word is the tree's truth — the ascending column said"
    );
    let name = node(&harness, Role::ColumnHeader, "Name").expect("the descending header rides");
    assert_eq!(
        name.sort_direction,
        Some(SortDirection::Descending),
        "the order the word carries is the column's own"
    );
    let added = node(&harness, Role::ColumnHeader, "Added").expect("the other header rides");
    assert_eq!(
        added.sort_direction,
        Some(SortDirection::Other),
        "the order ARIA names no word for rides the same word"
    );
    let plain = node(&harness, Role::ColumnHeader, "Notes").expect("the plain header rides");
    assert!(
        plain.sort_direction.is_none(),
        "a header that declared nothing answers no order"
    );
    let row = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| {
            tree.nodes
                .iter()
                .find(|n| n.role == Role::TableRow)
                .cloned()
        })
        .expect("the row rides");
    assert!(
        row.sort_direction.is_none(),
        "a member role inherits the honest absence — the word is a header word"
    );
    let sort = node(&harness, Role::Button, "Sort").expect("the button rides");
    assert!(
        sort.sort_direction.is_none(),
        "a non-header role inherits the honest absence"
    );

    // The word prices nothing but the word: the sorted table's
    // neighbour's write door answers exactly as any button's.
    assert!(
        harness.act(Role::Button, "Sort", A11yAction::DefaultPress),
        "the sort word is not a wall — the column's neighbour answers as ever"
    );
    harness.assert_all_named();
}

#[test]
fn m112_the_sort_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 160.0);
    let root = harness.mount(Column::new().boxed());
    let header = ColumnHeader::new("Score", 0);
    let header_id = harness.add_child(root, header.boxed());
    harness.settle();

    // The live door: the app's column order moved, the re-announcement
    // the mutate seam's request paid for — the kind riding beside the
    // word, and the honest absence restored on the clear.
    set_sort(&mut harness, header_id, Some(SortDirection::Descending));
    let ranked = node(&harness, Role::ColumnHeader, "Score").expect("the header survives");
    assert_eq!(
        ranked.sort_direction,
        Some(SortDirection::Descending),
        "the app's door moves the word the tree says, kind beside it"
    );
    set_sort(&mut harness, header_id, Some(SortDirection::Ascending));
    let reworked = node(&harness, Role::ColumnHeader, "Score").expect("the header survives");
    assert_eq!(
        reworked.sort_direction,
        Some(SortDirection::Ascending),
        "the same door moves the kind — the AT reads the new order"
    );
    set_sort(&mut harness, header_id, None);
    let cleared = node(&harness, Role::ColumnHeader, "Score").expect("the header survives");
    assert!(
        cleared.sort_direction.is_none(),
        "the same door carries the word away — the honest absence restored"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(header_id);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<ColumnHeader>()
        .expect("the header survives")
        .set_sort(None);
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m112_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the sort route's pair: the
    // 1,000-node header arena carrying the sort word — the word's cost
    // the measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(SORT_CASE),
        "the sort route's case is published ({SORT_CASE})"
    );
    assert!(
        benches.contains_key(HEADERS_BARE_CASE),
        "the sort route's bare denominator is published ({HEADERS_BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M112 cut"
    );

    // The contract set gates the pair and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    let gate = gates
        .iter()
        .find(|gate| gate["baseline"].as_str() == Some(SORT_CASE))
        .unwrap_or_else(|| panic!("the case carries a regression gate ({SORT_CASE})"));
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
        .find(|budget| budget["id"].as_str() == Some("sort_snapshot_overhead"))
        .expect("the sort budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(SORT_CASE),
        "the budget prices the sort route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(HEADERS_BARE_CASE),
        "the budget prices it against the same header arena, bare"
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
        "the m5 corpus pin re-agrees at the M112 cut"
    );
}
