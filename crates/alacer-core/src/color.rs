//! Colors: plain 8-bit RGBA, converted to renderer-native formats at the edges.

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
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn with_alpha_preserves_channels() {
        let c = Color::rgb(10, 20, 30).with_alpha(99);
        assert_eq!(c, Color::new(10, 20, 30, 99));
    }
}
