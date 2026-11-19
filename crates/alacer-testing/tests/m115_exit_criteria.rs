//! M115 exit-criteria integration tests (§10.3, ADR-0119): the
//! has-popup tier — the opener's target, said by the tree. Through
//! M114 the snapshot said what a control *does* (the description,
//! ADR-0099), what an empty field *hints* (the placeholder,
//! ADR-0100), the span a value rides (the range tier, ADR-0101), the
//! form's ask and the form's rejection (the required and invalid
//! tiers, ADR-0110/ADR-0111), the collection's pick, capacity and
//! disclosure (ADR-0112/ADR-0113/ADR-0114), the field's suggestion
//! contract (the auto-complete tier, ADR-0115), the sorted column's
//! order (the sort direction tier, ADR-0116), the nested member's
//! depth (the level tier, ADR-0117), the set's current item (the
//! current tier, ADR-0118) — and the word every menu-and-picker
//! vocabulary speaks about what a control opens ("this one opens a
//! menu, that one a dialog") had no tier: the AT announced the name,
//! the currency, the form facts — and never the target — the
//! declared-but-unreported gap (the M94 pattern), cashed at the
//! has-popup tier: the word grows beside the item words it opens
//! beside — [`alacer_core::Widget::has_popup`] beside
//! [`alacer_widgets::Button::has_popup`], AccessKit's `HasPopup`
//! property the sink maps in the same snapshot every other property
//! rides.
//!
//! The tier prices the word and nothing else: a popup opener's doors
//! answer exactly as they always did — the word names the target, it
//! walls nothing, the opener's press and focus routes the widget's
//! own. The kind mirrors AccessKit's own enum values (`Menu` /
//! `Listbox` / `Tree` / `Grid` / `Dialog`, one branch with no
//! translation), the absent attribute the absence spelled by `None` —
//! a control that opens nothing announces no target, never a guessed
//! one.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the word's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, HasPopupKind, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, ColumnHeader, TableRow};

/// The suite's corpus pin: the coordinated rule's count at the M115 cut —
/// the has-popup route's case joining the current route's pair (M114),
/// the level route's case (M113), the sort route's pair (M112), the
/// auto-complete route's case (M111), the expanded route's pair (M110),
/// the multiselectable route's pair (M109), the selected route's pair
/// (M108), the invalid route's case (M107) and every tier's before
/// them on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 144;

