//! The software rasterizer: one scene snapshot → one RGBA pixmap via
//! tiny-skia (ADR-2: tiny-skia + softbuffer is the headless/CI/low-end
//! backend, and A4: the test rig *is* this backend).
//!
//! Glyphs arrive as coverage bitmaps through the [`GlyphSource`] seam — the
//! exact same bitmaps the GPU backend uploads to its atlas.
//!
//! M3 partial paint (§9.3): [`SoftwareRenderer::render_partial`] repaints
//! only the frame's [`Damage`] regions onto the *persistent* framebuffer —
//! clear each region, redraw the primitives intersecting it in scene order —
//! and the equivalence contract (Q8) guarantees the result is pixel-identical
//! to a full re-render. `alacer-testing` fuzzes that equivalence.

use alacer_core::color::Color;
use alacer_core::damage::Damage;
use alacer_core::num::{f32_to_i32, f32_to_u8};
use alacer_core::scene::{Glyph, GlyphSource, Primitive, Quad, SceneSnapshot, primitive_extent};
use std::sync::Arc;
use tiny_skia::{
    FillRule, FilterQuality, Mask, Paint, Path, PathBuilder, Pixmap, PixmapPaint,
    PremultipliedColorU8, Rect, Stroke, Transform,
};

/// Renders scenes into CPU pixmaps.
#[derive(Debug, Default)]
pub struct SoftwareRenderer {
    glyphs_rendered: u64,
}

impl SoftwareRenderer {
    /// A fresh renderer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Glyphs blitted so far (instrumentation).
    #[must_use]
    pub const fn glyphs_rendered(&self) -> u64 {
        self.glyphs_rendered
    }

    /// Rasterizes `scene` into a `width × height` pixmap over `background`.
    ///
    /// # Panics
    ///
    /// Never; returns `None` only when the pixmap cannot be allocated (0×0 or
    /// OOM).
    #[must_use]
    pub fn render(
        &mut self,
        scene: &SceneSnapshot,
        width: u32,
        height: u32,
        background: Color,
        glyphs: &mut dyn GlyphSource,
    ) -> Option<Pixmap> {
        let mut pixmap = Pixmap::new(width, height)?;
        pixmap.fill(to_skia(background));
        {
            let mut painter = pixmap.as_mut();
            for primitive in &scene.primitives {
                match primitive {
                    Primitive::Quad(quad) => self.draw_quad(&mut painter, quad, None),
                    Primitive::Glyph(glyph) => {
                        self.draw_glyph(&mut painter, glyph, glyphs, None);
                    }
                }
            }
        }
        Some(pixmap)
    }

    /// Repaints only `damage`'s regions onto the persistent `frame`
    /// framebuffer (§9.3, M3): each region is cleared to `background`, then
    /// every primitive whose extent intersects the region is redrawn in
    /// scene order, scissored to the region.
    ///
    /// The equivalence contract (Q8): the result is pixel-identical to
    /// [`Self::render`] of the same scene over the same background, provided
    /// `frame` is the previous frame's rendering of the previous scene and
    /// the pipeline's damage covers every changed pixel (old layer extents
    /// are damage too — `PassPipeline::tick` guarantees this).
    ///
    /// `Damage::NONE` is a no-op (the present can be skipped entirely);
    /// `Damage::FULL` clears the whole frame and redraws everything.
    ///
    /// # Panics
    ///
    /// Never; returns `None` only when the frame cannot be allocated (0×0 or
    /// OOM).
    pub fn render_partial(
        &mut self,
        scene: &SceneSnapshot,
        frame: &mut Pixmap,
        damage: &Damage,
        background: Color,
        glyphs: &mut dyn GlyphSource,
    ) -> Option<()> {
        let viewport = alacer_core::geom::Size::new(
            f32::from(u16::try_from(frame.width()).unwrap_or(u16::MAX)),
            f32::from(u16::try_from(frame.height()).unwrap_or(u16::MAX)),
        );
        for region in damage.regions(viewport) {
            clear_region(frame, &region, background);
            // One scissor mask per region: a primitive may only touch pixels
            // the region owns (blending is not idempotent outside it).
            let scissor = tiny_region(&region, frame.width(), frame.height()).and_then(|r| {
                rect_mask(
                    alacer_core::geom::Rect::new(
                        alacer_core::geom::Point::new(r.left(), r.top()),
                        alacer_core::geom::Size::new(r.width(), r.height()),
                    ),
                    frame.width(),
                    frame.height(),
                )
            });
            let mut painter = frame.as_mut();
            for primitive in &scene.primitives {
                // Only primitives reaching into this region draw. Clean ones
                // are redrawn within the region; their pixels outside keep
                // the previous frame's identical values.
                if !primitive_extent(primitive).intersects(&region) {
                    continue;
                }
                match primitive {
                    Primitive::Quad(quad) => {
                        self.draw_quad(&mut painter, quad, scissor.as_ref());
                    }
                    Primitive::Glyph(glyph) => {
                        self.draw_glyph(&mut painter, glyph, glyphs, scissor.as_ref());
                    }
                }
            }
        }
        Some(())
    }

