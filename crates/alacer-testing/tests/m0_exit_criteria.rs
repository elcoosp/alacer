//! M0 exit-criteria integration tests (§18): counter + scrolling list render;
//! masks flow end-to-end; the ADR-6 scheduling contract holds.
//!
//! Every test runs the real pipeline with the software rasterizer (A4).

use alacer_core::color::Color;
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Point;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;

fn center(rect: alacer_core::geom::Rect) -> Point {
    Point::new(
        rect.origin.x + rect.size.width * 0.5,
        rect.origin.y + rect.size.height * 0.5,
    )
}
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, List, Panel, Row};

/// The M0 counter: a bound label, a button, one signal, one write batch per click.
fn mount_counter(harness: &mut TestHarness) -> (alacer_core::arena::NodeId, RenderBinding<String>) {
    use reactive_graph::traits::{GetUntracked as _, Set as _};

    let root = harness.mount(Column::new().gap(8.0).padding(12.0).boxed());

    let count = harness.pipeline().flux().signal(0_i32);
    let label_binding = RenderBinding::new("Clicked 0 times".to_string());
    let label = harness.add_child(root, Label::bound(label_binding.clone()).boxed());
    let button = harness.add_child(
        root,
        Button::new("Increment")
            .on_click(move || {
                // One write batch per click: the ADR-6 contract at the seam.
                alacer_core::flux::Flux::batch_of(|| {
                    let current = count.try_get_untracked().unwrap_or_default();
                    count.set(current + 1);
                });
            })
            .boxed(),
    );

    harness.pipeline_mut().flux_mut().bind_value(
        count,
        label,
        &label_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Clicked {n} times"),
    );

    harness.settle();
    (button, label_binding)
}

/// A scrolling list with 24 colored panels.
fn mount_list(
    harness: &mut TestHarness,
) -> (alacer_core::arena::NodeId, alacer_widgets::ListHandle) {
    let root = harness.mount(Column::new().boxed());
    let list = List::new(160.0);
    let handle = list.handle();
    let list_node = harness.add_child(root, list.boxed());

    let colors = [
        Color::rgb(239, 68, 68),
        Color::rgb(59, 130, 246),
        Color::rgb(34, 197, 94),
        Color::rgb(234, 179, 8),
    ];
    for i in 0..24 {
        let _ = harness.add_child(
            list_node,
            Panel::new(28.0, colors[i % colors.len()]).boxed(),
        );
    }
    harness.settle();
    (list_node, handle)
}

#[test]
fn m0_counter_renders_and_updates_through_masks() {
    // reactive_graph + taffy + parley + accesskit on one (headless) window:
    // masks flow end-to-end from signal write to pixels (§18 M0).
    let mut harness = TestHarness::new(320.0, 240.0);
    let (button, _binding) = mount_counter(&mut harness);

    // The a11y tree is the query surface: reachable, named (§11.5).
    let label = harness.expect_a11y(Role::Label, "Clicked 0 times");
    assert_ne!(label.id.to_string(), "", "the a11y id renders");

    // Click the button through the event pass (hit-test + bubble).
    let bounds = harness.bounds(button);
    harness.click(center(bounds));

    // The label's a11y name now reflects the new count — state flowed from
    // the click handler through flux to the widget via masks.
    let _ = harness.expect_a11y(Role::Label, "Clicked 1 times");

    // Counter scheduling contract (ADR-6, at E2E level): the click batch
    // produced exactly ONE layout and ONE paint cycle.
    let counters_after_first_click = harness.counters();

    harness.click(center(bounds));
    let counters_after_second = harness.counters();
    assert_eq!(
        counters_after_second.layout_runs - counters_after_first_click.layout_runs,
        1,
        "one write batch (click) => at most one layout cycle"
    );
    assert_eq!(
        counters_after_second.paint_runs - counters_after_first_click.paint_runs,
        1,
        "one write batch (click) => at most one paint cycle"
    );

    // Idle contract: an interaction-free frame does nothing (A7).
    let before = harness.counters();
    harness.tick();
    assert_eq!(harness.counters().layout_runs, before.layout_runs);
    assert_eq!(harness.counters().paint_runs, before.paint_runs);

    // All input controls are named (the type-level contract holds here).
    harness.assert_all_named();
}

