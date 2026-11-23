//! M98 exit-criteria integration tests (§8, ADR-0102): the orientation tier.
//! Since M89 (ADR-0093) a scroll container declares its axes at
//! construction — `List::new(height)` names the y axis its only story,
//! `List::two_axis(width, height)` declares both — and every scroll door
//! since M50 has answered per axis. The a11y tree could never say which:
//! the AT's user heard "scroll area" and did not know whether the arrows
//! ride a vertical track, a horizontal one, or both. The
//! declared-but-unreported gap, the M97 pattern at the orientation tier,
//! cashed: [`alacer_core::Widget::scroll_orientation`] beside the
//! constructors that already told the truth, the snapshot carrying it,
//! the sink mapping it — AccessKit's `Orientation` property, ARIA's
//! `aria-orientation`. The two-axis answer is the honest absence: ARIA's
//! `undefined` is the spec's own word for "both axes", and AccessKit's
//! enum has no both variant — the sink sets nothing rather than a wrong
//! half.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), the counters are the pass ledger's own (A4), the view
//! tier is the consumer's route (§13.1).

use alacer_core::passes::TreeNode;
use alacer_core::widget::{Role, ScrollOrientation};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, List, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M98 cut —
/// the orientation route's case joining the range route's case (M97),
/// the placeholder route's pair (M96), the description route's case
/// (M95), the toggle route's pair (M94), the derive route's pair (M92),
/// and the recording route's pair (M93) on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 152;

/// The suite's own read door into the published corpus: the orientation
/// route's case — the 1,000-node vertical list arena (the coordinated
/// rule's bench half, ADR-0102's exit criteria). The ratio's honest
/// denominator is the no-fact arena the toggle route already carries
/// (`toggle/snapshot_1k_binary`, the M94 binary case), the M95 pricing
/// rule: the same corpus measuring what the tier adds.
const ORIENTED_CASE: &str = "orientation/snapshot_1k_vertical";
const BARE_CASE: &str = "toggle/snapshot_1k_binary";

/// Every scroll area in the snapshot, tree order — the two-list arena
/// this suite mounts (the vertical list first, the two-axis beside it).
fn scroll_areas(harness: &TestHarness) -> Vec<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .map(|tree| {
            tree.find_all(Role::ScrollArea, "list")
                .into_iter()
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn m98_the_declared_axis_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The vertical-only constructor: the y axis is the declared story.
    let _ = harness.add_child(root, List::new(160.0).boxed());
    // The two-axis constructor: both axes are the declared story — the
    // answer ARIA calls `undefined`, the honest absence.
    let _ = harness.add_child(root, List::two_axis(120.0, 160.0).boxed());
    // The honest absence on a non-scroll role: a text input answers
    // nothing — the tree reads exactly as it read before the tier.
    let _ = harness.add_child(root, TextInput::new("Notes").boxed());
    harness.settle();

    let areas = scroll_areas(&harness);
    assert_eq!(
        areas.len(),
        2,
        "both lists ride the snapshot as scroll areas"
    );
    assert_eq!(
        areas[0].scroll_orientation,
        Some(ScrollOrientation::Vertical),
        "the vertical-only constructor declares its axis"
    );
    assert_eq!(
        areas[1].scroll_orientation, None,
        "the two-axis constructor keeps the honest absence — both axes, no single-axis word"
    );
    assert_eq!(
        harness
            .pipeline()
            .a11y_snapshot()
            .and_then(|tree| tree.find(Role::TextInput, "Notes").cloned())
            .and_then(|node| node.scroll_orientation),
        None,
        "a widget with no declared axis announces the absence"
    );
    harness.assert_all_named();
}

#[test]
fn m98_the_declaration_is_the_constructors_and_survives_the_scroll() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().boxed());
    // The handle is the test door into the shared scroll state — the
    // widget's own story stays where the constructor put it.
    let list = List::new(160.0);
    let handle = list.handle();
    let _ = harness.add_child(root, list.boxed());
    harness.settle();
    assert_eq!(
        scroll_areas(&harness)[0].scroll_orientation,
        Some(ScrollOrientation::Vertical),
        "the declaration lands at mount, where every declared fact lands"
    );

    // The scroll the tier constrains nothing of: the handle moves the
    // offset (the state's story), the declaration stays mounted (the
    // constructor's story) — the two facts the AT reads beside each
    // other, exactly the frame the canvas shows.
    handle.scroll_to(40.0);
    harness.settle();
    assert_eq!(
        scroll_areas(&harness)[0].scroll_orientation,
        Some(ScrollOrientation::Vertical),
        "the offset moved; the declared axis is the same fact"
    );
}

#[test]
fn m98_the_dsl_list_declares_its_axis() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);

    // The `height` word the list element has parsed since the M0 registry
    // builds the vertical-only constructor — the tree's truth rides the
    // word, at mount, with no new prop class to pay for.
    let view = alacer_macro::rsx! {
        column {
            list { height: 160.0 }
        }
    };
    let root = alacer_view::ViewRoot::mount(
        &view,
        harness.pipeline_mut(),
        alacer_widgets::default_registry(),
    );
    harness.settle();
    let areas = scroll_areas(&harness);
    assert_eq!(areas.len(), 1, "the view's list rides the snapshot");
    assert_eq!(
        areas[0].scroll_orientation,
        Some(ScrollOrientation::Vertical),
        "the DSL's vertical list declares its axis through the same door"
    );
    let _ = root;
}

#[test]
fn m98_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the orientation route's case: the
    // 1,000-node vertical list arena — the declaration's cost the
    // measurement, priced against the no-fact arena the toggle route
    // already carries.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(ORIENTED_CASE) && benches.contains_key(BARE_CASE),
        "the orientation route's case and its denominator are published ({ORIENTED_CASE}, {BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M98 cut"
    );

    // The contract set gates both cases and prices the declaration's
    // budget — a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [ORIENTED_CASE, BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("orientation_snapshot_overhead"))
        .expect("the orientation budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(ORIENTED_CASE),
        "the budget prices the oriented route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(BARE_CASE),
        "the budget prices it against the same corpus, bare"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the declaration must not outprice the snapshot it rides"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M98 cut"
    );
}
