//! The compose tier's opacity contract, at the record + render seam
//! (§14.2's W3 mechanism): a quad faded through the compose pass blends
//! against its background at the fade's own strength, a faded glyph fades
//! its coverage stamp, a compose translation moves the recorded geometry,
//! and the identity compose is *byte-identical* to the baseline — the
//! zero-regression gate for the static frames that never compose.
//!
//! The record side is [`alacer_core::scene::PaintCtx`] (the compose state
//! the pass installs); the render side is the software rasterizer — the
//! rig every test runs on (A4). The fade rides the alpha channel the
//! rasterizer already blends with, so a faded primitive is an ordinary
//! premultiplied-alpha citizen: no backend learned a new fact here, the
//! renderer tests that existed before this file pass untouched.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect, Size, Vector};
use alacer_core::scene::{Compose, Glyph, PaintCtx, PaintSink, Primitive, Quad, SceneSnapshot};
use alacer_render::SoftwareRenderer;
use std::sync::Arc;

/// A recording paint sink: the test's window into what a context records.
#[derive(Default)]
struct Collector(Vec<Primitive>);

impl PaintSink for Collector {
    fn push(&mut self, primitive: Primitive) {
        self.0.push(primitive);
    }
}

/// The context facts the tests share: origin (4, 6), 40×30 surface, canon
/// palette, no OS facts — the direct-caller shape `PaintCtx::new`
/// documents.
const ORIGIN: Point = Point::new(4.0, 6.0);
const SURFACE: Size = Size::new(40.0, 30.0);

/// Records one opaque red 20×10 quad at the local origin, under `compose`.
fn red_quad_under(compose: Compose) -> Vec<Primitive> {
    let mut sink = Collector::default();
    let mut ctx = PaintCtx::new(
        &mut sink,
        ORIGIN,
        SURFACE,
        None,
        alacer_core::theme::Palette::CANON,
        false,
        false,
        false,
        None,
    );
    ctx.set_compose(compose);
    ctx.push_quad(Quad::filled(
        Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)),
        Color::rgb(255, 0, 0),
    ));
    sink.0
}

/// The test glyph: solid full coverage, black, baseline at the local
/// (10, 20).
fn black_glyph() -> Glyph {
    Glyph {
        position: Point::new(10.0, 20.0),
        size: 16.0,
        color: Color::BLACK,
        font_id: 1,
        glyph_id: 65,
        clip: None,
    }
}

/// A fully-opaque coverage bitmap spanning 4×4 (the software crate's own
/// test helper shape).
fn solid_bitmap() -> alacer_core::scene::GlyphBitmap {
    alacer_core::scene::GlyphBitmap {
        left: 0,
        top: 0,
        width: 4,
        height: 4,
        coverage: Arc::from(vec![255_u8; 16].into_boxed_slice()),
    }
}

#[test]
fn half_opacity_quad_blends_against_the_background() {
    // Source-over arithmetic at the fade the compose recorded: the alpha
    // folded to 128 (255 × 0.5, rounded once), the red channel rides it
    // premultiplied. Over the green background: red = 255·(128/255), green
    // = 255·(1 − 128/255) — the quad's own half strength, nothing more.
    let primitives = red_quad_under(Compose::faded(0.5));
    let scene = SceneSnapshot { primitives };
    let mut no_glyphs =
        |_f: u64, _g: u32, _s: f32| -> Option<alacer_core::scene::GlyphBitmap> { None };
    let mut glyphs = alacer_render::ClosureGlyphSource::new(&mut no_glyphs);
    let pixmap = SoftwareRenderer::new()
        .render(&scene, 40, 30, Color::rgb(0, 255, 0), &mut glyphs)
        .expect("pixmap");
    let blended = pixmap.pixel(14, 11).expect("inside the quad");
    assert!(
        (i32::from(blended.red()) - 128).abs() <= 2,
        "half opacity contributes half the red, got {}",
        blended.red()
    );
    assert!(
        (i32::from(blended.green()) - 127).abs() <= 2,
        "half opacity leaves half the background green, got {}",
        blended.green()
    );
    assert_eq!(blended.blue(), 0, "neither source carries blue");
}

