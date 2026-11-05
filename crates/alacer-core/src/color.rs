//! Colors: plain 8-bit RGBA, converted to renderer-native formats at the edges.

use crate::num::f32_to_u8;

/// An 8-bit-per-channel RGBA color.
///
/// Storage is non-premultiplied; renderers convert at the boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Color {
    /// Red channel, 0–255.
    pub r: u8,
    /// Green channel, 0–255.
    pub g: u8,
    /// Blue channel, 0–255.
    pub b: u8,
    /// Alpha channel, 0–255; 0 is fully transparent.
    pub a: u8,
}

impl Color {
    /// Builds a color from components.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Builds a fully opaque color.
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Fully opaque white.
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    /// Fully opaque black.
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    /// Fully transparent.
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);

    /// Returns this color with a new alpha channel.
    #[must_use]
    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// Normalized non-premultiplied RGBA, for GPU uniform upload.
    #[must_use]
    pub fn to_f32_rgba(self) -> [f32; 4] {
        [
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
            f32::from(self.a) / 255.0,
        ]
    }

    /// Premultiplied RGBA8 bytes, for software blending into a `0RGB` pixel buffer.
    #[must_use]
    pub fn to_premultiplied_rgba8(self) -> [u8; 4] {
        let a = f32::from(self.a) / 255.0;
        let r = f32::from(self.r) * a;
        let g = f32::from(self.g) * a;
        let b = f32::from(self.b) * a;
        [f32_to_u8(r), f32_to_u8(g), f32_to_u8(b), self.a]
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn to_f32_rgba_is_normalized() {
        let rgba = Color::rgb(255, 128, 0).to_f32_rgba();
        assert_eq!(rgba, [1.0, 128.0 / 255.0, 0.0, 1.0]);
    }

    #[test]
    fn premultiplication_scales_channels_by_alpha() {
        let premul = Color::rgb(255, 128, 0)
            .with_alpha(128)
            .to_premultiplied_rgba8();
        assert_eq!(premul[3], 128);
        // 255 * (128/255) rounded == 128; 128 * (128/255) rounded == 64.
        assert_eq!(premul[0], 128);
        assert_eq!(premul[1], 64);
        assert_eq!(premul[2], 0);
    }

    #[test]
    fn with_alpha_preserves_channels() {
        let c = Color::rgb(10, 20, 30).with_alpha(99);
        assert_eq!(c, Color::new(10, 20, 30, 99));
    }
}
