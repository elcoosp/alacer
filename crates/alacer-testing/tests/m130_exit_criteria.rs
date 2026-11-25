//! M130 exit-criteria integration tests (§10.3, ADR-0134): the switch
//! tier — the instant on/off control, said by the tree. Through M129 the
//! snapshot said the tool strip (ADR-0133), the bounded number field
//! (ADR-0132), the page selector and its strip (ADR-0131), the section
//! title (ADR-0130), the hyperlink (ADR-0129), the one-of-N control and
//! its group (ADR-0128) — and the control every settings vocabulary
//! speaks as "flip this, now" had no tier: an app wiring an
//! instant-effect toggle reached for a checkbox wearing the wrong word
//! (a checkbox's write resolves a *form contract* — checked, unchecked,
//! the mixed summary the select-all parent needs; a switch's write is
//! the instant *on/off* the settings vocabulary speaks, and a switch has
//! no summary to hold) or for a button wearing no state at all.
//! [`alacer_core::Role::Switch`] carries the "writes one on/off fact"
//! AccessKit's own `Switch` role spells, [`alacer_widgets::Switch`]
//! answering every user door with the same one toggle — the checkbox's
//! three-doors-one-funnel shape at the switch tier, the binary pair
//! riding the M6 `toggled` contract (`On`/`Off`, never the summary a
//! switch does not hold), the availability wall (M22) refusing every
//! door on an unavailable control, and the programmatic write the one
//! door that reports the move (the M21 idle-frame property, at the
//! switch tier).
//!
//! The tier prices the control and nothing else: a mounted switch's
//! neighbours answer exactly as they always did — the checkbox beside it
//! keeps the tri-state the select-all parent needs, the button beside it
//! presses, the switch's doors are the on/off's. No patch-tier growth:
//! the role rides the `toggled` mapping the sink has carried since M6,
//! the corpus pin re-agreeing at 152.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), the keyboard's Space rides the focus the walk grants,
//! and the app's doors ride the mutate seam whose `PAINT | A11Y` request
//! re-announces exactly the toggle's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role, ToggleState};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Checkbox, Column, Switch, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M130
/// cut — unchanged from the M129 cut. The switch tier grows no snapshot
/// route (the role rides the `toggled` mapping the sink has carried
/// since M6 — the `Toggled` property beside the new role branch), and
/// its pricing door is the widget-level bench the tier carries in the
/// widgets crate (`switch/toggle_route_256` against
/// `switch/idle_write_256`), which rides *outside* the published
/// corpus: the published baseline's routes price the a11y snapshot
/// machinery (the M95 rule), and the coordinated rule grows it at the
/// next published cut, never at a widget bench's. The pin re-agrees at
/// 155.
const EXPECTED_BENCH_COUNT: usize = 205;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m130_the_switch_tier_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The switch tier, mounted through the constructor words: the on
    // switch and the off switch from the first frame, each named — and
    // the plain widgets the tier keeps honest. No switch word leaks onto
    // a checkbox or a button.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(root, Switch::new("Wi-Fi").on(true).boxed());
    let _ = harness.add_child(root, Switch::new("Bluetooth").boxed());
    let _ = harness.add_child(root, Checkbox::new("Gift wrap").boxed());
    let _ = harness.add_child(root, Button::new("Place order").boxed());
    let _ = harness.add_child(root, TextInput::new("Note").boxed());
    harness.settle();

    // The switches: the on/off fact AccessKit's own Switch role spells,
    // the binary pair the M6 `toggled` contract announces — never the
    // mixed summary a switch does not hold.
    let wifi = node(&harness, Role::Switch, "Wi-Fi").expect("the on switch rides");
    assert_eq!(
        wifi.toggled,
        Some(ToggleState::On),
        "the mounted-on switch announces on — the tree never guesses"
    );
    let bluetooth = node(&harness, Role::Switch, "Bluetooth").expect("the off switch rides");
    assert_eq!(
        bluetooth.toggled,
        Some(ToggleState::Off),
        "the mounted-off switch announces off — the honest absence's neighbour"
    );

    // The honest absences beside the tier: the neighbours' answers are
    // exactly what they were before the tier existed — the checkbox's
    // tri-state contract, the button's press, the field's edit.
    let wrap = node(&harness, Role::CheckBox, "Gift wrap").expect("the checkbox rides");
    assert_eq!(
        wrap.toggled,
        Some(ToggleState::Off),
        "the checkbox keeps the tri-state contract — the switch never took it"
    );
    let order = node(&harness, Role::Button, "Place order").expect("the button rides");
    assert_eq!(
        order.toggled, None,
        "a button carries no on/off fact — the switch's word, not the button's"
    );
    let note = node(&harness, Role::TextInput, "Note").expect("the plain field rides");
    assert_eq!(
        note.toggled, None,
        "a field carries no on/off fact — the switch's word, not the field's"
    );
    harness.assert_all_named();
}