    fn draw_quad(
        &self,
        painter: &mut tiny_skia::PixmapMut<'_>,
        quad: &Quad,
        extra_scissor: Option<&Mask>,
    ) {
        let Some(rect) = Rect::from_xywh(
            quad.rect.origin.x,
            quad.rect.origin.y,
            quad.rect.size.width,
            quad.rect.size.height,
        ) else {
            return;
        };
        let path = quad_path(rect, quad.corner_radius);
        let clip = combined_mask(quad.clip, extra_scissor, painter.width(), painter.height());
        let paint = Paint {
            blend_mode: tiny_skia::BlendMode::SourceOver,
            ..fill_paint(quad.color)
        };
        painter.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            clip.as_ref(),
        );

        if quad.border_width > 0.0 && quad.border_color.a > 0 {
            let stroke = Stroke {
                width: quad.border_width,
                ..Stroke::default()
            };
            let border_paint = fill_paint(quad.border_color);
            painter.stroke_path(
                &path,
                &border_paint,
                &stroke,
                Transform::identity(),
                clip.as_ref(),
            );
        }
    }

    fn draw_glyph(
        &mut self,
        painter: &mut tiny_skia::PixmapMut<'_>,
        glyph: &Glyph,
        source: &mut dyn GlyphSource,
        extra_scissor: Option<&Mask>,
    ) {
        let Some(bitmap) = source.glyph_bitmap(glyph.font_id, glyph.glyph_id, glyph.size) else {
            return;
        };
        if bitmap.width == 0 || bitmap.height == 0 || bitmap.coverage.is_empty() {
            return;
        }
        let Some(mut stamp) = Pixmap::new(bitmap.width, bitmap.height) else {
            return;
        };
        stamp_glyph(&mut stamp, &bitmap, glyph.color);
        let x = f32_to_i32(glyph.position.x) + bitmap.left;
        let y = f32_to_i32(glyph.position.y) + bitmap.top;
        let clip = combined_mask(glyph.clip, extra_scissor, painter.width(), painter.height());
        painter.draw_pixmap(
            x,
            y,
            stamp.as_ref(),
            &PixmapPaint {
                opacity: 1.0,
                blend_mode: tiny_skia::BlendMode::SourceOver,
                quality: FilterQuality::Nearest,
            },
            Transform::identity(),
            clip.as_ref(),
        );
        self.glyphs_rendered += 1;
    }
}

fn fill_paint(color: Color) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color(to_skia(color));
    paint.anti_alias = true;
    paint
}

fn to_skia(color: Color) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba8(color.r, color.g, color.b, color.a)
}

/// The mask for one optional clip rect (the primitive's own clip).
fn clip_mask(clip: Option<alacer_core::geom::Rect>, width: u32, height: u32) -> Option<Mask> {
    let clip = clip?;
    rect_mask(clip, width, height)
}

/// Combines the primitive's own clip with a partial-paint scissor region:
/// the mask covers their intersection (a primitive may only touch pixels the
/// region owns — blending is not idempotent outside it).
fn combined_mask(
    clip: Option<alacer_core::geom::Rect>,
    scissor: Option<&Mask>,
    width: u32,
    height: u32,
) -> Option<Mask> {
    let Some(scissor) = scissor else {
        return clip_mask(clip, width, height);
    };
    let Some(clip) = clip else {
        return Some(scissor.clone());
    };
    let clip_mask = rect_mask(clip, width, height)?;
    let mut combined = scissor.clone();
    for (a, b) in combined.data_mut().iter_mut().zip(clip_mask.data().iter()) {
        *a = (*a).min(*b);
    }
    Some(combined)
}

