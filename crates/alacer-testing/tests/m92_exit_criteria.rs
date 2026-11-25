//! M92 exit-criteria integration tests (§11.1, ADR-0096): the `#[derive(Widget)]`
//! form — architecture §19 Q5's own words pre-registered the cut: "Required
//! a11y methods won't poison ergonomics … answered by the shipped shape:
//! required trait methods since M0 (no derive yet); gather feedback at 0.1,
//! the `#[derive(Widget)]` form stays the option." The feedback the 0.17.0
//! cut gathered priced the option: a hand-written widget writes the required
//! pair — the role, the name, the `Any` upcast — by hand on every type, the
//! same three lines, the boilerplate the option existed to retire.
//!
//! The derive grows into `alacer-macro` beside the `rsx!` compiler it already
//! hosts: the attribute names the widget's role (the [`Role::name`]
//! vocabulary the whole framework prints — `button`, `container`,
//! `text_input`, …) and its accessible name (a string, or `from_children`
//! when the container aggregates its descendants — the Column and Table
//! rule). The expansion is the required pair and nothing else: `role`,
//! `label`, `as_any_mut` as `{ self }` — every other method rides the
//! trait's own default, so a derived widget is exactly as honest as the
//! hand-written one beside it.
//!
//! Every test runs the real pipeline (A4): real layout, real a11y passes,
//! the snapshot the AT reads — the derive's product is the snapshot, so the
//! snapshot is what the suite pins.

use alacer_core::passes::ImePurpose;
use alacer_core::widget::{A11yAction, LabelSource, Role, Widget};
// The derive beside the trait it implements — one name, two namespaces
// (the serde pattern the facade prelude ships).
use alacer_macro::Widget;
use alacer_testing::TestHarness;
use alacer_widgets::Column;

/// The suite's corpus pin: the coordinated rule's count at the M93 cut —
/// the derive route's two cases (M92) beside the recording route's pair
/// (M93) joining the published corpus in the same cut.
const EXPECTED_BENCH_COUNT: usize = 205;

/// The suite's own read doors into the published corpus: the derive route's
/// zero-overhead cases (the coordinated rule's bench half, ADR-0096's exit
/// criteria).
const DERIVED_CASE: &str = "derive/snapshot_1k_derived";
const HANDWRITTEN_CASE: &str = "derive/snapshot_1k_handwritten";

/// The derive with the full pair: a semantic role and a fixed name.
#[derive(Widget)]
#[widget(role = "container", label = "Toolbar")]
struct Toolbar;

/// The derive riding the container aggregation contract.
#[derive(Widget)]
#[widget(role = "container", label = from_children)]
struct Section;

/// The derive on an actionable role: reachable and named by the pair,
/// refused by the trait's own default answer.
#[derive(Widget)]
#[widget(role = "button", label = "Save")]
struct SaveButton;

/// The derive on a valueless reporting role.
#[derive(Widget)]
#[widget(role = "progress_indicator", label = "Indexing")]
struct Indexing;

#[test]
fn m92_the_derived_pair_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(4.0).boxed());
    let _toolbar = harness.add_child(root, Box::new(Toolbar) as Box<dyn Widget>);
    let _save = harness.add_child(root, Box::new(SaveButton) as Box<dyn Widget>);
    let _indexing = harness.add_child(root, Box::new(Indexing) as Box<dyn Widget>);
    harness.settle();

    // The role the attribute named is the role the snapshot carries —
    // the vocabulary the whole framework prints, spoken by the derive.
    let toolbar = harness.expect_a11y(Role::Container, "Toolbar");
    assert!(
        toolbar.bounds.is_some(),
        "the derived widget laid out like any other widget"
    );
    let _found = harness.expect_a11y(Role::Button, "Save");
    let _found = harness.expect_a11y(Role::ProgressIndicator, "Indexing");
    harness.assert_all_named();
}

