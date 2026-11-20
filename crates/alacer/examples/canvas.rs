//! The canvas (M82): the app's own paint closure through the paint seam —
//! a tone's level bar that follows the slider, one signal write away, over
//! a procedural raster the closure blits through `push_image` — the same
//! raster-tier primitive the `Image` widget records, proving the seam at
//! the app's own drawing surface.
//!
//! Run with `cargo run -p alacer --example canvas`. The slider's change
//! handler writes one signal inside one batch (the ADR-6 contract); the
//! flux bridge's `bind_value` door updates the level the painter reads and
//! queues the canvas's `PAINT` mask — the repaint currency every widget
//! shares, now at the app's own drawing surface. `ALACER_SOFTWARE=1`
//! forces the softbuffer path (VMs, CI, no-GPU machines).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Canvas, Column, Label, Slider};
use alacer_core::Change;
use alacer_core::color::Color;
use alacer_core::flux::RenderBinding;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::num::u32_to_f32;
use alacer_core::scene::{Image, ImageSampling};
use reactive_graph::traits::Set as _;

/// The checker's bitmap edge in pixels: 8 cells of 8 px.
const CHECKER_PX: u32 = 64;

/// Builds the painter's procedural raster: a `cells × cells` checkerboard
/// of `cell_px` squares, premultiplied RGBA8, row-major — every cell
/// opaque (a = 255), so the premultiplied bytes equal the straight ones,
/// the byte contract the image primitive documents.
fn checker_raster(cells: u32, cell_px: u32) -> Vec<u8> {
    let edge = cells * cell_px;
    let mut bytes = Vec::with_capacity((edge * edge * 4) as usize);
    for y in 0..edge {
        for x in 0..edge {
            let light = ((x / cell_px) + (y / cell_px)).is_multiple_of(2);
            let (r, g, b) = if light { (31, 122, 77) } else { (24, 24, 30) };
            bytes.extend_from_slice(&[r, g, b, 255]);
        }
    }
    bytes
}

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let _header = pipeline.add_child(root, Label::new("The canvas: the app paints").boxed());

    // The level the painter reads, carried by the reactive bridge: the
    // signal's writes land in the binding and queue the canvas's repaint.
    let level = pipeline.flux().signal(60.0_f32);
    let binding = RenderBinding::new(60.0_f32);
    // The painter's own read door: an `Rc`-cheap clone of the same
    // binding, so the bridge's writes are visible at paint time.
    let paint_binding = binding.clone();
    // The procedural raster the painter blits: built once outside the
    // closure, shared through one `Arc` allocation — the image primitive's
    // own sharing rule (app, journal, and renderers read the same bytes).
    let checker: std::sync::Arc<[u8]> = checker_raster(8, 8).into();

    let canvas = pipeline.add_child(
        root,
        Canvas::new("The tone level")
            .size(280.0, 80.0)
            .painter(move |ctx| {
                let size = ctx.size();
                let palette = ctx.palette();
                // The surface: a recessed rail the bar rides.
                ctx.push_quad(alacer_core::scene::Quad {
                    rect: Rect::new(Point::new(0.0, 0.0), size),
                    color: palette.surface,
                    corner_radius: 8.0,
                    border_color: palette.border,
                    border_width: 1.0,
                    clip: None,
                });
                // The raster tier through the app's own closure: the
                // checker blits through `push_image`, scaled into the
                // whole rail with crisp texel copies (`Nearest`) — the
                // same primitive the `Image` widget records, now proving
                // the seam at the app's own drawing surface.
                ctx.push_image(Image {
                    bytes: std::sync::Arc::clone(&checker),
                    width: CHECKER_PX,
                    height: CHECKER_PX,
                    src: Rect::new(
                        Point::new(0.0, 0.0),
                        Size::new(u32_to_f32(CHECKER_PX), u32_to_f32(CHECKER_PX)),
                    ),
                    dst: Rect::new(Point::new(0.0, 0.0), size),
                    clip: None,
                    sampling: ImageSampling::Nearest,
                });
                // The level bar: the binding's share of the rail, painted
                // in the canon accent (the fact a signal write moved).
                let share = paint_binding.with(|level| (level / 100.0).clamp(0.0, 1.0));
                ctx.push_quad(alacer_core::scene::Quad::filled(
                    Rect::new(
                        Point::new(6.0, 6.0),
                        alacer_core::geom::Size::new(
                            (size.width - 12.0) * share,
                            size.height - 12.0,
                        ),
                    ),
                    Color::rgb(31, 122, 77),
                ));
            })
            .boxed(),
    );

    // The bridge door: every signal write updates the level the painter
    // reads and queues the canvas's `PAINT` mask — one write batch, one
    // repaint, the scheduling contract at the app's own surface.
    pipeline
        .flux_mut()
        .bind_value(level, canvas, &binding, Change::PAINT, |value: &f32| *value);

    let _ = pipeline.add_child(
        root,
        Slider::new("The level", 0.0, 100.0)
            .value(60.0)
            .on_change(move |value: f32| {
                alacer_core::flux::Flux::batch_of(|| level.set(value));
            })
            .boxed(),
    );

    if let Err(error) =
        alacer::run_window(pipeline, "Alacer canvas", (360.0, 260.0), backend_choice())
    {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}
