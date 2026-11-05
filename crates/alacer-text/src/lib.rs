//! Alacer's text: a parley/fontique/swash facade (ADR-4).
//!
//! This crate owns shaping and measuring. Glyph *rasterization* lives here
//! too — `zeno` (swash's own rasterizer) turns scaled outlines into coverage
//! bitmaps that both backends consume, so there is exactly one rasterization
//! path (A8).

pub mod cache;
pub mod raster;
pub mod registry;

use crate::cache::LayoutCache;
use crate::registry::FontRegistry;
use parley::layout::PositionedLayoutItem;
use parley::{
    FontContext, FontFamily, FontFamilyName, GenericFamily, LayoutContext, StyleProperty,
};

/// One positioned, shaped glyph.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedGlyph {
    /// X of the glyph origin (baseline-relative from the line layout).
    pub x: f32,
    /// Y of the glyph origin (baseline).
    pub y: f32,
    /// Glyph index within the font.
    pub glyph_id: u32,
    /// Interned font tag — resolvable through the [`FontRegistry`].
    pub font_id: u64,
    /// Font size this glyph was shaped at.
    pub size: f32,
}

/// A fully shaped, positioned text layout.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextLayout {
    /// Ink width in pixels.
    pub width: f32,
    /// Line height total in pixels.
    pub height: f32,
    /// Positioned glyphs in paint order.
    pub glyphs: Vec<PositionedGlyph>,
}

/// The text subsystem: font collection, shaping, registry, cache, and the
/// glyph rasterizer.
pub struct TextContext {
    font: FontContext,
    layout: LayoutContext<[u8; 4]>,
    registry: FontRegistry,
    cache: LayoutCache,
    raster: raster::GlyphRasterizer,
}

impl Default for TextContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TextContext {
    /// Creates the context; fontique scans system fonts lazily on first use.
    #[must_use]
    pub fn new() -> Self {
        Self {
            font: FontContext::new(),
            layout: LayoutContext::new(),
            registry: FontRegistry::new(),
            cache: LayoutCache::new(256, 8),
            raster: raster::GlyphRasterizer::new(),
        }
    }

    /// The font registry (glyph rasterization resolves font tags through it).
    #[must_use]
    pub const fn registry(&self) -> &FontRegistry {
        &self.registry
    }

    /// The layout cache (frame-windowed eviction, §9.4).
    #[must_use]
    pub const fn cache(&self) -> &LayoutCache {
        &self.cache
    }

    /// Drops cache entries not touched within the eviction window.
    pub fn advance_frame(&mut self) {
        self.cache.advance_frame();
    }

    /// Shapes `text` at `font_size`, wrapping at `max_width` when given.
    ///
    /// Results are cached; repeated calls with the same inputs cost a lookup.
    #[must_use]
    pub fn layout_text(
        &mut self,
        text: &str,
        font_size: f32,
        max_width: Option<f32>,
    ) -> TextLayout {
        let key = cache::CacheKey::new(text, font_size, max_width);
        if let Some(hit) = self.cache.get(&key) {
            return hit;
        }

        let mut builder = self.layout.ranged_builder(&mut self.font, text, 1.0, true);
        builder.push_default(StyleProperty::FontFamily(FontFamily::Single(
            FontFamilyName::Generic(GenericFamily::SystemUi),
        )));
        builder.push_default(StyleProperty::FontSize(font_size));
        builder.push_default(StyleProperty::Brush([255, 255, 255, 255]));

        let mut laid: parley::Layout<[u8; 4]> = builder.build(text);
        laid.break_all_lines(max_width);

        let mut glyphs = Vec::new();
        for line in laid.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                    let font = glyph_run.run().font();
                    let font_id = self.registry.intern(font);
                    let size = glyph_run.run().font_size();
                    for glyph in glyph_run.positioned_glyphs() {
                        glyphs.push(PositionedGlyph {
                            x: glyph.x,
                            y: glyph.y,
                            glyph_id: glyph.id,
                            font_id,
                            size,
                        });
                    }
                }
            }
        }

        let result = TextLayout {
            width: laid.width(),
            height: laid.height(),
            glyphs,
        };
        self.cache.put(key, result.clone());
        result
    }

    /// Intrinsic size of a text run — the [`alacer_core::style::MeasureFn`] payload.
    #[must_use]
    pub fn measure(
        &mut self,
        spec: &alacer_core::style::TextSpec,
        max_width: Option<f32>,
    ) -> alacer_core::geom::Size {
        let layout = self.layout_text(&spec.text, spec.font_size, max_width);
        alacer_core::geom::Size::new(layout.width, layout.height)
    }
}

thread_local! {
    static GLOBAL_TEXT: std::cell::RefCell<TextContext> = std::cell::RefCell::new(TextContext::new());
}

/// Runs `f` with the thread's shared text context.
///
/// Widgets paint and measure through this single instance so shaping caches
/// are shared app-wide. Main-thread only (the frame loop's thread).
pub fn with_text<R>(f: impl FnOnce(&mut TextContext) -> R) -> R {
    GLOBAL_TEXT.with(|cell| f(&mut cell.borrow_mut()))
}

/// The `MeasureFn` adapter routed through the thread-local context.
///
/// A zero-sized marker: the reactive closure plumbing requires `Send + Sync`
/// types, while parley's `FontContext` is main-thread state — the adapter
/// satisfies both by routing calls into [`with_text`] (they always execute on
/// the frame thread).
#[derive(Debug, Default, Clone, Copy)]
pub struct ThreadLocalMeasurer;