#[test]
fn half_opacity_glyph_fades_the_coverage_stamp() {
    // The glyph stamp multiplies coverage by the (faded) alpha: black ink
    // at half strength over white lands at the half-coverage blend the
    // software suite already pins — 0 + (1 − 128/255)·255 ≈ 127.
    let mut sink = Collector::default();
    let mut ctx = PaintCtx::new(
        &mut sink,
        ORIGIN,
        SURFACE,
        None,
        alacer_core::theme::Palette::CANON,
        false,
        false,
        false,
        None,
    );
    ctx.set_compose(Compose::faded(0.5));
    ctx.push_glyph(black_glyph());
    let scene = SceneSnapshot { primitives: sink.0 };
    let mut stamp = |_f: u64, _g: u32, _s: f32| Some(solid_bitmap());
    let mut glyphs = alacer_render::ClosureGlyphSource::new(&mut stamp);
    let pixmap = SoftwareRenderer::new()
        .render(&scene, 40, 30, Color::WHITE, &mut glyphs)
        .expect("pixmap");
    let inked = pixmap.pixel(14, 26).expect("inside the glyph stamp");
    assert!(
        (i32::from(inked.red()) - 127).abs() <= 4,
        "the faded glyph paints at half strength, got {}",
        inked.red()
    );
}

#[test]
fn identity_compose_is_byte_identical_to_the_baseline() {
    // The baseline: the same quad painted without any compose state — the
    // pre-compose pipeline's own record. The identity must never move a
    // byte, in the journal or in the pixels.
    let mut plain = Collector::default();
    let mut ctx = PaintCtx::new(
        &mut plain,
        ORIGIN,
        SURFACE,
        None,
        alacer_core::theme::Palette::CANON,
        false,
        false,
        false,
        None,
    );
    ctx.push_quad(Quad::filled(
        Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)),
        Color::rgb(255, 0, 0),
    ));
    let baseline = SceneSnapshot {
        primitives: plain.0,
    };

    let composed = SceneSnapshot {
        primitives: red_quad_under(Compose::IDENTITY),
    };
    assert_eq!(
        baseline.primitives, composed.primitives,
        "the identity records the same primitives"
    );

    let mut stamp = |_f: u64, _g: u32, _s: f32| Some(solid_bitmap());
    let mut glyphs = alacer_render::ClosureGlyphSource::new(&mut stamp);
    let left = SoftwareRenderer::new()
        .render(&baseline, 40, 30, Color::WHITE, &mut glyphs)
        .expect("baseline pixmap");
    let right = SoftwareRenderer::new()
        .render(&composed, 40, 30, Color::WHITE, &mut glyphs)
        .expect("composed pixmap");
    assert_eq!(
        left.data(),
        right.data(),
        "the identity renders byte-identical pixels"
    );
}

#[test]
fn compose_translation_moves_the_recorded_geometry() {
    // The compose translation rides the same record-time fold the origin
    // does: the quad lands shifted in the journal, and the pixels follow.
    let primitives = red_quad_under(Compose::shifted(Vector::new(6.0, 2.0)));
    let Primitive::Quad(quad) = &primitives[0] else {
        panic!("expected quad");
    };
    assert_eq!(
        quad.rect.origin,
        Point::new(10.0, 8.0),
        "origin (4, 6) beside the compose translation (6, 2)"
    );
    assert_eq!(quad.color.a, 255, "a shift is not a fade");

    let scene = SceneSnapshot { primitives };
    let mut no_glyphs =
        |_f: u64, _g: u32, _s: f32| -> Option<alacer_core::scene::GlyphBitmap> { None };
    let mut glyphs = alacer_render::ClosureGlyphSource::new(&mut no_glyphs);
    let pixmap = SoftwareRenderer::new()
        .render(&scene, 40, 30, Color::rgb(0, 255, 0), &mut glyphs)
        .expect("pixmap");
    let moved = pixmap.pixel(26, 17).expect("inside the moved quad alone");
    assert!(
        moved.red() > 200,
        "the quad paints where the compose translation put it"
    );
    let vacated = pixmap.pixel(5, 7).expect("inside the unshifted quad alone");
    assert!(
        vacated.green() > 200,
        "the background shows where the quad no longer is"
    );
}
