//! M110 exit-criteria integration tests (§10.3, ADR-0114): the
//! expanded tier — the collection member's disclosure, said by the
//! tree. Through M109 the snapshot said which member the collection
//! picked (the selected tier, ADR-0112) beside the capacity the
//! multiselectable tier named (ADR-0113), the form's ask and the form's
//! rejection beside them — and the word every collection vocabulary
//! speaks about the member that opens and closes ("this one is
//! expanded, that one collapsed") had no tree tier: the AT announced
//! the picks, the positions, the names — and never the disclosure —
//! the declared-but-unreported gap (the M94 pattern), cashed at the
//! expanded tier: the word grows beside the member words it explains —
//! [`alacer_core::Widget::expanded`] beside
//! [`alacer_widgets::TableRow::expanded`], AccessKit's `Expanded`
//! property (ARIA's `aria-expanded`) the sink maps in the same snapshot
//! every other property rides.
//!
//! The tier prices the word and nothing else: an expanded member's
//! doors answer exactly as they always did — the word names state, it
//! walls nothing, the open/close routes the widgets' own. The trio is
//! the honest shape (the ADR-0087 presence rule at the tier, the
//! selected shape): `None` the member that opens and closes not — the
//! absence ARIA spells `undefined`, never a guessed `false`;
//! `Some(false)` the member standing collapsed; `Some(true)` the
//! member standing open. AccessKit's own contract names the mapping
//! ("setting this to `false` means the node is collapsed; omitting it
//! means this state isn't applicable") — the tier mirrors it verbatim.
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

/// The suite's corpus pin: the coordinated rule's count at the M110 cut —
/// the expanded route's pair joining the multiselectable route's pair
/// (M109), the selected route's pair (M108), the invalid route's case
/// (M107), the required route's case (M106) and every tier's before them
/// on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 140;

/// The suite's own read door into the published corpus: the expanded
/// route's pair — the 1,000-node row arena carrying the disclosure word,
/// against the same arena bare. The ratio's honest denominator is the
/// same row arena with the word absent (the M95 pricing rule: the same
/// corpus measuring what the tier adds — the word is a member word, so
/// the honest comparison is rows against rows).
const EXPANDED_CASE: &str = "expanded/snapshot_1k_expanded";
const ROWS_BARE_CASE: &str = "expanded/snapshot_1k_rows_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's expanded door: the mutate seam the app's code rides — the
/// word moved, `PAINT | A11Y` requested on the row, the frames the
/// re-announcement rides settled before the read. `None` clears the
/// word — the honest absence restored.
fn set_row_expanded(
    harness: &mut TestHarness,
    node_id: alacer_core::NodeId,
    expanded: Option<bool>,
) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(row) = guard.widget_mut().as_any_mut().downcast_mut::<TableRow>() {
        row.set_expanded(expanded);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m110_the_expanded_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The collection members, mounted through the constructor word: the
    // disclosure the tree says from the first frame — the row standing
    // open, the row standing collapsed, the wordless row between them —
    // the trio the AT reads while it walks the rows.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let folders = Table::new("Folders", 1);
    let folders_id = harness.add_child(root, folders.boxed());
    let open = harness.add_child(folders_id, TableRow::new().expanded(true).boxed());
    let _ = harness.add_child(open, TableCell::new("Archive", 0, 0).boxed());
    let shut = harness.add_child(folders_id, TableRow::new().expanded(false).boxed());
    let _ = harness.add_child(shut, TableCell::new("Drafts", 1, 0).boxed());
    let wordless = harness.add_child(folders_id, TableRow::new().boxed());
    let _ = harness.add_child(wordless, TableCell::new("Spam", 2, 0).boxed());
    // The plain widgets beside the collection: no disclosure word to say.
    let _ = harness.add_child(root, Column::new().gap(4.0).boxed());
    let _ = harness.add_child(root, TextInput::new("Note").boxed());
    let _ = harness.add_child(root, Label::new("Plain").boxed());
    let _ = harness.add_child(root, Button::new("Save").boxed());
    harness.settle();

    // The rows read through their cells (the nameless-container rule),
    // so the disclosure word is found by walking the snapshot's rows in
    // tree order: the open row, the collapsed row, the wordless row.
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
        3,
        "the collection's three members ride the snapshot"
    );
    assert_eq!(
        rows[0].expanded,
        Some(true),
        "the constructor's word is the tree's truth — the open row said"
    );
    assert_eq!(
        rows[1].expanded,
        Some(false),
        "the collapsed row's word is the explicit `false`, never the absence"
    );
    assert_eq!(
        rows[2].expanded, None,
        "a member that opens and closes not announces no disclosure — the honest absence"
    );
    // The word is a member word, not a container or widget word: the
    // plain widgets beside the collection inherit the absence.
    let note = node(&harness, Role::TextInput, "Note").expect("the plain field rides");
    assert!(
        note.expanded.is_none(),
        "a field that declared nothing answers no disclosure word"
    );
    let save = node(&harness, Role::Button, "Save").expect("the button rides");
    assert!(
        save.expanded.is_none(),
        "a non-member role inherits the honest absence"
    );
    // The word prices nothing but the word: the worded collection's
    // neighbours answer exactly as they always did — the disclosure
    // names state, it walls nothing.
    assert!(
        harness.act(Role::Button, "Save", A11yAction::DefaultPress),
        "the press route answers unchanged beside the member's word"
    );
    harness.assert_all_named();
}

