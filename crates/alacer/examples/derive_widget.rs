//! The `Widget` derive (M92, ADR-0096) driven from the consumer side:
//! the a11y contract's required pair — the semantic role, the accessible
//! name, the `Any` upcast — generated at the type from two
//! `#[widget(...)]` attributes, every other method riding the trait's
//! own defaults. The beats: the derived pair landing in the a11y
//! snapshot the AT reads (the attribute's words are the snapshot's role
//! and name), the `from_children` form riding the container aggregation
//! contract Column and Table declare themselves, the defaults keeping
//! the derived widget exactly as honest as the hand-written one beside
//! it, and the AT's press meeting the trait's own refusal.
//!
//! Run with `cargo run -p alacer --example derive_widget`. The example
//! is headless — the real pass pipeline (A4) — so the same beats run
//! anywhere the tests do.

use alacer_core::passes::ImePurpose;
use alacer_core::widget::{A11yAction, LabelSource, Role, Widget};
// The derive beside the trait it implements — one name, two namespaces
// (the serde pattern the prelude ships).
use alacer_macro::Widget;
use alacer_testing::TestHarness;
use alacer_widgets::Column;

/// The toolbar: the pair in its smallest shape — a semantic role and a
/// fixed name, two attributes where three trait methods stood. The word
/// is the M129 tier's own (`Role::Toolbar`) — the derive speaks the
/// whole vocabulary the snapshot prints, the tool strip saying its own
/// word instead of borrowing the container's.
#[derive(Widget)]
#[widget(role = "toolbar", label = "Toolbar")]
struct Toolbar;

/// The save door: an actionable role. Reachable and named by the pair;
/// the focus derivation reads the role (§7.2 pass 2), and the AT's
/// press meets the trait's own refusal — nothing is consumed the
/// widget did not accept.
#[derive(Widget)]
#[widget(role = "button", label = "Save")]
struct SaveButton;

/// The section: the `from_children` form — the container aggregation
/// contract ([`LabelSource::FromChildren`]), the exact variant Column
/// and Table declare themselves, so the derived container consumes
/// whatever aggregation the pass grows without a second form existing.
#[derive(Widget)]
#[widget(role = "container", label = from_children)]
struct Section;

/// Beat 1: the derived pair lands in the a11y snapshot — mounted
/// through the real pipeline, laid out like any other widget, the
/// attribute's words carried as the snapshot's role and name.
fn the_pair_lands_in_the_snapshot() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(4.0).boxed());
    let _toolbar = harness.add_child(root, Box::new(Toolbar) as Box<dyn Widget>);
    let _save = harness.add_child(root, Box::new(SaveButton) as Box<dyn Widget>);
    harness.settle();

    let toolbar = harness.expect_a11y(Role::Toolbar, "Toolbar");
    assert!(
        toolbar.bounds.is_some(),
        "the derived container laid out like any other widget"
    );
    let save = harness.expect_a11y(Role::Button, "Save");
    assert!(
        save.bounds.is_some(),
        "the derived button laid out like any other widget"
    );
    harness.assert_all_named();
    println!(
        "beat 1: the snapshot reads the pair — toolbar \"Toolbar\", button \"Save\", both laid out, all named"
    );
}

/// Beat 2: the two name forms answer their own contracts — a string
/// rides [`LabelSource::text`], the keyword rides
/// [`LabelSource::FromChildren`], and no third form exists.
fn the_name_forms_answer_their_contracts() {
    let toolbar = Toolbar;
    assert!(
        matches!(toolbar.label(), LabelSource::Text(text) if text.as_ref() == "Toolbar"),
        "the string form is the fixed-text source"
    );
    let section = Section;
    assert!(
        matches!(section.label(), LabelSource::FromChildren),
        "the keyword is the container contract, not a new species"
    );
    println!(
        "beat 2: the string form rides LabelSource::text, `from_children` rides the container contract"
    );
}

/// Beat 3: the defaults ride the trait — the availability tier, the
/// valueless answers, the IME flavor, and the focus derivation reading
/// the role (actionable roles focus, reporting roles do not). A derived
/// widget is exactly as honest as the hand-written one beside it.
fn the_defaults_ride_the_trait() {
    let save = SaveButton;
    assert!(save.enabled(), "the availability default holds");
    assert_eq!(save.value(), None, "the value default holds");
    assert_eq!(save.toggled(), None, "the toggle default holds");
    assert_eq!(save.ime_purpose(), ImePurpose::Off, "the IME default holds");
    assert!(save.focusable(), "a derived button is a focus candidate");

    let toolbar = Toolbar;
    assert!(
        !toolbar.focusable(),
        "a derived tool strip is not a focus candidate"
    );
    println!(
        "beat 3: the defaults ride the trait — available, valueless, IME-off, focusable by the role's own predicate"
    );
}

/// Beat 4: the `Any` hatch reaches the derived type — the patching
/// escape hatch (§6.1) downcasts the retained widget through the
/// generated `{ self }` upcast, the same discipline Masonry's
/// `WidgetMut` machinery uses.
fn the_any_hatch_reaches_the_derived_type() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let node = harness.mount(Box::new(Toolbar) as Box<dyn Widget>);
    harness.settle();

    let patched = harness
        .pipeline_mut()
        .arena_mut()
        .mutate(node)
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Toolbar>()
        .is_some();
    assert!(patched, "the generated hatch downcasts to the derived type");
    println!("beat 4: the Any hatch downcasts the retained widget to its derived type");
}

/// Beat 5: the AT's ask meets the honest refusal — the derived button
/// is reachable and named, and the default `on_a11y_action` refuses
/// the press (the M11 rule: refused, never munged; the route's
/// answer, never a dropped route).
fn the_at_s_ask_meets_the_honest_refusal() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(4.0).boxed());
    let _save = harness.add_child(root, Box::new(SaveButton) as Box<dyn Widget>);
    harness.settle();

    assert!(
        !harness.act(Role::Button, "Save", A11yAction::DefaultPress),
        "the default on_a11y_action is the honest refusal"
    );
    println!(
        "beat 5: the AT's press on the derived button met the trait's own refusal — reachable, named, and honest"
    );
}

fn main() {
    println!("=== the Widget derive (M92, ADR-0096) ===");
    the_pair_lands_in_the_snapshot();
    the_name_forms_answer_their_contracts();
    the_defaults_ride_the_trait();
    the_any_hatch_reaches_the_derived_type();
    the_at_s_ask_meets_the_honest_refusal();
    println!("=== every beat answered — the required pair is two attributes ===");
}
