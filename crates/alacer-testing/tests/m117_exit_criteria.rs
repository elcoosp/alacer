//! M117 exit-criteria integration tests (§10.3, ADR-0121): the busy
//! tier — the subtree's updating fact, said by the tree. Through M116
//! the snapshot said what a control is read as (the role-description
//! tier, ADR-0120), the opener's target (ADR-0119), the set's current
//! item (ADR-0118), the nested member's depth (ADR-0117), the sorted
//! column's order (ADR-0116), the field's suggestion contract
//! (ADR-0115), the member's disclosure (ADR-0114), the container's
//! capacity (ADR-0113), the member's pick (ADR-0112), the form's
//! rejection (ADR-0111), the form's ask (ADR-0110), the field's secret
//! entry (ADR-0104) — and the word every bulk-update vocabulary speaks
//! about the region under construction ("this list is still loading,
//! hold what you would announce") had no tree tier: the AT announced
//! every half-settled intermediate the stream wrote into the subtree —
//! the declared-but-unreported gap (the M94 pattern), cashed at the
//! busy tier: the word grows beside the announcement machinery it
//! paces — [`alacer_core::Widget::busy`] beside
//! [`alacer_widgets::List::busy`] and [`alacer_widgets::Table::busy`],
//! AccessKit's `Busy` property (ARIA's `aria-busy`) the sink maps in
//! the same snapshot every other property rides.
//!
//! The tier prices the word and nothing else: a busy region's doors
//! answer exactly as they always did — the word names the subtree's
//! updating fact, it walls nothing, the children's routes the widgets'
//! own. `false` is the honest absence (the ADR-0087 presence rule at
//! the tier, the multiselectable shape): a container that declared
//! nothing announces nothing, never a guessed "updating".
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the word's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{
    BoxedWidget as _, Button, Column, Label, List, Table, TableCell, TableRow, TextInput,
};

/// The suite's corpus pin: the coordinated rule's count at the M117 cut —
/// the busy route's pair joining the role-description route's case
/// (M116), the has-popup route's case (M115), the current route's pair
/// (M114), the level route's case (M113), the sort route's pair (M112),
/// the auto-complete route's case (M111), the expanded route's pair
/// (M110), the multiselectable route's pair (M109) and every tier's
/// before them on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 165;

/// The suite's own read door into the published corpus: the busy
/// route's pair — the 1,000-node list arena carrying the updating word,
/// against the same arena bare. The ratio's honest denominator is the
/// same list arena with the word absent (the M95 pricing rule: the
/// same corpus measuring what the tier adds; the word is a container
/// word, so the honest comparison is lists against lists — the
/// M108/M109 denominator shape).
const BUSY_CASE: &str = "busy/snapshot_1k_lists_busy";
const LISTS_BARE_CASE: &str = "busy/snapshot_1k_lists_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's busy door: the mutate seam the app's code rides — the word
/// moved, `PAINT | A11Y` requested on the container, the frames the
/// re-announcement rides settled before the read.
fn set_container_busy(harness: &mut TestHarness, node_id: alacer_core::NodeId, busy: bool) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(list) = guard.widget_mut().as_any_mut().downcast_mut::<List>() {
        list.set_busy(busy);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m117_the_busy_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The updating regions, mounted through the constructor word: the
    // fact the tree says from the first frame — the list still filling,
    // the table still streaming — beside the settled containers, the
    // honest absence the tier keeps honest.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let feed = List::new(80.0).busy(true);
    let feed_id = harness.add_child(root, feed.boxed());
    let _ = harness.add_child(feed_id, Label::new("Row 1").boxed());
    let ledger = Table::new("Ledger", 2).busy(true);
    let ledger_id = harness.add_child(root, ledger.boxed());
    let ledger_row = harness.add_child(ledger_id, TableRow::new().boxed());
    let _ = harness.add_child(ledger_row, TableCell::new("Entry 1", 0, 0).boxed());
    let archive = Table::new("Archive", 2);
    let archive_id = harness.add_child(root, archive.boxed());
    let archive_row = harness.add_child(archive_id, TableRow::new().boxed());
    let _ = harness.add_child(archive_row, TableCell::new("Entry 2", 0, 0).boxed());
    // The plain widgets beside them: no updating word to say.
    let _ = harness.add_child(root, Button::new("Save").boxed());
    let _ = harness.add_child(root, TextInput::new("Note").boxed());
    harness.settle();

    let feed = node(&harness, Role::ScrollArea, "list").expect("the busy list rides");
    assert!(
        feed.busy,
        "the constructor's word is the tree's truth — the updating fact said"
    );
    let ledger = node(&harness, Role::Table, "Ledger").expect("the busy table rides");
    assert!(
        ledger.busy,
        "the table's constructor word lands in the same snapshot — the tier's second producer"
    );
    let archive = node(&harness, Role::Table, "Archive").expect("the settled table rides");
    assert!(
        !archive.busy,
        "a container that declared nothing announces no updating fact — the honest absence"
    );
    let save = node(&harness, Role::Button, "Save").expect("the button rides");
    assert!(
        !save.busy,
        "a non-container role inherits the honest absence"
    );
    let note = node(&harness, Role::TextInput, "Note").expect("the plain field rides");
    assert!(
        !note.busy,
        "a field that declared nothing answers no updating word"
    );
    // The word prices nothing but the word: the worded region's
    // neighbours answer exactly as they always did — the updating fact
    // names state, it walls nothing.
    assert!(
        harness.act(Role::Button, "Save", A11yAction::DefaultPress),
        "the press route answers unchanged beside the container's word"
    );
    harness.assert_all_named();
}

#[test]
fn m117_the_busy_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let feed = List::new(80.0);
    let feed_id = harness.add_child(root, feed.boxed());
    let _ = harness.add_child(feed_id, Label::new("Row 1").boxed());
    harness.settle();

    // The live door: the app's updating fact moved, the re-announcement
    // the mutate seam's request paid for — and the honest absence
    // restored on the clear (the bulk update finished, the AT speaks).
    set_container_busy(&mut harness, feed_id, true);
    let loading = node(&harness, Role::ScrollArea, "list").expect("the list survives");
    assert!(loading.busy, "the app's door moves the word the tree says");
    set_container_busy(&mut harness, feed_id, false);
    let settled = node(&harness, Role::ScrollArea, "list").expect("the list survives");
    assert!(
        !settled.busy,
        "the same door carries the word away — the honest absence restored"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(feed_id);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<List>()
        .expect("the list survives")
        .set_busy(false);
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m117_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the busy route's pair: the
    // 1,000-node list arena carrying the updating word — the word's
    // cost the measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(BUSY_CASE) && benches.contains_key(LISTS_BARE_CASE),
        "the busy route's pair is published ({BUSY_CASE}, {LISTS_BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M117 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [BUSY_CASE, LISTS_BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("busy_snapshot_overhead"))
        .expect("the busy budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(BUSY_CASE),
        "the budget prices the busy route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(LISTS_BARE_CASE),
        "the budget prices it against the same list arena, bare"
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
        "the m5 corpus pin re-agrees at the M117 cut"
    );
}