fn rect_mask(clip: alacer_core::geom::Rect, width: u32, height: u32) -> Option<Mask> {
    let mut mask = Mask::new(width, height)?;
    let rect = Rect::from_xywh(
        clip.origin.x,
        clip.origin.y,
        clip.size.width,
        clip.size.height,
    )?;
    let path = PathBuilder::from_rect(rect);
    mask.fill_path(&path, FillRule::Winding, false, Transform::identity());
    Some(mask)
}

/// Clears one region to `background` (the partial path's erase step).
fn clear_region(frame: &mut Pixmap, region: &alacer_core::geom::Rect, background: Color) {
    let Some(rect) = tiny_region(region, frame.width(), frame.height()) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(to_skia(background));
    frame.fill_rect(rect, &paint, Transform::identity(), None);
}

/// A damage region as a clamped tiny-skia rect.
fn tiny_region(region: &alacer_core::geom::Rect, width: u32, height: u32) -> Option<Rect> {
    let x = region.origin.x.max(0.0);
    let y = region.origin.y.max(0.0);
    let right = region
        .max_x()
        .min(f32::from(u16::try_from(width).unwrap_or(u16::MAX)));
    let bottom = region
        .max_y()
        .min(f32::from(u16::try_from(height).unwrap_or(u16::MAX)));
    if right <= x || bottom <= y {
        return None;
    }
    Rect::from_xywh(x, y, right - x, bottom - y)
}

/// Builds the quad path — sharp when `radius` is 0, rounded otherwise.
#[must_use]
fn quad_path(rect: Rect, radius: f32) -> Path {
    let r = radius
        .max(0.0)
        .min(rect.width() * 0.5)
        .min(rect.height() * 0.5);
    if r <= 0.0 {
        return PathBuilder::from_rect(rect);
    }
    let x0 = rect.left();
    let y0 = rect.top();
    let x1 = rect.right();
    let y1 = rect.bottom();
    let mut b = PathBuilder::new();
    b.move_to(x0 + r, y0);
    b.line_to(x1 - r, y0);
    b.quad_to(x1, y0, x1, y0 + r);
    b.line_to(x1, y1 - r);
    b.quad_to(x1, y1, x1 - r, y1);
    b.line_to(x0 + r, y1);
    b.quad_to(x0, y1, x0, y1 - r);
    b.line_to(x0, y0 + r);
    b.quad_to(x0, y0, x0 + r, y0);
    b.close();
    b.finish().expect("rounded path")
}

/// Converts coverage + color into a premultiplied RGBA stamp.
fn stamp_glyph(stamp: &mut Pixmap, bitmap: &alacer_core::scene::GlyphBitmap, color: Color) {
    let pixels = stamp.pixels_mut();
    let cov_scale = f32::from(color.a) / 255.0;
    for (index, coverage) in bitmap.coverage.iter().enumerate() {
        let Some(pixel) = pixels.get_mut(index) else {
            break;
        };
        if *coverage == 0 {
            continue;
        }
        let alpha = f32::from(*coverage) * cov_scale;
        let a = f32_to_u8(alpha);
        // Premultiplied channels scale by alpha/255 — the raw 0..=255 alpha
        // would push every non-black channel past 255 (clamped to white).
        let ink = alpha / 255.0;
        let premul = PremultipliedColorU8::from_rgba(
            f32_to_u8(f32::from(color.r) * ink),
            f32_to_u8(f32::from(color.g) * ink),
            f32_to_u8(f32::from(color.b) * ink),
            a,
        );
        if let Some(p) = premul {
            *pixel = p;
        }
    }
}

/// A glyph source backed by a closure — for tests and benches.
pub struct ClosureGlyphSource<'a, F>
where
    F: FnMut(u64, u32, f32) -> Option<alacer_core::scene::GlyphBitmap>,
{
    source: &'a mut F,
}

impl<'a, F> ClosureGlyphSource<'a, F>
where
    F: FnMut(u64, u32, f32) -> Option<alacer_core::scene::GlyphBitmap>,
{
    /// Wraps a closure.
    #[must_use]
    pub fn new(source: &'a mut F) -> Self {
        Self { source }
    }
}

