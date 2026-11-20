//! The headless screenshot seam: one scene snapshot → one PNG file.
//!
//! The launch-asset path the README's pixels ride: a mounted pipeline's
//! ticked [`SceneSnapshot`](alacer_core::scene::SceneSnapshot) renders
//! through the software rasterizer (the same stage the softbuffer present
//! path runs), encodes, and lands on disk — no window, no event loop, no
//! display server. The refusal vocabulary is the platform's: an
//! allocation, an encode, or a write that cannot happen returns a typed
//! [`ScreenshotError`] whose chain a consumer walks, never a guessed file.

use crate::software::SoftwareRenderer;
use alacer_core::color::Color;
use alacer_core::scene::{GlyphSource, SceneSnapshot};
use std::path::Path;

/// The screenshot save path's refusals. Each names its stage: the raster
/// the frame could not get, the encode the pixels could not survive, the
/// write the file could not take.
#[derive(Debug)]
pub enum ScreenshotError {
    /// The rasterizer could not allocate the viewport pixmap (a 0×0 or
    /// oversized viewport, or OOM): no pixels exist, so no file is
    /// written. A fact about the request, not a wrapped failure — the
    /// chain ends here.
    Raster,
    /// The PNG encoder refused the rendered frame. The encoder's error
    /// type is not part of tiny-skia's public surface, so this variant is
    /// an honest leaf — the stage names itself, the cause stays inside
    /// the encoder that produced it.
    Encode,
    /// The file could not be written; the io error rides the chain.
    Io(std::io::Error),
}

impl std::fmt::Display for ScreenshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Raster => write!(f, "screenshot: the viewport pixmap could not be allocated"),
            Self::Encode => write!(f, "screenshot: the PNG encoder refused the frame"),
            Self::Io(error) => write!(f, "screenshot: the file could not be written: {error}"),
        }
    }
}

impl std::error::Error for ScreenshotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Raster | Self::Encode => None,
        }
    }
}

/// Renders `scene` into a `width × height` PNG over `background` and saves
/// it to `path` — the full headless save path (rasterize → encode → write)
/// in one call, the software tier's own stage order. Returns the written
/// file's byte count, the number a caller's size gate asserts.
///
/// # Errors
///
/// [`ScreenshotError::Raster`] when the pixmap cannot be allocated,
/// [`ScreenshotError::Encode`] when the encoder refuses, and
/// [`ScreenshotError::Io`] when the write fails (the io error on the chain).
pub fn save_png(
    scene: &SceneSnapshot,
    width: u32,
    height: u32,
    background: Color,
    glyphs: &mut dyn GlyphSource,
    path: &Path,
) -> Result<usize, ScreenshotError> {
    let mut renderer = SoftwareRenderer::new();
    let pixmap = renderer
        .render(scene, width, height, background, glyphs)
        .ok_or(ScreenshotError::Raster)?;
    // The encoder's error type is not nameable through tiny-skia's public
    // surface (it is not re-exported), so the refusal drops the payload
    // and names the stage — the `Encode` leaf's own documented honesty.
    let bytes = pixmap.encode_png().map_err(|_| ScreenshotError::Encode)?;
    std::fs::write(path, &bytes).map_err(ScreenshotError::Io)?;
    Ok(bytes.len())
}

#[cfg(test)]
mod tests {
    use super::{ScreenshotError, save_png};
    use crate::journal::SceneJournalImpl;
    use crate::software::ClosureGlyphSource;
    use alacer_core::arena::NodeId;
    use alacer_core::color::Color;
    use alacer_core::geom::{Point, Rect, Size};
    use alacer_core::passes::SceneJournal as _;
    use alacer_core::scene::{Image, ImageSampling, Primitive, Quad, SceneSnapshot};
    use std::path::PathBuf;
    use std::sync::Arc;

    const VIEWPORT: (u32, u32) = (64, 48);

    /// The scratch directory the test's files land in, named per process so
    /// parallel test binaries never share a path.
    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("alacer-screenshot-tests-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("the scratch directory creates");
        dir.join(name)
    }

    /// An 8×8 premultiplied RGBA8 bitmap: the top half red, the bottom half
    /// blue — the byte contract the image primitive spells, built in code
    /// so the expected pixels are known without a fixture file.
    fn red_blue_bitmap() -> Arc<[u8]> {
        let mut bytes = Vec::with_capacity(8 * 8 * 4);
        for _ in 0..4 {
            for _ in 0..8 {
                bytes.extend([255, 0, 0, 255]);
            }
        }
        for _ in 0..4 {
            for _ in 0..8 {
                bytes.extend([0, 0, 255, 255]);
            }
        }
        bytes.into()
    }

