//! The software rasterizer: one scene snapshot → one RGBA pixmap via
//! tiny-skia (ADR-2: tiny-skia + softbuffer is the headless/CI/low-end
//! backend, and A4: the test rig *is* this backend).
//!
//! Glyphs arrive as coverage bitmaps through the [`GlyphSource`] seam — the
//! exact same bitmaps the GPU backend uploads to its atlas.

use alacer_core::color::Color;
use alacer_core::num::{f32_to_i32, f32_to_u8};
use alacer_core::scene::{Glyph, GlyphSource, Primitive, Quad, SceneSnapshot};
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
                    Primitive::Quad(quad) => self.draw_quad(&mut painter, quad),
                    Primitive::Glyph(glyph) => self.draw_glyph(&mut painter, glyph, glyphs),
                }
            }
        }
        Some(pixmap)
    }

    fn draw_quad(&self, painter: &mut tiny_skia::PixmapMut<'_>, quad: &Quad) {
        let Some(rect) = Rect::from_xywh(
            quad.rect.origin.x,
            quad.rect.origin.y,
            quad.rect.size.width,
            quad.rect.size.height,
        ) else {
            return;
        };
        let path = quad_path(rect, quad.corner_radius);
        let clip = clip_mask(quad.clip, painter.width(), painter.height());
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
        let clip = clip_mask(glyph.clip, painter.width(), painter.height());
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

fn clip_mask(clip: Option<alacer_core::geom::Rect>, width: u32, height: u32) -> Option<Mask> {
    let clip = clip?;
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
    use alacer_core::geom::{Point, Rect, Size};
    use alacer_core::scene::{Glyph, Primitive, Quad, SceneSnapshot};
    use std::sync::Arc;

    fn render_with<F>(scene: &SceneSnapshot, source: &mut F) -> Option<tiny_skia::Pixmap>
    where
        F: FnMut(u64, u32, f32) -> Option<alacer_core::scene::GlyphBitmap>,
    {
        let mut src = ClosureGlyphSource::new(source);
        SoftwareRenderer::new().render(scene, 40, 30, Color::WHITE, &mut src)
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
