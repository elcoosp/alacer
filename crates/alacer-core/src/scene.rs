//! The scene vocabulary: primitives the paint pass records and both backends
//! render (A8: one scene journal, two honest backends).
//!
//! Primitive types are core-owned because the pass system records them through
//! [`PaintCtx`]; the journal storage and rasterizers live in `alacer-render`.

use crate::color::Color;
use crate::geom::{Point, Rect, Size, Vector};
use crate::num::f32_to_u8;

/// The compose tier's per-node paint state (§14.2's W3 mechanism): the
/// fade and the translation a node paints with. The compose pass folds a
/// node's own state beside every ancestor's into the paint context
/// (fades multiply down the tree — a faded parent fades its subtree, the
/// CSS rule; translations add), and pushes apply the fold at record time:
/// the fade into the alpha channel the renderers already blend with, the
/// translation into the recorded coordinates the origin already rides.
///
/// The state is a *paint* fact, never a layout one (styles stay
/// layout-only, §7.2 pass 5): an animated frame moves it without
/// re-measuring — the M8 scrolling contract's tier, generalized from the
/// scroll offset to any compose state. `is_identity` gates the fold: a
/// frame where nobody composes records exactly the primitives the
/// pre-compose pipeline recorded — the byte-identical baseline the
/// compose-opacity suite pins.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Compose {
    /// The node's paint opacity, clamped to 0.0–1.0 at use; 1.0 paints
    /// untouched.
    pub opacity: f32,
    /// The translation the node's paint rides, in pixels.
    pub translation: Vector,
}

impl Compose {
    /// The identity: full opacity, no translation. Every push records
    /// exactly what it recorded before the compose tier existed.
    pub const IDENTITY: Self = Self {
        opacity: 1.0,
        translation: euclid::Vector2D::new(0.0, 0.0),
    };

    /// A fade-only compose: the subtree paints at `opacity`.
    #[must_use]
    pub const fn faded(opacity: f32) -> Self {
        Self {
            opacity,
            translation: euclid::Vector2D::new(0.0, 0.0),
        }
    }

    /// A translation-only compose: the subtree paints shifted by
    /// `translation`.
    #[must_use]
    pub const fn shifted(translation: Vector) -> Self {
        Self {
            opacity: 1.0,
            translation,
        }
    }

    /// Whether this state composes nothing: the gate that keeps a static
    /// frame's recorded scene byte-identical to the baseline. The compare
    /// is the bit pattern, not a margin: 1.0 is the exact identity (a ×1.0
    /// fold is the no-op the byte-identity contract names), so the gate
    /// answers "is this the identity constant", never "is this close to
    /// one".
    #[must_use]
    pub fn is_identity(&self) -> bool {
        self.opacity.to_bits() == Self::IDENTITY.opacity.to_bits()
            && self.translation == euclid::Vector2D::new(0.0, 0.0)
    }
}

/// One color's fade: the alpha `opacity` scales, the channels keep —
/// storage is non-premultiplied, the renderers premultiply at their own
/// boundary. A ×1.0 fold is the exact identity (`f32::from` × 1.0 is the
/// same f32, the rounding is the same), so a fully opaque compose can
/// never move a byte; callers gate on [`Compose::is_identity`] anyway.
fn faded(color: Color, opacity: f32) -> Color {
    Color::new(
        color.r,
        color.g,
        color.b,
        f32_to_u8(f32::from(color.a) * opacity),
    )
}

/// A filled (optionally rounded, optionally bordered) rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// How a raster image samples its source pixels when `src` and `dst` differ
/// in size: crisp texel copies (`Nearest`, the honest default for UI
/// rasters) or blended edges (`Bilinear`, for scaled thumbnails).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageSampling {
    /// Texel copies: each destination pixel reads its nearest source pixel.
    Nearest,
    /// Blended sampling: destination pixels average their source neighbors.
    Bilinear,
}