    /// The test scene: a white background, a black quad over its top-left
    /// corner, and the red/blue raster blitted at 2× into the right half —
    /// every expected pixel a fact the test spells itself.
    fn scene() -> SceneSnapshot {
        let mut journal = SceneJournalImpl::new();
        let node = NodeId::allocate();
        journal.commit_layer(
            node,
            vec![
                Primitive::Quad(Quad::filled(
                    Rect::new(Point::new(0.0, 0.0), Size::new(16.0, 16.0)),
                    Color::BLACK,
                )),
                Primitive::Image(Image {
                    bytes: red_blue_bitmap(),
                    width: 8,
                    height: 8,
                    src: Rect::new(Point::new(0.0, 0.0), Size::new(8.0, 8.0)),
                    dst: Rect::new(Point::new(32.0, 8.0), Size::new(16.0, 16.0)),
                    clip: None,
                    sampling: ImageSampling::Nearest,
                }),
            ],
        );
        journal.snapshot(&[node])
    }

    fn save_to(scene: &SceneSnapshot, name: &str) -> (PathBuf, usize) {
        let path = scratch(name);
        let mut no_glyphs = |_font: u64,
                             _glyph: u32,
                             _size: f32|
         -> Option<alacer_core::scene::GlyphBitmap> { None };
        let written = save_png(
            scene,
            VIEWPORT.0,
            VIEWPORT.1,
            Color::WHITE,
            &mut ClosureGlyphSource::new(&mut no_glyphs),
            &path,
        )
        .expect("the scene saves");
        (path, written)
    }

    #[test]
    fn the_saved_file_is_a_valid_png_of_the_viewport() {
        let scene = scene();
        let (path, written) = save_to(&scene, "valid.png");
        let bytes = std::fs::read(&path).expect("the saved file reads back");
        assert_eq!(written, bytes.len(), "the returned count is the file's own");
        assert!(written > 0, "a real frame is not an empty file");
        // The PNG signature, then the IHDR's dimensions at the documented
        // offsets: width at 16, height at 20, both big-endian.
        assert_eq!(
            &bytes[..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
            "the file opens with the PNG signature"
        );
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        assert_eq!((width, height), VIEWPORT, "the IHDR names the viewport");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn the_png_decodes_to_the_scene_pixels() {
        let scene = scene();
        let (path, _) = save_to(&scene, "pixels.png");
        let bytes = std::fs::read(&path).expect("the saved file reads back");
        let pixmap = tiny_skia::Pixmap::decode_png(&bytes).expect("the PNG decodes");
        assert_eq!((pixmap.width(), pixmap.height()), VIEWPORT);
        let pixel = |x: u32, y: u32| {
            let p = pixmap.pixel(x, y).expect("inside the pixmap");
            (p.red(), p.green(), p.blue(), p.alpha())
        };
        assert_eq!(pixel(1, 1), (0, 0, 0, 255), "the quad covers its corner");
        assert_eq!(pixel(24, 40), (255, 255, 255, 255), "the background stands");
        // The blit landed at 2×: the source's top half (red) maps to the
        // destination's top half, the source's bottom half (blue) below it.
        assert_eq!(pixel(40, 12), (255, 0, 0, 255), "the red band blitted");
        assert_eq!(pixel(40, 20), (0, 0, 255, 255), "the blue band blitted");
        assert_eq!(pixel(36, 8), (255, 0, 0, 255), "the scaled edge samples");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn the_render_is_deterministic_across_saves() {
        let scene = scene();
        let (first, _) = save_to(&scene, "first.png");
        let (second, _) = save_to(&scene, "second.png");
        let a = std::fs::read(&first).expect("the first save reads back");
        let b = std::fs::read(&second).expect("the second save reads back");
        assert_eq!(
            a, b,
            "the same scene encodes to the same bytes — the known hash"
        );
        let _ = std::fs::remove_file(&first);
        let _ = std::fs::remove_file(&second);
    }

    #[test]
    fn the_refusals_are_typed() {
        let scene = scene();
        let mut no_glyphs = |_font: u64,
                             _glyph: u32,
                             _size: f32|
         -> Option<alacer_core::scene::GlyphBitmap> { None };
        // A zero-width viewport cannot allocate a pixmap: the raster
        // refusal is the seam's own variant, not a panic.
        let refused = save_png(
            &scene,
            0,
            VIEWPORT.1,
            Color::WHITE,
            &mut ClosureGlyphSource::new(&mut no_glyphs),
            &scratch("raster.png"),
        )
        .expect_err("0×0 refuses");
        assert!(
            matches!(refused, ScreenshotError::Raster),
            "the raster refusal names its stage: {refused}"
        );
        // A path that is a directory cannot hold a file: the write
        // refusal carries the io error on its chain.
        let dir = scratch("a-directory");
        std::fs::create_dir_all(&dir).expect("the directory creates");
        let refused = save_png(
            &scene,
            VIEWPORT.0,
            VIEWPORT.1,
            Color::WHITE,
            &mut ClosureGlyphSource::new(&mut no_glyphs),
            &dir,
        )
        .expect_err("a directory path refuses");
        assert!(
            matches!(refused, ScreenshotError::Io(_)),
            "the io refusal: {refused}"
        );
        assert!(
            std::error::Error::source(&refused).is_some(),
            "the io error rides the chain"
        );
        let _ = std::fs::remove_dir(&dir);
    }
}
