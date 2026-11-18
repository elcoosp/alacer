//! The a11y query API driven from the consumer side: the vocabulary the
//! README's Testing section names — `query(role).named(..)`, `count`,
//! `bounds`, `press` through the AT's own door — as one runnable
//! walkthrough. Tests query the accessibility tree, never widget
//! internals (A4's rule at the test tier): the same questions a screen
//! reader asks are the questions the harness answers.
//!
//! The beat the example exists to show: mount a small panel — one
//! button, one checkbox, one bound label — and drive it entirely
//! through the tree: find the controls by role and name, press the
//! button through the query door (the AT's `DefaultPress`, the same
//! door a screen reader's activation rides), toggle the checkbox with
//! a click landed at the tree's own bounds, and read the label's word
//! back as the tree resolves it. No signal is read, no field is
//! downcast, no widget internals are touched — the tree is the API.
//!
//! Run with `cargo run -p alacer --example a11y_queries`. The example
//! is headless — the real pass pipeline with the software raster, no
//! window (A4) — and its assertions run on every CI test pass.

use alacer::prelude::*;
use alacer_core::GetUntracked as _;
use alacer_core::flux::RenderBinding;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Checkbox, Column, Label};
use reactive_graph::traits::Set as _;

/// The tree's bounds, centered — the click lands where the tree says
/// the control is, never at a coordinate the test guessed.
fn center(bounds: alacer_core::geom::Rect) -> alacer_core::geom::Point {
    alacer_core::geom::Point::new(
        bounds.origin.x + bounds.size.width / 2.0,
        bounds.origin.y + bounds.size.height / 2.0,
    )
}

fn main() {
    let mut harness = TestHarness::new(420.0, 200.0);
    let root = harness.mount(Column::new().gap(8.0).padding(12.0).boxed());
    let _ = harness.add_child(
        root,
        Label::new("The a11y query API: the tree is the test's API (A4)").boxed(),
    );

    let sent = harness.pipeline_mut().flux().signal(0_u32);
    let binding = RenderBinding::new("Sent 0 messages".to_string());
    let label = harness
        .pipeline_mut()
        .add_child(root, Label::bound(binding.clone()).boxed());
    harness.pipeline_mut().flux_mut().bind_value(
        sent,
        label,
        &binding,
        Change::LAYOUT.union(Change::PAINT),
        |n: &u32| format!("Sent {n} messages"),
    );
    let _ = harness.add_child(
        root,
        Button::new("Send")
            .on_click(move || sent.set(sent.get_untracked() + 1))
            .boxed(),
    );
    let _ = harness.add_child(root, Checkbox::new("Archive after sending").boxed());
    harness.settle();
    harness.assert_all_named();

    // Find by role and name: the questions a screen reader asks.
    let button = harness.query(Role::Button).named("Send").bounds();
    let toggle = harness
        .query(Role::CheckBox)
        .named("Archive after sending")
        .bounds();
    println!(
        "found     : Button \"Send\" at {:.0},{:.0}; Checkbox at {:.0},{:.0}",
        button.origin.x, button.origin.y, toggle.origin.x, toggle.origin.y
    );
    assert!(
        harness.query(Role::Button).named("Send").exists()
            && harness
                .query(Role::CheckBox)
                .named("Archive after sending")
                .exists(),
        "the controls the panel mounts are the tree's answers"
    );

    // Press through the AT's own door: `DefaultPress` — the same act a
    // screen reader's activation rides — and the bound label follows.
    harness.press(Role::Button, "Send");
    harness.press(Role::Button, "Send");
    assert!(
        harness.query(Role::Label).named("Sent 2 messages").exists(),
        "the tree says what the presses landed"
    );
    println!("pressed   : the tree says \"Sent 2 messages\" — the AT's door, twice");

    // A click at the tree's own bounds: the checkbox resolves, and the
    // tree's word is the test's answer.
    harness.click(center(toggle));
    println!("toggled   : the checkbox the tree located answers the click landed at its bounds");
    println!("the API   : query, bounds, press, click — the tree, never the internals");
}
