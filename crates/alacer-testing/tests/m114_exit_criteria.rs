//! M114 exit-criteria integration tests (§10.3, ADR-0118): the current
//! tier — the set's current item, said by the tree. Through M113 the
//! snapshot said where a member sits in its grid (the M6 table slot),
//! what a control *does* (the description, ADR-0099), what an empty
//! field *hints* (the placeholder, ADR-0100), the span a value rides
//! (the range tier, ADR-0101), the form's ask and the form's rejection
//! (the required and invalid tiers, ADR-0110/ADR-0111), the collection's
//! pick, capacity and disclosure (ADR-0112/ADR-0113/ADR-0114), the
//! field's suggestion contract (the auto-complete tier, ADR-0115), the
//! sorted column's order (the sort direction tier, ADR-0116), the nested
//! member's depth (the level tier, ADR-0117) — and the word every
//! navigation and wizard vocabulary speaks about which item the reader
//! stands in ("this one is the page we're on, that step is where we
//! are") had no tier: the AT announced the name, the position the M6
//! slot carried, the form facts — and never the currency — the
//! declared-but-unreported gap (the M94 pattern), cashed at the current
//! tier: the word grows beside the collection words it stands among —
//! [`alacer_core::Widget::current`] beside
//! [`alacer_widgets::Button::current`], AccessKit's `AriaCurrent`
//! property the sink maps in the same snapshot every other property
//! rides.
//!
//! The tier prices the word and nothing else: a current item's doors
//! answer exactly as they always did — the word names the currency, it
//! walls nothing, the item's press and focus routes the widget's own.
//! The kind mirrors AccessKit's own enum values (`True` / `Page` /
//! `Step` / `Location` / `Date` / `Time`, one branch with no
//! translation), ARIA's `false` the absence spelled by `None` — an
//! item that declared nothing announces no currency, never a guessed
//! `false`.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the word's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, CurrentKind, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, ColumnHeader, TableRow};

/// The suite's corpus pin: the coordinated rule's count at the M114 cut —
/// the current route's pair joining the level route's case (M113), the
/// sort route's pair (M112), the auto-complete route's case (M111), the
/// expanded route's pair (M110), the multiselectable route's pair (M109),
/// the selected route's pair (M108), the invalid route's case (M107) and
/// every tier's before them on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 208;