impl<F> GlyphSource for ClosureGlyphSource<'_, F>
where
    F: FnMut(u64, u32, f32) -> Option<alacer_core::scene::GlyphBitmap>,
{
    fn glyph_bitmap(
        &mut self,
        font_id: u64,
        glyph_id: u32,
        size: f32,
    ) -> Option<alacer_core::scene::GlyphBitmap> {
        (self.source)(font_id, glyph_id, size)
    }
}

/// Builds a fully-opaque coverage bitmap spanning `w × h` (test helper).
#[must_use]
pub fn solid_bitmap(w: u32, h: u32, left: i32, top: i32) -> alacer_core::scene::GlyphBitmap {
    let len = usize::try_from(w * h).unwrap_or(0);
    alacer_core::scene::GlyphBitmap {
        left,
        top,
        width: w,
        height: h,
        coverage: Arc::from(vec![255_u8; len].into_boxed_slice()),
    }
}

#[cfg(test)]
mod tests {
    use super::{ClosureGlyphSource, SoftwareRenderer, solid_bitmap};
    use alacer_core::color::Color;
    use alacer_core::damage::{Damage, DamageBuilder};
    use alacer_core::geom::{Point, Rect, Size};
    use alacer_core::scene::{Glyph, Primitive, Quad, SceneSnapshot, primitive_extent};
    use std::sync::Arc;

    fn render_with<F>(scene: &SceneSnapshot, source: &mut F) -> Option<tiny_skia::Pixmap>
    where
        F: FnMut(u64, u32, f32) -> Option<alacer_core::scene::GlyphBitmap>,
    {
        let mut src = ClosureGlyphSource::new(source);
        SoftwareRenderer::new().render(scene, 40, 30, Color::WHITE, &mut src)
    }

    /// Renders `scene` partially onto `prev` with `damage` and compares the
    /// result pixel-by-pixel with a full render of the same scene.
    fn assert_partial_equals_full(
        prev_scene: &SceneSnapshot,
        scene: &SceneSnapshot,
        damage: &Damage,
        source: &mut dyn alacer_core::scene::GlyphSource,
        width: u32,
        height: u32,
    ) {
        let mut full_renderer = SoftwareRenderer::new();
        let mut partial_renderer = SoftwareRenderer::new();
        let expected = full_renderer
            .render(scene, width, height, Color::WHITE, source)
            .expect("full render");
        let mut no_glyphs =
            |_f: u64, _g: u32, _s: f32| -> Option<alacer_core::scene::GlyphBitmap> { None };
        let mut prev = full_renderer
            .render(
                prev_scene,
                width,
                height,
                Color::WHITE,
                &mut ClosureGlyphSource::new(&mut no_glyphs),
            )
            .expect("prev render");
        partial_renderer
            .render_partial(scene, &mut prev, damage, Color::WHITE, source)
            .expect("partial render");
        for y in 0..height {
            for x in 0..width {
                let a = prev.pixel(x, y).expect("in bounds");
                let b = expected.pixel(x, y).expect("in bounds");
                assert_eq!(
                    (a.red(), a.green(), a.blue()),
                    (b.red(), b.green(), b.blue()),
                    "partial != full at ({x},{y})"
                );
            }
        }
    }

    #[test]
    fn partial_repaint_matches_full_repaint_on_a_color_flip() {
        let prev_scene = SceneSnapshot {
            primitives: vec![Primitive::Quad(Quad::filled(
                Rect::new(Point::new(4.0, 4.0), Size::new(20.0, 16.0)),
                Color::rgb(255, 0, 0),
            ))],
        };
        let scene = SceneSnapshot {
            primitives: vec![Primitive::Quad(Quad::filled(
                Rect::new(Point::new(4.0, 4.0), Size::new(20.0, 16.0)),
                Color::rgb(0, 0, 255),
            ))],
        };
        let mut damage = DamageBuilder::new();
        damage.add(Rect::new(Point::new(4.0, 4.0), Size::new(20.0, 16.0)));
        let damage = damage.finish(Size::new(40.0, 30.0));
        assert!(!damage.is_full());
        let mut glyphs =
            |_f: u64, _g: u32, _s: f32| -> Option<alacer_core::scene::GlyphBitmap> { None };
        let mut source = ClosureGlyphSource::new(&mut glyphs);
        assert_partial_equals_full(&prev_scene, &scene, &damage, &mut source, 40, 30);
    }

