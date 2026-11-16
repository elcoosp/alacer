//! The canvas against the real headless pipeline (M82): the custom-paint
//! capability end-to-end — `rsx!` mounts a `canvas`, the app's paint closure
//! records primitives through the paint seam, the software renderer
//! materializes them, and the a11y tree carries the name.

use alacer::prelude::*;
use alacer::view::Painter;
use alacer::widgets::{BoxedWidget as _, Canvas, Column};
use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::scene::Quad;
use alacer_testing::{Pixmap, TestHarness};

const CANVAS_WASH: (u8, u8, u8) = (31, 122, 77);
const CANVAS_SWAP: (u8, u8, u8) = (178, 88, 24);
const SURFACE: (u8, u8, u8) = (245, 246, 248);

/// The wash quad a test painter records: full-bleed in a fixed color.
fn wash_quad(rgb: (u8, u8, u8)) -> Quad {
    Quad {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(120.0, 120.0)),
        color: Color::rgb(rgb.0, rgb.1, rgb.2),
        corner_radius: 0.0,
        border_color: Color::TRANSPARENT,
        border_width: 0.0,
        clip: None,
    }
}

/// One full-bleed wash in a fixed color: the painter the tests read.
fn wash(rgb: (u8, u8, u8)) -> Painter {
    Painter::new(move |ctx| {
        ctx.push_quad(Quad {
            rect: Rect::new(Point::new(0.0, 0.0), ctx.size()),
            ..wash_quad(rgb)
        });
    })
}

/// The view under test: a padded column holding one named canvas.
fn padded_view() -> ViewNode {
    rsx! {
        column { padding: 16.0,
            canvas { key: 0_u64, name: "The waveform", width: 120.0, height: 120.0,
                on_paint: wash(CANVAS_WASH) }
        }
    }
}

/// The pixel at `x, y` as the straight RGB triple the painter recorded.
fn rgb_at(pixels: &Pixmap, x: u32, y: u32) -> Option<(u8, u8, u8)> {
    pixels
        .pixel(x, y)
        .map(|pixel| (pixel.red(), pixel.green(), pixel.blue()))
}

#[test]
fn canvas_mounts_names_and_paints_through_the_pipeline() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let view = padded_view();
    let _root = alacer::mount_view(&view, harness.pipeline_mut());
    harness.settle();

    // Reachable and named through the a11y tree — the only query API.
    let node = harness.expect_a11y(Role::Canvas, "The waveform");
    let bounds = node.bounds.expect("the canvas laid out");
    assert_eq!(bounds.origin.x, 16.0, "the padded column places the canvas");
    assert_eq!(bounds.size.width, 120.0);
    harness.assert_all_named();

    // The painter's quad reached the scene journal in the frame's paint.
    let scene = harness.tick().scene;
    let painted = scene.primitives.iter().any(|primitive| {
        matches!(
            primitive,
            alacer_core::scene::Primitive::Quad(quad)
                if quad.color == Color::rgb(CANVAS_WASH.0, CANVAS_WASH.1, CANVAS_WASH.2)
        )
    });
    assert!(painted, "the canvas's wash is in the recorded scene");

    // And materialized pixels: the wash inside, the padding outside.
    let pixels = harness.pixels();
    assert_eq!(
        rgb_at(&pixels, 80, 80),
        Some(CANVAS_WASH),
        "the wash rasterized"
    );
    assert_eq!(
        rgb_at(&pixels, 4, 4),
        Some(SURFACE),
        "the padding stays background"
    );
}

#[test]
fn canvas_painter_swap_repaints_on_the_next_view_run() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let view = padded_view();
    let mut root = alacer::mount_view(&view, harness.pipeline_mut());
    harness.settle();

    // The next view run hangs a new painter: a rebuild may carry a new
    // closure, so the patch reports the canvas visual and it repaints.
    let next = rsx! {
        column { padding: 16.0,
            canvas { key: 0_u64, name: "The waveform", width: 120.0, height: 120.0,
                on_paint: wash(CANVAS_SWAP) }
        }
    };
    let report = root.update(&next, harness.pipeline_mut());
    harness.settle();

    assert_eq!(report.patched, 1, "the new painter is a visual change");
    let pixels = harness.pixels();
    assert_eq!(
        rgb_at(&pixels, 80, 80),
        Some(CANVAS_SWAP),
        "the swap rasterized"
    );
}

#[test]
fn canvas_hoisted_painter_is_an_idle_frame() {
    let mut harness = TestHarness::new(320.0, 240.0);
    // The same `Painter` value across view runs — hoisted once, cloned by
    // reference into each run (an `Rc` clone keeps the identity): the
    // comparison says nothing moved, so the re-run queues no repaint (the
    // idle-frame property the storm budget leans on, at the canvas seam).
    let hoisted = wash(CANVAS_WASH);
    let view = || {
        rsx! {
            column { padding: 16.0,
                canvas { key: 0_u64, name: "The waveform", width: 120.0, height: 120.0,
                    on_paint: hoisted.clone() }
            }
        }
    };
    let mut root = alacer::mount_view(&view(), harness.pipeline_mut());
    harness.settle();
    let before = harness.counters();

    let report = root.update(&view(), harness.pipeline_mut());
    harness.settle();
    let after = harness.counters();

    assert_eq!(report.patched, 0, "the hoisted painter re-hangs quietly");
    assert_eq!(
        after.paint_runs, before.paint_runs,
        "no repaint ran — the hoisted painter is the idle-frame tool"
    );
}

#[test]
fn canvas_mounts_imperatively_and_paints_on_the_software_path() {
    // The example's shape, headless: the builder (not the DSL) mounts a
    // canvas on the composition every CI build ships — the software-first
    // guarantee, painted end-to-end.
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().padding(12.0).boxed());
    let _ = harness.add_child(
        root,
        Canvas::new("The headless wash")
            .size(160.0, 90.0)
            .painter(|ctx| {
                ctx.push_quad(Quad {
                    rect: Rect::new(Point::new(0.0, 0.0), ctx.size()),
                    color: Color::rgb(CANVAS_WASH.0, CANVAS_WASH.1, CANVAS_WASH.2),
                    corner_radius: 0.0,
                    border_color: Color::TRANSPARENT,
                    border_width: 0.0,
                    clip: None,
                });
            })
            .boxed(),
    );
    harness.settle();

    let node = harness.expect_a11y(Role::Canvas, "The headless wash");
    let bounds = node.bounds.expect("the canvas laid out");
    assert_eq!(bounds.size.width, 160.0);
    assert_eq!(bounds.size.height, 90.0);

    let pixels = harness.pixels();
    assert_eq!(rgb_at(&pixels, 80, 60), Some(CANVAS_WASH));
}