/// The suite's own read door into the published corpus: the has-popup
/// route's case — the 1,000-node button arena carrying the target
/// word — priced against the same 1,000-node button arena bare
/// (`current/snapshot_1k_buttons_bare`, the M114 case the M111
/// precedent reuses; the M95 pricing rule — the word is a button word,
/// so the honest comparison is buttons against buttons, the
/// M108/M110 denominator shape).
const HAS_POPUP_CASE: &str = "has_popup/snapshot_1k_has_popups";
const BUTTONS_BARE_CASE: &str = "current/snapshot_1k_buttons_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's has-popup door: the mutate seam the app's code rides — the
/// word moved, `PAINT | A11Y` requested on the opener, the frames the
/// re-announcement rides settled before the read. `None` clears the
/// word — the honest absence restored.
fn set_has_popup(
    harness: &mut TestHarness,
    node_id: alacer_core::NodeId,
    has_popup: Option<HasPopupKind>,
) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(button) = guard.widget_mut().as_any_mut().downcast_mut::<Button>() {
        button.set_has_popup(has_popup);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m115_the_has_popup_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 320.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The worded openers, mounted through the constructor word: the
    // target the tree says from the first frame — the menu button, the
    // combobox button, the tree toggle, the grid picker, the dialog
    // opener — beside the plain button, the honest absence the tier
    // keeps honest (the control that opens nothing, never a guessed
    // target).
    let _ = harness.add_child(
        root,
        Button::new("Actions").has_popup(HasPopupKind::Menu).boxed(),
    );
    let _ = harness.add_child(
        root,
        Button::new("Pick a color")
            .has_popup(HasPopupKind::Listbox)
            .boxed(),
    );
    let _ = harness.add_child(
        root,
        Button::new("Browse folders")
            .has_popup(HasPopupKind::Tree)
            .boxed(),
    );
    let _ = harness.add_child(
        root,
        Button::new("Pick a date")
            .has_popup(HasPopupKind::Grid)
            .boxed(),
    );
    let _ = harness.add_child(
        root,
        Button::new("Confirm")
            .has_popup(HasPopupKind::Dialog)
            .boxed(),
    );
    let _ = harness.add_child(root, Button::new("Send").boxed());
    // The header and the row beside them: no target to say — the word
    // is an opener word, the container and member words the tiers' own
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
        6,
        "the six controls ride the snapshot the way the harness mounted them"
    );
    assert_eq!(
        buttons[0].has_popup,
        Some(HasPopupKind::Menu),
        "the menu button said — AccessKit's own kind, one branch with no translation"
    );
    assert_eq!(
        buttons[1].has_popup,
        Some(HasPopupKind::Listbox),
        "the combobox button said the target the constructor named"
    );
    assert_eq!(
        buttons[2].has_popup,
        Some(HasPopupKind::Tree),
        "the tree toggle said the target the constructor named"
    );
    assert_eq!(
        buttons[3].has_popup,
        Some(HasPopupKind::Grid),
        "the grid picker said the target the constructor named"
    );
    assert_eq!(
        buttons[4].has_popup,
        Some(HasPopupKind::Dialog),
        "the dialog opener said the target the constructor named"
    );
    assert!(
        buttons[5].has_popup.is_none(),
        "a control that opens nothing answers no target — the ordinary button, never a guessed one"
    );
    let header = node(&harness, Role::ColumnHeader, "Price").expect("the header rides");
    assert!(
        header.has_popup.is_none(),
        "a header role inherits the honest absence — the word is an opener word"
    );
    let row = node(&harness, Role::TableRow, "").expect("the row rides");
    assert!(
        row.has_popup.is_none(),
        "a member role inherits the honest absence — the word is an opener word"
    );

    // The word prices nothing but the word: the plain control's write
    // door answers exactly as any button's.
    assert!(
        harness.act(Role::Button, "Send", A11yAction::DefaultPress),
        "the has-popup word is not a wall — the plain control answers as ever"
    );
    harness.assert_all_named();
}

#[test]
fn m115_the_has_popup_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 160.0);
    let root = harness.mount(Column::new().boxed());
    let button = Button::new("Actions");
    let button_id = harness.add_child(root, button.boxed());
    harness.settle();

    // The live door: the app's target moved, the re-announcement the
    // mutate seam's request paid for — the kind riding the word, and
    // the honest absence restored on the clear.
    set_has_popup(&mut harness, button_id, Some(HasPopupKind::Menu));
    let menu = node(&harness, Role::Button, "Actions").expect("the opener survives");
    assert_eq!(
        menu.has_popup,
        Some(HasPopupKind::Menu),
        "the app's door moves the word the tree says, the kind beside it"
    );
    set_has_popup(&mut harness, button_id, Some(HasPopupKind::Dialog));
    let dialog = node(&harness, Role::Button, "Actions").expect("the opener survives");
    assert_eq!(
        dialog.has_popup,
        Some(HasPopupKind::Dialog),
        "the same door moves the target — the AT reads the new kind"
    );
    set_has_popup(&mut harness, button_id, None);
    let cleared = node(&harness, Role::Button, "Actions").expect("the opener survives");
    assert!(
        cleared.has_popup.is_none(),
        "the same door carries the word away — the honest absence restored"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(button_id);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Button>()
        .expect("the opener survives")
        .set_has_popup(None);
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m115_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the has-popup route's case: the
    // 1,000-node button arena carrying the target word — the word's
    // cost the measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(HAS_POPUP_CASE),
        "the has-popup route's case is published ({HAS_POPUP_CASE})"
    );
    assert!(
        benches.contains_key(BUTTONS_BARE_CASE),
        "the tier's bare denominator is published ({BUTTONS_BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M115 cut"
    );

    // The contract set gates the case and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    let gate = gates
        .iter()
        .find(|gate| gate["baseline"].as_str() == Some(HAS_POPUP_CASE))
        .unwrap_or_else(|| panic!("the case carries a regression gate ({HAS_POPUP_CASE})"));
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
        .find(|budget| budget["id"].as_str() == Some("has_popup_snapshot_overhead"))
        .expect("the has-popup budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(HAS_POPUP_CASE),
        "the budget prices the has-popup route"
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
        "the m5 corpus pin re-agrees at the M115 cut"
    );
}
