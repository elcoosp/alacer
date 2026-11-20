//! The raster tier's pricing bench: the moving blit (256 rasters redrawn
//! through their damage regions — the paint the scene journal's image
//! primitive takes when the pictures move) against the idle write (the
//! same scene landing where the frame already is — `Damage::NONE`, the
//! honest floor of a frame the pipeline can skip the present for).
//!
//! This bench rides *outside* the published corpus: the coordinated rule
//! grows the baseline at the next published cut, never at a crate bench's
//! (the m138 widget-bench rule's shape).

use alacer_core::color::Color;
use alacer_core::damage::{Damage, DamageBuilder};
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::scene::{
    GlyphSource, Image, ImageSampling, Primitive, SceneSnapshot, primitive_extent,
};
use alacer_render::SoftwareRenderer;
use criterion::{Criterion, criterion_group, criterion_main};
use std::sync::Arc;

/// The raster handle the route shares: `Arc<[u8]>` behind an alias (the
/// byte contract the image primitive spells).
type Raster = Arc<[u8]>;

/// The route's raster count: a 16×16 grid of pictures.
const COUNT: usize = 256;
/// Each picture's side, in pixels.
const RASTER_SIDE: u32 = 16;
/// The frame the route renders into.
const FRAME: u32 = 512;

/// A deterministic no-glyph source: image scenes carry no text.
struct NoGlyphs;

impl GlyphSource for NoGlyphs {
    fn glyph_bitmap(
        &mut self,
        _font_id: u64,
        _glyph_id: u32,
        _size: f32,
    ) -> Option<alacer_core::scene::GlyphBitmap> {
        None
    }
}

/// The route's raster: a 16×16 premultiplied RGBA8 checkerboard (opaque,
/// red/blue) — small enough to blit 256 of, textured enough to see.
fn route_raster() -> Raster {
    let mut bytes = Vec::with_capacity(RASTER_SIDE as usize * RASTER_SIDE as usize * 4);
    for y in 0..RASTER_SIDE {
        for x in 0..RASTER_SIDE {
            if (x + y) % 2 == 0 {
                bytes.extend_from_slice(&[220, 40, 40, 255]);
            } else {
                bytes.extend_from_slice(&[40, 40, 220, 255]);
            }
        }
    }
    Arc::from(bytes.into_boxed_slice())
}

/// The route's scene: the grid of pictures at `offset` (the moving arm
/// shifts every picture by a few pixels).
fn scene(raster: &Raster, offset: f32) -> SceneSnapshot {
    let columns = usize::try_from(FRAME / 32).unwrap_or(COUNT);
    let primitives = (0..COUNT)
        .map(|index| {
            let column = index % columns;
            let row = index / columns;
            Primitive::Image(Image {
                bytes: Arc::clone(raster),
                width: RASTER_SIDE,
                height: RASTER_SIDE,
                src: Rect::new(
                    Point::new(0.0, 0.0),
                    Size::new(
                        alacer_core::num::u32_to_f32(RASTER_SIDE),
                        alacer_core::num::u32_to_f32(RASTER_SIDE),
                    ),
                ),
                dst: Rect::new(
                    Point::new(
                        alacer_core::num::usize_to_f32(column * 32) + offset,
                        alacer_core::num::usize_to_f32(row * 32) + offset,
                    ),
                    Size::new(
                        alacer_core::num::u32_to_f32(RASTER_SIDE),
                        alacer_core::num::u32_to_f32(RASTER_SIDE),
                    ),
                ),
                clip: None,
                sampling: ImageSampling::Nearest,
            })
        })
        .collect();
    SceneSnapshot { primitives }
}

/// The route's damage: every old and new primitive extent (the
/// pipeline's own rule).
fn route_damage(previous: &SceneSnapshot, scene: &SceneSnapshot) -> Damage {
    let mut damage = DamageBuilder::new();
    for primitive in previous.primitives.iter().chain(scene.primitives.iter()) {
        damage.add(primitive_extent(primitive));
    }
    damage.finish(Size::new(
        alacer_core::num::u32_to_f32(FRAME),
        alacer_core::num::u32_to_f32(FRAME),
    ))
}

/// The moving blit: 256 rasters redrawn through their damage regions.
fn bench_image_blit(c: &mut Criterion) {
    let mut group = c.benchmark_group("image");
    group.throughput(criterion::Throughput::Elements(COUNT as u64));
    group.bench_function("image_blit_256", |b| {
        let raster = route_raster();
        let previous = scene(&raster, 0.0);
        let moved = scene(&raster, 2.0);
        let damage = route_damage(&previous, &moved);
        let mut glyphs = NoGlyphs;
        b.iter_batched(
            || {
                SoftwareRenderer::new()
                    .render(&previous, FRAME, FRAME, Color::WHITE, &mut NoGlyphs)
                    .expect("previous frame")
            },
            |mut frame| {
                SoftwareRenderer::new()
                    .render_partial(&moved, &mut frame, &damage, Color::WHITE, &mut glyphs)
                    .expect("partial render");
                frame.width()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// The idle write: the same 256-raster scene with no damage — the write
/// that lands where the frame already is (no regions, no blits, the
/// honest floor the moving blit is priced against).
fn bench_idle_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("image");
    group.throughput(criterion::Throughput::Elements(COUNT as u64));
    group.bench_function("idle_write_256", |b| {
        let raster = route_raster();
        let scene = scene(&raster, 0.0);
        let mut glyphs = NoGlyphs;
        b.iter_batched(
            || {
                SoftwareRenderer::new()
                    .render(&scene, FRAME, FRAME, Color::WHITE, &mut NoGlyphs)
                    .expect("previous frame")
            },
            |mut frame| {
                SoftwareRenderer::new()
                    .render_partial(&scene, &mut frame, &Damage::NONE, Color::WHITE, &mut glyphs)
                    .expect("idle render");
                frame.width()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_image_blit, bench_idle_write);
criterion_main!(benches);