/// One raster image blit (the raster tier): premultiplied RGBA8 pixels
/// sampled from `src` (a window in bitmap space) into `dst` (absolute
/// scene space). The bytes ride an [`std::sync::Arc`] — app, journal, and
/// renderers share one allocation, the glyph coverage's own sharing rule.
///
/// The byte contract is **premultiplied** RGBA8, row-major, exactly
/// `width · height · 4` bytes: blending (software and GPU both) consumes
/// premultiplied colors directly, and the premultiplied form is what PNG
/// decode produces. A raster whose bytes cannot hold its declared
/// dimensions, whose source or destination is empty, or whose premultiplied
/// channels exceed their alpha is refused honestly — renderers draw
/// nothing for it, never a guessed pixel.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    /// Premultiplied RGBA8 pixels, row-major.
    pub bytes: std::sync::Arc<[u8]>,
    /// Bitmap width in pixels.
    pub width: u32,
    /// Bitmap height in pixels.
    pub height: u32,
    /// The source window, in bitmap pixel coordinates.
    pub src: Rect,
    /// The destination rect, in absolute scene coordinates.
    pub dst: Rect,
    /// Effective clip rect at record time.
    pub clip: Option<Rect>,
    /// How `src` scales into `dst` when their sizes differ.
    pub sampling: ImageSampling,
}

/// One recorded paint operation.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    /// A filled rectangle.
    Quad(Quad),
    /// A positioned glyph.
    Glyph(Glyph),
    /// A raster image blit (the raster tier).
    Image(Image),
}

/// Sink the paint pass records primitives into — implemented by the
/// pipeline's crate-private journal adapter (the paint pass in
/// [`crate::passes`]), which streams the primitives into the injected
/// [`crate::passes::SceneJournal`] — the layer store whose production
/// implementation is `alacer-render`'s journal. The renderer replays
/// the recorded layers; it never implements this trait itself.
pub trait PaintSink {
    /// Records one primitive; order defines paint order.
    fn push(&mut self, primitive: Primitive);
}

/// The paint pass hands this to [`crate::widget::Widget::paint`].
///
/// Coordinates are local: the pass adds the node's absolute origin when
/// recording, and applies the clip stack (scroll viewports clip children).
///
/// The context also carries the frame's resolved palette (ADR-0079): the
/// pass resolves the installed [`crate::theme::ThemeSeam`] once per paint
/// pass — one dynamic call per frame, never per node — and every widget
/// paints the roles its app did not override from the copy it holds.
/// Beside the palette it carries the frame's resolved OS facts
/// (ADR-0080, ADR-0083, ADR-0085, ADR-0087, ADR-0158): the installed
/// [`crate::settings::SettingSeam`]'s dark-mode, reduced-motion, and
/// high-contrast answers, and its accent-color answer riding its
/// presence (`None` is the honest absence), resolved the same
/// once-per-pass shape, the role-independent decisions the vocabulary
/// consults.
#[expect(
    clippy::struct_excessive_bools,
    reason = "the context holds the frame's resolved OS facts the vocabulary consults — the bools *are* the facts (ADR-0080's rule), one field per fact the pass resolved once; a bundle would trade every probe's direct read for nothing (the M154 cut grew the transparency beside the four)"
)]
pub struct PaintCtx<'a> {
    sink: &'a mut dyn PaintSink,
    origin: Point,
    size: crate::geom::Size,
    clip: Option<Rect>,
    palette: crate::theme::Palette,
    dark: bool,
    reduced_motion: bool,
    high_contrast: bool,
    accent: Option<Color>,
    reduced_transparency: bool,
    /// The compose state this context's pushes record with — the compose
    /// pass's inheritance write ([`Self::set_compose`]), identity until
    /// the pass says otherwise.
    compose: Compose,
}

