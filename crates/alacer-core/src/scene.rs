//! The scene vocabulary: primitives the paint pass records and both backends
//! render (A8: one scene journal, two honest backends).
//!
//! Primitive types are core-owned because the pass system records them through
//! [`PaintCtx`]; the journal storage and rasterizers live in `alacer-render`.

use crate::color::Color;
use crate::geom::{Point, Rect};

/// A filled (optionally rounded, optionally bordered) rectangle.
#[derive(Debug, Clone, PartialEq)]
pub struct Quad {
    /// Rectangle in local coordinates (origin added by the pass system).
    pub rect: Rect,
    /// Fill color.
    pub color: Color,
    /// Corner radius; 0.0 renders a sharp rect.
    pub corner_radius: f32,
    /// Border color (ignored when `border_width` is 0).
    pub border_color: Color,
    /// Border stroke width; 0.0 means no border.
    pub border_width: f32,
    /// Effective clip rect at record time, already in absolute coordinates.
    pub clip: Option<Rect>,
}

impl Quad {
    /// A sharp, unbordered filled rectangle.
    #[must_use]
    pub fn filled(rect: Rect, color: Color) -> Self {
        Self {
            rect,
            color,
            corner_radius: 0.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        }
    }
}

/// One positioned glyph, resolved by the renderers through the font registry.
#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    /// Absolute position of the glyph's baseline origin.
    pub position: Point,
    /// Font size in pixels.
    pub size: f32,
    /// Text color.
    pub color: Color,
    /// Interned font tag (see `alacer-text`'s font registry).
    pub font_id: u64,
    /// Glyph index within the font.
    pub glyph_id: u32,
    /// Effective clip rect at record time.
    pub clip: Option<Rect>,
}

/// One recorded paint operation.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    /// A filled rectangle.
    Quad(Quad),
    /// A positioned glyph.
    Glyph(Glyph),
}

/// Sink the paint pass records primitives into — implemented by the scene
/// journal in `alacer-render`.
pub trait PaintSink {
    /// Records one primitive; order defines paint order.
    fn push(&mut self, primitive: Primitive);
}

/// The paint pass hands this to [`crate::widget::Widget::paint`].
///
/// Coordinates are local: the pass adds the node's absolute origin when
/// recording, and applies the clip stack (scroll viewports clip children).
pub struct PaintCtx<'a> {
    sink: &'a mut dyn PaintSink,
    origin: Point,
    size: crate::geom::Size,
    clip: Option<Rect>,
}

impl<'a> PaintCtx<'a> {
    /// Creates a context painting at `origin` over `size`, with an optional clip.
    #[must_use]
    pub fn new(
        sink: &'a mut dyn PaintSink,
        origin: Point,
        size: crate::geom::Size,
        clip: Option<Rect>,
    ) -> Self {
        Self {
            sink,
            origin,
            size,
            clip,
        }
    }

    /// The laid-out size of the node being painted.
    #[must_use]
    pub const fn size(&self) -> crate::geom::Size {
        self.size
    }

    /// Records one quad, translated to absolute coordinates.
    pub fn push_quad(&mut self, quad: Quad) {
        self.sink.push(Primitive::Quad(Quad {
            rect: quad.rect.translate(self.origin.to_vector()),
            clip: self.clip,
            ..quad
        }));
    }

    /// Records one glyph; `position` is relative to this node's origin.
    pub fn push_glyph(&mut self, glyph: Glyph) {
        self.sink.push(Primitive::Glyph(Glyph {
            position: glyph.position + self.origin.to_vector(),
            clip: self.clip,
            ..glyph
        }));
    }

    /// Runs `f` with children clipped to `rect` (absolute) intersected with the
    /// current clip — the scroll-viewport mechanism.
    pub fn with_clip(&mut self, rect: Rect, f: impl FnOnce(&mut Self)) {
        let outer = self.clip;
        self.clip = Some(match outer {
            Some(o) => o.intersection(&rect).unwrap_or_default(),
            None => rect,
        });
        f(self);
        self.clip = outer;
    }

