//! M97 exit-criteria integration tests (§8, ADR-0101): the range tier.
//! Every value-tier write since M20 has funneled through a range the
//! widget declares at construction — `Slider::new(name, min, max)` and
//! `Progress::new(name, min, max)` clamp every door into it — and the
//! a11y tree could never say it: the AT read "42" and did not know
//! whether that was the middle of 0..100 or the top of 0..50. The
//! declared-but-unreported gap, the M94 pattern at the range tier,
//! cashed: [`alacer_core::Widget::value_range`] beside the clamp that
//! already told the truth, the snapshot carrying it, the sink mapping
//! it — AccessKit's `MinNumericValue`/`MaxNumericValue` beside the
//! `NumericValueStep` the slider declares and the progress honestly
//! lacks (aria-valuemin / aria-valuemax / the step).
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), the counters are the pass ledger's own (A4), the view
//! tier is the consumer's route (§13.1).

use alacer_core::widget::{Role, ValueRange};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Progress, Slider, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M97 cut —
/// the range route's case joining the placeholder route's pair (M96),
/// the description route's case (M95), the toggle route's pair (M94),
/// the derive route's pair (M92), and the recording route's pair (M93)
/// on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 204;

/// The suite's own read door into the published corpus: the range
/// route's case — the ranged 1,000-node slider arena (the coordinated
/// rule's bench half, ADR-0101's exit criteria). The ratio's honest
/// denominator is the no-range arena the toggle route already carries
/// (`toggle/snapshot_1k_binary`, the M94 binary case), the M95 pricing
/// rule: the same corpus measuring what the tier adds.
const RANGED_CASE: &str = "range/snapshot_1k_ranged";
const BARE_CASE: &str = "toggle/snapshot_1k_binary";

/// The tree-carried range, read through the only official query API
/// (ADR-10).
fn carried_range(harness: &TestHarness, role: Role, name: &str) -> Option<ValueRange> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name))
        .and_then(|node| node.value_range)
}

#[test]
fn m97_the_range_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    let _ = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .step(5.0)
            .value(40.0)
            .boxed(),
    );
    let _ = harness.add_child(root, Progress::new("Upload", 0.0, 100.0).boxed());
    let _ = harness.add_child(root, TextInput::new("Notes").boxed());
    harness.settle();

    // The range every write clamps into is the range the tree announces.
    let slider = carried_range(&harness, Role::Slider, "Volume");
    assert_eq!(
        slider.as_ref().map(|r| (r.min, r.max)),
        Some((0.0, 100.0)),
        "the declared span is the tree's truth"
    );
    assert_eq!(
        slider.and_then(|r| r.step),
        Some(5.0),
        "the step the arrows and the AT's acts snap to rides beside it"
    );

    // The progress declares its span and honestly lacks a step: the
    // absence is the absence, never a guessed default.
    let progress = carried_range(&harness, Role::ProgressIndicator, "Upload");
    assert_eq!(
        progress.as_ref().map(|r| (r.min, r.max)),
        Some((0.0, 100.0)),
        "the report's span is the tree's truth too"
    );
    assert_eq!(
        progress.and_then(|r| r.step),
        None,
        "a report with no step announces the absence, never a default"
    );

    // The honest absence on a valueless role: a text input answers
    // nothing — the tree reads exactly as it read before the tier.
    assert_eq!(
        carried_range(&harness, Role::TextInput, "Notes"),
        None,
        "a widget with no span announces the absence"
    );
    harness.assert_all_named();
}

#[test]
fn m97_the_value_write_moves_inside_the_declared_range() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .step(5.0)
            .value(40.0)
            .boxed(),
    );
    harness.settle();
    assert!(carried_range(&harness, Role::Slider, "Volume").is_some());

    // The write the range constrains: the click-to-focus seam arms the
    // slider, Home jumps to the wall the tree already carries, and the
    // value moves while the range stays mounted — the two facts the AT
    // reads beside each other, exactly the frame the canvas shows.
    let bounds = harness
        .query(Role::Slider)
        .named("Volume")
        .first()
        .bounds
        .expect("the laid-out slider has bounds");
    harness.click(alacer_core::geom::Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + bounds.height() * 0.5,
    ));
    harness.press_key(alacer_core::events::Key::Home);
    harness.settle();
    let node = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::Slider, "Volume").cloned())
        .expect("the slider still in the tree");
    assert_eq!(
        node.value.as_deref(),
        Some("0"),
        "the jump landed on the wall the tree carries"
    );
    let range = node.value_range.expect("the range stays mounted");
    assert_eq!(
        (range.min, range.max, range.step),
        (0.0, 100.0, Some(5.0)),
        "the write moved the value; the range is the same fact"
    );
}

#[test]
fn m97_the_dsl_words_carry_the_range() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);

    // The `min`/`max`/`step` words the slider element has parsed since
    // M32 now land where every other declared fact lands: the tree.
    let view = alacer_macro::rsx! {
        column {
            slider { name: "Volume", min: 0.0_f32, max: 100.0_f32, step: 5.0_f32, value: 40.0_f32 }
        }
    };
    let mut root = alacer_view::ViewRoot::mount(
        &view,
        harness.pipeline_mut(),
        alacer_widgets::default_registry(),
    );
    harness.settle();
    assert_eq!(
        carried_range(&harness, Role::Slider, "Volume")
            .as_ref()
            .map(|r| (r.min, r.max)),
        Some((0.0, 100.0)),
        "the words carry the span through the DSL"
    );

    // The re-render that lands where the range already is: the range is
    // constructor-immutable (there is no `set_range` door to patch —
    // `Slider::new` owns the span's invariant), so the honest absence of
    // masks — an idle frame at the range tier.
    let before = harness.counters();
    let again = alacer_macro::rsx! {
        column {
            slider { name: "Volume", min: 0.0_f32, max: 100.0_f32, step: 5.0_f32, value: 40.0_f32 }
        }
    };
    let _ = root.update(&again, harness.pipeline_mut());
    harness.tick();
    let after = harness.counters();
    assert_eq!(
        after.a11y_runs, before.a11y_runs,
        "a no-op re-render re-announces nothing"
    );
    assert_eq!(
        after.paint_runs, before.paint_runs,
        "a no-op re-render repaints nothing"
    );
}

#[test]
fn m97_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the range route's case: the ranged
    // 1,000-node slider arena — the span's cost the measurement, priced
    // against the no-range arena the toggle route already carries.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(RANGED_CASE) && benches.contains_key(BARE_CASE),
        "the range route's case and its denominator are published ({RANGED_CASE}, {BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M97 cut"
    );

    // The contract set gates both cases and prices the span's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [RANGED_CASE, BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("range_snapshot_overhead"))
        .expect("the range budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(RANGED_CASE),
        "the budget prices the ranged route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(BARE_CASE),
        "the budget prices it against the same arena, bare"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the span must not outprice the snapshot it rides"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M97 cut"
    );
}