impl<'a> PaintCtx<'a> {
    /// Creates a context painting at `origin` over `size`, with an optional
    /// clip, resolving paint roles from `palette`, the OS's dark-mode
    /// fact from `dark`, the OS's reduced-motion fact from
    /// `reduced_motion`, the OS's high-contrast fact from
    /// `high_contrast`, and the OS's accent-color fact from `accent`
    /// (riding its presence — `None` is the honest absence), and the
    /// OS's reduced-transparency fact from `reduced_transparency` (the
    /// paint pass resolves the installed seams once per pass; direct
    /// callers without a theme resolve the canon palette and the canon
    /// facts).
    #[expect(
        clippy::too_many_arguments,
        reason = "the constructor mirrors the context's own resolved facts — the palette beside the OS facts the pass resolved once each; packing them into a bundle would trade every direct caller's readability for nothing (the M79 cut grew the fact beside the two, the M81 cut the accent beside the three, the M154 cut the transparency beside the four)"
    )]
    #[expect(
        clippy::fn_params_excessive_bools,
        reason = "the facts ride the plain per-fact parameters the pass resolved — one dynamic call per fact per frame, the seam every widget's probe reads (ADR-0083, ADR-0085, ADR-0087, ADR-0158); the bundle refactor would trade the constructor's readability for nothing"
    )]
    #[must_use]
    pub fn new(
        sink: &'a mut dyn PaintSink,
        origin: Point,
        size: crate::geom::Size,
        clip: Option<Rect>,
        palette: crate::theme::Palette,
        dark: bool,
        reduced_motion: bool,
        high_contrast: bool,
        accent: Option<Color>,
        reduced_transparency: bool,
    ) -> Self {
        Self {
            sink,
            origin,
            size,
            clip,
            palette,
            dark,
            reduced_motion,
            high_contrast,
            accent,
            reduced_transparency,
            compose: Compose::IDENTITY,
        }
    }

    /// The laid-out size of the node being painted.
    #[must_use]
    pub const fn size(&self) -> crate::geom::Size {
        self.size
    }

    /// The frame's resolved palette (ADR-0079): the roles a widget
    /// paints for every visual its app did not override. `Copy` — the
    /// read is the pass's own per-frame resolution, never a per-node
    /// dynamic call.
    #[must_use]
    pub const fn palette(&self) -> crate::theme::Palette {
        self.palette
    }

    /// The frame's resolved OS fact (ADR-0080): the dark-mode answer
    /// the installed setting seam gave this pass — the fact a widget
    /// consults for the decisions no palette role answers (an inset
    /// shadow's direction, a thumbnail's border weight). The fact never
    /// munges the palette: the roles re-resolve from the installed
    /// palette exactly as they were.
    #[must_use]
    pub const fn dark(&self) -> bool {
        self.dark
    }

    /// The frame's resolved OS fact beside the dark one (ADR-0083): the
    /// reduced-motion answer the installed setting seam gave this pass
    /// — the fact a widget consults when motion itself is the question
    /// (a spinner's spin, a thumbnail's autoplay), the same
    /// once-per-pass shape the dark fact rides, never a per-node call.
    #[must_use]
    pub const fn reduced_motion(&self) -> bool {
        self.reduced_motion
    }

    /// The frame's resolved OS fact beside the two (ADR-0085): the
    /// high-contrast answer the installed setting seam gave this pass —
    /// the fact a widget consults when contrast itself is the question
    /// (a hairline's weight, a focus ring's width), the same
    /// once-per-pass shape the dark and motion facts ride, never a
    /// per-node call.
    #[must_use]
    pub const fn high_contrast(&self) -> bool {
        self.high_contrast
    }

    /// The frame's resolved OS fact beside the three (ADR-0087): the
    /// accent-color answer the installed setting seam gave this pass —
    /// the fact a widget consults when tint itself is the question (an
    /// accent that follows the OS's pick, a selection riding it), the
    /// same once-per-pass shape the dark, motion, and contrast facts
    /// ride, never a per-node call. The fact rides its presence: `None`
    /// is the honest absence — a platform that reports no accent
    /// answers `None`, never a guessed color.
    #[must_use]
    pub const fn accent_color(&self) -> Option<Color> {
        self.accent
    }

    /// The frame's resolved OS fact beside the four (ADR-0158): the
    /// reduced-transparency answer the installed setting seam gave this
    /// pass — the fact a widget consults when translucency itself is
    /// the question (a backdrop that mutes its glass, a selection that
    /// holds its fill, an overlay that goes solid), the same
    /// once-per-pass shape the dark, motion, contrast, and accent facts
    /// ride, never a per-node call. The fact answers the ask: `false`
    /// is effects allowed, the canon answer the default build resolves.
    #[must_use]
    pub const fn reduced_transparency(&self) -> bool {
        self.reduced_transparency
    }

    /// Records one quad, translated to absolute coordinates: the node's
    /// origin beside the compose translation (the compose tier rides the
    /// same record-time fold the origin does), and the compose fade folded
    /// into the fill's and border's alpha — the channel the renderers
    /// already blend with, so a faded quad is an ordinary quad to every
    /// backend (the premultiplied pipelines need no new fact).
    pub fn push_quad(&mut self, quad: Quad) {
        let mut quad = Quad {
            rect: quad
                .rect
                .translate(self.origin.to_vector() + self.compose.translation),
            clip: self.clip,
            ..quad
        };
        if self.compose.opacity.to_bits() != Compose::IDENTITY.opacity.to_bits() {
            quad.color = faded(quad.color, self.compose.opacity);
            quad.border_color = faded(quad.border_color, self.compose.opacity);
        }
        self.sink.push(Primitive::Quad(quad));
    }

    /// Records one glyph; `position` is relative to this node's origin,
    /// beside the compose translation (the compose tier's ride). The
    /// compose fade folds into the glyph's alpha — the coverage stamp
    /// multiplies it in, the same channel the atlas blend consumes.
    pub fn push_glyph(&mut self, glyph: Glyph) {
        let mut glyph = Glyph {
            position: glyph.position + self.origin.to_vector() + self.compose.translation,
            clip: self.clip,
            ..glyph
        };
        if self.compose.opacity.to_bits() != Compose::IDENTITY.opacity.to_bits() {
            glyph.color = faded(glyph.color, self.compose.opacity);
        }
        self.sink.push(Primitive::Glyph(glyph));
    }

    /// Records one raster image blit; `dst` is relative to this node's
    /// origin beside the compose translation (the compose tier's ride).
    /// The source window rides bitmap coordinates untouched.
    ///
    /// The compose fade does *not* fold into an image: its bytes are a
    /// shared `Arc` (the app, journal, and renderers own one allocation —
    /// the glyph coverage's sharing rule), and fading would fork the
    /// allocation the warm-path budget forbids (§14.3, ADR-0088). Images
    /// compose translation only; their own alpha rides the premultiplied
    /// bytes as decoded.
    pub fn push_image(&mut self, image: Image) {
        self.sink.push(Primitive::Image(Image {
            dst: image
                .dst
                .translate(self.origin.to_vector() + self.compose.translation),
            clip: self.clip,
            ..image
        }));
    }

    /// This node's absolute origin.
    #[must_use]
    pub fn origin(&self) -> Point {
        self.origin
    }

    /// The compose pass's inheritance write: installs the compose state in
    /// effect at this node — the node's own beside every ancestor's, the
    /// product the pass's ancestor walk folded ([`Compose`]'s contract).
    /// Called once per painted node, before the widget paints; the
    /// untouched context composes nothing.
    pub fn set_compose(&mut self, compose: Compose) {
        self.compose = compose;
    }

    /// The compose state this context records with — the pass's
    /// inheritance product. A widget composing its own parts consults it;
    /// every push applies it whether the widget reads it or not.
    #[must_use]
    pub const fn compose(&self) -> Compose {
        self.compose
    }
}

