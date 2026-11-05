//! The workspace's single cast seam.
//!
//! Clippy pedantic denies lossy casts; pixel math still needs float→int
//! conversion at exactly one documented point (architecture §8.3: one rounding
//! point per axis, in device pixels). Every such conversion in the workspace
//! goes through these saturating helpers, each with a `#[expect]` + reason —
//! there are no other casts and no `allow` attributes anywhere.

/// Rounds and saturatingly casts to `u8` (coverage/color quantization).
#[must_use]
pub fn f32_to_u8(v: f32) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the single documented rounding point; saturating clamp bounds the result"
    )]
    {
        v.round().clamp(0.0, 255.0) as u8
    }
}

/// Rounds and saturatingly casts to `u32` (pixel extents).
#[must_use]
pub fn f32_to_u32(v: f32) -> u32 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the single documented rounding point; saturating clamp bounds the result"
    )]
    {
        v.round().clamp(0.0, 4_294_967_295.0) as u32
    }
}

/// Rounds and saturatingly casts to `i32` (glyph origins, sub-pixel offsets).
#[must_use]
pub fn f32_to_i32(v: f32) -> i32 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the single documented rounding point; saturating clamp bounds the result"
    )]
    {
        v.round().clamp(-2_147_483_648.0, 2_147_483_647.0) as i32
    }
}

/// Clamps and narrows `usize` extents into `u32` APIs with an explicit
/// guarantee (the one place the workspace does this).
#[must_use]
pub fn usize_to_u32(v: usize) -> u32 {
    v.min(u32::MAX as usize) as u32
}

/// Lossy-but-bounded widening to `f32` (pixel-count ratios; values below
/// 2^24 are exact).
#[must_use]
pub fn usize_to_f32(v: usize) -> f32 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "pixel counts far exceed f32 precision only past 2^24 pixels"
    )]
    {
        v as f32
    }
}

/// Widens to `f32` (device-pixel coordinates; exact below 2^24).
#[must_use]
pub fn i32_to_f32(v: i32) -> f32 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "device-pixel coordinates are exact far below the f32 mantissa limit"
    )]
    {
        v as f32
    }
}

/// Widens to `f32` (frame counters; exact below 2^24).
#[must_use]
pub fn u32_to_f32(v: u32) -> f32 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "counter values are exact far below the f32 mantissa limit"
    )]
    {
        v as f32
    }
}

/// Widens to `f32` (nanosecond timing ratios; exact below 2^24 ns).
#[must_use]
pub fn u64_to_f32(v: u64) -> f32 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "timing ratios compare magnitudes; f32 precision is ample past 2^24 ns"
    )]
    {
        v as f32
    }
}

/// Narrows a `u128` duration to `u64` nanoseconds (saturating; a frame longer
/// than 584 years is not a frame).
#[must_use]
pub fn u128_to_u64(v: u128) -> u64 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "saturating clamp bounds the result to u64::MAX before the cast"
    )]
    {
        v.min(u128::from(u64::MAX)) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::{f32_to_i32, f32_to_u8, f32_to_u32, i32_to_f32, usize_to_f32, usize_to_u32};

    #[test]
    fn u8_helpers_saturate_and_round() {
        assert_eq!(f32_to_u8(127.4), 127);
        assert_eq!(f32_to_u8(127.5), 128);
        assert_eq!(f32_to_u8(-3.0), 0);
        assert_eq!(f32_to_u8(900.0), 255);
    }

    #[test]
    fn u32_helpers_saturate_and_round() {
        assert_eq!(f32_to_u32(10.49), 10);
        assert_eq!(f32_to_u32(-1.0), 0);
        assert_eq!(f32_to_u32(4_294_967_295.0), u32::MAX);
    }

    #[test]
    fn i32_helpers_round_toward_nearest() {
        assert_eq!(f32_to_i32(-1.5), -2);
        assert_eq!(f32_to_i32(1.5), 2);
        assert_eq!(f32_to_i32(f32::MIN), i32::MIN);
    }

    #[test]
    fn usize_helpers_convert() {
        assert_eq!(usize_to_u32(42), 42);
        assert_eq!(usize_to_u32(usize::MAX), u32::MAX);
        assert_eq!(usize_to_f32(1024), 1024.0);
        assert_eq!(i32_to_f32(-7), -7.0);
    }
}
