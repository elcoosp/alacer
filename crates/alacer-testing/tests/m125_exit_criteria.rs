//! M125 exit-criteria integration tests (§10.3, ADR-0129): the link
//! tier — the hyperlink control and its activation verb, said by the
//! tree. Through M124 the snapshot said the one-of-N control and the
//! group it reads as (ADR-0128), the set words, the form facts, the
//! collection facts, the declared live region, the chord, the capacity,
//! the language — and the control every document vocabulary speaks as
//! "this goes somewhere" had no tier: an app wiring a hyperlink reached
//! for a button (the wrong verb — a button's activation is a *press*,
//! a link's is an *open*, and the AT's vocabulary names the difference)
//! or for styled text wearing no activation fact at all — the
//! declared-but-unreported gap (the M94 pattern), cashed at the link
//! tier: the control grows beside the actionable machinery it rides —
//! [`alacer_core::Role::Link`] carrying the "opens something" fact
//! AccessKit's own `Link` role spells, [`alacer_widgets::Link`]
//! answering the activation door with the open act (the handler the
//! app's navigation rides), the keyboard's verb the link's own (Enter,
//! the platform convention — Space stays the scroller's key the
//! framework hands on).
//!
//! The tier prices the control and nothing else: a mounted link's
//! neighbours answer exactly as they always did — the button beside it
//! presses, the radio beside it selects, the link's doors are the
//! open's. An unavailable link refuses every door honestly (the M22
//! wall, the M11 discipline); a consumed activation streams the
//! activation fact the keyboard names itself with (M70, ADR-0076).
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's doors ride the mutate seam and the
//! dispatch funnel whose `PAINT | A11Y` request re-announces exactly
//! the link's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Link, Radio};

/// The suite's corpus pin: the coordinated rule's count at the M125
/// cut — unchanged from the M124 cut. The link tier's pricing door is
/// the widget-level bench the tier carries in the widgets crate (the
/// open route priced beside the M124 radio routes), which rides
/// *outside* the published corpus: the published baseline's routes
/// price the a11y snapshot machinery (the M95 rule), and the link tier
/// grows no snapshot route. The pin re-agrees at 144.
const EXPECTED_BENCH_COUNT: usize = 144;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m125_the_link_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The link tier, mounted through the constructor words: the two
    // links (the described one beside the plain one) and the plain
    // widgets the tier keeps honest — no link word leaks onto a button
    // or a radio.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(root, Link::new("Read the architecture").boxed());
    let _ = harness.add_child(
        root,
        Link::new("API reference")
            .description("Opens the docs site in your browser")
            .boxed(),
    );
    let _ = harness.add_child(root, Button::new("Place order").boxed());
    let _ = harness.add_child(root, Radio::new("Express").checked(true).boxed());
    harness.settle();

    // The links: the "opens something" fact the role spells, the name
    // the ADR-5 contract requires, the description the tier beside it.
    let read = node(&harness, Role::Link, "Read the architecture").expect("the link rides");
    assert_eq!(
        read.description, None,
        "an undescribed link announces the absence"
    );
    let api = node(&harness, Role::Link, "API reference").expect("the described link rides");
    assert_eq!(
        api.description.as_deref(),
        Some("Opens the docs site in your browser"),
        "the help text the tree carries beside the name"
    );

    // The honest absences beside the tier: the neighbours' roles are
    // exactly what they were before the tier existed.
    let order = node(&harness, Role::Button, "Place order").expect("the button rides");
    assert_eq!(order.role, Role::Button, "the button keeps its own verb");
    let express = node(&harness, Role::Radio, "Express").expect("the radio rides");
    assert_eq!(express.role, Role::Radio, "the radio keeps its own verb");
    harness.assert_all_named();
}

#[test]
fn m125_the_activation_door_opens_and_the_wall_refuses() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Link::new("Changelog").boxed());
    let _ = harness.add_child(root, Link::new("Archived").enabled(false).boxed());
    harness.settle();

    // The AT's ask opens: the link's door consumes the activation.
    assert!(
        harness.act(Role::Link, "Changelog", A11yAction::DefaultPress),
        "the link's door answers the open ask"
    );
    harness.settle();

    // The M22 wall, at the link tier: the unavailable link's ask is
    // refused before it is parsed.
    assert!(
        !harness.act(Role::Link, "Archived", A11yAction::DefaultPress),
        "an unavailable link has no write door"
    );
    harness.assert_all_named();
}

#[test]
fn m125_the_corpus_pin_re_agrees_at_the_m125_cut() {
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

    // The published corpus pin re-agrees: the link tier grew no
    // snapshot route, and its pricing door is the widgets crate's own
    // bench — outside the published corpus, the coordinated rule
    // growing it at the next published cut.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M125 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M125 cut"
    );
}
