//! M124 exit-criteria integration tests (§10.3, ADR-0128): the radio
//! tier — the one-of-N control and the group it reads as, said by the
//! tree. Through M123 the snapshot said which member a collection picks
//! (the selected tier, ADR-0112), the set's span and the member's place
//! (the set tier, ADR-0123), the form facts, the collection facts, the
//! declared live region, the chord, the capacity, the language — and
//! the control every settings vocabulary speaks as "one of these, not
//! many" had no tier: an app wiring exclusive choice reached for a
//! checkbox that toggles off (the wrong verb — a radio selects, it
//! never deselects itself) or for a button wearing no selection fact at
//! all — the declared-but-unreported gap (the M94 pattern), cashed at
//! the radio tier: the control grows beside the set machinery it rides —
//! [`alacer_core::Role::Radio`] carrying the selection the M6
//! `toggled` contract announces, [`alacer_core::Role::RadioGroup`] the
//! named container the set reads as, [`alacer_widgets::Radio`]
//! answering every user door with *select* (never deselect — the
//! redundant ask consumed without moving a mask), and the set tier's
//! own words (`position_in_set` on the member, `size_of_set` on the
//! group) answering "which one, of how many" without a second tier.
//!
//! The tier prices the control and nothing else: a mounted radio's
//! neighbours answer exactly as they always did — the checkbox beside
//! it toggles, the button beside it presses, the radio's doors are the
//! selection's. An unavailable radio refuses every door honestly (the
//! M22 wall, the M11 discipline); a redundant select on the already-
//! selected member consumes the ask and moves nothing (the M21
//! idle-frame property at the widget's own doors — the AT-route's
//! coarse `consumed` contract rides the funnel's, documented at the
//! widget). The honest absence (`None`) is the set word's truth: a
//! radio that declared no place and a group that declared no size
//! announce neither — never a guessed count.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's doors ride the mutate seam and the
//! dispatch funnel whose `PAINT | A11Y` request re-announces exactly
//! the selection's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role, ToggleState};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Checkbox, Column, Radio, RadioGroup, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M124
/// cut — unchanged from the M123 cut. The radio tier's pricing door is
/// the widget-level bench the tier carries in the widgets crate
/// (`alacer-widgets`' own bench target, the press/select route priced
/// against the checkbox's), which rides *outside* the published corpus:
/// the published baseline's routes price the a11y snapshot machinery
/// (the M95 rule), and the radio tier grows no snapshot route — the
/// words it says are the set tier's, published since M119. The pin
/// re-agrees at 155, and the coordinated rule grows it at the next
/// published cut, never at a widget bench's.
const EXPECTED_BENCH_COUNT: usize = 163;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's selection doors: the mutate seam the app's code rides — a
/// selection moved, `PAINT | A11Y` requested on the control, the frames
/// the re-announcement rides settled before the read.
fn set_checked(harness: &mut TestHarness, radio_id: alacer_core::NodeId, checked: bool) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(radio_id);
    if let Some(radio) = guard.widget_mut().as_any_mut().downcast_mut::<Radio>() {
        radio.set_checked(checked);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m124_the_radio_tier_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The radio tier, mounted through the constructor words: the named
    // group the set reads as, the members with their selection facts
    // from the first frame, and the set words the AT reads as "2 of 3".
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let group = RadioGroup::new("Shipping").size_of_set(3);
    let group_id = harness.add_child(root, group.boxed());
    let _ = harness.add_child(group_id, Radio::new("Standard").position_in_set(1).boxed());
    let _picked_id = harness.add_child(
        group_id,
        Radio::new("Express")
            .checked(true)
            .position_in_set(2)
            .boxed(),
    );
    let _ = harness.add_child(group_id, Radio::new("Overnight").position_in_set(3).boxed());
    // The plain widgets beside them: the tier keeps their answers
    // exactly as they were — no radio word leaks onto a checkbox or a
    // button.
    let _ = harness.add_child(root, Checkbox::new("Gift wrap").boxed());
    let _ = harness.add_child(root, Button::new("Place order").boxed());
    let _ = harness.add_child(root, TextInput::new("Note").boxed());
    harness.settle();

    // The group: the named container the set reads as.
    let group_node = node(&harness, Role::RadioGroup, "Shipping").expect("the group rides");
    assert_eq!(
        group_node.size_of_set,
        Some(3),
        "the group's declared span is the tree's truth — the set word said"
    );
    assert_eq!(
        group_node.toggled, None,
        "a container carries no selection fact — the control's word, not the group's"
    );

    // The members: the selection the M6 `toggled` contract announces,
    // the place the set tier's word names.
    let picked = node(&harness, Role::Radio, "Express").expect("the picked radio rides");
    assert_eq!(
        picked.toggled,
        Some(ToggleState::On),
        "the checked member announces checked — the checkbox contract, at the radio"
    );
    assert_eq!(
        picked.position_in_set,
        Some(2),
        "the member's place lands in the same snapshot — the set tier's word"
    );
    let plain = node(&harness, Role::Radio, "Standard").expect("the unchecked radio rides");
    assert_eq!(
        plain.toggled,
        Some(ToggleState::Off),
        "an unchecked member announces off — the tree never guesses"
    );
    let third = node(&harness, Role::Radio, "Overnight").expect("the third radio rides");
    assert_eq!(
        third.position_in_set,
        Some(3),
        "the places answer one-based"
    );

    // The honest absences beside the tier: the neighbours' answers are
    // exactly what they were before the tier existed.
    let wrap = node(&harness, Role::CheckBox, "Gift wrap").expect("the checkbox rides");
    assert_eq!(
        wrap.toggled,
        Some(ToggleState::Off),
        "the checkbox's own contract answers unchanged beside the radio tier"
    );
    let order = node(&harness, Role::Button, "Place order").expect("the button rides");
    assert_eq!(
        order.position_in_set, None,
        "a non-collection role inherits the honest absence"
    );
    let note = node(&harness, Role::TextInput, "Note").expect("the plain field rides");
    assert_eq!(
        note.toggled, None,
        "a field carries no selection fact — the radio's word, not the field's"
    );
    harness.assert_all_named();
}

