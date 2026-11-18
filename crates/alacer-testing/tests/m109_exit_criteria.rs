//! M109 exit-criteria integration tests (§10.3, ADR-0113): the
//! multiselectable tier — the collection's capacity, said by the tree.
//! Through M108 the snapshot said which member the collection picked
//! (the selected tier, ADR-0112) beside the position the M6 slot
//! carried — and the word every collection vocabulary speaks about the
//! container ("this one allows more than one pick") had no tree tier:
//! the AT announced the picks, the positions, the names — and never
//! the capacity. The declared-but-unreported gap (the M94 pattern)
//! cashed at the multiselectable tier: the word grows beside the
//! member word it explains —
//! [`alacer_core::Widget::multiselectable`] beside
//! [`alacer_widgets::Table::multiselectable`], AccessKit's
//! `Multiselectable` property (ARIA's `aria-multiselectable`) the sink
//! maps in the same snapshot every other property rides.
//!
//! The tier prices the word and nothing else: a multiselectable
//! container's doors answer exactly as they always did — the word
//! names capacity, it walls nothing, the members' routes the widgets'
//! own. `false` is the honest absence (the ADR-0087 presence rule at
//! the tier, the read-only shape): a container that declared nothing
//! announces nothing, never a guessed capacity.
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

/// The suite's corpus pin: the coordinated rule's count at the M109 cut —
/// the multiselectable route's pair joining the selected route's pair
/// (M108), the invalid route's case (M107), the required route's case
/// (M106) and every tier's before them on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 131;

/// The suite's own read door into the published corpus: the
/// multiselectable route's pair — the 1,000-node table arena carrying
/// the capacity word, against the same arena bare. The ratio's honest
/// denominator is the same table arena with the word absent (the M95
/// pricing rule: the same corpus measuring what the tier adds).
const MULTISELECTABLE_CASE: &str = "multiselectable/snapshot_1k_multiselectable";
const TABLES_BARE_CASE: &str = "multiselectable/snapshot_1k_tables_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's multiselectable door: the mutate seam the app's code
/// rides — the word moved, `PAINT | A11Y` requested on the container,
/// the frames the re-announcement rides settled before the read.
fn set_table_multiselectable(
    harness: &mut TestHarness,
    node_id: alacer_core::NodeId,
    multiselectable: bool,
) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(table) = guard.widget_mut().as_any_mut().downcast_mut::<Table>() {
        table.set_multiselectable(multiselectable);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m109_the_multiselectable_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The multi-pick collections, mounted through the constructor word:
    // the capacity the tree says from the first frame — the table whose
    // rows pick many — beside the single-pick table, the honest absence
    // the tier keeps honest.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let leads = Table::new("Leads", 2).multiselectable(true);
    let leads_id = harness.add_child(root, leads.boxed());
    let row = harness.add_child(leads_id, TableRow::new().selected(true).boxed());
    let _ = harness.add_child(row, TableCell::new("Ada", 0, 0).boxed());
    let single = Table::new("Themes", 1);
    let single_id = harness.add_child(root, single.boxed());
    let single_row = harness.add_child(single_id, TableRow::new().boxed());
    let _ = harness.add_child(single_row, TableCell::new("Dusk", 0, 0).boxed());
    // The plain containers beside them: no capacity word to say.
    let _ = harness.add_child(root, Column::new().gap(4.0).boxed());
    let _ = harness.add_child(root, TextInput::new("Note").boxed());
    let _ = harness.add_child(root, Label::new("Plain").boxed());
    let _ = harness.add_child(root, Button::new("Save").boxed());
    harness.settle();

    let leads = node(&harness, Role::Table, "Leads").expect("the multi-pick table rides");
    assert!(
        leads.multiselectable,
        "the constructor's word is the tree's truth — the capacity said"
    );
    let themes = node(&harness, Role::Table, "Themes").expect("the single-pick table rides");
    assert!(
        !themes.multiselectable,
        "a container that declared nothing announces no capacity — the honest absence"
    );
    // The word is a container word, not a member or widget word: the
    // plain widgets beside the collections inherit the absence.
    let note = node(&harness, Role::TextInput, "Note").expect("the plain field rides");
    assert!(
        !note.multiselectable,
        "a field that declared nothing answers no capacity word"
    );
    let save = node(&harness, Role::Button, "Save").expect("the button rides");
    assert!(
        !save.multiselectable,
        "a non-collection role inherits the honest absence"
    );
    // The word prices nothing but the word: the worded collection's
    // neighbours answer exactly as they always did — the capacity names
    // state, it walls nothing.
    assert!(
        harness.act(Role::Button, "Save", A11yAction::DefaultPress),
        "the press route answers unchanged beside the container's word"
    );
    harness.assert_all_named();
}

#[test]
fn m109_the_multiselectable_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let themes = Table::new("Themes", 1);
    let themes_id = harness.add_child(root, themes.boxed());
    let row = harness.add_child(themes_id, TableRow::new().boxed());
    let _ = harness.add_child(row, TableCell::new("Dusk", 0, 0).boxed());
    harness.settle();

    // The live door: the app's capacity moved, the re-announcement the
    // mutate seam's request paid for — and the honest absence restored
    // on the clear.
    set_table_multiselectable(&mut harness, themes_id, true);
    let marked = node(&harness, Role::Table, "Themes").expect("the table survives");
    assert!(
        marked.multiselectable,
        "the app's door moves the word the tree says"
    );
    set_table_multiselectable(&mut harness, themes_id, false);
    let cleared = node(&harness, Role::Table, "Themes").expect("the table survives");
    assert!(
        !cleared.multiselectable,
        "the same door carries the word away — the honest absence restored"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(themes_id);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Table>()
        .expect("the table survives")
        .set_multiselectable(false);
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m109_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the multiselectable route's pair:
    // the 1,000-node table arena carrying the capacity word — the
    // word's cost the measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(MULTISELECTABLE_CASE) && benches.contains_key(TABLES_BARE_CASE),
        "the multiselectable route's pair is published ({MULTISELECTABLE_CASE}, {TABLES_BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M109 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [MULTISELECTABLE_CASE, TABLES_BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("multiselectable_snapshot_overhead"))
        .expect("the multiselectable budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(MULTISELECTABLE_CASE),
        "the budget prices the multiselectable route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(TABLES_BARE_CASE),
        "the budget prices it against the same table arena, bare"
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
        "the m5 corpus pin re-agrees at the M109 cut"
    );
}
