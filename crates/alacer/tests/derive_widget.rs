//! The `Widget` derive against the real headless pipeline (§11.1, the
//! `#[derive(Widget)]` option architecture §19 Q5 named, ADR-0096): the
//! a11y contract's required pair — the semantic role, the accessible
//! name, the `Any` upcast — generated at the type; every other method
//! riding the trait's own defaults. The assertions drive the a11y tree,
//! the only official query API (ADR-10).

use alacer::prelude::*;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label};

/// The derive with the full pair: a semantic role and a fixed name.
#[derive(Widget)]
#[widget(role = "toolbar", label = "Toolbar")]
struct Toolbar;

/// The derive riding the container aggregation contract (the Column and
/// Table rule: the name is the descendants').
#[derive(Widget)]
#[widget(role = "container", label = from_children)]
struct Section;

/// The derive on an actionable role: the defaults make it reachable and
/// named, and its honest refusal is the trait's own.
#[derive(Widget)]
#[widget(role = "button", label = "Save")]
struct SaveButton;

#[test]
fn the_derived_pair_lands_in_the_a11y_snapshot() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(4.0).boxed());
    let toolbar = harness.add_child(root, Box::new(Toolbar) as Box<dyn Widget>);
    let section = harness.add_child(root, Box::new(Section) as Box<dyn Widget>);
    let _ = harness.add_child(section, Label::new("Cut").boxed());
    let _ = harness.add_child(toolbar, Label::new("Copy").boxed());
    harness.settle();

    // Reachable and named through the a11y tree — the derive's pair is
    // the snapshot's role and label, laid out like any other widget.
    let toolbar = harness.expect_a11y(Role::Toolbar, "Toolbar");
    assert!(
        toolbar.bounds.is_some(),
        "the derived container laid out like any other widget"
    );
    let _found = harness.expect_a11y(Role::Label, "Cut");
    let _found = harness.expect_a11y(Role::Label, "Copy");
    harness.assert_all_named();
}

#[test]
fn from_children_rides_the_container_contract() {
    // The keyword emits the variant the containers declare themselves:
    // the same `LabelSource::FromChildren` Column and Table answer by.
    let section = Section;
    assert!(matches!(section.label(), LabelSource::FromChildren));
    let toolbar = Toolbar;
    assert!(
        matches!(toolbar.label(), LabelSource::Text(text) if text.as_ref() == "Toolbar"),
        "a string label rides LabelSource::text"
    );
}

#[test]
fn the_any_hatch_reaches_the_derived_type() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let node = harness.mount(Box::new(Toolbar) as Box<dyn Widget>);
    harness.settle();

    // The patching escape hatch (§6.1) downcasts the retained widget to
    // its concrete type — the generated `{ self }` shape is the hatch.
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
fn the_trait_defaults_ride_the_derive() {
    let mut toolbar = Toolbar;
    let save = SaveButton;
    // The availability tier's default: a reporting widget is available.
    assert!(toolbar.enabled(), "the derive's availability default holds");
    assert!(save.enabled(), "the derive's availability default holds");
    // Valueless, non-toggle, IME-off: the defaults the trait ships.
    assert_eq!(toolbar.value(), None);
    assert_eq!(toolbar.toggled(), None);
    assert_eq!(save.ime_purpose(), alacer_core::passes::ImePurpose::Off);
    // The focus derivation reads the role: actionable roles focus,
    // containers do not (§7.2 pass 2's predicate, unchanged by the derive).
    assert!(save.focusable(), "a derived button is a focus candidate");
    assert!(!toolbar.focusable(), "a derived container is not");
    // The AT's ask meets the trait's own refusal: derived widgets are
    // reachable and named, and nothing is consumed they did not accept.
    assert!(
        !toolbar.on_a11y_action(A11yAction::DefaultPress),
        "the default is the honest refusal"
    );
}

#[test]
fn the_at_s_ask_meets_the_honest_refusal_in_the_pipeline() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(4.0).boxed());
    let _ = harness.add_child(root, Box::new(SaveButton) as Box<dyn Widget>);
    harness.settle();

    assert!(
        !harness.act(Role::Button, "Save", A11yAction::DefaultPress),
        "the derived button refuses the AT's press — the default is the refusal, never a munged no-op"
    );
    harness.assert_all_named();
}