#[test]
fn m0_counter_snapshot_is_stable() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let (button, _binding) = mount_counter(&mut harness);
    let bounds = harness.bounds(button);
    harness.click(center(bounds));
    harness.snapshot("counter_clicked_once");
}

#[test]
fn m0_scrolling_list_translates_without_re_measuring() {
    let mut harness = TestHarness::new(240.0, 320.0);
    let (list_node, handle) = mount_list(&mut harness);

    handle.set_content_height(24.0 * 28.0);
    let viewport = harness.bounds(list_node);
    let before = harness.bounds(harness.pipeline().arena().children(list_node)[0]);
    harness.reset_measure_calls();

    // Wheel down 56px = two item heights.
    harness.wheel(center(viewport), (0.0, -56.0));
    assert_eq!(handle.offset(), 56.0);

    // Translation-only: ZERO measure calls for the scroll (§8.2 item 4).
    assert_eq!(
        harness.measure_calls(),
        0,
        "scroll must not re-measure anything"
    );

    // Children moved by the offset; sizes untouched.
    let first_child = harness.pipeline().arena().children(list_node)[0];
    let after = harness.bounds(first_child);
    assert_eq!(after.origin.y, before.origin.y - 56.0);
    assert_eq!(after.size.height, before.size.height);

    // Pixels actually changed (partial paint on the software path).
    let counters = harness.counters();
    assert!(counters.paint_runs >= 1);
    harness.snapshot("list_scrolled_56");
}

#[test]
fn m0_list_clips_content_to_the_viewport() {
    let mut harness = TestHarness::new(240.0, 320.0);
    let (list_node, handle) = mount_list(&mut harness);
    handle.set_content_height(24.0 * 28.0);

    // Scroll far down: the first items are far above the viewport top.
    handle.scroll_to(600.0);
    harness.settle();

    let viewport = harness.viewport();
    let scene = harness.pipeline_mut().tick(viewport).expect("tick").scene;
    let clip_bounds = viewport_intersection(list_node, &harness).expect("list bounds after layout");
    // Nothing may RENDER outside the scroll viewport: a primitive is legal
    // when it lies inside, or is clipped to the viewport (clip_overflow).
    for primitive in &scene.primitives {
        let (rect, clip) = match primitive {
            alacer_core::scene::Primitive::Quad(q) => (q.rect, q.clip),
            alacer_core::scene::Primitive::Glyph(g) => (
                alacer_core::geom::Rect::new(g.position, alacer_core::geom::Size::new(1.0, 12.0)),
                g.clip,
            ),
        };
        let effective = match clip {
            Some(clip) => clip.intersection(&rect),
            None => Some(rect),
        };
        if let Some(effective) = effective
            && (effective.max_y() < clip_bounds.min_y() || effective.min_y() > clip_bounds.max_y())
        {
            panic!("primitive renders outside the scroll viewport: {rect:?}");
        }
    }
}

fn viewport_intersection(
    list_node: alacer_core::arena::NodeId,
    harness: &TestHarness,
) -> Option<alacer_core::geom::Rect> {
    harness.pipeline().arena().bounds(list_node)
}

#[test]
fn m0_a11y_walker_fails_on_unnamed_inputs() {
    // The §11.1 debug walker: an unnamed button cannot pass a build.
    let mut harness = TestHarness::new(200.0, 120.0);
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Button::new("").boxed()); // unnamed!
    harness.settle();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.assert_all_named();
    }));
    assert!(result.is_err(), "unnamed input must fail the walker");
}

#[test]
fn m0_named_buttons_pass_the_walker() {
    let mut harness = TestHarness::new(200.0, 120.0);
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Button::new("Save").boxed());
    let _ = harness.add_child(root, Row::new().gap(4.0).boxed());
    harness.settle();
    harness.assert_all_named();
}

#[test]
fn m0_first_frame_is_a_full_pass_and_idle_frames_cost_nothing() {
    let mut harness = TestHarness::new(160.0, 120.0);
    let _ = harness.mount(Column::new().boxed());
    let report = harness.tick();
    assert!(
        report.layout_ran && report.paint_ran && report.a11y_ran,
        "first frame runs every pass"
    );

    let counters = harness.counters();
    let idle = harness.tick();
    assert!(idle.is_idle());
    assert_eq!(harness.counters(), counters, "idle frame: zero work");
}
