//! M10 exit-criteria integration tests (§11.5, ADR-0016): the a11y action
//! tier joins the dispatch contract — the platform maps `ActionRequest`s
//! faithfully instead of munging every request into a press, a scroll area
//! answers the AT scroll/step vocabulary through pass 10, a focused button
//! is keyboard-operable (Space/Enter), and the corpus grows 47 → 48 by the
//! coordinated rule.
//!
//! Every test runs the real pipeline (A4); the platform mapping is pinned
//! by source, matching the M6/M7/M8 suite convention.

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers, PointerEvent};
use alacer_core::geom::{Point, Vector};
use alacer_core::widget::{A11yAction, Role, ScrollUnit};
use alacer_testing::TestHarness;
use alacer_widgets::List;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves")
}

fn read_workspace_file(path: &str) -> String {
    let path = workspace_root().join(path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("required file {} missing: {error}", path.display()))
}

fn parse_workspace_json(path: &str) -> serde_json::Value {
    let text = read_workspace_file(path);
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path} is not valid JSON: {error}"))
}

/// A list with 24 fixed panels: the M8 mount, reused so the scroll stays
/// translation-only under the a11y path too.
fn mount_list(harness: &mut TestHarness) -> alacer_widgets::ListHandle {
    let root = harness.mount(alacer_widgets::Column::new().boxed());
    let list = List::new(160.0);
    let handle = list.handle();
    let list_node = harness.add_child(root, list.boxed());
    for i in 0..24 {
        let colors = [
            alacer_core::color::Color::rgb(239, 68, 68),
            alacer_core::color::Color::rgb(59, 130, 246),
            alacer_core::color::Color::rgb(34, 197, 94),
            alacer_core::color::Color::rgb(234, 179, 8),
        ];
        let _ = harness.add_child(
            list_node,
            alacer_widgets::Panel::new(28.0, colors[i % colors.len()]).boxed(),
        );
    }
    harness.settle();
    handle.set_content_height(24.0 * 28.0);
    handle
}

#[test]
fn m10_the_a11y_action_tier_scrolls_a_list_end_to_end() {
    let mut harness = TestHarness::new(400.0, 400.0);
    let handle = mount_list(&mut harness);
    harness.reset_measure_calls();

    // The AT path: pass 10 routes the action, the list consumes it, the
    // next frame recomposes translation-only.
    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollDown(ScrollUnit::Item)
        ),
        "the list consumes a one-item scroll"
    );
    assert_eq!(handle.offset(), 40.0, "one item = the 40 px line");

    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollDown(ScrollUnit::Page)
        ),
        "the list consumes a one-page scroll"
    );
    assert_eq!(handle.offset(), 200.0, "one page = the 160 px viewport");

    assert!(
        harness.act(Role::ScrollArea, "list", A11yAction::Increment),
        "the numeric step scrolls by a page"
    );
    assert_eq!(handle.offset(), 360.0);

    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollUp(ScrollUnit::Page)
        ),
        "scroll up mirrors scroll down"
    );
    assert_eq!(handle.offset(), 200.0);

    // The one-currency audit: scrolling is translation-only through the
    // a11y path too — zero measure calls across all four actions.
    assert_eq!(
        harness.measure_calls(),
        0,
        "a11y scrolling stays translation-only (§8.2)"
    );

    // The consumed actions rode PAINT|A11Y: the a11y tree re-announced the
    // moved rows (the sink's delta carried the list's children).
    assert!(
        harness.query(Role::ScrollArea).named("list").exists(),
        "the list is reachable and named after the action storm"
    );
}

#[test]
fn m10_an_action_the_widget_does_not_implement_is_never_munged_into_a_press() {
    let mut harness = TestHarness::new(400.0, 400.0);
    let clicks = Rc::new(Cell::new(0u32));
    let counter = Rc::clone(&clicks);
    let root = harness.mount(alacer_widgets::Column::new().boxed());
    let _ = harness.add_child(
        root,
        Box::new(
            alacer_widgets::Button::new("Increment")
                .on_click(move || counter.set(counter.get() + 1)),
        ),
    );
    harness.settle();

    // An AT increment on a button must be *refused*, not silently turned
    // into the pre-M10 blanket press.
    assert!(
        !harness.act(Role::Button, "Increment", A11yAction::Increment),
        "a button does not implement Increment"
    );
    assert_eq!(clicks.get(), 0, "the refused action fired nothing");

    assert!(
        !harness.act(
            Role::Button,
            "Increment",
            A11yAction::ScrollDown(ScrollUnit::Item)
        ),
        "a button does not implement scrolling"
    );
    assert_eq!(clicks.get(), 0);

    // The default press still works, and only the default press.
    assert!(harness.act(Role::Button, "Increment", A11yAction::DefaultPress));
    assert_eq!(clicks.get(), 1, "exactly one click from exactly one press");
}

