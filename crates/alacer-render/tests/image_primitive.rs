//! The raster image primitive's renderer contract (the raster tier, §9.1's
//! journal vocabulary growth): pixel-exact software rendering of a known
//! bitmap, clip + partial-repaint equivalence (Q8), damage extents, honest
//! refusal paths, and the journal partition routing the new kind.
//!
//! These tests ride the software backend — the A4 test rig — the same
//! discipline the quad and glyph contracts pin by.

use alacer_core::color::Color;
use alacer_core::damage::DamageBuilder;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::scene::{Image, ImageSampling, Primitive, SceneSnapshot, primitive_extent};
use std::sync::Arc;

/// A deterministic no-glyph source: image scenes carry no text.
struct NoGlyphs;

impl alacer_core::scene::GlyphSource for NoGlyphs {
    fn glyph_bitmap(
        &mut self,
        _font_id: u64,
        _glyph_id: u32,
        _size: f32,
    ) -> Option<alacer_core::scene::GlyphBitmap> {
        None
    }
}

/// A 4×2 premultiplied RGBA8 raster: the left half opaque red, the right
/// half opaque blue.
fn red_blue_raster() -> Image {
    let mut bytes = Vec::with_capacity(4 * 2 * 4);
    for _y in 0..2_u32 {
        for x in 0..4_u32 {
            if x < 2 {
                bytes.extend_from_slice(&[255, 0, 0, 255]);
            } else {
                bytes.extend_from_slice(&[0, 0, 255, 255]);
            }
        }
    }
    Image {
        bytes: Arc::from(bytes.into_boxed_slice()),
        width: 4,
        height: 2,
        src: Rect::new(Point::new(0.0, 0.0), Size::new(4.0, 2.0)),
        dst: Rect::new(Point::new(0.0, 0.0), Size::new(4.0, 2.0)),
        clip: None,
        sampling: ImageSampling::Nearest,
    }
}

#[test]
fn image_pixels_render_pixel_exact() {
    let mut image = red_blue_raster();
    image.dst = Rect::new(Point::new(2.0, 1.0), Size::new(4.0, 2.0));
    let scene = SceneSnapshot {
        primitives: vec![Primitive::Image(image)],
    };
    let mut glyphs = NoGlyphs;
    let pixmap = alacer_render::SoftwareRenderer::new()
        .render(&scene, 12, 8, Color::WHITE, &mut glyphs)
        .expect("pixmap");

    // Inside the destination: the bitmap's own colors, row-major columns.
    let left = pixmap.pixel(2, 1).expect("in bounds");
    assert_eq!(
        (left.red(), left.green(), left.blue()),
        (255, 0, 0),
        "red column"
    );
    let right = pixmap.pixel(5, 1).expect("in bounds");
    assert_eq!(
        (right.red(), right.green(), right.blue()),
        (0, 0, 255),
        "blue column"
    );
    let second_row = pixmap.pixel(3, 2).expect("in bounds");
    assert_eq!(
        (second_row.red(), second_row.green(), second_row.blue()),
        (255, 0, 0),
        "the bitmap's second row lands under the first"
    );

    // Outside the destination: the background stands untouched.
    let outside = pixmap.pixel(0, 0).expect("in bounds");
    assert_eq!(
        (outside.red(), outside.green(), outside.blue()),
        (255, 255, 255)
    );
    let below = pixmap.pixel(2, 5).expect("in bounds");
    assert_eq!(below.blue(), 255, "below the dst the background stands");
}

#[test]
fn image_clip_keeps_pixels_inside_the_region() {
    let mut image = red_blue_raster();
    image.clip = Some(Rect::new(Point::new(0.0, 0.0), Size::new(2.0, 8.0)));
    let scene = SceneSnapshot {
        primitives: vec![Primitive::Image(image)],
    };
    let mut glyphs = NoGlyphs;
    let pixmap = alacer_render::SoftwareRenderer::new()
        .render(&scene, 12, 8, Color::WHITE, &mut glyphs)
        .expect("pixmap");

    // Inside the clip: the image paints.
    let inside = pixmap.pixel(1, 1).expect("in bounds");
    assert_eq!((inside.red(), inside.green(), inside.blue()), (255, 0, 0));
    // Outside the clip: the background stands, even where the dst reaches.
    let clipped = pixmap.pixel(3, 1).expect("in bounds");
    assert!(
        clipped.red() > 200 && clipped.blue() > 200,
        "the blue column is clipped away, got {clipped:?}"
    );
}

