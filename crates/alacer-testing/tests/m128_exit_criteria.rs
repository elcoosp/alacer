//! M128 exit-criteria integration tests (§10.3, ADR-0132): the
//! spin-button tier — the bounded number field the AT walks by arrows,
//! said by the tree. Through M127 the snapshot said the page selector
//! and its strip (ADR-0131), the section title (ADR-0130), the
//! hyperlink (ADR-0129), the one-of-N control and its group
//! (ADR-0128), the set words, the form facts, the collection facts —
//! and the number field had no tier: an app wiring one reached for a
//! slider wearing text (the wrong shape — a slider's write is the
//! *continuous drag*, a spin button's write is the *discrete step* the
//! arrows speak) or for a bare text input carrying no value words at
//! all. [`alacer_core::Role::SpinButton`] carries the "writes one
//! bounded number" fact AccessKit's own `SpinButton` role spells; the
//! value rides the M3 `value()` contract, the span the M97 range word,
//! the number the M99 numeric word — the value tier's own vocabulary,
//! never a second one.
//!
//! The tier prices the field and nothing else: a mounted field's
//! neighbours answer exactly as they always did — the button beside it
//! presses, the heading beside it names, the field's doors are the
//! value's. An unavailable field refuses every door honestly (the M22
//! wall, the M11 discipline); a write that lands where the value
//! already is is the M21 idle frame; the walls the funnel lands at
//! name themselves on the value log (the M49 rule).
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's doors ride the mutate seam and the
//! dispatch funnel whose `PAINT | A11Y` request re-announces exactly
//! the field's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Heading, SpinButton};

/// The suite's corpus pin: the coordinated rule's count at the M128
/// cut — unchanged from the M127 cut. The spin-button tier grows no
/// snapshot route (its words are the value tier's, published since
/// M3/M97/M99), and its pricing door is the widget-level bench the
/// tier carries in the widgets crate (`spin_button/step_route_256`
/// against `spin_button/redundant_ask_256`), which rides *outside* the
/// published corpus: the published baseline's routes price the a11y
/// snapshot machinery (the M95 rule), and the coordinated rule grows
/// it at the next published cut, never at a widget bench's. The pin
/// re-agrees at 155.
const EXPECTED_BENCH_COUNT: usize = 207;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m128_the_spin_button_words_land_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The field, mounted through the constructor words: the named
    // bounded number the AT walks by arrows — the value the M3
    // contract carries, the span the M97 range word announces, the
    // number the M99 numeric word beside it — and the plain widgets
    // the tier keeps honest. No spin-button word leaks onto a button
    // or a heading.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(root, Heading::new("Checkout").level(1).boxed());
    let _ = harness.add_child(
        root,
        SpinButton::new("Quantity", 1.0, 12.0)
            .step(1.0)
            .value(3.0)
            .description("How many, one to twelve")
            .boxed(),
    );
    let _ = harness.add_child(root, Button::new("Pay").boxed());
    harness.settle();

    // The field: the "writes one bounded number" fact the role spells,
    // the name the ADR-5 contract requires, the value words the tree
    // carries — the spoken word, the number beside it, the span and
    // the step the AT reads before it asks.
    let quantity = node(&harness, Role::SpinButton, "Quantity").expect("the field rides");
    assert_eq!(
        quantity.role,
        Role::SpinButton,
        "the field keeps its own word"
    );
    assert_eq!(
        quantity.value.as_deref(),
        Some("3"),
        "the value the funnel holds is the word the tree carries"
    );
    assert_eq!(
        quantity.numeric_value,
        Some(3.0),
        "the number behind the spoken word rides beside it"
    );
    let range = quantity
        .value_range
        .expect("the span the AT reads before it asks");
    assert_eq!(range.min, 1.0, "the lower wall");
    assert_eq!(range.max, 12.0, "the upper wall");
    assert_eq!(range.step, Some(1.0), "the step the arrows move");

    // The honest absences beside the tier: the neighbours' roles are
    // exactly what they were before the tier existed.
    let checkout = node(&harness, Role::Heading, "Checkout").expect("the heading rides");
    assert_eq!(
        checkout.role,
        Role::Heading,
        "the heading keeps its own word"
    );
    let pay = node(&harness, Role::Button, "Pay").expect("the button rides");
    assert_eq!(pay.role, Role::Button, "the button keeps its own press");
    harness.assert_all_named();
}

#[test]
fn m128_the_at_doors_step_and_the_wall_refuses() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(
        root,
        SpinButton::new("Quantity", 1.0, 12.0).value(3.0).boxed(),
    );
    let _ = harness.add_child(
        root,
        SpinButton::new("Archived", 0.0, 10.0)
            .enabled(false)
            .boxed(),
    );
    harness.settle();

    // The AT's ask steps: the field's door consumes the increment and
    // the value moves one declared step — the tree re-announces
    // exactly the field's node.
    assert!(
        harness.act(Role::SpinButton, "Quantity", A11yAction::Increment),
        "the field's door answers the step ask"
    );
    harness.settle();
    let quantity = node(&harness, Role::SpinButton, "Quantity").expect("the moved field rides");
    assert_eq!(
        quantity.numeric_value,
        Some(4.0),
        "the ask stepped the value one declared step"
    );

    // The AT's write: SetValue parses and rides the same clamp-and-snap
    // funnel every other door rides (the M20 rule at the field tier).
    assert!(
        harness.act(
            Role::SpinButton,
            "Quantity",
            A11yAction::SetValue("10".to_string()),
        ),
        "the field's door answers the write ask"
    );
    harness.settle();
    let quantity = node(&harness, Role::SpinButton, "Quantity").expect("the written field rides");
    assert_eq!(quantity.numeric_value, Some(10.0), "the write landed");

    // The M22 wall, at the spin-button tier: the unavailable field's
    // ask is refused before it is parsed.
    assert!(
        !harness.act(Role::SpinButton, "Archived", A11yAction::Increment),
        "an unavailable field has no write door"
    );
    harness.settle();
    harness.assert_all_named();
}

#[test]
fn m128_the_field_paints_its_own_snapshot() {
    // The tier's visual arrangement — the value word beside its
    // steppers — is the pixels the regression net holds: one blessed
    // frame, the corpus growing exactly one route (§13.4's hygiene,
    // the recipe docs/examples.md names).
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);
    let root = harness.mount(Column::new().padding(8.0).boxed());
    let _ = harness.add_child(
        root,
        SpinButton::new("Quantity", 1.0, 12.0)
            .step(1.0)
            .value(3.0)
            .boxed(),
    );
    harness.settle();
    harness.snapshot("spin_button");
}

#[test]
fn m128_the_corpus_pin_re_agrees_at_the_m128_cut() {
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

    // The published corpus pin re-agrees: the spin-button tier grew no
    // snapshot route, and its pricing door is the widgets crate's own
    // bench — outside the corpus at its own cut, joined by the corpus catch-up's published cut — the coordinated rule's next published cut cashed.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M128 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M128 cut"
    );
}