#[test]
fn m124_the_press_route_selects_and_never_deselects() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let group_id = harness.add_child(root, RadioGroup::new("Plan").boxed());
    let first_id = harness.add_child(group_id, Radio::new("Monthly").boxed());
    let second_id = harness.add_child(group_id, Radio::new("Yearly").checked(true).boxed());
    harness.settle();

    // The AT's ask selects the unchecked member: the tree's truth moves.
    assert!(
        harness.act(Role::Radio, "Monthly", A11yAction::DefaultPress),
        "the radio's door answers the select ask"
    );
    harness.settle();
    let monthly = node(&harness, Role::Radio, "Monthly").expect("the radio survives");
    assert_eq!(
        monthly.toggled,
        Some(ToggleState::On),
        "the ask landed — the member announces checked"
    );
    let yearly = node(&harness, Role::Radio, "Yearly").expect("the radio survives");
    assert_eq!(
        yearly.toggled,
        Some(ToggleState::On),
        "the tier is the control's: the radio announces its own selection — the
         exclusivity is the app's wiring (the handler the set's writer rides),
         never a tree-level rewrite of the siblings"
    );

    // The redundant ask: the already-selected member's door consumes the
    // ask and the selection does not move — the tree's truth is stable.
    assert!(
        harness.act(Role::Radio, "Monthly", A11yAction::DefaultPress),
        "the ask is consumable — the door exists and answers"
    );
    harness.settle();
    let monthly = node(&harness, Role::Radio, "Monthly").expect("the radio survives");
    assert_eq!(
        monthly.toggled,
        Some(ToggleState::On),
        "a radio never deselects itself — the redundant ask moved nothing"
    );

    // The app's own door: the mutate seam's write moves the selection
    // both ways — programmatic state, never a user door (the checkbox's
    // `set_checked` rule, at the radio).
    set_checked(&mut harness, second_id, false);
    let yearly = node(&harness, Role::Radio, "Yearly").expect("the radio survives");
    assert_eq!(
        yearly.toggled,
        Some(ToggleState::Off),
        "the app's write lands — the tree announces the cleared state"
    );
    set_checked(&mut harness, first_id, false);
    let monthly = node(&harness, Role::Radio, "Monthly").expect("the radio survives");
    assert_eq!(
        monthly.toggled,
        Some(ToggleState::Off),
        "the programmatic door is the one deselect door — the user's never was"
    );
    harness.assert_all_named();
}

#[test]
fn m124_an_unavailable_radio_refuses_every_door() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let group_id = harness.add_child(root, RadioGroup::new("Plan").boxed());
    let _ = harness.add_child(
        group_id,
        Radio::new("Fax").enabled(false).checked(true).boxed(),
    );
    harness.settle();

    // The M22 wall, at the radio tier: the AT's ask is refused before
    // it is parsed, and the selection stays the app's own.
    assert!(
        !harness.act(Role::Radio, "Fax", A11yAction::DefaultPress),
        "an unavailable radio has no write door"
    );
    harness.settle();
    let fax = node(&harness, Role::Radio, "Fax").expect("the radio rides");
    assert_eq!(
        fax.toggled,
        Some(ToggleState::On),
        "the refusal moved nothing — the selection is the app's truth"
    );
    harness.assert_all_named();
}

#[test]
fn m124_the_corpus_pin_re_agrees_at_the_m124_cut() {
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

    // The published corpus pin re-agrees: the radio tier grew no
    // snapshot route (its words are the set tier's, published since
    // M119), and its pricing door is the widgets crate's own bench —
    // outside the published corpus, the coordinated rule growing it at
    // the next published cut.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M124 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M124 cut"
    );
}
