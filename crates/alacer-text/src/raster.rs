//! Glyph rasterization: scaled outlines → coverage bitmaps, via zeno (swash's
//! own rasterizer). One rasterization path serves both backends (A8).
//!
//! The GPU backend uploads the same coverage bitmaps into its atlas; the
//! software backend blends them directly. There is no second rasterizer.

use alacer_core::scene::GlyphBitmap;
use std::sync::Arc;
use swash::scale::{ScaleContext, Scaler};
use swash::{FontRef, GlyphId};

/// Cached placement + coverage handle per raster key.
type RasterCache = std::collections::HashMap<RasterKey, Option<(i32, i32, Arc<[u8]>, u32, u32)>>;

/// The rasterizer: one `ScaleContext` reused across glyphs (the documented
/// swash pattern — contexts are expensive, scalers are cheap).
#[derive(Default)]
pub struct GlyphRasterizer {
    context: ScaleContext,
    cache: RasterCache,
    hits: u64,
    misses: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RasterKey {
    font_tag: u64,
    glyph_id: u32,
    size_bits: u32,
}

impl GlyphRasterizer {
    /// A fresh rasterizer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Rasterizes one glyph at `size`, resolving font bytes through
    /// `resolve(font_id)`.
    ///
    /// Returns `None` for empty outlines (spaces) or unknown fonts.
    #[must_use]
    pub fn rasterize(
        &mut self,
        font_tag: u64,
        glyph_id: u32,
        size: f32,
        resolve: &dyn Fn(u64) -> Option<(Arc<[u8]>, u32)>,
    ) -> Option<GlyphBitmap> {
        let key = RasterKey {
            font_tag,
            glyph_id,
            size_bits: size.to_bits(),
        };
        if let Some(cached) = self.cache.get(&key) {
            self.hits += 1;
            return cached
                .as_ref()
                .map(|(left, top, coverage, w, h)| GlyphBitmap {
                    left: *left,
                    top: *top,
                    width: *w,
                    height: *h,
                    coverage: Arc::clone(coverage),
                });
        }
        self.misses += 1;

        let outcome = self.rasterize_uncached(font_tag, glyph_id, size, resolve);
        self.cache.insert(
            key,
            outcome
                .as_ref()
                .map(|b| (b.left, b.top, Arc::clone(&b.coverage), b.width, b.height)),
        );
        outcome
    }

    fn rasterize_uncached(
        &mut self,
        font_tag: u64,
        glyph_id: u32,
        size: f32,
        resolve: &dyn Fn(u64) -> Option<(Arc<[u8]>, u32)>,
    ) -> Option<GlyphBitmap> {
        let (data, index) = resolve(font_tag)?;
        let font_ref = FontRef::from_index(
            &data,
            usize::try_from(index).expect("font index fits usize"),
        )
        .expect("valid font index");
        let mut scaler: Scaler<'_> = self.context.builder(font_ref).size(size).hint(true).build();

        let glyph_id = u16::try_from(glyph_id).ok()?;
        let outline = scaler.scale_outline(GlyphId::from(glyph_id))?;
        if outline.is_empty() {
            return None;
        }

        let mask = zeno::Mask::new(outline.path());
        let (mut coverage, placement) = mask.render();
        // zeno reads the swash y-down outline as y-up: its buffer's row 0 is
        // the path's *lowest* point, and `placement.top` is that bottom edge's
        // offset from the baseline. The `GlyphBitmap` contract is row 0 =
        // visual top with `top` measured downward from the baseline — flip
        // the rows and re-derive the offset (empirically pinned by the
        // `rasterized_glyphs_sit_upright_on_the_baseline` test: an L's top
        // row is the narrow stem and `top + height == 0`).
        let rows = placement.height as usize;
        let cols = placement.width as usize;
        for row in 0..rows / 2 {
            let upper = row * cols;
            let lower = (rows - 1 - row) * cols;
            let (head, tail) = coverage.split_at_mut(lower);
            head[upper..upper + cols].swap_with_slice(&mut tail[..cols]);
        }
        let top = -(placement.top + i32::try_from(placement.height).unwrap_or(i32::MAX));
        let bitmap = GlyphBitmap {
            left: placement.left,
            top,
            width: placement.width,
            height: placement.height,
            coverage: Arc::from(coverage.into_boxed_slice()),
        };
        Some(bitmap)
    }

    /// Rasterization cache hits.
    #[must_use]
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Rasterization cache misses.
    #[must_use]
    pub fn misses(&self) -> u64 {
        self.misses
    }
}

#[cfg(test)]
mod tests {
    use super::GlyphRasterizer;

    // The rasterizer needs real font bytes; the harness integration tests
    // exercise it through the full pipeline. Here we pin the cache counters
    // with a resolution that always fails — proving the miss path and the
    // None propagation for unknown fonts.
    #[test]
    fn unknown_fonts_yield_none_and_count_misses() {
        let mut raster = GlyphRasterizer::new();
        let resolve = |_tag: u64| -> Option<(std::sync::Arc<[u8]>, u32)> { None };
        let first = raster.rasterize(1, 65, 16.0, &resolve);
        assert!(first.is_none());
        assert_eq!(raster.misses(), 1);

        let second = raster.rasterize(1, 65, 16.0, &resolve);
        assert!(second.is_none());
        assert_eq!(raster.hits(), 1, "negative results are cached too");
    }
}
