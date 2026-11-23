//! M116 exit-criteria integration tests (§10.3, ADR-0120): the
//! role-description tier — the role a custom control is *read as*, said
//! by the tree. Through M115 the snapshot said what a control *does*
//! (the description, ADR-0099), what an empty field *hints* (the
//! placeholder, ADR-0100), the span a value rides (the range tier,
//! ADR-0101), the form's ask and the form's rejection (the required and
//! invalid tiers, ADR-0110/ADR-0111), the collection's pick, capacity
//! and disclosure (ADR-0112/ADR-0113/ADR-0114), the field's suggestion
//! contract (the auto-complete tier, ADR-0115), the sorted column's
//! order (the sort direction tier, ADR-0116), the nested member's depth
//! (the level tier, ADR-0117), the set's current item (the current
//! tier, ADR-0118), the opener's target (the has-popup tier,
//! ADR-0119) — and the word every custom-control vocabulary speaks
//! about what a widget is read as ("this slider is a star rating, that
//! canvas is a chart") had no tier: the AT announced the generic role —
//! "slider", "button" — and never the specialized one, the
//! declared-but-unreported gap (the M94 pattern), cashed at the
//! role-description tier: the word grows beside the roles it specializes
//! — [`alacer_core::Widget::role_description`] beside
//! [`alacer_widgets::Button::role_description`] and
//! [`alacer_widgets::Slider::role_description`], AccessKit's
//! `RoleDescription` property the sink maps in the same snapshot every
//! other property rides.
//!
//! The tier prices the word and nothing else: a custom control's doors
//! answer exactly as they always did — the word names the read-as role,
//! it walls nothing, the control's press and focus routes the widget's
//! own. The honest absence is `None` (the ADR-0087 presence rule at the
//! tier): a control that declared nothing is read as the generic role
//! it already was — never a guessed specialization.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the word's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, ColumnHeader, Slider, TableRow};

/// The suite's corpus pin: the coordinated rule's count at the M116 cut —
/// the role-description route's case joining the has-popup route's case
/// (M115), the current route's pair (M114), the level route's case
/// (M113), the sort route's pair (M112), the auto-complete route's case
/// (M111), the expanded route's pair (M110), the multiselectable route's
/// pair (M109), the selected route's pair (M108), the invalid route's
/// case (M107) and every tier's before them on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 159;

