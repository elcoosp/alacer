//! M94 exit-criteria integration tests (§8, ADR-0098): the checkbox grows
//! the mixed state. The vocabulary has carried the state since M6 —
//! [`alacer_core::widget::ToggleState::Mixed`] mapped onto AccessKit's
//! `Toggled::Mixed` in the same cut — and no widget has ever produced it:
//! the tri-state parent, the classic select-all summary, was a story the
//! a11y tree could carry but the widget vocabulary could not tell. This
//! cut ships the producer: the [`alacer_widgets::Checkbox`] holds the
//! tri-state, every user door (click, Space, the AT's press) *resolves*
//! the summary — the first press lands fully checked, never a third bit —
//! the painter draws the dash, and the view DSL's `state` word carries
//! the tri-state through reconciliation.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), the pixels are the painter's own (A4), the view tier is
//! the consumer's route (§13.1).

use alacer_core::widget::{Role, ToggleState};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Checkbox, Column, Label};

/// The suite's corpus pin: the coordinated rule's count at the M94 cut —
/// the toggle route's honesty pair joining the derive route's pair (M92)
/// and the recording route's pair (M93) on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 118;

/// The suite's own read doors into the published corpus: the toggle
/// route's tri-state honesty cases (the coordinated rule's bench half,
/// ADR-0098's exit criteria).
const TRI_CASE: &str = "toggle/snapshot_1k_tri_state";
const BINARY_CASE: &str = "toggle/snapshot_1k_binary";

/// The select-all canon: one tri-state parent over three children in
/// every state — the summary the mixed state exists to tell.
fn select_all_group(harness: &mut TestHarness) -> alacer_core::arena::NodeId {
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    let _ = harness.add_child(
        root,
        Checkbox::new("Select all")
            .state(ToggleState::Mixed)
            .boxed(),
    );
    let _ = harness.add_child(root, Checkbox::new("Ships").checked(true).boxed());
    let _ = harness.add_child(root, Checkbox::new("Invoice").boxed());
    let _ = harness.add_child(
        root,
        Checkbox::new("Receipt").state(ToggleState::Mixed).boxed(),
    );
    let _ = harness.add_child(root, Label::new("the group's footer").boxed());
    harness.settle();
    root
}

/// The tri-state the a11y tree carries, read through the only official
/// query API (ADR-10).
fn carried_state(harness: &TestHarness, name: &str) -> Option<ToggleState> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::CheckBox, name))
        .and_then(|node| node.toggled)
}

/// The bounds center, the click point every pointer test rides.
fn center(bounds: alacer_core::geom::Rect) -> alacer_core::geom::Point {
    alacer_core::geom::Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + bounds.height() * 0.5,
    )
}

#[test]
fn m94_the_mixed_summary_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 280.0);
    select_all_group(&mut harness);

    // The vocabulary's own states ride the tree: the parents carry the
    // mixed summary the pipeline has mapped since M6, the children carry
    // their binary states.
    assert_eq!(
        carried_state(&harness, "Select all"),
        Some(ToggleState::Mixed),
        "the tri-state parent's summary is the tree's truth"
    );
    assert_eq!(
        carried_state(&harness, "Receipt"),
        Some(ToggleState::Mixed),
        "the second parent carries its own summary"
    );
    assert_eq!(
        carried_state(&harness, "Ships"),
        Some(ToggleState::On),
        "the checked child is announced checked"
    );
    assert_eq!(
        carried_state(&harness, "Invoice"),
        Some(ToggleState::Off),
        "the unchecked child is announced unchecked"
    );
    harness.assert_all_named();
}

#[test]
fn m94_the_user_doors_resolve_the_summary() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 280.0);
    let resolved = std::rc::Rc::new(std::cell::Cell::new(false));
    let sink = std::rc::Rc::clone(&resolved);
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(
        root,
        Checkbox::new("Select all")
            .state(ToggleState::Mixed)
            .on_toggle(move |checked| sink.set(checked))
            .boxed(),
    );
    harness.settle();

    // The AT's press resolves: the summary lands fully checked — the
    // handler sees the resolved boolean, never a third bit.
    harness.press(Role::CheckBox, "Select all");
    assert_eq!(
        carried_state(&harness, "Select all"),
        Some(ToggleState::On),
        "the AT's press resolves the summary to checked"
    );
    assert!(resolved.get(), "the handler saw the resolved state");

    // The second press toggles the resolved bit honestly.
    harness.press(Role::CheckBox, "Select all");
    assert_eq!(
        carried_state(&harness, "Select all"),
        Some(ToggleState::Off),
        "the resolved control toggles like any checkbox"
    );

    // A fresh summary resolves through the pointer's door the same way.
    let _ = harness.add_child(
        root,
        Checkbox::new("Pointer summary")
            .state(ToggleState::Mixed)
            .boxed(),
    );
    harness.settle();
    let bounds = harness
        .query(Role::CheckBox)
        .named("Pointer summary")
        .bounds();
    harness.click(center(bounds));
    assert_eq!(
        carried_state(&harness, "Pointer summary"),
        Some(ToggleState::On),
        "the click resolves the summary the press resolves"
    );
}