/// The suite's own read door into the published corpus: the current
/// route's pair — the 1,000-node button arena carrying the currency
/// word, priced against the same 1,000-node button arena bare (the
/// M95 pricing rule: the word is a button word, so the honest
/// comparison is buttons against buttons, the M108/M110 denominator
/// shape).
const CURRENT_CASE: &str = "current/snapshot_1k_currents";
const BUTTONS_BARE_CASE: &str = "current/snapshot_1k_buttons_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's current door: the mutate seam the app's code rides — the
/// word moved, `PAINT | A11Y` requested on the item, the frames the
/// re-announcement rides settled before the read. `None` clears the
/// word — the honest absence restored.
fn set_current(
    harness: &mut TestHarness,
    node_id: alacer_core::NodeId,
    current: Option<CurrentKind>,
) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(button) = guard.widget_mut().as_any_mut().downcast_mut::<Button>() {
        button.set_current(current);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m114_the_current_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 320.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The worded items, mounted through the constructor word: the
    // currency the tree says from the first frame — the nav's page, the
    // wizard's step, the calendar's date — beside the plain item, the
    // honest absence the tier keeps honest (the ordinary button, never
    // a guessed `false`).
    let _ = harness.add_child(root, Button::new("Home").current(CurrentKind::Page).boxed());
    let _ = harness.add_child(
        root,
        Button::new("Step 2 of 4")
            .current(CurrentKind::Step)
            .boxed(),
    );
    let _ = harness.add_child(
        root,
        Button::new("Oct 14").current(CurrentKind::Date).boxed(),
    );
    let _ = harness.add_child(root, Button::new("Send").boxed());
    // The header and the row beside them: no currency to say — the word
    // is an item word, the container and member words the tiers' own
    // facts.
    let _ = harness.add_child(root, ColumnHeader::new("Price", 0).boxed());
    let _ = harness.add_child(root, TableRow::new().boxed());
    harness.settle();

    let buttons: Vec<TreeNode> = harness
        .pipeline()
        .a11y_snapshot()
        .map(|tree| {
            tree.nodes
                .iter()
                .filter(|n| n.role == Role::Button)
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(
        buttons.len(),
        4,
        "the four items ride the snapshot the way the harness mounted them"
    );
    assert_eq!(
        buttons[0].current,
        Some(CurrentKind::Page),
        "the nav's page said — AccessKit's own kind, one branch with no translation"
    );
    assert_eq!(
        buttons[1].current,
        Some(CurrentKind::Step),
        "the wizard's step said the currency the constructor named"
    );
    assert_eq!(
        buttons[2].current,
        Some(CurrentKind::Date),
        "the calendar's date said the currency the constructor named"
    );
    assert!(
        buttons[3].current.is_none(),
        "an item that declared nothing answers no currency — the ordinary button, never a guessed `false`"
    );
    let header = node(&harness, Role::ColumnHeader, "Price").expect("the header rides");
    assert!(
        header.current.is_none(),
        "a header role inherits the honest absence — the word is an item word"
    );
    let row = node(&harness, Role::TableRow, "").expect("the row rides");
    assert!(
        row.current.is_none(),
        "a member role inherits the honest absence — the word is an item word"
    );

    // The word prices nothing but the word: the plain item's write door
    // answers exactly as any button's.
    assert!(
        harness.act(Role::Button, "Send", A11yAction::DefaultPress),
        "the current word is not a wall — the plain item answers as ever"
    );
    harness.assert_all_named();
}

#[test]
fn m114_the_current_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 160.0);
    let root = harness.mount(Column::new().boxed());
    let button = Button::new("Page 1");
    let button_id = harness.add_child(root, button.boxed());
    harness.settle();

    // The live door: the app's currency moved, the re-announcement the
    // mutate seam's request paid for — the kind riding the word, and
    // the honest absence restored on the clear.
    set_current(&mut harness, button_id, Some(CurrentKind::Page));
    let paged = node(&harness, Role::Button, "Page 1").expect("the item survives");
    assert_eq!(
        paged.current,
        Some(CurrentKind::Page),
        "the app's door moves the word the tree says, the kind beside it"
    );
    set_current(&mut harness, button_id, Some(CurrentKind::Step));
    let stepped = node(&harness, Role::Button, "Page 1").expect("the item survives");
    assert_eq!(
        stepped.current,
        Some(CurrentKind::Step),
        "the same door moves the currency — the AT reads the new kind"
    );
    set_current(&mut harness, button_id, None);
    let cleared = node(&harness, Role::Button, "Page 1").expect("the item survives");
    assert!(
        cleared.current.is_none(),
        "the same door carries the word away — the honest absence restored"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(button_id);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Button>()
        .expect("the item survives")
        .set_current(None);
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m114_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the current route's pair: the
    // 1,000-node button arena carrying the currency word — the word's
    // cost the measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(CURRENT_CASE),
        "the current route's case is published ({CURRENT_CASE})"
    );
    assert!(
        benches.contains_key(BUTTONS_BARE_CASE),
        "the tier's bare denominator is published ({BUTTONS_BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M114 cut"
    );

    // The contract set gates the case and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    let gate = gates
        .iter()
        .find(|gate| gate["baseline"].as_str() == Some(CURRENT_CASE))
        .unwrap_or_else(|| panic!("the case carries a regression gate ({CURRENT_CASE})"));
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
        .find(|budget| budget["id"].as_str() == Some("current_snapshot_overhead"))
        .expect("the current budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(CURRENT_CASE),
        "the budget prices the current route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(BUTTONS_BARE_CASE),
        "the budget prices it against the same button arena, bare"
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
        "the m5 corpus pin re-agrees at the M114 cut"
    );
}
