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

    /// The engine's UAX #29 word segmentation (ADR-0019): ICU's
    /// `WordSegmenter` — the same segmenter parley's line breaking adopts,
    /// dictionary-backed for the complex scripts — yields byte offsets with
    /// per-segment classification; the seam converts them into the editing
    /// contract's UTF-16 units. Segments cover the run exactly (the
    /// iterator always breaks at 0 and the length).
    fn word_segments(&self, text: &str) -> Vec<alacer_core::text_seam::WordSegment> {
        let segmenter = icu_segmenter::WordSegmenter::new_auto(
            icu_segmenter::options::WordBreakInvariantOptions::default(),
        );
        let mut segments = Vec::new();
        let mut previous: Option<(usize, icu_segmenter::options::WordType)> = None;
        for (byte, word_type) in segmenter.segment_str(text).iter_with_word_type() {
            if let Some((start_byte, _)) = previous {
                let start = alacer_core::utf16::utf16_offset(text, start_byte);
                let end = alacer_core::utf16::utf16_offset(text, byte);
                if start < end {
                    segments.push(alacer_core::text_seam::WordSegment {
                        start,
                        end,
                        word_like: word_type.is_word_like(),
                    });
                }
            }
            previous = Some((byte, word_type));
        }
        segments
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TextContext, ThreadLocalGlyphSource, ThreadLocalMeasurer, ThreadLocalTextSeam, with_text,
    };
    use alacer_core::style::{MeasureFn, TextSpec};
    use alacer_core::text_seam::TextSeam as _;
    use alacer_core::utf16;

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

    #[test]
    fn hit_test_round_trips_the_prefix_widths_it_inverts() {
        use alacer_core::text_seam::TextSeam as _;
        use alacer_core::utf16;

        let seam = ThreadLocalTextSeam;
        let text = "hello world";
        let total = utf16::len_utf16(text);

        // Clicking exactly on boundary i's prefix width must land on that
        // boundary: the hit-test inverts the same measurements the caret
        // paints from, so the mapping closes (ASCII — every boundary has a
        // distinct advance).
        for i in 0..=total {
            let at = utf16::byte_offset(text, i);
            let x = seam.layout_glyphs(&text[..at], 16.0, None).width;
            assert_eq!(
                seam.hit_test(text, 16.0, x),
                i,
                "the prefix width of boundary {i} maps back to {i}"
            );
        }

        // The clamped edges: before the run is the start, past it the end.
        assert_eq!(seam.hit_test(text, 16.0, -10.0), 0);
        assert_eq!(
            seam.hit_test(
                text,
                16.0,
                with_text(|ctx| ctx.layout_text(text, 16.0, None).width) + 500.0
            ),
            total
        );
    }

    // ---- The word contract (ADR-0019) -----------------------------------

    /// The engine contract every consumer of `word_segments` relies on:
    /// consecutive half-open segments cover the run exactly — first start
    /// 0, no gaps, no overlaps, last end the run's UTF-16 length — and
    /// every edge sits on a scalar-value boundary (surrogate pairs are
    /// never sliced, whatever the segmentation decides).
    fn assert_covers(text: &str, segments: &[alacer_core::text_seam::WordSegment]) {
        let mut units = 0_usize;
        for segment in segments {
            assert_eq!(
                segment.start, units,
                "no gap before {segment:?} in {text:?}"
            );
            assert!(
                segment.start < segment.end,
                "empty segment {segment:?} in {text:?}"
            );
            assert!(
                utf16::is_boundary(text, segment.start) && utf16::is_boundary(text, segment.end),
                "segment {segment:?} slices a scalar value in {text:?}"
            );
            units = segment.end;
        }
        assert_eq!(
            units,
            utf16::len_utf16(text),
            "the segments cover {text:?} exactly"
        );
    }

    #[test]
    fn word_segments_classify_and_cover_the_run() {
        let seam = ThreadLocalTextSeam;

        let hello = seam.word_segments("hello, world");
        assert_covers("hello, world", &hello);
        assert_eq!(
            hello,
            vec![
                seg(0, 5, true),  // hello
                seg(5, 6, false), // ","
                seg(6, 7, false), // " " — UAX #29 breaks between them
                seg(7, 12, true), // world
            ],
            "letters are words; the separator segments are not word-like"
        );

        let plain = seam.word_segments("hello world");
        assert_covers("hello world", &plain);
        assert_eq!(
            plain,
            vec![seg(0, 5, true), seg(5, 6, false), seg(6, 11, true)],
        );
    }

    #[test]
    fn apostrophized_words_stay_one_segment() {
        let seam = ThreadLocalTextSeam;
        let segments = seam.word_segments("don't stop");
        assert_covers("don't stop", &segments);
        assert_eq!(
            segments,
            vec![
                seg(0, 5, true),  // don't — the MidLetter apostrophe joins
                seg(5, 6, false), // space
                seg(6, 10, true), // stop
            ],
            "UAX #29 keeps the contraction whole"
        );
    }

    #[test]
    fn cjk_and_kana_segment_by_script_and_count_one_unit_each() {
        let seam = ThreadLocalTextSeam;
        // The ICU reference run (its doctest): "こんにちは世界" breaks
        // between the kana run and the ideographs — UTF-16 units 5 and 2.
        let segments = seam.word_segments("こんにちは世界");
        assert_covers("こんにちは世界", &segments);
        assert_eq!(
            segments,
            vec![seg(0, 5, true), seg(5, 7, true)],
            "each BMP ideograph is one UTF-16 unit; the script change breaks"
        );
    }

    #[test]
    fn surrogate_pairs_segment_on_scalar_boundaries() {
        let seam = ThreadLocalTextSeam;
        let text = "a\u{1F600}b";
        let segments = seam.word_segments(text);
        assert_covers(text, &segments);
        // Whatever classification the pictograph draws, its two UTF-16
        // units must never split: the coverage property above already pins
        // every edge to a scalar boundary.
    }

    #[test]
    fn dictionary_scripts_break_into_words() {
        let seam = ThreadLocalTextSeam;
        // Thai needs the complex-script dictionary (ICU's own doctest
        // run): the rule-only fallback would keep the run unbroken.
        let segments = seam.word_segments("ทุกสองสัปดาห์");
        assert_covers("ทุกสองสัปดาห์", &segments);
        assert!(
            segments.len() >= 2,
            "the dictionary breaks Thai into words, got {segments:?}"
        );
        assert!(
            segments.iter().all(|s| s.word_like),
            "Thai letters are word-like"
        );
    }

    #[test]
    fn empty_and_separator_only_runs_segment_honestly() {
        let seam = ThreadLocalTextSeam;
        assert!(
            seam.word_segments("").is_empty(),
            "the empty run has no segments"
        );
        let spaces = seam.word_segments("   ");
        assert_covers("   ", &spaces);
        assert!(
            spaces.iter().all(|s| !s.word_like),
            "whitespace is never word-like"
        );
    }

    /// Shorthand for the expected-segment assertions.
    fn seg(start: usize, end: usize, word_like: bool) -> alacer_core::text_seam::WordSegment {
        alacer_core::text_seam::WordSegment {
            start,
            end,
            word_like,
        }
    }

    #[test]
    fn rasterized_glyphs_sit_upright_on_the_baseline() {
        use crate::raster::GlyphRasterizer;
        let mut ctx = TextContext::new();
        let layout = ctx.layout_text("L", 32.0, None);
        let glyph = layout.glyphs.first().expect("L shaped").clone();
        let mut raster = GlyphRasterizer::new();
        let bitmap = raster
            .rasterize(glyph.font_id, glyph.glyph_id, glyph.size, &|tag| {
                ctx.registry().font_data(tag)
            })
            .expect("L rasterizes");
        // An L has no descender: its ink must end exactly at the baseline
        // (the y-down contract: `top` is the bitmap's top row offset).
        assert_eq!(
            bitmap.top + i32::try_from(bitmap.height).expect("raster extents fit i32"),
            0,
            "the L's last row sits on the baseline"
        );
        // Orientation: the L's top row is the bare stem, its bottom row the
        // full-width foot — a vertically flipped buffer inverts the counts.
        let width = bitmap.width as usize;
        let ink = |row: usize| {
            bitmap.coverage[row * width..(row + 1) * width]
                .iter()
                .filter(|coverage| **coverage > 128)
                .count()
        };
        let top_row = ink(0);
        let bottom_row = ink(bitmap.height as usize - 1);
        assert!(
            top_row * 3 < bottom_row,
            "top row ({top_row}) must be the narrow stem, bottom row ({bottom_row}) the foot"
        );
    }
}
