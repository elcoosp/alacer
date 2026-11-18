//! M99 exit-criteria integration tests (§10.3, ADR-0103): the numeric
//! tier. Since M97 (ADR-0101) the a11y tree carries the span every
//! value-tier write funnels through — the walls beside the step — while
//! the value itself rode as *text*: the AT read "40" (aria-valuetext)
//! and never the number 40 (aria-valuenow), the gap ADR-0101's re-open
//! rule pre-registered verbatim: "a consumer needs the *numeric now*
//! (the value the automation and the voice-control tier read as a
//! number, `aria-valuenow`) → the sink's mapping is one setter beside
//! the walls (`set_numeric_value`), the trait door growing the answer
//! beside `value_range` where the value widgets already report their
//! scalar." Cashed at the numeric tier: [`alacer_core::Widget::
//! numeric_value`] beside the range it answers, the snapshot carrying
//! it (`TreeNode::numeric_value`), the sink mapping it — AccessKit's
//! `NumericValue` property, ARIA's `aria-valuenow`, riding the text the
//! tree already carried as the value's spoken word.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), the counters are the pass ledger's own (A4), the view
//! tier is the consumer's route (§13.1).

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Progress, Slider, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M99 cut —
/// the numeric route's case joining the orientation route's case (M98),
/// the range route's case (M97), the placeholder route's pair (M96),
/// the description route's case (M95), the toggle route's pair (M94),
/// the derive route's pair (M92), and the recording route's pair (M93)
/// on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 120;

/// The suite's own read door into the published corpus: the numeric
/// route's case — the 1,000-node value arena whose every node answers
/// the number beside the text (the coordinated rule's bench half,
/// ADR-0103's exit criteria). The ratio's honest denominator is the
/// no-fact arena the toggle route already carries
/// (`toggle/snapshot_1k_binary`, the M94 binary case), the M95 pricing
/// rule: the same corpus measuring what the tier adds.
const VALUED_CASE: &str = "numeric/snapshot_1k_valued";
const BARE_CASE: &str = "toggle/snapshot_1k_binary";

/// The first slider in the snapshot, if the tree carries one.
fn slider_node(harness: &TestHarness) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::Slider, "Volume").cloned())
}

#[test]
fn m99_the_numeric_now_lands_beside_the_text_value() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The value tier's control: the scalar the funnel holds, the span
    // the tree carried since M97, and now the number beside the text.
    let _ = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .step(5.0)
            .value(40.0)
            .boxed(),
    );
    // The report half: the app writes it, the AT reads it — the number
    // riding the same door the text does.
    let _ = harness.add_child(
        root,
        Progress::new("Upload", 0.0, 100.0).value(30.0).boxed(),
    );
    // The honest absences: a text field's value is prose, not a scalar
    // — the tree announces the text and no number; a button never
    // carried a value at all.
    let _ = harness.add_child(root, TextInput::new("Notes").boxed());
    let _ = harness.add_child(root, Button::new("Save").boxed());
    harness.settle();

    let slider = slider_node(&harness).expect("the slider rides the snapshot");
    assert_eq!(
        slider.value.as_deref(),
        Some("40"),
        "the text value rides exactly as it always has — the spoken word"
    );
    assert_eq!(
        slider.numeric_value,
        Some(40.0),
        "the numeric now rides beside the text — aria-valuenow to its aria-valuetext"
    );
    assert_eq!(
        slider.value_range.map(|range| range.min),
        Some(0.0),
        "the M97 walls ride beside the number, unchanged"
    );

    let progress = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::ProgressIndicator, "Upload").cloned())
        .expect("the report rides the snapshot");
    assert_eq!(
        progress.numeric_value,
        Some(30.0),
        "the report half answers the number too — the read side of the value tier"
    );

    let absence = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, "Notes").cloned())
        .expect("the field rides the snapshot");
    assert_eq!(
        absence.numeric_value, None,
        "a text field's value is prose — the honest absence, never a parsed guess"
    );
    let button = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::Button, "Save").cloned())
        .expect("the button rides the snapshot");
    assert_eq!(
        button.numeric_value, None,
        "a valueless role announces nothing — the absence the tree always carried"
    );
    harness.assert_all_named();
}

#[test]
fn m99_the_write_moves_the_number_the_tree_carries() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 160.0);
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Slider::new("Volume", 0.0, 100.0).value(40.0).boxed());
    harness.settle();
    assert_eq!(
        slider_node(&harness).and_then(|node| node.numeric_value),
        Some(40.0),
        "the fact lands at mount, where every declared fact lands"
    );

    // The AT's write: the same funnel the drag rides, the number and
    // the text moving together — one tier, two shapes of one answer.
    assert!(
        harness.act(Role::Slider, "Volume", A11yAction::SetValue("85".into())),
        "the slider consumes the AT's write"
    );
    let node = slider_node(&harness).expect("the slider survives the write");
    assert_eq!(
        node.numeric_value,
        Some(85.0),
        "the number the tree carries follows the funnel's landing"
    );
    assert_eq!(
        node.value.as_deref(),
        Some("85"),
        "the text follows the same landing — the pair never disagrees"
    );

    // The write that lands where the value already is: the idle-frame
    // property the value tier has answered since M20 — the funnel
    // queues nothing, and the tree's pair stays where it was.
    assert!(
        harness.act(Role::Slider, "Volume", A11yAction::SetValue("85".into())),
        "the write is consumed; the value is where the ask puts it"
    );
    assert_eq!(
        slider_node(&harness).and_then(|node| node.numeric_value),
        Some(85.0),
        "the no-op write leaves the pair exactly where it was"
    );
}

#[test]
fn m99_the_dsl_slider_declares_its_number() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);

    // The `value` word the slider element has parsed since M32 builds
    // the scalar the funnel holds — the tree's number rides the word,
    // at mount, with no new prop class to pay for.
    let view = alacer_macro::rsx! {
        column {
            slider { name: "Volume", min: 0.0, max: 100.0, step: 5.0, value: 40.0 }
        }
    };
    let root = alacer_view::ViewRoot::mount(
        &view,
        harness.pipeline_mut(),
        alacer_widgets::default_registry(),
    );
    harness.settle();
    let node = slider_node(&harness).expect("the view's slider rides the snapshot");
    assert_eq!(
        node.numeric_value,
        Some(40.0),
        "the DSL's slider declares its number through the same door"
    );
    let _ = root;
}

#[test]
fn m99_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the numeric route's case: the
    // 1,000-node value arena — the number's cost the measurement,
    // priced against the no-fact arena the toggle route already
    // carries.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(VALUED_CASE) && benches.contains_key(BARE_CASE),
        "the numeric route's case and its denominator are published ({VALUED_CASE}, {BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M99 cut"
    );

    // The contract set gates both cases and prices the number's
    // budget — a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [VALUED_CASE, BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("numeric_snapshot_overhead"))
        .expect("the numeric budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(VALUED_CASE),
        "the budget prices the valued route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(BARE_CASE),
        "the budget prices it against the same corpus, bare"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the number must not outprice the snapshot it rides"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M99 cut"
    );
}