/// The ordered materialization of the journal that renderers consume.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SceneSnapshot {
    /// Every recorded primitive, in paint order (pre-order tree walk).
    pub primitives: Vec<Primitive>,
}

/// The conservative margin (pixels) wrapped around every primitive extent
/// before it becomes damage (§9.3): anti-aliased edges and glyph bitmap
/// overhang must never fall outside a region, or partial frames leave stale
/// pixels. Overestimating is safe — it only widens the repaint area.
pub const PAINT_DAMAGE_MARGIN: f32 = 2.0;

/// Inflates `rect` by `margin` on every side.
#[must_use]
pub fn inflate(rect: Rect, margin: f32) -> Rect {
    Rect::new(
        Point::new(rect.origin.x - margin, rect.origin.y - margin),
        Size::new(
            rect.size.width + margin * 2.0,
            rect.size.height + margin * 2.0,
        ),
    )
}

/// The conservative painted extent of one primitive, in absolute coordinates:
/// the area any correct renderer may touch. Quads span their rect plus half
/// the border stroke (the stroke is centered on the path) plus the AA margin;
/// glyphs span a generous box around the baseline origin, because bitmap
/// metrics belong to the font (up to ~1.05 em above the baseline, ~0.45 em
/// below, ±0.5 em horizontally) and the pipeline must not depend on font
/// internals to compute damage; images span their destination rect plus the
/// AA margin — sampling reads inside `src` and writes inside `dst`, so the
/// destination is the whole render footprint.
#[must_use]
pub fn primitive_extent(primitive: &Primitive) -> Rect {
    match primitive {
        Primitive::Quad(quad) => {
            let stroke = quad.border_width * 0.5;
            inflate(quad.rect, stroke + PAINT_DAMAGE_MARGIN)
        }
        Primitive::Glyph(glyph) => {
            let s = glyph.size;
            Rect::new(
                Point::new(glyph.position.x - s * 0.6, glyph.position.y - s * 1.1),
                Size::new(
                    s * 2.2 + PAINT_DAMAGE_MARGIN * 2.0,
                    s * 1.7 + PAINT_DAMAGE_MARGIN * 2.0,
                ),
            )
        }
        Primitive::Image(image) => inflate(image.dst, PAINT_DAMAGE_MARGIN),
    }
}