#[test]
fn m130_every_user_door_toggles_and_the_idle_write_queues_nothing() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let wifi_id = harness.add_child(root, Switch::new("Wi-Fi").boxed());
    harness.settle();

    // The AT's ask toggles: a switch flips — and flips again. A switch
    // is a toggle, not a select: the second ask moves the tree's truth
    // back (the radio's never-deselect rule is the radio's own).
    assert!(
        harness.act(Role::Switch, "Wi-Fi", A11yAction::DefaultPress),
        "the switch's door answers the toggle ask"
    );
    harness.settle();
    let wifi = node(&harness, Role::Switch, "Wi-Fi").expect("the switch survives");
    assert_eq!(
        wifi.toggled,
        Some(ToggleState::On),
        "the ask landed — the switch announces on"
    );
    assert!(
        harness.act(Role::Switch, "Wi-Fi", A11yAction::DefaultPress),
        "the door answers again"
    );
    harness.settle();
    let wifi = node(&harness, Role::Switch, "Wi-Fi").expect("the switch survives");
    assert_eq!(
        wifi.toggled,
        Some(ToggleState::Off),
        "a switch toggles — the second ask lands off"
    );

    // The keyboard's door: Tab focuses the control, Space toggles it —
    // the checkbox's activation parity, at the switch tier.
    harness.press_key(alacer_core::events::Key::Tab);
    harness.press_key(alacer_core::events::Key::Space);
    harness.settle();
    let wifi = node(&harness, Role::Switch, "Wi-Fi").expect("the switch survives");
    assert_eq!(
        wifi.toggled,
        Some(ToggleState::On),
        "Space toggled the focused switch — the tree announces on"
    );

    // The programmatic door: the mutate seam's write moves the state
    // without firing a user door — the app's own switch state.
    let moved = {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(wifi_id);
        let moved = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<Switch>()
            .is_some_and(|wifi_switch| wifi_switch.set_on(false));
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
        drop(guard);
        moved
    };
    harness.settle();
    let wifi = node(&harness, Role::Switch, "Wi-Fi").expect("the switch survives");
    assert!(
        moved,
        "the programmatic write reports the move — the app's own door"
    );
    assert_eq!(
        wifi.toggled,
        Some(ToggleState::Off),
        "the app's write landed — the tree announces the cleared state"
    );

    // The idle write: the write that lands where the flag already is
    // answers *not moved* and queues no masks — the tick it would have
    // paid reports idle (the M21 idle-frame property, at the switch
    // tier).
    let idle = {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(wifi_id);
        let moved = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<Switch>()
            .is_some_and(|wifi_switch| wifi_switch.set_on(false));
        drop(guard);
        moved
    };
    assert!(
        !idle,
        "the write landed where the flag already is — not moved, no masks"
    );
    let report = harness.tick();
    assert!(
        report.is_idle(),
        "the idle write queues nothing — the frame after it is idle"
    );
    harness.assert_all_named();
}

#[test]
fn m130_an_unavailable_switch_refuses_every_door() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let bluetooth_id = harness.add_child(root, Switch::new("Bluetooth").on(true).boxed());
    let wifi_id = harness.add_child(root, Switch::new("Wi-Fi").enabled(false).on(true).boxed());
    harness.settle();

    // The M22 wall, at the switch tier: the AT's ask is refused before
    // it is parsed, and the on/off fact stays the app's own.
    assert!(
        !harness.act(Role::Switch, "Wi-Fi", A11yAction::DefaultPress),
        "an unavailable switch has no write door"
    );
    harness.settle();
    let wifi = node(&harness, Role::Switch, "Wi-Fi").expect("the switch rides");
    assert_eq!(
        wifi.toggled,
        Some(ToggleState::On),
        "the refusal moved nothing — the state is the app's truth"
    );
    assert!(
        !wifi.enabled,
        "the tree carries the availability word the doors answered"
    );

    // The keyboard never reaches an unavailable control either: the
    // available neighbour holds the walk's focus, the wall does not.
    let focusables = harness.pipeline().focusables();
    assert!(
        !focusables.contains(&wifi_id),
        "an unavailable switch holds no focus"
    );
    assert!(
        focusables.contains(&bluetooth_id),
        "the available neighbour keeps its focus door"
    );
    harness.assert_all_named();
}

#[test]
fn m130_the_corpus_pin_re_agrees_at_the_m130_cut() {
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

    // The published corpus pin re-agrees: the switch tier grew no
    // snapshot route (the role rides the `toggled` mapping the sink has
    // carried since M6), and its pricing door is the widgets crate's own
    // bench — outside the corpus at its own cut, joined by the corpus catch-up's published cut — the coordinated rule's next published cut cashed.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M130 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M130 cut"
    );
}
