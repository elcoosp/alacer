//! M95 exit-criteria integration tests (§8, ADR-0099): the description
//! tier. The a11y tree has always carried the name and the value — the
//! two answers a screen reader speaks first — and never the *description*:
//! the help text the AT reads when the name alone does not answer "what
//! does this control do?" (AccessKit's `Description` property, ARIA's
//! `aria-description`). The vocabulary was declared at the boundary
//! (the property every platform adapter maps) and produced by no widget —
//! the M94 pattern, the declared-but-unproduced gap, at the description
//! tier. This cut ships the producer: [`alacer_core::Widget::description`]
//! beside the name it explains, the snapshot carrying it, the sink mapping
//! it, and the view DSL's `description` word carrying it through
//! reconciliation — a patch that moves only the tree's truth queues
//! exactly `A11Y`, the pixels never paying for a fact the painter did not
//! write.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), the counters are the pass ledger's own (A4), the view
//! tier is the consumer's route (§13.1).

use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Checkbox, Column, TextInput};

/// The suite's corpus pin: the coordinated rule's count at this cut —
/// the description route's case (M95) joining the placeholder route's
/// pair (M96), the toggle route's honesty pair (M94), the derive route's
/// pair (M92), and the recording route's pair (M93) on the published
/// corpus.
const EXPECTED_BENCH_COUNT: usize = 207;

/// The suite's own read door into the published corpus: the description
/// route's 1,000-node snapshot case (the coordinated rule's bench half,
/// ADR-0099's exit criteria) — the same 1,000-node checkbox arena the
/// toggle route's binary case carries, every node described.
const DESCRIBED_CASE: &str = "describe/snapshot_1k_described";

/// The tree-carried description, read through the only official query
/// API (ADR-10).
fn carried_description(harness: &TestHarness, role: Role, name: &str) -> Option<String> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name))
        .and_then(|node| node.description.clone())
}

/// The description canon: one described control per producer door — the
/// button, the checkbox, the text input — beside undescribed controls of
/// the same roles, the honest absence riding beside the fact.
fn description_canon(harness: &mut TestHarness) -> alacer_core::arena::NodeId {
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    let _ = harness.add_child(
        root,
        Button::new("Publish")
            .description("Saves the draft and queues it for review")
            .boxed(),
    );
    let _ = harness.add_child(root, Button::new("Cancel").boxed());
    let _ = harness.add_child(
        root,
        Checkbox::new("Ship in bulk")
            .description("Marks every open order for the carrier run")
            .boxed(),
    );
    let _ = harness.add_child(root, Checkbox::new("Ship retail").boxed());
    let _ = harness.add_child(
        root,
        TextInput::new("Card number")
            .description("The 16 digits on the front of the card")
            .boxed(),
    );
    let _ = harness.add_child(root, TextInput::new("Coupon code").boxed());
    harness.settle();
    root
}

#[test]
fn m95_the_description_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 400.0);
    description_canon(&mut harness);

    // Every producer door's description is the tree's truth.
    assert_eq!(
        carried_description(&harness, Role::Button, "Publish").as_deref(),
        Some("Saves the draft and queues it for review"),
        "the described button's help text rides the snapshot"
    );
    assert_eq!(
        carried_description(&harness, Role::CheckBox, "Ship in bulk").as_deref(),
        Some("Marks every open order for the carrier run"),
        "the described checkbox's help text rides the snapshot"
    );
    assert_eq!(
        carried_description(&harness, Role::TextInput, "Card number").as_deref(),
        Some("The 16 digits on the front of the card"),
        "the described input's help text rides the snapshot"
    );

    // The absent descriptions are the honest absence — the name alone,
    // exactly the tree M94 handed over.
    assert_eq!(
        carried_description(&harness, Role::Button, "Cancel"),
        None,
        "an undescribed button announces the absence, never a guess"
    );
    assert_eq!(
        carried_description(&harness, Role::CheckBox, "Ship retail"),
        None,
        "an undescribed checkbox announces the absence"
    );
    assert_eq!(
        carried_description(&harness, Role::TextInput, "Coupon code"),
        None,
        "an undescribed input announces the absence"
    );
    harness.assert_all_named();
}

#[test]
fn m95_the_patch_tier_writes_the_description_in_place() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 280.0);

    let view = alacer_macro::rsx! {
        column { gap: 6.0,
            button { text: "Publish", description: "Saves the draft" }
        }
    };
    let mut root = alacer_view::ViewRoot::mount(
        &view,
        harness.pipeline_mut(),
        alacer_widgets::default_registry(),
    );
    harness.settle();
    assert_eq!(
        carried_description(&harness, Role::Button, "Publish").as_deref(),
        Some("Saves the draft"),
        "the `description` word carries the help text through the DSL"
    );

    // The description-only patch: the tree's truth moves, the pixels do
    // not — exactly one a11y pass, the paint pass untouched (the mask
    // the tier costs is the tier's own).
    let before = harness.counters();
    let patched = alacer_macro::rsx! {
        column { gap: 6.0,
            button { text: "Publish", description: "Queues the draft for review" }
        }
    };
    let report = root.update(&patched, harness.pipeline_mut());
    assert!(report.patched >= 1, "the description word patched in place");
    harness.tick();
    let after = harness.counters();
    assert_eq!(
        after.a11y_runs,
        before.a11y_runs + 1,
        "the description patch re-announces exactly that node's pass"
    );
    assert_eq!(
        after.paint_runs, before.paint_runs,
        "a fact the painter did not write repaints nothing"
    );
    assert_eq!(
        carried_description(&harness, Role::Button, "Publish").as_deref(),
        Some("Queues the draft for review"),
        "the patched help text is the tree's truth"
    );

    // The re-render that lands where the description already is: the
    // honest absence of masks — an idle frame at the description tier.
    let again = alacer_macro::rsx! {
        column { gap: 6.0,
            button { text: "Publish", description: "Queues the draft for review" }
        }
    };
    let before = harness.counters();
    let _ = root.update(&again, harness.pipeline_mut());
    harness.tick();
    let after = harness.counters();
    assert_eq!(
        after.a11y_runs, before.a11y_runs,
        "a no-op description patch queues no masks"
    );
    assert_eq!(
        after.paint_runs, before.paint_runs,
        "a no-op description patch repaints nothing"
    );
}

#[test]
fn m95_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the description route's case: the same
    // 1,000-node checkbox arena the toggle route's binary case carries,
    // every node described — the help text's cost the measurement.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(DESCRIBED_CASE),
        "the description route's case is published ({DESCRIBED_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M95 cut"
    );

    // The contract set gates the case and prices the description's cost —
    // a ratio against the toggle route's binary arena, the same tree the
    // described one carries (the trail-reader budget's cross-group shape).
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    let gate = gates
        .iter()
        .find(|gate| gate["baseline"].as_str() == Some(DESCRIBED_CASE))
        .expect("the case carries a regression gate");
    assert_eq!(
        gate["scope"].as_str(),
        Some("same-host"),
        "the gate is same-host scoped like the route tier's own"
    );
    let budgets = contracts["budgets"]
        .as_array()
        .expect("the contract set carries budgets");
    let budget = budgets
        .iter()
        .find(|budget| budget["id"].as_str() == Some("describe_snapshot_overhead"))
        .expect("the description budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(DESCRIBED_CASE),
        "the budget prices the described route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some("toggle/snapshot_1k_binary"),
        "the budget prices it against the same arena, undescribed"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the help text must not outprice the snapshot it rides"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M95 cut"
    );
}
