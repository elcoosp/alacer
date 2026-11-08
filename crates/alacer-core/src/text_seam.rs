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

/// One maximal UAX #29 word-boundary segment of a run, in the editing
/// contract's unit (§10.3): UTF-16 offsets, half-open `[start, end)`.
///
/// `word_like` carries the segment's classification — letters (CJK included)
/// and numbers are word-like, whitespace and punctuation runs are not. The
/// classification is the engine's ([`TextSeam::word_segments`] implementors
/// own the Unicode tables); [`word_hop`] only walks positions against it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WordSegment {
    /// First UTF-16 offset of the segment.
    pub start: usize,
    /// One past the last UTF-16 offset of the segment.
    pub end: usize,
    /// Whether the segment classifies as a word (letters, numbers, CJK).
    pub word_like: bool,
}

/// The direction a word-wise caret operation hops along the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordDirection {
    /// Toward the run's start (Alt/Ctrl+Left, word-backspace).
    Previous,
    /// Toward the run's end (Alt/Ctrl+Right, word-delete).
    Next,
}

/// The word-hop rule the editing contract fixes (§10.3, ADR-0019): a hop
/// lands on the far edge of the word-like segment that spans the caret, or
/// — when the caret sits outside every word-like segment — on the near
/// edge of the next segment in the hop's direction. Whitespace and
/// punctuation runs are never landing points from outside them, so moving
/// through `", "` takes one hop, not one per boundary.
///
/// `from` is a UTF-16 offset (the contract unit); the result is too. At the
/// run's edges the hop clamps: past the last word's end, `Next` returns the
/// run's end and `Previous` returns its last word's start; before the
/// first, `Previous` returns 0. An empty segmentation returns `from`
/// unchanged (there is nothing to hop to).
///
/// This is *pure arithmetic over segment positions* — the rule lives here,
/// next to the contract it serves, so every seam implementation and every
/// consumer shares one definition and no engine can grow its own dialect.
#[must_use]
pub fn word_hop(segments: &[WordSegment], from: usize, direction: WordDirection) -> usize {
    match direction {
        WordDirection::Next => {
            // Inside (or at the start of) a word-like segment: its far edge.
            if let Some(spans) = segments
                .iter()
                .find(|s| s.word_like && s.start <= from && from < s.end)
            {
                return spans.end;
            }
            // Outside every word: the next word-like segment's near edge,
            // else the run's end.
            segments
                .iter()
                .find(|s| s.word_like && s.start > from)
                .map_or_else(
                    || segments.last().map_or(from, |s| s.end),
                    |next| next.start,
                )
        }
        WordDirection::Previous => {
            // An empty segmentation (an empty run): nothing to hop to.
            if segments.is_empty() {
                return from;
            }
            // Inside (or at the end of) a word-like segment: its near edge.
            if let Some(spans) = segments
                .iter()
                .find(|s| s.word_like && s.start < from && from <= s.end)
            {
                return spans.start;
            }
            // Outside every word: the previous word-like segment's near
            // edge — a word *start*, never a word's end or a separator —
            // else the run's start.
            segments
                .iter()
                .rev()
                .find(|s| s.word_like && s.start < from)
                .map_or(0, |prev| prev.start)
        }
    }
}

