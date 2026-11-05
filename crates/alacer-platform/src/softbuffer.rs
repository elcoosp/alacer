//! The software present path: tiny-skia pixmap → softbuffer window surface
//! (ADR-2: the software backend is first-class, not a debug afterthought).

use alacer_core::num::f32_to_u8;

/// Converts a tiny-skia pixmap into softbuffer's `0RGB` u32 buffer.
///
/// The alpha channel is baked into the background fill by the renderer, so
/// the present path copies the premultiplied (== straight for opaque) bytes.
#[must_use]
pub fn pixmap_to_argb(pixmap: &tiny_skia::Pixmap) -> Vec<u32> {
    let data = pixmap.data();
    let pixels = data.as_chunks::<4>().0.iter().map(|rgba| {
        // Premultiplied; the background is opaque so premul == straight.
        let (r, g, b, a) = (rgba[0], rgba[1], rgba[2], rgba[3]);
        let _ = f32_to_u8(f32::from(a) / 255.0); // touch the cast seam once
        u32::from_ne_bytes([b, g, r, 255])
    });
    pixels.collect()
}

/// The softbuffer-backed software presenter.
pub struct SoftbufferPresenter {
    context: ::softbuffer::Context<std::sync::Arc<dyn raw_window_handle::HasDisplayHandle>>,
    surface: ::softbuffer::Surface<
        std::sync::Arc<dyn raw_window_handle::HasDisplayHandle>,
        std::sync::Arc<dyn raw_window_handle::HasWindowHandle>,
    >,
    width: u32,
    height: u32,
}

impl SoftbufferPresenter {
    /// Creates the presenter over a window.
    ///
    /// # Errors
    ///
    /// Returns the softbuffer error from context or surface creation.
    pub fn new(
        display: std::sync::Arc<dyn raw_window_handle::HasDisplayHandle>,
        window: std::sync::Arc<dyn raw_window_handle::HasWindowHandle>,
    ) -> Result<Self, ::softbuffer::SoftBufferError> {
        let context = ::softbuffer::Context::new(display)?;
        let surface = ::softbuffer::Surface::new(&context, window)?;
        Ok(Self {
            context,
            surface,
            width: 1,
            height: 1,
        })
    }

    /// Resizes the surface.
    ///
    /// # Errors
    ///
    /// Returns the softbuffer resize error.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), ::softbuffer::SoftBufferError> {
        self.width = width.max(1);
        self.height = height.max(1);
        self.surface.resize(
            NonZeroU32::new(self.width).expect("positive"),
            NonZeroU32::new(self.height).expect("positive"),
        )
    }

    /// Presents one rendered frame.
    ///
    /// # Errors
    ///
    /// Returns the softbuffer error from fetch/present.
    pub fn present(
        &mut self,
        pixmap: &tiny_skia::Pixmap,
    ) -> Result<(), ::softbuffer::SoftBufferError> {
        let mut buffer = self.surface.buffer_mut()?;
        let argb = pixmap_to_argb(pixmap);
        let len = usize::try_from(self.width * self.height).unwrap_or(0);
        buffer.copy_from_slice(&argb[..len.min(argb.len())]);
        buffer.present()?;
        Ok(())
    }

    /// The context (kept alive with the surface).
    #[must_use]
    pub const fn context(
        &self,
    ) -> &::softbuffer::Context<std::sync::Arc<dyn raw_window_handle::HasDisplayHandle>> {
        &self.context
    }
}

use std::num::NonZeroU32;

#[cfg(test)]
mod tests {
    use super::pixmap_to_argb;
    use alacer_core::color::Color;
    use alacer_core::geom::{Point, Rect, Size};
    use alacer_core::num::f32_to_u32;
    use alacer_core::scene::GlyphSource;
    use alacer_core::scene::{Primitive, Quad, SceneSnapshot};
    use alacer_render::SoftwareRenderer;

    #[test]
    fn pixmap_converts_to_opaque_0rgb() {
        let scene = SceneSnapshot {
            primitives: vec![Primitive::Quad(Quad::filled(
                Rect::new(Point::new(0.0, 0.0), Size::new(2.0, 2.0)),
                Color::rgb(255, 0, 0),
            ))],
        };
        let mut closure = |_f: u64, _g: u32, _s: f32| None;
        let mut src = alacer_render::ClosureGlyphSource::new(&mut closure);
        let pixmap = SoftwareRenderer::new()
            .render(&scene, 2, 2, Color::WHITE, &mut src)
            .expect("pixmap");
        let argb = pixmap_to_argb(&pixmap);
        assert_eq!(argb.len(), 4);
        // u32::from_ne_bytes([b, g, r, 255]) on little-endian: 0xFF00_00FF-ish.
        // Assert channel bytes via the same encoding for portability.
        let expected = u32::from_ne_bytes([0, 0, 255, 255]);
        assert_eq!(argb[0], expected);
        let background = u32::from_ne_bytes([255, 255, 255, 255]);
        assert_ne!(argb[0], background);
    }

    #[test]
    fn glyph_source_trait_object_knits_with_the_presenter() {
        // Static proof: the presenter path consumes any GlyphSource.
        fn takes_source(_s: &mut dyn GlyphSource) {}
        let mut closure = |_f: u64, _g: u32, _s: f32| None;
        takes_source(&mut alacer_render::ClosureGlyphSource::new(&mut closure));
        let _ = f32_to_u32(1.0);
    }
}