/// The Q8 equivalence at the raster tier: partial repaint of a moved image
/// is pixel-identical to a full render of the moved scene.
#[test]
fn partial_repaint_matches_full_repaint_on_an_image_move() {
    let width = 32_u32;
    let height = 24_u32;
    let mut previous = red_blue_raster();
    previous.dst = Rect::new(Point::new(2.0, 2.0), Size::new(4.0, 2.0));
    let prev_scene = SceneSnapshot {
        primitives: vec![
            Primitive::Quad(alacer_core::scene::Quad::filled(
                Rect::new(Point::new(12.0, 12.0), Size::new(4.0, 4.0)),
                Color::rgb(30, 200, 30),
            )),
            Primitive::Image(previous),
        ],
    };
    let mut moved = red_blue_raster();
    moved.dst = Rect::new(Point::new(20.0, 8.0), Size::new(4.0, 2.0));
    let scene = SceneSnapshot {
        primitives: vec![
            Primitive::Quad(alacer_core::scene::Quad::filled(
                Rect::new(Point::new(12.0, 12.0), Size::new(4.0, 4.0)),
                Color::rgb(30, 200, 30),
            )),
            Primitive::Image(moved),
        ],
    };

    // The pipeline's damage rule: every old and new primitive extent.
    let mut damage = DamageBuilder::new();
    for primitive in scene.primitives.iter().chain(prev_scene.primitives.iter()) {
        damage.add(primitive_extent(primitive));
    }
    let damage = damage.finish(Size::new(
        f32::from(u16::try_from(width).unwrap_or(0)),
        f32::from(u16::try_from(height).unwrap_or(0)),
    ));
    assert!(!damage.is_full(), "the moved image's extents stay partial");

    let mut renderer = alacer_render::SoftwareRenderer::new();
    let mut glyphs = NoGlyphs;
    let expected = renderer
        .render(&scene, width, height, Color::WHITE, &mut NoGlyphs)
        .expect("full render");
    let mut prev = renderer
        .render(&prev_scene, width, height, Color::WHITE, &mut glyphs)
        .expect("previous frame");
    let mut glyphs_partial = NoGlyphs;
    renderer
        .render_partial(
            &scene,
            &mut prev,
            &damage,
            Color::WHITE,
            &mut glyphs_partial,
        )
        .expect("partial render");
    for y in 0..height {
        for x in 0..width {
            let a = prev.pixel(x, y).expect("in bounds");
            let b = expected.pixel(x, y).expect("in bounds");
            assert_eq!(
                (a.red(), a.green(), a.blue()),
                (b.red(), b.green(), b.blue()),
                "partial != full at ({x},{y})"
            );
        }
    }
}

#[test]
fn damage_extent_covers_the_image_dst() {
    let image = red_blue_raster();
    let extent = primitive_extent(&Primitive::Image(image));
    let dst = Rect::new(Point::new(0.0, 0.0), Size::new(4.0, 2.0));
    assert!(
        extent.intersection(&dst) == Some(dst),
        "the extent must cover the destination (the area any renderer may touch), got {extent:?}"
    );
    // The AA margin rides: the extent is strictly larger than the dst.
    assert!(extent.size.width > dst.size.width && extent.size.height > dst.size.height);
}

#[test]
fn image_sampling_scales_the_bitmap() {
    let mut image = red_blue_raster();
    // 4×2 bitmap scaled 2× into an 8×4 destination: the red/blue column
    // seam (between source columns 1 and 2) lands inside the destination,
    // where the two sampling modes must differ.
    image.dst = Rect::new(Point::new(1.0, 1.0), Size::new(8.0, 4.0));
    let nearest_scene = SceneSnapshot {
        primitives: vec![Primitive::Image(image.clone())],
    };
    image.sampling = ImageSampling::Bilinear;
    let bilinear_scene = SceneSnapshot {
        primitives: vec![Primitive::Image(image)],
    };

    let mut glyphs = NoGlyphs;
    let nearest = alacer_render::SoftwareRenderer::new()
        .render(&nearest_scene, 16, 10, Color::WHITE, &mut glyphs)
        .expect("pixmap");
    let bilinear = alacer_render::SoftwareRenderer::new()
        .render(&bilinear_scene, 16, 10, Color::WHITE, &mut glyphs)
        .expect("pixmap");

    // Nearest keeps the pure texel color on the red side of the seam;
    // bilinear blends across it toward blue.
    let nearest_pixel = nearest.pixel(4, 2).expect("in bounds");
    let bilinear_pixel = bilinear.pixel(4, 2).expect("in bounds");
    assert_eq!(
        (
            nearest_pixel.red(),
            nearest_pixel.green(),
            nearest_pixel.blue()
        ),
        (255, 0, 0),
        "nearest sampling keeps the pure texel color, got {nearest_pixel:?}"
    );
    assert_ne!(
        (
            bilinear_pixel.red(),
            bilinear_pixel.green(),
            bilinear_pixel.blue()
        ),
        (255, 0, 0),
        "bilinear blends across the color seam, got {bilinear_pixel:?}"
    );
}

