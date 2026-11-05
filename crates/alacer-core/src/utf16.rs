//! UTF-16 index arithmetic: the IME contract's unit of measure (§10.3).
//!
//! GPUI's `EntityInputHandler` semantics — adopted as the framework
//! contract — express every caret offset, selection range, and IME cursor
//! as **UTF-16 code units**, because that is what the OS IME protocols
//! (and Windows' Korean preedit quirk, `egui-winit/src/lib.rs:741-763`
//! @ `fc9d66d`) count in. Rust strings are UTF-8, so every seam between
//! the two encodings funnels through this module: offsets are *floored*
//! to the nearest scalar-value boundary before use, never silently
//! misinterpreted mid-surrogate.

/// The length of `s` in UTF-16 code units.
#[must_use]
pub fn len_utf16(s: &str) -> usize {
    s.chars().map(char::len_utf16).sum()
}

/// Whether `utf16_offset` sits on a scalar-value boundary of `s`. Strict:
/// offsets past the end are *not* boundaries — the read path that tolerates
/// them is [`floor_boundary`]/[`byte_offset`], which clamp to the end.
#[must_use]
pub fn is_boundary(s: &str, utf16_offset: usize) -> bool {
    if utf16_offset == 0 {
        return true;
    }
    let mut units = 0_usize;
    for ch in s.chars() {
        if units == utf16_offset {
            return true;
        }
        units += ch.len_utf16();
        if units > utf16_offset {
            return false;
        }
    }
    units == utf16_offset
}

/// Floors `utf16_offset` to the nearest scalar-value boundary at or below
/// it — the forgiving read side of the contract (surrogate-pair middle is
/// clamped left, never an out-of-range index). Offsets beyond the end clamp
/// to the end.
#[must_use]
pub fn floor_boundary(s: &str, utf16_offset: usize) -> usize {
    let mut units = 0_usize;
    for ch in s.chars() {
        if units >= utf16_offset {
            break;
        }
        if units + ch.len_utf16() > utf16_offset {
            break; // inside this scalar value: floor to its start
        }
        units += ch.len_utf16();
    }
    units
}

/// Converts a UTF-16 offset into a UTF-8 byte offset, flooring to the
/// nearest scalar-value boundary — an offset that lands inside a surrogate
/// pair floors to the pair's *start*, never slices mid-scalar.
#[must_use]
pub fn byte_offset(s: &str, utf16_offset: usize) -> usize {
    let mut units = 0_usize;
    for (index, ch) in s.char_indices() {
        if units >= utf16_offset {
            return index;
        }
        if units + ch.len_utf16() > utf16_offset {
            return index; // inside this scalar value: floor to its start
        }
        units += ch.len_utf16();
    }
    s.len()
}

/// Converts a UTF-8 byte offset into a UTF-16 offset.
///
/// # Panics
///
/// Panics when `byte_offset` is not a `char` boundary — a caller bug, not
/// a runtime condition.
#[must_use]
pub fn utf16_offset(s: &str, byte_offset: usize) -> usize {
    assert!(
        s.is_char_boundary(byte_offset),
        "utf16_offset: {byte_offset} is not a char boundary"
    );
    s[..byte_offset].chars().map(char::len_utf16).sum()
}

/// The UTF-8 byte range for a UTF-16 range `[start, end)`, both floored to
/// scalar-value boundaries. Inverted or empty-at-mid-scalar ranges collapse
/// to the leftmost floored position (a zero-length range, never a panic).
#[must_use]
pub fn byte_range(s: &str, start_utf16: usize, end_utf16: usize) -> std::ops::Range<usize> {
    let start = byte_offset(s, start_utf16);
    let end_floor = floor_boundary(s, end_utf16);
    let start_units = utf16_offset(s, start);
    if end_floor <= start_units {
        // Empty or inverted: collapse at the leftmost floored position.
        let at = byte_offset(s, end_floor).min(start);
        return at..at;
    }
    start..byte_offset(s, end_floor)
}

/// Splits `s` at the caret (UTF-16 offset) into `(before, after)` byte slices.
#[must_use]
pub fn split_at(s: &str, utf16_caret: usize) -> (&str, &str) {
    let at = byte_offset(s, utf16_caret);
    s.split_at(at)
}

#[cfg(test)]
mod tests {
    use super::{
        byte_offset, byte_range, floor_boundary, is_boundary, len_utf16, split_at, utf16_offset,
    };

    #[test]
    fn ascii_offsets_are_identity() {
        assert_eq!(len_utf16("hello"), 5);
        assert_eq!(byte_offset("hello", 3), 3);
        assert_eq!(utf16_offset("hello", 3), 3);
    }

    #[test]
    fn cjk_counts_one_unit_per_char() {
        // CJK ideographs are BMP: one UTF-16 code unit each.
        let s = "にほんご";
        assert_eq!(len_utf16(s), 4);
        assert_eq!(byte_offset(s, 2), 6, "2 chars × 3 UTF-8 bytes");
        assert_eq!(utf16_offset(s, 6), 2);
        assert!(is_boundary(s, 2));
        assert!(!is_boundary(s, 5));
    }

    #[test]
    fn surrogate_pairs_count_two_units() {
        // U+1F600 GRINNING FACE: 4 UTF-8 bytes, 2 UTF-16 code units.
        let s = "\u{1F600}x";
        assert_eq!(len_utf16(s), 3);
        assert_eq!(byte_offset(s, 2), 4, "the pair is one scalar value");
        assert_eq!(byte_offset(s, 1), 0, "mid-surrogate floors to 0");
        assert_eq!(floor_boundary(s, 1), 0);
        assert!(is_boundary(s, 2));
        assert!(!is_boundary(s, 1));
    }

    #[test]
    fn ranges_floor_and_never_invert() {
        let s = "a\u{1F600}b";
        // "a" + pair + "b" = 4 UTF-16 units.
        assert_eq!(byte_range(s, 1, 3), 1..5, "the pair's byte span");
        assert_eq!(
            byte_range(s, 2, 2),
            1..1,
            "empty mid-surrogate range floors"
        );
        assert_eq!(byte_range(s, 3, 1), 1..1, "inverted ranges collapse");
        assert_eq!(byte_range(s, 0, 99), 0..s.len(), "out-of-range clamps");
    }

    #[test]
    fn split_and_floor_agree() {
        let s = "ab\u{1F600}cd";
        // Offset 4 is the true boundary after the pair (a=0, b=1, pair=2..3).
        let (before, after) = split_at(s, 4);
        assert_eq!(before, "ab\u{1F600}");
        assert_eq!(after, "cd");
        // Offsets 2 (pair start) and 3 (mid-pair) both floor left to the
        // pair's start — a caret never slices a surrogate pair.
        let (before, after) = split_at(s, 2);
        assert_eq!(before, "ab", "the pair's start boundary floors in place");
        assert_eq!(after, "\u{1F600}cd");
        let (before, after) = split_at(s, 3);
        assert_eq!(before, "ab", "mid-surrogate caret floors left");
        assert_eq!(after, "\u{1F600}cd");
    }

    #[test]
    fn offsets_beyond_the_end_clamp() {
        let s = "hi";
        assert_eq!(byte_offset(s, 100), 2);
        assert_eq!(floor_boundary(s, 100), 2);
        assert!(is_boundary(s, 2), "the end itself is always a boundary");
        assert!(
            !is_boundary(s, 100),
            "beyond the end is not a boundary — readers clamp"
        );
    }
}
