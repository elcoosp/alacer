//! The text seam: how widgets shape and measure text without depending on
//! any text engine (ADR-9 tightening, M1).
//!
//! Widgets live two layers below the text stack, yet paint glyphs and report
//! intrinsic sizes. Rather than dragging `alacer-text` into every widget's
//! dependency set, the assembly site installs one [`TextSeam`] implementation
//! and widgets call it through a thread-local indirection — the same
//! main-thread discipline the text context already imposes.
//!
//! ```text
//! // Assembly (facade/testing/platform) — alacer_text depends on this crate:
//! alacer_core::text_seam::install(std::rc::Rc::new(alacer_text::ThreadLocalTextSeam));
//!
//! // Widgets (core-only dependencies):
//! use alacer_core::text_seam::with_seam;
//! let run = with_seam(|seam| seam.layout_glyphs("Hello", 16.0, None));
//! ```

use crate::geom::Size;
use crate::style::TextSpec;
use std::cell::RefCell;
use std::rc::Rc;

/// One shaped glyph, positioned relative to the run's origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacedGlyph {
    /// X offset from the run origin, in pixels.
    pub x: f32,
    /// Y offset from the run origin, in pixels.
    pub y: f32,
    /// Font size this glyph was shaped at.
    pub size: f32,
    /// The registry id of the font (identity shared with the raster seam).
    pub font_id: u64,
    /// The font's glyph index.
    pub glyph_id: u32,
}

/// A shaped text run: its measured extent and its placed glyphs.
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRun {
    /// Run width in pixels.
    pub width: f32,
    /// Run height (line height) in pixels.
    pub height: f32,
    /// The shaped glyphs, in visual order.
    pub glyphs: Vec<PlacedGlyph>,
}

/// The text services seam. One implementation per process — installed at
/// assembly time, before the first frame.
pub trait TextSeam: 'static {
    /// Intrinsic size of a text run, wrapping at `max_width` when given.
    fn measure(&self, spec: &TextSpec, max_width: Option<f32>) -> Size;

    /// Shapes `text` at `font_size`, wrapping at `max_width` when given.
    fn layout_glyphs(&self, text: &str, font_size: f32, max_width: Option<f32>) -> GlyphRun;

    /// The UTF-16 offset whose boundary is nearest to pixel `x` within the
    /// run `text` shapes at `font_size` (unwrapped — the same shaping the
    /// caret and the selection highlight measure their prefix widths with,
    /// so the mapping closes against what paint draws). This is the
    /// pointer side of the editing contract (§10.3, ADR-0018): a click's
    /// x-coordinate becomes a caret offset through this seam.
    ///
    /// The provided implementation inverts the prefix widths
    /// [`TextSeam::layout_glyphs`] already measures: a binary search over
    /// the run's scalar boundaries, the nearest boundary wins, and ties
    /// break to the earlier boundary — a click inside a cluster never
    /// lands past it. Engines may override with a cluster-precise mapping
    /// (parley carries cluster indexes); the inversion stays honest for
    /// the single-line, left-to-right runs the contract covers because
    /// prefix advances are non-decreasing along them.
    fn hit_test(&self, text: &str, font_size: f32, x: f32) -> usize {
        if text.is_empty() || x <= 0.0 {
            return 0;
        }
        let total = crate::utf16::len_utf16(text);
        // The run's scalar boundaries in UTF-16 units, ascending: one per
        // scalar value, plus the end — every placement the caret legally
        // takes.
        let mut boundaries: Vec<usize> = Vec::with_capacity(total + 1);
        boundaries.push(0);
        let mut offset = 0_usize;
        for ch in text.chars() {
            offset += ch.len_utf16();
            boundaries.push(offset);
        }
        debug_assert_eq!(offset, total, "the boundary walk covers the run");
        let width_of = |boundary: usize| -> f32 {
            let at = crate::utf16::byte_offset(text, boundary);
            self.layout_glyphs(&text[..at], font_size, None).width
        };
        // The widths are non-decreasing along the boundaries: binary search
        // for the last boundary whose width is <= x, then take the nearer
        // of it and its successor; ties break to the earlier boundary.
        let (mut lo, mut hi) = (0_usize, boundaries.len() - 1);
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            if width_of(boundaries[mid]) <= x {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        let left = lo;
        let Some(&right) = boundaries.get(left + 1) else {
            // x is at or past the run's width: the caret parks at the end.
            return total;
        };
        let (left_width, right_width) = (width_of(boundaries[left]), width_of(right));
        if x - left_width > right_width - x {
            right
        } else {
            boundaries[left]
        }
    }
}