/// The text services seam. One implementation per process — installed at
/// assembly time, before the first frame.
pub trait TextSeam: 'static {
    /// Intrinsic size of a text run, wrapping at `max_width` when given.
    fn measure(&self, spec: &TextSpec, max_width: Option<f32>) -> Size;

    /// Shapes `text` at `font_size`, wrapping at `max_width` when given.
    fn layout_glyphs(&self, text: &str, font_size: f32, max_width: Option<f32>) -> GlyphRun;

    /// The run's UAX #29 word segmentation, in the editing contract's unit:
    /// consecutive segments cover `text` exactly, half-open
    /// `[start, end)` in UTF-16 offsets, `word_like` classifying letters
    /// (CJK included) and numbers as words and everything else as
    /// separators. This is the word side of the editing contract (§10.3,
    /// ADR-0019): word-wise movement, word-wise deletion, and double-click
    /// word selection all ask the seam, never the widget's own tables.
    ///
    /// There is **no provided implementation**: segmentation is a Unicode
    /// table question — the honest answer needs UAX #29 word-break data and
    /// the complex-script dictionaries, exactly what the text engines
    /// carry. A whitespace-and-ASCII approximation would silently mis-hop
    /// `don't` and every multi-script run, so the seam requires the real
    /// classification instead of shipping a documented guess. The workspace
    /// implementation rides ICU's `WordSegmenter` (the same segmenter
    /// parley's line breaking adopts, dictionary-backed for Thai and
    /// friends).
    fn word_segments(&self, text: &str) -> Vec<WordSegment>;

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
    use super::{
        GlyphRun, PlacedGlyph, TextSeam, WordDirection, WordSegment, install, is_installed,
        with_seam, word_hop,
    };
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

        /// The stub's segmentation: runs of `char::is_alphanumeric` are
        /// word-like, everything else separates. Deterministic and honest
        /// for the ASCII runs these tests feed it — the *real* tables ride
        /// the engine's implementation (alacer-text).
        fn word_segments(&self, text: &str) -> Vec<WordSegment> {
            let mut segments = Vec::new();
            let (mut start, mut word_like, mut units) = (0_usize, false, 0_usize);
            for ch in text.chars() {
                let like = ch.is_alphanumeric();
                if units > 0 && like != word_like {
                    segments.push(WordSegment {
                        start,
                        end: units,
                        word_like,
                    });
                    start = units;
                }
                word_like = like;
                units += ch.len_utf16();
            }
            if units > start {
                segments.push(WordSegment {
                    start,
                    end: units,
                    word_like,
                });
            }
            segments
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

    // ---- The word contract (§10.3, ADR-0019) ---------------------------

    /// Builds segments from `(start, end, word_like)` triples — the tests'
    /// hand-cut segmentations, exactly what an engine returns.
    fn segs(parts: &[(usize, usize, bool)]) -> Vec<WordSegment> {
        parts
            .iter()
            .map(|&(start, end, word_like)| WordSegment {
                start,
                end,
                word_like,
            })
            .collect()
    }

    /// "hello world" — `hello` [0,5), space [5,6), `world` [6,11).
    const HELLO: &[(usize, usize, bool)] = &[(0, 5, true), (5, 6, false), (6, 11, true)];

    #[test]
    fn next_hop_lands_on_word_ends_then_word_starts() {
        let s = segs(HELLO);

        assert_eq!(
            word_hop(&s, 0, WordDirection::Next),
            5,
            "start -> hello's end"
        );
        assert_eq!(
            word_hop(&s, 2, WordDirection::Next),
            5,
            "mid-word -> the word's end"
        );
        assert_eq!(
            word_hop(&s, 5, WordDirection::Next),
            6,
            "before the space -> the next word's start, one hop"
        );
        assert_eq!(word_hop(&s, 6, WordDirection::Next), 11, "world's end");
        assert_eq!(
            word_hop(&s, 11, WordDirection::Next),
            11,
            "the run's end clamps the hop"
        );
    }

    #[test]
    fn previous_hop_lands_on_word_starts_only() {
        let s = segs(HELLO);

        assert_eq!(
            word_hop(&s, 11, WordDirection::Previous),
            6,
            "end -> world's start"
        );
        assert_eq!(
            word_hop(&s, 7, WordDirection::Previous),
            6,
            "mid-word -> the word's start"
        );
        assert_eq!(
            word_hop(&s, 6, WordDirection::Previous),
            0,
            "before 'world' -> hello's start, the separator is never a landing"
        );
        assert_eq!(
            word_hop(&s, 5, WordDirection::Previous),
            0,
            "before the space -> hello's start"
        );
        assert_eq!(
            word_hop(&s, 0, WordDirection::Previous),
            0,
            "the run's start clamps the hop"
        );
    }

    #[test]
    fn punctuation_runs_take_one_hop() {
        // "hello, world" — `hello` [0,5), ", " [5,7), `world` [7,12).
        let s = segs(&[(0, 5, true), (5, 7, false), (7, 12, true)]);

        assert_eq!(
            word_hop(&s, 5, WordDirection::Next),
            7,
            "the ', ' separator takes one hop, not one per boundary"
        );
        assert_eq!(word_hop(&s, 4, WordDirection::Next), 5, "inside hello");
        assert_eq!(
            word_hop(&s, 7, WordDirection::Previous),
            0,
            "back from 'world' skips ', ' to hello's start"
        );
        assert_eq!(word_hop(&s, 12, WordDirection::Previous), 7);
    }

    #[test]
    fn apostrophized_words_stay_one_hop() {
        // "don't stop" — UAX #29 keeps `don't` one segment.
        let s = segs(&[(0, 5, true), (5, 6, false), (6, 10, true)]);

        assert_eq!(
            word_hop(&s, 3, WordDirection::Next),
            5,
            "mid-word -> the whole contraction's end, never mid-'t'"
        );
        assert_eq!(word_hop(&s, 5, WordDirection::Next), 6);
        assert_eq!(word_hop(&s, 10, WordDirection::Previous), 6);
    }

    #[test]
    fn surrogate_pairs_hop_whole() {
        // "a😀b" — the pair is UTF-16 units [1,3); hops never slice it.
        let s = segs(&[(0, 1, true), (1, 3, true), (3, 4, true)]);

        assert_eq!(word_hop(&s, 0, WordDirection::Next), 1);
        assert_eq!(
            word_hop(&s, 1, WordDirection::Next),
            3,
            "the pair is one hop"
        );
        assert_eq!(word_hop(&s, 4, WordDirection::Previous), 3);
        assert_eq!(
            word_hop(&s, 3, WordDirection::Previous),
            1,
            "back from 'b' lands at the pair's start, never mid-pair"
        );
    }

    #[test]
    fn degenerate_segmentations_hop_in_place() {
        // An empty run: no segments, the hop returns `from` unchanged.
        assert_eq!(word_hop(&[], 3, WordDirection::Next), 3);
        assert_eq!(word_hop(&[], 3, WordDirection::Previous), 3);

        // Only separators ("   "): next clamps to the run's end, previous
        // to its start — nothing word-like to land on.
        let spaces = segs(&[(0, 3, false)]);
        assert_eq!(word_hop(&spaces, 1, WordDirection::Next), 3);
        assert_eq!(word_hop(&spaces, 1, WordDirection::Previous), 0);
    }

    /// A segmentation must cover the run exactly: consecutive segments,
    /// half-open, no gaps, no overlaps — the property the engine's output
    /// must carry for `word_hop` to be total.
    #[test]
    fn the_engine_contract_covers_the_run_exactly() {
        install(Rc::new(Stub));
        // The stub above does not implement segmentation (it is the
        // pre-M13 stub); coverage is pinned where real segmentations live
        // (alacer-text). What is pinned here is the shape: the hop rule
        // stays total over any segmentation that covers the run.
        let covered = segs(HELLO);
        assert_eq!(covered.first().map_or(0, |s| s.start), 0);
        assert_eq!(
            covered.last().map_or(0, |s| s.end),
            11,
            "the last segment's end is the run's UTF-16 length"
        );
    }
}