    #[test]
    fn partial_repaint_matches_full_repaint_on_a_move() {
        let prev_scene = SceneSnapshot {
            primitives: vec![Primitive::Quad(Quad::filled(
                Rect::new(Point::new(2.0, 2.0), Size::new(12.0, 12.0)),
                Color::rgb(30, 200, 30),
            ))],
        };
        let scene = SceneSnapshot {
            primitives: vec![Primitive::Quad(Quad::filled(
                Rect::new(Point::new(24.0, 14.0), Size::new(12.0, 12.0)),
                Color::rgb(30, 200, 30),
            ))],
        };
        let mut damage = DamageBuilder::new();
        damage.add(Rect::new(Point::new(2.0, 2.0), Size::new(12.0, 12.0)));
        damage.add(Rect::new(Point::new(24.0, 14.0), Size::new(12.0, 12.0)));
        let damage = damage.finish(Size::new(40.0, 30.0));
        let mut glyphs =
            |_f: u64, _g: u32, _s: f32| -> Option<alacer_core::scene::GlyphBitmap> { None };
        let mut source = ClosureGlyphSource::new(&mut glyphs);
        assert_partial_equals_full(&prev_scene, &scene, &damage, &mut source, 40, 30);
    }

    #[test]
    fn partial_repaint_matches_full_repaint_with_glyphs() {
        let glyph = |x: f32| {
            Primitive::Glyph(Glyph {
                position: Point::new(x, 20.0),
                size: 16.0,
                color: Color::BLACK,
                font_id: 1,
                glyph_id: 65,
                clip: None,
            })
        };
        let prev_scene = SceneSnapshot {
            primitives: vec![glyph(4.0), glyph(12.0)],
        };
        let scene = SceneSnapshot {
            primitives: vec![glyph(4.0), glyph(26.0)],
        };
        let mut damage = DamageBuilder::new();
        // The pipeline damages the old layer extents and the new ones.
        for primitive in scene.primitives.iter().chain(prev_scene.primitives.iter()) {
            damage.add(primitive_extent(primitive));
        }
        let damage = damage.finish(Size::new(40.0, 30.0));
        let mut glyphs = |_f: u64, _g: u32, _s: f32| Some(solid_bitmap(6, 12, 0, -10));
        let mut source = ClosureGlyphSource::new(&mut glyphs);
        assert_partial_equals_full(&prev_scene, &scene, &damage, &mut source, 40, 30);
    }

    #[test]
    fn full_damage_repaints_everything() {
        let prev_scene = SceneSnapshot {
            primitives: vec![Primitive::Quad(Quad::filled(
                Rect::new(Point::new(0.0, 0.0), Size::new(40.0, 30.0)),
                Color::rgb(10, 10, 10),
            ))],
        };
        let scene = SceneSnapshot::default();
        let mut prev = SoftwareRenderer::new()
            .render(
                &prev_scene,
                40,
                30,
                Color::WHITE,
                &mut ClosureGlyphSource::new(&mut |_f, _g, _s| None),
            )
            .expect("prev");
        let expected = SoftwareRenderer::new()
            .render(
                &scene,
                40,
                30,
                Color::WHITE,
                &mut ClosureGlyphSource::new(&mut |_f, _g, _s| None),
            )
            .expect("expected");
        SoftwareRenderer::new()
            .render_partial(
                &scene,
                &mut prev,
                &Damage::FULL,
                Color::WHITE,
                &mut ClosureGlyphSource::new(&mut |_f, _g, _s| None),
            )
            .expect("partial");
        for y in 0..30_u32 {
            for x in 0..40_u32 {
                let a = prev.pixel(x, y).expect("in bounds");
                let b = expected.pixel(x, y).expect("in bounds");
                assert_eq!(
                    (a.red(), a.green(), a.blue()),
                    (b.red(), b.green(), b.blue())
                );
            }
        }
    }

