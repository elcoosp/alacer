//! M96 exit-criteria integration tests (§8, ADR-0100): the placeholder
//! tier. The field has painted its hint since M6 — the grey sentence an
//! empty input shows — and the a11y tree could never say it: the AT read
//! an empty value where the sighted user read "Search…". The
//! declared-but-unreported gap, the M94 pattern at the placeholder tier,
//! cashed: [`alacer_core::Widget::placeholder`] beside the paint that
//! already told the truth, the snapshot carrying it, the sink mapping
//! it — an empty placeholder is the honest absence, never an empty hint.
//! The view's patch tier pays both masks the tier costs: the paint the
//! painter draws and the re-announcement the tree carries (the a11y-
//! *also* prop class, ADR-0100's own rule).
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), the counters are the pass ledger's own (A4), the view
//! tier is the consumer's route (§13.1).

use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M96 cut —
/// the placeholder route's honesty pair joining the description route's
/// case (M95), the toggle route's pair (M94), the derive route's pair
/// (M92), and the recording route's pair (M93) on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 131;

/// The suite's own read doors into the published corpus: the placeholder
/// route's pair — the hinted 1,000-node input arena beside the bare one
/// (the coordinated rule's bench half, ADR-0100's exit criteria).
const PLACEHOLDERED_CASE: &str = "placeholder/snapshot_1k_placeholdered";
const BARE_CASE: &str = "placeholder/snapshot_1k_bare";

/// The tree-carried placeholder, read through the only official query
/// API (ADR-10).
fn carried_placeholder(harness: &TestHarness, name: &str) -> Option<String> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, name))
        .and_then(|node| node.placeholder.clone())
}

#[test]
fn m96_the_placeholder_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 200.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    let _ = harness.add_child(
        root,
        TextInput::new("Search")
            .placeholder("Find a parts number…")
            .boxed(),
    );
    let _ = harness.add_child(root, TextInput::new("Notes").boxed());
    harness.settle();

    // The hint the field paints is the hint the tree announces.
    assert_eq!(
        carried_placeholder(&harness, "Search").as_deref(),
        Some("Find a parts number…"),
        "the painted hint is the tree's truth"
    );
    // The empty placeholder is the honest absence, never an empty hint.
    assert_eq!(
        carried_placeholder(&harness, "Notes"),
        None,
        "a field with no hint announces the absence"
    );
    harness.assert_all_named();
}

#[test]
fn m96_the_value_arrives_and_the_hint_stays_mounted() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(
        root,
        TextInput::new("Search")
            .placeholder("Find a parts number…")
            .boxed(),
    );
    harness.settle();
    assert!(carried_placeholder(&harness, "Search").is_some());

    // Focus the field the way the pointer does, then the typing door's
    // own beat: the committed value replaces the hint on the canvas, and
    // the tree's truth follows — the value is what the AT reads first,
    // the placeholder the field's own static hint beside it (the
    // property stays mounted; the painter stops drawing it once the
    // value lands).
    let bounds = harness
        .query(Role::TextInput)
        .named("Search")
        .first()
        .bounds
        .expect("the laid-out field has bounds");
    harness.click(alacer_core::geom::Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + bounds.height() * 0.5,
    ));
    harness.settle();
    harness.type_text("axle");
    harness.settle();
    let node = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, "Search").cloned())
        .expect("the field still in the tree");
    assert_eq!(
        node.value.as_deref(),
        Some("axle"),
        "the value is the tree's truth once typed"
    );
    assert_eq!(
        node.placeholder.as_deref(),
        Some("Find a parts number…"),
        "the placeholder is the field's own hint — it answers while empty, \
         and the AT reads the value first"
    );
}

#[test]
fn m96_the_patch_tier_pays_both_masks() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);

    let view = alacer_macro::rsx! {
        column {
            text_input { name: "Search", placeholder: "Find a parts number…" }
        }
    };
    let mut root = alacer_view::ViewRoot::mount(
        &view,
        harness.pipeline_mut(),
        alacer_widgets::default_registry(),
    );
    harness.settle();
    assert_eq!(
        carried_placeholder(&harness, "Search").as_deref(),
        Some("Find a parts number…"),
        "the `placeholder` word carries the hint through the DSL"
    );

    // The hint patch: the painter draws the new hint AND the tree
    // re-announces — both masks the tier costs, the a11y-*also* class
    // adding the announcement beside the paint (ADR-0100's own rule).
    let before = harness.counters();
    let patched = alacer_macro::rsx! {
        column {
            text_input { name: "Search", placeholder: "Find a torque spec…" }
        }
    };
    let report = root.update(&patched, harness.pipeline_mut());
    assert!(report.patched >= 1, "the placeholder word patched in place");
    harness.tick();
    let after = harness.counters();
    assert_eq!(
        after.paint_runs,
        before.paint_runs + 1,
        "the painter draws the new hint"
    );
    assert_eq!(
        after.a11y_runs,
        before.a11y_runs + 1,
        "the tree re-announces beside the paint"
    );
    assert_eq!(
        carried_placeholder(&harness, "Search").as_deref(),
        Some("Find a torque spec…"),
        "the patched hint is the tree's truth"
    );

    // The re-render that lands where the hint already is: the honest
    // absence of masks — an idle frame at the placeholder tier.
    let again = alacer_macro::rsx! {
        column {
            text_input { name: "Search", placeholder: "Find a torque spec…" }
        }
    };
    let before = harness.counters();
    let _ = root.update(&again, harness.pipeline_mut());
    harness.tick();
    let after = harness.counters();
    assert_eq!(
        after.a11y_runs, before.a11y_runs,
        "a no-op placeholder patch queues no masks"
    );
    assert_eq!(
        after.paint_runs, before.paint_runs,
        "a no-op placeholder patch repaints nothing"
    );
}

#[test]
fn m96_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the placeholder route's pair: the
    // hinted 1,000-node input arena beside the bare one — the hint's
    // cost the measurement.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(PLACEHOLDERED_CASE) && benches.contains_key(BARE_CASE),
        "the placeholder route's pair is published ({PLACEHOLDERED_CASE}, {BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M96 cut"
    );

    // The contract set gates both cases and prices the hint's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [PLACEHOLDERED_CASE, BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("placeholder_snapshot_overhead"))
        .expect("the placeholder budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(PLACEHOLDERED_CASE),
        "the budget prices the hinted route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(BARE_CASE),
        "the budget prices it against the same arena, bare"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the hint must not outprice the snapshot it rides"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M96 cut"
    );
}