#[test]
fn m92_the_label_forms_answer_their_own_contracts() {
    // A string label rides `LabelSource::text` — the fixed-text source
    // every hand-written named widget answers by.
    let toolbar = Toolbar;
    assert!(
        matches!(toolbar.label(), LabelSource::Text(text) if text.as_ref() == "Toolbar"),
        "the string form is the fixed-text source"
    );
    // The `from_children` keyword rides the container aggregation
    // contract — the exact variant Column and Table declare themselves,
    // so a derived container consumes whatever aggregation the pass
    // grows without a second form existing.
    let section = Section;
    assert!(
        matches!(section.label(), LabelSource::FromChildren),
        "the keyword is the container contract, not a new species"
    );
}

#[test]
fn m92_the_defaults_ride_the_trait() {
    let save = SaveButton;
    let indexing = Indexing;
    // The availability tier: reporting widgets are available, the
    // trait's own default — the derive never invents an availability.
    assert!(save.enabled(), "the availability default holds");
    assert!(indexing.enabled(), "the availability default holds");
    // Valueless, non-toggle, IME-off: the defaults, unchanged.
    assert_eq!(save.value(), None);
    assert_eq!(save.toggled(), None);
    assert_eq!(save.ime_purpose(), ImePurpose::Off);
    assert_eq!(save.table_slot(), None);
    assert_eq!(save.selection(), None);
    // The focus derivation reads the role: actionable roles focus,
    // reporting roles do not — the §7.2 pass 2 predicate, unchanged.
    assert!(save.focusable(), "a derived button is a focus candidate");
    assert!(!indexing.focusable(), "a report is not a focus candidate");
    // The paint default emits nothing: children are painted by the pass
    // system walking the tree, the container's own contract.
    let toolbar = Toolbar;
    assert!(toolbar.accepts_pointer(), "the pointer default holds");
}

#[test]
fn m92_the_any_hatch_reaches_the_derived_type() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let node = harness.mount(Box::new(Toolbar) as Box<dyn Widget>);
    harness.settle();

    // The patching escape hatch (§6.1) downcasts the retained widget to
    // its concrete type — the generated `{ self }` shape is the hatch,
    // the same discipline Masonry's `WidgetMut` machinery uses.
    let patched = harness
        .pipeline_mut()
        .arena_mut()
        .mutate(node)
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Toolbar>()
        .is_some();
    assert!(patched, "the generated hatch downcasts to the derived type");
}

#[test]
fn m92_the_at_s_ask_meets_the_honest_refusal() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(4.0).boxed());
    let _save = harness.add_child(root, Box::new(SaveButton) as Box<dyn Widget>);
    harness.settle();

    // The derived button is reachable and named, and the AT's press
    // meets the trait's own refusal — never a munged no-op, never a
    // dropped route (the M11 rule the route answers by).
    assert!(
        !harness.act(Role::Button, "Save", A11yAction::DefaultPress),
        "the default on_a11y_action is the honest refusal"
    );
}

#[test]
fn m92_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the derive route's zero-overhead
    // pair: the derived tree and the hand-written tree, the same roles,
    // the same names, the same 1,000-node arena.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(DERIVED_CASE) && benches.contains_key(HANDWRITTEN_CASE),
        "the derive route's pair is published ({DERIVED_CASE}, {HANDWRITTEN_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M93 cut"
    );

    // The contract set gates both cases (same-host, the route tier's
    // own scope) and prices the zero-overhead budget — a ratio, the
    // host-independent contract class the M5 design shipped.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [DERIVED_CASE, HANDWRITTEN_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("derive_zero_overhead"))
        .expect("the zero-overhead budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(DERIVED_CASE),
        "the budget prices the derived tree"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(HANDWRITTEN_CASE),
        "the budget prices it against the hand-written tree"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "derived must not cost more than hand-written"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 205;"),
        "the m5 corpus pin re-agrees at the M93 cut"
    );
}