#[test]
fn m94_the_resolution_costs_its_masks() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 280.0);
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let parent = harness.add_child(
        root,
        Checkbox::new("Select all")
            .state(ToggleState::Mixed)
            .boxed(),
    );
    harness.settle();
    let before = harness.counters();

    // The AT's press resolves the summary — the funnel's own masks (the
    // M6 dispatch's `PAINT | A11Y`) land: the next snapshot announces
    // the resolved state, the paint pass redraws the dash as a check.
    harness.press(Role::CheckBox, "Select all");
    let after = harness.counters();
    assert!(
        after.paint_runs > before.paint_runs && after.a11y_runs > before.a11y_runs,
        "the resolution's frame ran the passes the masks priced \
         (paint {}→{}, a11y {}→{})",
        before.paint_runs,
        after.paint_runs,
        before.a11y_runs,
        after.a11y_runs
    );
    assert_eq!(
        carried_state(&harness, "Select all"),
        Some(ToggleState::On),
        "the re-announced tree carries the resolved state"
    );

    // The programmatic door stays programmatic (view reconciliation):
    // the state write moves the summary, fires no handler, and the
    // write tier queues its own masks (the m22 discipline: the write
    // that moved is the only mask the tier costs).
    let mut guard = harness.pipeline_mut().arena_mut().mutate(parent);
    let checkbox = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Checkbox>()
        .expect("the parent is the checkbox");
    checkbox.set_state(ToggleState::Off);
    guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    drop(guard);
    harness.settle();
    assert_eq!(
        carried_state(&harness, "Select all"),
        Some(ToggleState::Off),
        "the programmatic write re-announces through the same masks"
    );
}

#[test]
fn m94_the_painter_tells_the_tri_state_truth() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 280.0);
    select_all_group(&mut harness);

    // The dash is the mixed painter's own pixels: deterministic quads,
    // no glyph dependency — the same discipline the check mark shipped
    // by (the snapshot corpus stays font-free at the box).
    harness.snapshot("tri_state_group");
}

#[test]
fn m94_the_view_lands_the_state_word() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 280.0);

    let view = alacer_macro::rsx! {
        column { gap: 6.0,
            checkbox { name: "Select all", state: "mixed" }
            checkbox { name: "Ships", checked: true }
        }
    };
    let mut root = alacer_view::ViewRoot::mount(
        &view,
        harness.pipeline_mut(),
        alacer_widgets::default_registry(),
    );
    harness.settle();
    assert_eq!(
        carried_state(&harness, "Select all"),
        Some(ToggleState::Mixed),
        "the `state` word carries the tri-state through the DSL"
    );

    // The patch re-renders the word in place — the summary lands On.
    let patched = alacer_macro::rsx! {
        column { gap: 6.0,
            checkbox { name: "Select all", state: "on" }
            checkbox { name: "Ships", checked: true }
        }
    };
    let report = root.update(&patched, harness.pipeline_mut());
    assert!(report.patched >= 1, "the state word patched in place");
    harness.settle();
    assert_eq!(
        carried_state(&harness, "Select all"),
        Some(ToggleState::On),
        "the patched summary announces checked"
    );

    // The binary flag word keeps its own contract beside the tri-state.
    assert_eq!(
        carried_state(&harness, "Ships"),
        Some(ToggleState::On),
        "the `checked` flag rides On as always"
    );
}

#[test]
fn m94_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the toggle route's honesty pair: the
    // tri-state summary against the binary toggle on the same snapshot
    // route — the dash's cost the measurement.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(TRI_CASE) && benches.contains_key(BINARY_CASE),
        "the toggle route's pair is published ({TRI_CASE}, {BINARY_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M94 cut"
    );

    // The contract set gates both cases and prices the summary's
    // budget — a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [TRI_CASE, BINARY_CASE] {
        let gate = gates
            .iter()
            .find(|gate| gate["baseline"].as_str() == Some(case))
            .unwrap_or_else(|| panic!("the case carries a regression gate ({case})"));
        assert_eq!(
            gate["scope"].as_str(),
            Some("same-host"),
            "the gate is same-host scoped like the route tier's own"
        );
    }
    let budgets = contracts["budgets"]
        .as_array()
        .expect("the contract set carries budgets");
    let budget = budgets
        .iter()
        .find(|budget| budget["id"].as_str() == Some("toggle_tri_state_overhead"))
        .expect("the tri-state budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(TRI_CASE),
        "the budget prices the tri-state route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(BINARY_CASE),
        "the budget prices it against the binary toggle"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the summary must not outprice the toggle it summarizes"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 118;"),
        "the m5 corpus pin re-agrees at the M94 cut"
    );
}