/// The union of every primitive's extent, inflated once more by the AA
/// margin — the damage contribution of one committed layer.
#[must_use]
pub fn primitives_extent(primitives: &[Primitive]) -> Option<Rect> {
    let mut iter = primitives.iter().map(primitive_extent);
    let first = iter.next()?;
    Some(iter.fold(first, |acc, r| acc.union(&r)))
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
    use super::{Compose, Glyph, PaintSink, Primitive, Quad};
    use crate::color::Color;
    use crate::geom::{Point, Rect, Size, Vector};
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
        let mut ctx = PaintCtx::new(
            &mut sink,
            origin,
            crate::geom::Size::new(10.0, 10.0),
            None,
            crate::theme::Palette::CANON,
            false,
            false,
            false,
            None,
            false,
        );

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

    /// The test's quad: an opaque red 20×10 rect at the local origin.
    fn red_quad() -> Quad {
        Quad::filled(
            Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)),
            Color::rgb(255, 0, 0),
        )
    }

    #[test]
    fn identity_compose_records_the_baseline_primitive() {
        // The byte-identical contract: a context the pass never touched
        // and one set to the identity record the *same* primitive — the
        // static frame's zero-regression gate.
        let mut plain = Collector::default();
        let mut composed = Collector::default();
        let facts = (
            Point::new(4.0, 6.0),
            Size::new(40.0, 30.0),
            crate::theme::Palette::CANON,
        );
        {
            let mut ctx = PaintCtx::new(
                &mut plain, facts.0, facts.1, None, facts.2, false, false, false, None, false,
            );
            ctx.push_quad(red_quad());
        }
        {
            let mut ctx = PaintCtx::new(
                &mut composed,
                facts.0,
                facts.1,
                None,
                facts.2,
                false,
                false,
                false,
                None,
                false,
            );
            ctx.set_compose(Compose::IDENTITY);
            ctx.push_quad(red_quad());
        }
        assert_eq!(plain.0, composed.0, "the identity composes nothing");
    }

    #[test]
    fn compose_fade_folds_into_the_recorded_alpha() {
        let mut sink = Collector::default();
        let mut ctx = PaintCtx::new(
            &mut sink,
            Point::new(0.0, 0.0),
            Size::new(40.0, 30.0),
            None,
            crate::theme::Palette::CANON,
            false,
            false,
            false,
            None,
            false,
        );
        ctx.set_compose(Compose::faded(0.5));
        ctx.push_quad(red_quad());

        let Primitive::Quad(quad) = &sink.0[0] else {
            panic!("expected quad");
        };
        assert_eq!(
            quad.color.a, 128,
            "the fade folded into the alpha (255 × 0.5, rounded once), channels untouched"
        );
        assert_eq!(
            (quad.color.r, quad.color.g, quad.color.b),
            (255, 0, 0),
            "the fade is an alpha fact, never a color rewrite"
        );
        assert_eq!(
            quad.rect,
            Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)),
            "a fade moves no geometry"
        );
    }

    #[test]
    fn compose_translation_rides_the_recorded_geometry() {
        let mut sink = Collector::default();
        let mut ctx = PaintCtx::new(
            &mut sink,
            Point::new(10.0, 20.0),
            Size::new(40.0, 30.0),
            None,
            crate::theme::Palette::CANON,
            false,
            false,
            false,
            None,
            false,
        );
        ctx.set_compose(Compose::shifted(Vector::new(5.0, 3.0)));
        ctx.push_quad(red_quad());
        ctx.push_glyph(Glyph {
            position: Point::new(2.0, 8.0),
            size: 12.0,
            color: Color::BLACK,
            font_id: 1,
            glyph_id: 65,
            clip: None,
        });

        let Primitive::Quad(quad) = &sink.0[0] else {
            panic!("expected quad");
        };
        assert_eq!(
            quad.rect.origin,
            Point::new(15.0, 23.0),
            "origin beside the compose translation"
        );
        let Primitive::Glyph(glyph) = &sink.0[1] else {
            panic!("expected glyph");
        };
        assert_eq!(glyph.position, Point::new(17.0, 31.0));
        assert_eq!(glyph.color.a, 255, "a shift is not a fade");
    }

    #[test]
    fn paint_ctx_translates_image_destinations() {
        let mut sink = Collector::default();
        let mut ctx = PaintCtx::new(
            &mut sink,
            Point::new(100.0, 50.0),
            crate::geom::Size::new(10.0, 10.0),
            Some(crate::geom::Rect::new(
                Point::new(0.0, 0.0),
                Size::new(50.0, 50.0),
            )),
            crate::theme::Palette::CANON,
            false,
            false,
            false,
            None,
            false,
        );

        ctx.push_image(super::Image {
            bytes: std::sync::Arc::from(vec![255_u8, 0, 0, 255].into_boxed_slice()),
            width: 1,
            height: 1,
            src: Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0)),
            dst: Rect::new(Point::new(2.0, 3.0), Size::new(4.0, 4.0)),
            clip: None,
            sampling: super::ImageSampling::Nearest,
        });

        assert_eq!(sink.0.len(), 1);
        let Primitive::Image(image) = &sink.0[0] else {
            panic!("expected image");
        };
        assert_eq!(
            image.dst,
            Rect::new(Point::new(102.0, 53.0), Size::new(4.0, 4.0)),
            "the destination rides absolute coordinates"
        );
        assert_eq!(
            image.clip,
            Some(Rect::new(Point::new(0.0, 0.0), Size::new(50.0, 50.0))),
            "the clip stack rides"
        );
        assert_eq!(
            image.src.size,
            Size::new(1.0, 1.0),
            "the source window stays in bitmap space"
        );
    }
}