    /// The absolute clip currently in effect.
    #[must_use]
    pub fn clip(&self) -> Option<Rect> {
        self.clip
    }

    /// This node's absolute origin.
    #[must_use]
    pub fn origin(&self) -> Point {
        self.origin
    }
}

/// The ordered materialization of the journal that renderers consume.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SceneSnapshot {
    /// Every recorded primitive, in paint order (pre-order tree walk).
    pub primitives: Vec<Primitive>,
}

/// A rasterized glyph: 8-bit coverage plus sub-pixel placement.
///
/// Produced by `alacer-text` (swash/zeno), consumed by both backends — one
/// rasterization path (A8). Stored via `Arc` so atlas builders and software
/// blending share the same allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphBitmap {
    /// Horizontal offset from the glyph origin (can be negative).
    pub left: i32,
    /// Vertical offset from the glyph origin (baseline sits below the top).
    pub top: i32,
    /// Bitmap width in pixels.
    pub width: u32,
    /// Bitmap height in pixels.
    pub height: u32,
    /// Row-major 8-bit coverage.
    pub coverage: std::sync::Arc<[u8]>,
}

/// The atlas identity of one glyph: font, glyph index, and the exact size bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    /// Interned font tag.
    pub font_id: u64,
    /// Glyph index within the font.
    pub glyph_id: u32,
    /// `font_size.to_bits()` — exact float identity.
    pub size_bits: u32,
}

/// The glyph-rasterization seam: renderers query, `alacer-text` serves.
pub trait GlyphSource {
    /// The coverage bitmap for one glyph at `size`, or `None` for empty
    /// outlines (spaces) and unknown fonts.
    fn glyph_bitmap(&mut self, font_id: u64, glyph_id: u32, size: f32) -> Option<GlyphBitmap>;
}

#[cfg(test)]
mod tests {
    use super::{Glyph, PaintSink, Primitive, Quad};
    use crate::color::Color;
    use crate::geom::{Point, Rect, Size};
    use crate::scene::PaintCtx;

    #[derive(Default)]
    struct Collector(Vec<Primitive>);

    impl PaintSink for Collector {
        fn push(&mut self, primitive: Primitive) {
            self.0.push(primitive);
        }
    }

    #[test]
    fn paint_ctx_translates_to_absolute_coordinates() {
        let mut sink = Collector::default();
        let origin = Point::new(100.0, 50.0);
        let mut ctx = PaintCtx::new(&mut sink, origin, crate::geom::Size::new(10.0, 10.0), None);

        ctx.push_quad(Quad::filled(
            Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)),
            Color::BLACK,
        ));
        ctx.push_glyph(Glyph {
            position: Point::new(2.0, 8.0),
            size: 12.0,
            color: Color::BLACK,
            font_id: 1,
            glyph_id: 65,
            clip: None,
        });

        assert_eq!(sink.0.len(), 2);
        let Primitive::Quad(quad) = &sink.0[0] else {
            panic!("expected quad");
        };
        assert_eq!(quad.rect.origin, Point::new(100.0, 50.0));
        assert_eq!(quad.clip, None);
        let Primitive::Glyph(glyph) = &sink.0[1] else {
            panic!("expected glyph");
        };
        assert_eq!(glyph.position, Point::new(102.0, 58.0));
    }

    #[test]
    fn nested_clip_intersects() {
        let mut sink = Collector::default();
        let mut ctx = PaintCtx::new(
            &mut sink,
            Point::zero(),
            Size::new(50.0, 50.0),
            Some(Rect::new(Point::new(0.0, 0.0), Size::new(50.0, 50.0))),
        );

        let mut inner = None;
        ctx.with_clip(
            Rect::new(Point::new(40.0, 40.0), Size::new(50.0, 50.0)),
            |c| {
                inner = c.clip();
            },
        );
        assert_eq!(
            inner,
            Some(Rect::new(Point::new(40.0, 40.0), Size::new(10.0, 10.0)))
        );
        assert_eq!(
            ctx.clip(),
            Some(Rect::new(Point::new(0.0, 0.0), Size::new(50.0, 50.0))),
            "outer clip restored"
        );
    }
}