impl alacer_core::style::MeasureFn for ThreadLocalMeasurer {
    fn measure(
        &self,
        spec: &alacer_core::style::TextSpec,
        max_width: Option<f32>,
    ) -> alacer_core::geom::Size {
        with_text(|ctx| ctx.measure(spec, max_width))
    }
}

/// A [`GlyphSource`](alacer_core::scene::GlyphSource) routed through the
/// thread-local context.
#[derive(Debug, Default, Clone, Copy)]
pub struct ThreadLocalGlyphSource;

impl alacer_core::scene::GlyphSource for ThreadLocalGlyphSource {
    fn glyph_bitmap(
        &mut self,
        font_id: u64,
        glyph_id: u32,
        size: f32,
    ) -> Option<alacer_core::scene::GlyphBitmap> {
        with_text(|ctx| {
            let registry = &ctx.registry;
            let raster = &mut ctx.raster;
            raster.rasterize(font_id, glyph_id, size, &|tag| registry.font_data(tag))
        })
    }
}

/// The [`TextSeam`](alacer_core::text_seam::TextSeam) implementation routed
/// through the thread-local context — what assembly sites install so widgets
/// can shape and measure text without depending on this crate (ADR-9 M1
/// tightening).
///
/// A zero-sized marker: the seam executes on the frame thread, so routing
/// through [`with_text`] shares the process's shaping caches.
#[derive(Debug, Default, Clone, Copy)]
pub struct ThreadLocalTextSeam;

impl alacer_core::text_seam::TextSeam for ThreadLocalTextSeam {
    fn measure(
        &self,
        spec: &alacer_core::style::TextSpec,
        max_width: Option<f32>,
    ) -> alacer_core::geom::Size {
        with_text(|ctx| ctx.measure(spec, max_width))
    }

    fn layout_glyphs(
        &self,
        text: &str,
        font_size: f32,
        max_width: Option<f32>,
    ) -> alacer_core::text_seam::GlyphRun {
        let layout = with_text(|ctx| ctx.layout_text(text, font_size, max_width));
        alacer_core::text_seam::GlyphRun {
            width: layout.width,
            height: layout.height,
            glyphs: layout
                .glyphs
                .iter()
                .map(|glyph| alacer_core::text_seam::PlacedGlyph {
                    x: glyph.x,
                    y: glyph.y,
                    size: glyph.size,
                    font_id: glyph.font_id,
                    glyph_id: glyph.glyph_id,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TextContext, ThreadLocalGlyphSource, ThreadLocalMeasurer, with_text};
    use alacer_core::style::{MeasureFn, TextSpec};

    fn spec(text: &str) -> TextSpec {
        TextSpec {
            text: text.to_string(),
            font_id: 0,
            font_size: 16.0,
        }
    }

    #[test]
    fn text_measures_nonzero_and_monotonic_in_length() {
        let mut ctx = TextContext::new();
        let short = ctx.measure(&spec("hello"), None);
        assert!(short.width > 0.0 && short.height > 0.0, "real font metrics");

        let longer = ctx.measure(&spec("hello world, longer text"), None);
        assert!(longer.width > short.width, "more text is wider");
    }

    #[test]
    fn wrapping_reduces_width_and_increases_height() {
        let mut ctx = TextContext::new();
        let unwrapped = ctx.measure(&spec("aaaa bbbb cccc dddd eeee ffff gggg"), None);
        let wrapped = ctx.measure(&spec("aaaa bbbb cccc dddd eeee ffff gggg"), Some(60.0));
        assert!(wrapped.width <= 60.0 + 8.0, "respects the wrap width");
        assert!(wrapped.height > unwrapped.height, "wrapping adds lines");
    }

    #[test]
    fn cache_makes_repeat_layouts_cheap() {
        let mut ctx = TextContext::new();
        let first = ctx.layout_text("cache me", 16.0, None);
        let second = ctx.layout_text("cache me", 16.0, None);
        assert_eq!(first, second, "cached result identical");
        assert!(!ctx.cache().is_empty());
    }

    #[test]
    fn glyph_positions_reference_a_registered_font() {
        let mut ctx = TextContext::new();
        let layout = ctx.layout_text("g", 16.0, None);
        assert!(!layout.glyphs.is_empty(), "at least one glyph shaped");
        for glyph in &layout.glyphs {
            let resolved = ctx.registry().font_data(glyph.font_id);
            assert!(resolved.is_some(), "every glyph's font is interned");
        }
    }

    #[test]
    fn thread_local_measurer_implements_the_seam() {
        let measurer = ThreadLocalMeasurer;
        let size = measurer.measure(&spec("seam"), None);
        assert!(size.width > 0.0 && size.height > 0.0);
        let direct = with_text(|ctx| ctx.layout_text("seam", 16.0, None));
        assert_eq!(
            size.width, direct.width,
            "seam routes through the same layout"
        );
        assert_eq!(size.height, direct.height);
    }

    #[test]
    fn thread_local_glyph_source_serves_registered_fonts() {
        use alacer_core::scene::GlyphSource as _;
        let mut source = ThreadLocalGlyphSource;
        let layout = with_text(|ctx| ctx.layout_text("Ag", 20.0, None));
        let glyph = layout.glyphs.first().expect("shaped glyph");
        let bitmap = source.glyph_bitmap(glyph.font_id, glyph.glyph_id, glyph.size);
        assert!(bitmap.is_some(), "real font rasterizes a real glyph");
        let bitmap = bitmap.expect("bitmap");
        assert!(bitmap.width > 0 && bitmap.height > 0);
        assert_eq!(
            bitmap.coverage.len(),
            (bitmap.width * bitmap.height) as usize,
            "coverage is w×h"
        );
    }
}