/// The suite's own read door into the published corpus: the
/// role-description route's case — the 1,000-node button arena carrying
/// the read-as word — priced against the same 1,000-node button arena
/// bare (`current/snapshot_1k_buttons_bare`, the M114 case the M111
/// precedent reuses; the M95 pricing rule — the word is a button word,
/// so the honest comparison is buttons against buttons, the
/// M108/M110 denominator shape).
const ROLE_DESCRIPTION_CASE: &str = "role_description/snapshot_1k_role_descriptions";
const BUTTONS_BARE_CASE: &str = "current/snapshot_1k_buttons_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's role-description door: the mutate seam the app's code rides —
/// the word moved, `PAINT | A11Y` requested on the control, the frames
/// the re-announcement rides settled before the read. `None` clears the
/// word — the honest absence restored.
fn set_role_description(
    harness: &mut TestHarness,
    node_id: alacer_core::NodeId,
    role_description: Option<&str>,
) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(button) = guard.widget_mut().as_any_mut().downcast_mut::<Button>() {
        button.set_role_description(role_description.map(str::to_owned));
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m116_the_role_description_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 320.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The worded custom controls, mounted through the constructor word:
    // the read-as role the tree says from the first frame — the star
    // rating slider, the chart canvas stand-in, the wizard-page button —
    // beside the plain controls, the honest absence the tier keeps
    // honest (a control that declared nothing is read as the generic
    // role it already was, never a guessed specialization).
    let _ = harness.add_child(
        root,
        Slider::new("Rating", 0.0, 5.0)
            .role_description("star rating")
            .boxed(),
    );
    let _ = harness.add_child(
        root,
        Button::new("Chapter 3")
            .role_description("wizard page")
            .boxed(),
    );
    let _ = harness.add_child(root, Button::new("Send").boxed());
    let _ = harness.add_child(root, Slider::new("Volume", 0.0, 10.0).boxed());
    // The header and the row beside them: no read-as role to say — the
    // word is a custom-control word, the collection roles the tiers' own
    // facts.
    let _ = harness.add_child(root, ColumnHeader::new("Price", 0).boxed());
    let _ = harness.add_child(root, TableRow::new().boxed());
    harness.settle();

    let rating = node(&harness, Role::Slider, "Rating").expect("the rating rides");
    assert_eq!(
        rating.role_description.as_deref(),
        Some("star rating"),
        "the slider said the role it is read as — the specialization beside the generic role"
    );
    let chapter = node(&harness, Role::Button, "Chapter 3").expect("the chapter rides");
    assert_eq!(
        chapter.role_description.as_deref(),
        Some("wizard page"),
        "the button said the read-as word the constructor named"
    );
    let volume = node(&harness, Role::Slider, "Volume").expect("the volume rides");
    assert!(
        volume.role_description.is_none(),
        "a control that declared nothing is read as the generic role — the ordinary slider, never a guessed one"
    );
    let send = node(&harness, Role::Button, "Send").expect("the send rides");
    assert!(
        send.role_description.is_none(),
        "an ordinary control announces no read-as role — the honest absence"
    );
    let header = node(&harness, Role::ColumnHeader, "Price").expect("the header rides");
    assert!(
        header.role_description.is_none(),
        "a header role inherits the honest absence — the word is a custom-control word"
    );
    let row = node(&harness, Role::TableRow, "").expect("the row rides");
    assert!(
        row.role_description.is_none(),
        "a member role inherits the honest absence — the word is a custom-control word"
    );

    // The word prices nothing but the word: the plain control's write
    // door answers exactly as any button's.
    assert!(
        harness.act(Role::Button, "Send", A11yAction::DefaultPress),
        "the role-description word is not a wall — the plain control answers as ever"
    );
    harness.assert_all_named();
}

#[test]
fn m116_the_role_description_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 160.0);
    let root = harness.mount(Column::new().boxed());
    let button = Button::new("Chapter 3");
    let button_id = harness.add_child(root, button.boxed());
    harness.settle();

    // The live door: the app's read-as role moved, the re-announcement
    // the mutate seam's request paid for — the word the tree says
    // following the app's own, and the honest absence restored on the
    // clear.
    set_role_description(&mut harness, button_id, Some("wizard page"));
    let page = node(&harness, Role::Button, "Chapter 3").expect("the control survives");
    assert_eq!(
        page.role_description.as_deref(),
        Some("wizard page"),
        "the app's door moves the word the tree says"
    );
    set_role_description(&mut harness, button_id, Some("checklist step"));
    let step = node(&harness, Role::Button, "Chapter 3").expect("the control survives");
    assert_eq!(
        step.role_description.as_deref(),
        Some("checklist step"),
        "the same door moves the word — the AT reads the new specialization"
    );
    set_role_description(&mut harness, button_id, None);
    let cleared = node(&harness, Role::Button, "Chapter 3").expect("the control survives");
    assert!(
        cleared.role_description.is_none(),
        "the same door carries the word away — the honest absence restored"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(button_id);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Button>()
        .expect("the control survives")
        .set_role_description(None);
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m116_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the role-description route's case: the
    // 1,000-node button arena carrying the read-as word — the word's
    // cost the measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(ROLE_DESCRIPTION_CASE),
        "the role-description route's case is published ({ROLE_DESCRIPTION_CASE})"
    );
    assert!(
        benches.contains_key(BUTTONS_BARE_CASE),
        "the tier's bare denominator is published ({BUTTONS_BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M116 cut"
    );

    // The contract set gates the case and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    let gate = gates
        .iter()
        .find(|gate| gate["baseline"].as_str() == Some(ROLE_DESCRIPTION_CASE))
        .unwrap_or_else(|| panic!("the case carries a regression gate ({ROLE_DESCRIPTION_CASE})"));
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
        .find(|budget| budget["id"].as_str() == Some("role_description_snapshot_overhead"))
        .expect("the role-description budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(ROLE_DESCRIPTION_CASE),
        "the budget prices the role-description route"
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
        "the m5 corpus pin re-agrees at the M116 cut"
    );
}