    #[test]
    fn none_damage_leaves_the_frame_untouched() {
        let prev_scene = SceneSnapshot {
            primitives: vec![Primitive::Quad(Quad::filled(
                Rect::new(Point::new(0.0, 0.0), Size::new(40.0, 30.0)),
                Color::rgb(10, 10, 10),
            ))],
        };
        let scene = SceneSnapshot::default();
        let mut prev = SoftwareRenderer::new()
            .render(
                &prev_scene,
                40,
                30,
                Color::WHITE,
                &mut ClosureGlyphSource::new(&mut |_f, _g, _s| None),
            )
            .expect("prev");
        SoftwareRenderer::new()
            .render_partial(
                &scene,
                &mut prev,
                &Damage::NONE,
                Color::WHITE,
                &mut ClosureGlyphSource::new(&mut |_f, _g, _s| None),
            )
            .expect("partial");
        let pixel = prev.pixel(1, 1).expect("in bounds");
        assert_eq!((pixel.red(), pixel.green(), pixel.blue()), (10, 10, 10));
    }

    /// A tiny deterministic LCG — the equivalence fuzz (Q8) needs
    /// reproducible pseudo-randomness, no external crate.
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.0 >> 11
        }

        fn below(&mut self, bound: u64) -> u64 {
            self.next() % bound
        }
    }

    #[test]
    fn equivalence_fuzz_partial_matches_full_over_random_edits() {
        let mut rng = Lcg(0x5EED);
        let width = 64_u32;
        let height = 48_u32;
        let viewport = Size::new(
            f32::from(u16::try_from(width).unwrap_or(0)),
            f32::from(u16::try_from(height).unwrap_or(0)),
        );

        let mut prev_scene = SceneSnapshot::default();
        let mut prev = SoftwareRenderer::new()
            .render(
                &prev_scene,
                width,
                height,
                Color::WHITE,
                &mut ClosureGlyphSource::new(&mut |_f, _g, _s| None),
            )
            .expect("initial frame");

        let mut renderer = SoftwareRenderer::new();
        for _step in 0..64 {
            // Mutate the scene: move/recolor one or two quads, add or remove.
            let mut primitives = prev_scene.primitives.clone();
            let action = rng.below(4);
            match action {
                0 if !primitives.is_empty() => {
                    let index = rng.below(u64::try_from(primitives.len()).unwrap_or(1));
                    if let Some(Primitive::Quad(quad)) = primitives.get_mut(index as usize) {
                        quad.color = Color::rgb(
                            u8::try_from(rng.below(256)).unwrap_or(0),
                            u8::try_from(rng.below(256)).unwrap_or(0),
                            u8::try_from(rng.below(256)).unwrap_or(0),
                        );
                    }
                }
                1 if !primitives.is_empty() => {
                    let index = rng.below(u64::try_from(primitives.len()).unwrap_or(1));
                    if let Some(Primitive::Quad(quad)) = primitives.get_mut(index as usize) {
                        quad.rect = Rect::new(
                            Point::new(
                                f32::from(u8::try_from(rng.below(50)).unwrap_or(0)),
                                f32::from(u8::try_from(rng.below(38)).unwrap_or(0)),
                            ),
                            Size::new(
                                f32::from(u8::try_from(rng.below(20)).unwrap_or(1)) + 1.0,
                                f32::from(u8::try_from(rng.below(20)).unwrap_or(1)) + 1.0,
                            ),
                        );
                    }
                }
                2 => {
                    primitives.push(Primitive::Quad(Quad::filled(
                        Rect::new(
                            Point::new(
                                f32::from(u8::try_from(rng.below(50)).unwrap_or(0)),
                                f32::from(u8::try_from(rng.below(38)).unwrap_or(0)),
                            ),
                            Size::new(
                                f32::from(u8::try_from(rng.below(16)).unwrap_or(1)) + 1.0,
                                f32::from(u8::try_from(rng.below(16)).unwrap_or(1)) + 1.0,
                            ),
                        ),
                        Color::rgb(
                            u8::try_from(rng.below(256)).unwrap_or(0),
                            u8::try_from(rng.below(256)).unwrap_or(0),
                            u8::try_from(rng.below(256)).unwrap_or(0),
                        ),
                    )));
                }
                _ if !primitives.is_empty() => {
                    let index = rng.below(u64::try_from(primitives.len()).unwrap_or(1));
                    primitives.remove(index as usize);
                }
                _ => {}
            }
            let scene = SceneSnapshot { primitives };

            // The pipeline's damage rule: every old and new primitive extent.
            let mut damage = DamageBuilder::new();
            for primitive in prev_scene.primitives.iter().chain(scene.primitives.iter()) {
                damage.add(primitive_extent(primitive));
            }
            let damage = damage.finish(viewport);

            let expected = SoftwareRenderer::new()
                .render(
                    &scene,
                    width,
                    height,
                    Color::WHITE,
                    &mut ClosureGlyphSource::new(&mut |_f, _g, _s| None),
                )
                .expect("full render");
            let mut glyphs =
                |_f: u64, _g: u32, _s: f32| -> Option<alacer_core::scene::GlyphBitmap> { None };
            let mut source = ClosureGlyphSource::new(&mut glyphs);
            renderer
                .render_partial(&scene, &mut prev, &damage, Color::WHITE, &mut source)
                .expect("partial render");

            for y in 0..height {
                for x in 0..width {
                    let a = prev.pixel(x, y).expect("in bounds");
                    let b = expected.pixel(x, y).expect("in bounds");
                    assert_eq!(
                        (a.red(), a.green(), a.blue()),
                        (b.red(), b.green(), b.blue()),
                        "step fuzz partial != full at ({x},{y}) after step {action}"
                    );
                }
            }
            prev_scene = scene;
        }
    }

    #[test]
    fn blank_scene_renders_the_background() {
        let pixmap = render_with(
            &SceneSnapshot::default(),
            &mut |_f: u64, _g: u32, _s: f32| None,
        )
        .expect("pixmap");
        let pixel = pixmap.pixel(20, 15).expect("in bounds");
        assert_eq!((pixel.red(), pixel.green(), pixel.blue()), (255, 255, 255));
    }

    #[test]
    fn quad_fill_lands_on_expected_pixels() {
        let scene = SceneSnapshot {
            primitives: vec![Primitive::Quad(Quad::filled(
                Rect::new(Point::new(5.0, 5.0), Size::new(20.0, 10.0)),
                Color::rgb(255, 0, 0),
            ))],
        };
        let pixmap = render_with(&scene, &mut |_f: u64, _g: u32, _s: f32| None).expect("pixmap");
        let inside = pixmap.pixel(15, 10).expect("in bounds");
        assert_eq!((inside.red(), inside.green(), inside.blue()), (255, 0, 0));
        let outside = pixmap.pixel(2, 2).expect("in bounds");
        assert_eq!(outside.red(), 255, "background untouched outside the quad");
        assert_eq!(outside.green(), 255);
    }

    #[test]
    fn rounded_corners_leave_the_sharp_corner_untouched() {
        let scene = SceneSnapshot {
            primitives: vec![Primitive::Quad(Quad {
                rect: Rect::new(Point::new(5.0, 5.0), Size::new(20.0, 20.0)),
                color: Color::BLACK,
                corner_radius: 10.0,
                border_color: Color::TRANSPARENT,
                border_width: 0.0,
                clip: None,
            })],
        };
        let pixmap = render_with(&scene, &mut |_f: u64, _g: u32, _s: f32| None).expect("pixmap");
        let corner = pixmap.pixel(6, 6).expect("in bounds");
        assert!(
            corner.red() > 200,
            "sharp corner outside the radius stays background"
        );
        let center = pixmap.pixel(15, 15).expect("in bounds");
        assert!(center.red() < 50, "center is filled");
    }

    #[test]
    fn glyphs_blend_coverage_with_color() {
        let scene = SceneSnapshot {
            primitives: vec![Primitive::Glyph(Glyph {
                position: Point::new(10.0, 20.0),
                size: 16.0,
                color: Color::BLACK,
                font_id: 1,
                glyph_id: 65,
                clip: None,
            })],
        };
        let mut source = |_f: u64, _g: u32, _s: f32| Some(solid_bitmap(4, 4, 0, 0));
        let pixmap = render_with(&scene, &mut source).expect("pixmap");
        let inked = pixmap.pixel(10, 20).expect("in bounds");
        assert!(inked.red() < 50, "full coverage paints the glyph color");
    }

    #[test]
    fn glyph_color_channels_survive_the_coverage_blend() {
        // Non-black colors catch premultiply overflows: every channel scaled
        // by the raw 0..=255 alpha would clamp to white.
        let scene = SceneSnapshot {
            primitives: vec![Primitive::Glyph(Glyph {
                position: Point::new(10.0, 20.0),
                size: 16.0,
                color: Color::rgb(200, 30, 40),
                font_id: 1,
                glyph_id: 65,
                clip: None,
            })],
        };
        let mut source = |_f: u64, _g: u32, _s: f32| Some(solid_bitmap(4, 4, 0, 0));
        let pixmap = render_with(&scene, &mut source).expect("pixmap");
        let inked = pixmap.pixel(10, 20).expect("in bounds");
        assert!(
            (i32::from(inked.red()) - 200).abs() <= 6,
            "red channel must match the glyph color, got {}",
            inked.red()
        );
        assert!(
            (i32::from(inked.green()) - 30).abs() <= 6,
            "green channel must match the glyph color, got {}",
            inked.green()
        );
        assert!(
            (i32::from(inked.blue()) - 40).abs() <= 6,
            "blue channel must match the glyph color, got {}",
            inked.blue()
        );
        assert_eq!(inked.alpha(), 255, "full coverage is fully opaque");
    }

    #[test]
    fn half_coverage_blends_to_half_alpha() {
        let scene = SceneSnapshot {
            primitives: vec![Primitive::Glyph(Glyph {
                position: Point::new(10.0, 20.0),
                size: 16.0,
                color: Color::rgb(200, 30, 40),
                font_id: 1,
                glyph_id: 65,
                clip: None,
            })],
        };
        let mut source = |_f: u64, _g: u32, _s: f32| {
            let mut bitmap = solid_bitmap(2, 2, 0, 0);
            bitmap.coverage = Arc::from(vec![128_u8; 4].into_boxed_slice());
            Some(bitmap)
        };
        let pixmap = render_with(&scene, &mut source).expect("pixmap");
        let blended = pixmap.pixel(10, 20).expect("in bounds");
        // Source-over onto the opaque white background: the result stays
        // opaque, and red = premul_src + (1 - a_src)·dst
        //             = 200·128/255 + (1 − 128/255)·255 ≈ 100 + 127 = 227.
        assert!(
            (i32::from(blended.red()) - 227).abs() <= 8,
            "half coverage blends src over dst, got {}",
            blended.red()
        );
        // A black stamp at the same coverage proves the dst contribution is
        // the background, not a color error: 0 + 127 ≈ 127.
        let scene_dark = SceneSnapshot {
            primitives: vec![Primitive::Glyph(Glyph {
                position: Point::new(10.0, 20.0),
                size: 16.0,
                color: Color::BLACK,
                font_id: 1,
                glyph_id: 65,
                clip: None,
            })],
        };
        let pixmap_dark = render_with(&scene_dark, &mut source).expect("pixmap");
        let dark = pixmap_dark.pixel(10, 20).expect("in bounds");
        assert!(
            (i32::from(dark.red()) - 127).abs() <= 8,
            "black at half coverage leaves the white background at half strength, got {}",
            dark.red()
        );
    }

    #[test]
    fn clip_keeps_glyphs_inside_the_region() {
        let scene = SceneSnapshot {
            primitives: vec![Primitive::Glyph(Glyph {
                position: Point::new(10.0, 20.0),
                size: 16.0,
                color: Color::BLACK,
                font_id: 1,
                glyph_id: 65,
                clip: Some(Rect::new(Point::new(0.0, 0.0), Size::new(5.0, 5.0))),
            })],
        };
        let mut source = |_f: u64, _g: u32, _s: f32| Some(solid_bitmap(4, 4, 0, 0));
        let pixmap = render_with(&scene, &mut source).expect("pixmap");
        let outside = pixmap.pixel(10, 20).expect("in bounds");
        assert!(
            outside.red() > 200,
            "glyph outside the clip stays background"
        );
    }

    #[test]
    fn borders_stroke_the_outline() {
        let scene = SceneSnapshot {
            primitives: vec![Primitive::Quad(Quad {
                rect: Rect::new(Point::new(5.0, 5.0), Size::new(20.0, 20.0)),
                color: Color::WHITE,
                corner_radius: 0.0,
                border_color: Color::BLACK,
                border_width: 2.0,
                clip: None,
            })],
        };
        let pixmap = render_with(&scene, &mut |_f: u64, _g: u32, _s: f32| None).expect("pixmap");
        // The stroke is centered on the path: the rect edge sits at x=5, so the
        // darkest pixels are exactly on the edge line.
        let edge = pixmap.pixel(5, 15).expect("in bounds");
        assert!(edge.red() < 100, "border stroke darkens the edge");
        let middle = pixmap.pixel(15, 15).expect("in bounds");
        assert!(middle.red() > 200, "fill stays background inside");
    }
}