#[test]
fn m10_a_focused_button_is_keyboard_operable() {
    let mut harness = TestHarness::new(400.0, 400.0);
    let clicks = Rc::new(Cell::new(0u32));
    let counter = Rc::clone(&clicks);
    let root = harness.mount(alacer_widgets::Column::new().boxed());
    let _ = harness.add_child(
        root,
        Box::new(
            alacer_widgets::Button::new("Increment")
                .on_click(move || counter.set(counter.get() + 1)),
        ),
    );
    harness.settle();

    // Nothing is focused yet: space must not fall through to the button.
    harness.press_key(Key::Space);
    assert_eq!(clicks.get(), 0, "an unfocused button eats no keys");

    // Tab focuses the button (the only focusable); space activates.
    harness.press_key(Key::Tab);
    harness.press_key(Key::Space);
    assert_eq!(clicks.get(), 0, "the click waits for the release");
    harness.key(&KeyEvent {
        key: Key::Space,
        state: KeyState::Released,
        modifiers: Modifiers::default(),
        text: None,
    });
    assert_eq!(clicks.get(), 1, "space press-and-release fires one click");

    // Enter too.
    harness.press_key(Key::Enter);
    harness.key(&KeyEvent {
        key: Key::Enter,
        state: KeyState::Released,
        modifiers: Modifiers::default(),
        text: None,
    });
    assert_eq!(clicks.get(), 2, "enter fires one click per release");

    // The pointer path still works alongside (one currency, three doors).
    harness.click(Point::new(60.0, 30.0));
    assert_eq!(clicks.get(), 3, "pointer, keyboard, and a11y all converge");
}

#[test]
fn m10_the_platform_maps_action_requests_instead_of_munging() {
    let source = read_workspace_file("crates/alacer-platform/src/lib.rs");
    assert!(
        source.contains("Action::Click"),
        "the platform maps the click action explicitly"
    );
    assert!(
        source.contains("Action::ScrollDown") && source.contains("Action::ScrollUp"),
        "the platform maps the scroll actions explicitly"
    );
    assert!(
        source.contains("Action::Increment") && source.contains("Action::Decrement"),
        "the platform maps the step actions explicitly"
    );
    assert!(
        !source.contains("let action = alacer_core::widget::A11yAction::DefaultPress;"),
        "the pre-M10 blanket press is gone: actions are routed, not munged"
    );
}

// ---------------------------------------------------------------------------
// The published corpus (the M5 coordinated rule)
// ---------------------------------------------------------------------------

/// The corpus pin, mirroring the coordinated rule: 48 criterion cases — the
/// 47 pre-M10 cases plus the `a11y_action` route bench. Growing the corpus
/// moves the bench file, contracts.json, the baseline, and the M5–M9 pins
/// together.
const EXPECTED_BENCH_COUNT: usize = 48;

#[test]
fn m10_the_corpus_pin_agrees_with_the_published_data() {
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");

    let benches = baselines["benches"].as_object().expect("benches object");
    let regressions = contracts["regressions"].as_array().expect("regressions");
    assert_eq!(
        benches.len(),
        EXPECTED_BENCH_COUNT,
        "the corpus grew by the action-route bench"
    );
    assert_eq!(regressions.len(), EXPECTED_BENCH_COUNT);
    let id = "a11y_action/route_100_actions_1k";
    assert!(benches.contains_key(id), "{id} published");
    assert!(
        regressions
            .iter()
            .any(|gate| gate["baseline"] == serde_json::json!(id)),
        "{id} contracted"
    );

    // The earlier suites pin the same corpus — the chain cannot drift.
    for suite in [
        "m5_exit_criteria.rs",
        "m6_exit_criteria.rs",
        "m7_exit_criteria.rs",
        "m8_exit_criteria.rs",
        "m9_exit_criteria.rs",
    ] {
        let source = read_workspace_file(&format!("crates/alacer-testing/tests/{suite}"));
        assert!(
            source.contains("const EXPECTED_BENCH_COUNT: usize = 48;"),
            "{suite} must pin the 48-case corpus"
        );
    }
}

/// The wheel path is untouched by M10: a regression guard over the M8
/// contract, so the a11y scroll vocabulary cannot silently drift the
/// platform's line convention.
#[test]
fn m10_the_wheel_and_the_a11y_line_agree() {
    let mut harness = TestHarness::new(400.0, 400.0);
    let handle = mount_list(&mut harness);

    harness.queue_pointer(PointerEvent::Wheel {
        delta: Vector::new(0.0, -40.0),
        position: Point::new(50.0, 80.0),
    });
    harness.settle();
    assert_eq!(handle.offset(), 40.0, "one wheel notch = 40 px");

    let source = read_workspace_file("crates/alacer-widgets/src/list.rs");
    assert!(
        source.contains("const A11Y_LINE: f32 = 40.0"),
        "the a11y item unit is the same 40 px line the wheel notch travels"
    );
}