#[test]
fn refusal_paths_are_honest_no_ops() {
    // Zero-size destination.
    let mut image = red_blue_raster();
    image.dst = Rect::new(Point::new(2.0, 2.0), Size::new(0.0, 2.0));
    let zero_dst = SceneSnapshot {
        primitives: vec![Primitive::Image(image)],
    };
    // Missing bytes.
    let mut image = red_blue_raster();
    image.bytes = Arc::from(Vec::new().into_boxed_slice());
    let missing_bytes = SceneSnapshot {
        primitives: vec![Primitive::Image(image)],
    };
    // Zero-size source window.
    let mut image = red_blue_raster();
    image.src = Rect::new(Point::new(0.0, 0.0), Size::new(0.0, 0.0));
    let zero_src = SceneSnapshot {
        primitives: vec![Primitive::Image(image)],
    };

    let mut glyphs = NoGlyphs;
    for (name, scene) in [
        ("zero dst", zero_dst),
        ("missing bytes", missing_bytes),
        ("zero src", zero_src),
    ] {
        let pixmap = alacer_render::SoftwareRenderer::new()
            .render(&scene, 12, 8, Color::WHITE, &mut glyphs)
            .expect("the render itself never fails on a refused image");
        for y in 0..8_u32 {
            for x in 0..12_u32 {
                let pixel = pixmap.pixel(x, y).expect("in bounds");
                assert_eq!(
                    (pixel.red(), pixel.green(), pixel.blue()),
                    (255, 255, 255),
                    "{name}: the refused image leaves the frame untouched at ({x},{y})"
                );
            }
        }
    }
}

#[test]
fn decode_png_rgba_round_trips_premultiplied_bytes() {
    // tiny-skia encodes too: a known 2×1 pixmap (opaque red, opaque blue)
    // rides the same codec the example's asset uses.
    let mut pixmap = tiny_skia::Pixmap::new(2, 1).expect("pixmap");
    pixmap.fill(tiny_skia::Color::from_rgba8(255, 0, 0, 255));
    let png = pixmap.encode_png().expect("encode");
    let (width, height, bytes) = alacer_render::software::decode_png_rgba(&png).expect("decode");
    assert_eq!((width, height), (2, 1));
    assert_eq!(bytes.len(), 8, "one RGBA8 quad per pixel");
    assert_eq!(
        &bytes[0..4],
        &[255, 0, 0, 255],
        "the red pixel survives the codec"
    );
}

#[test]
fn partition_routes_images_beside_quads_and_glyphs() {
    let scene = SceneSnapshot {
        primitives: vec![
            Primitive::Quad(alacer_core::scene::Quad::filled(
                Rect::new(Point::new(0.0, 0.0), Size::new(2.0, 2.0)),
                Color::BLACK,
            )),
            Primitive::Image(red_blue_raster()),
            Primitive::Glyph(alacer_core::scene::Glyph {
                position: Point::new(1.0, 2.0),
                size: 14.0,
                color: Color::BLACK,
                font_id: 1,
                glyph_id: 65,
                clip: None,
            }),
        ],
    };
    let batches = alacer_render::partition(&scene);
    assert_eq!(batches.quads.len(), 1);
    assert_eq!(batches.images.len(), 1);
    assert_eq!(batches.glyphs.len(), 1);
    // The image batch keeps the raster: the GPU side uploads from it.
    assert_eq!(batches.images[0].width, 4);
    assert_eq!(batches.images[0].height, 2);
}