#[test]
fn m110_the_expanded_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let folders = Table::new("Folders", 1);
    let folders_id = harness.add_child(root, folders.boxed());
    let row = harness.add_child(folders_id, TableRow::new().boxed());
    let _ = harness.add_child(row, TableCell::new("Dusk", 0, 0).boxed());
    harness.settle();

    // The live door: the app's disclosure moved, the re-announcement the
    // mutate seam's request paid for — and the honest absence restored
    // on the clear.
    set_row_expanded(&mut harness, row, Some(true));
    let opened = harness.pipeline().a11y_snapshot().and_then(|tree| {
        tree.nodes
            .iter()
            .find(|n| n.role == Role::TableRow)
            .cloned()
    });
    assert_eq!(
        opened.and_then(|row| row.expanded),
        Some(true),
        "the app's door moves the word the tree says"
    );
    set_row_expanded(&mut harness, row, Some(false));
    let collapsed = harness.pipeline().a11y_snapshot().and_then(|tree| {
        tree.nodes
            .iter()
            .find(|n| n.role == Role::TableRow)
            .cloned()
    });
    assert_eq!(
        collapsed.and_then(|row| row.expanded),
        Some(false),
        "the same door carries the collapsed word — the trio's explicit false"
    );
    set_row_expanded(&mut harness, row, None);
    let cleared = harness.pipeline().a11y_snapshot().and_then(|tree| {
        tree.nodes
            .iter()
            .find(|n| n.role == Role::TableRow)
            .cloned()
    });
    assert_eq!(
        cleared.and_then(|row| row.expanded),
        None,
        "the same door carries the word away — the honest absence restored"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(row);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<TableRow>()
        .expect("the row survives")
        .set_expanded(None);
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m110_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the expanded route's pair: the
    // 1,000-node row arena carrying the disclosure word — the word's
    // cost the measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(EXPANDED_CASE) && benches.contains_key(ROWS_BARE_CASE),
        "the expanded route's pair is published ({EXPANDED_CASE}, {ROWS_BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M110 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [EXPANDED_CASE, ROWS_BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("expanded_snapshot_overhead"))
        .expect("the expanded budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(EXPANDED_CASE),
        "the budget prices the expanded route"
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
        "the m5 corpus pin re-agrees at the M110 cut"
    );
}