thread_local! {
    static SEAM: RefCell<Option<Rc<dyn TextSeam>>> = const { RefCell::new(None) };
}

/// Installs the thread's text seam. Re-installing replaces the previous one
/// (tests may swap implementations; production installs once).
pub fn install(seam: Rc<dyn TextSeam>) {
    SEAM.with(|cell| *cell.borrow_mut() = Some(seam));
}

/// Whether a seam is installed on this thread.
#[must_use]
pub fn is_installed() -> bool {
    SEAM.with_borrow(Option::is_some)
}

/// Runs `f` with the installed seam.
///
/// # Panics
///
/// Panics with an actionable message when no seam was installed: a widget
/// reached text before assembly did — a wiring bug, not a runtime condition.
pub fn with_seam<R>(f: impl FnOnce(&dyn TextSeam) -> R) -> R {
    SEAM.with_borrow(|seam| {
        let Some(seam) = seam.as_ref() else {
            panic!(
                "no text seam installed — call `alacer_core::text_seam::install` \
                 (e.g. with `alacer_text::ThreadLocalTextSeam`) before the first frame"
            );
        };
        f(seam.as_ref())
    })
}

#[cfg(test)]
mod tests {
    use super::{GlyphRun, PlacedGlyph, TextSeam, install, is_installed, with_seam};
    use crate::geom::Size;
    use crate::style::TextSpec;
    use std::rc::Rc;

    /// A seam stub recording calls (proves the indirection, no text engine).
    struct Stub;

    impl TextSeam for Stub {
        fn measure(&self, _spec: &TextSpec, _max_width: Option<f32>) -> Size {
            Size::new(7.0, 9.0)
        }

        fn layout_glyphs(&self, text: &str, _font_size: f32, _max_width: Option<f32>) -> GlyphRun {
            GlyphRun {
                width: f32::from(u16::try_from(text.len()).unwrap_or(u16::MAX)),
                height: 10.0,
                glyphs: vec![PlacedGlyph {
                    x: 0.0,
                    y: 8.0,
                    size: 16.0,
                    font_id: 0,
                    glyph_id: 1,
                }],
            }
        }
    }

    #[test]
    fn the_seam_routes_installed_implementations() {
        assert!(!is_installed(), "nothing installed on a fresh thread");
        install(Rc::new(Stub));

        let size = with_seam(|seam| {
            seam.measure(
                &TextSpec {
                    text: "hi".to_string(),
                    font_id: 0,
                    font_size: 16.0,
                },
                None,
            )
        });
        assert_eq!(size, Size::new(7.0, 9.0));

        let run = with_seam(|seam| seam.layout_glyphs("abc", 16.0, None));
        assert_eq!(run.glyphs.len(), 1);
        assert_eq!(run.glyphs[0].glyph_id, 1);
    }

    #[test]
    #[should_panic(expected = "no text seam installed")]
    fn an_uninstalled_seam_panics_with_direction() {
        // Fresh thread: this test owns its own thread-local state, so the
        // install from the previous test is invisible here.
        let _ = with_seam(|_| 0_u8);
    }

    /// The stub's widths are the text's byte lengths (1 px per byte), so
    /// the hit-test's boundaries are exactly the scalar offsets for ASCII.
    #[test]
    fn hit_test_returns_the_nearest_boundary() {
        install(Rc::new(Stub));

        let at = |x: f32| with_seam(|seam| seam.hit_test("abcd", 16.0, x));

        assert_eq!(at(0.0), 0, "the run's origin is the first boundary");
        assert_eq!(at(2.5), 2, "nearest boundary wins: 2 (width 2) over 3");
        assert_eq!(at(3.2), 3, "past the midpoint the next boundary wins");
        assert_eq!(at(-5.0), 0, "before the run clamps to the start");
        assert_eq!(at(99.0), 4, "past the run clamps to the end");
        assert_eq!(with_seam(|s| s.hit_test("", 16.0, 5.0)), 0, "empty is 0");
    }

    #[test]
    fn hit_test_breaks_ties_toward_the_cluster_start() {
        install(Rc::new(Stub));

        // x = 2.0 sits exactly on boundary 2; x = 2.5 is equidistant from
        // boundaries 2 and 3 — both resolve to the earlier boundary, so a
        // click inside a cluster never lands past it.
        assert_eq!(with_seam(|s| s.hit_test("abcd", 16.0, 2.0)), 2);
        assert_eq!(with_seam(|s| s.hit_test("abcd", 16.0, 2.5)), 2);
    }
}
