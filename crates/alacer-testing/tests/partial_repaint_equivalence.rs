//! The Q8 partial-repaint equivalence contract, proven end-to-end through
//! the real pipeline (M1).
//!
//! `SoftwareRenderer::render_partial` documents an equivalence contract
//! (Q8, `crates/alacer-render/src/software.rs`): a partial repaint of the
//! frame's [`Damage`] regions onto a persistent framebuffer is
//! *pixel-identical* to a full render of the same scene over the same
//! background. The renderer's own unit tests prove the claim at the
//! primitive level; this suite proves it through the shape a real frame
//! takes — widgets mounted on the [`TestHarness`], a `tick()` scene
//! snapshot, and the software raster's scissored redraw:
//!
//! 1. mount a solid background and a foreground canvas at a known offset,
//! 2. render fully and clone the pixmap as the persistent framebuffer,
//! 3. damage only the foreground's bounds (plus a 1 px margin exercising
//!    the scissor's boundary discipline),
//! 4. repaint partially and demand a zero pixel-diff ratio.
//!
//! The second test runs the *stale-framebuffer* flow the contract's
//! premise names: the persistent buffer holds the previous scene's
//! rendering (the foreground in its old color), the damage covers the
//! changed rect, and the partial repaint must converge on the full render
//! of the new scene. A pre-repaint diff against the new full render
//! guards the test's own sensitivity: a framebuffer that was never stale
//! could not distinguish a repaint from a no-op.

use alacer_core::arena::NodeId;
use alacer_core::color::Color;
use alacer_core::damage::DamageBuilder;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::num::f32_to_u32;
use alacer_core::scene::{GlyphBitmap, GlyphSource, PaintCtx, Primitive, Quad, SceneSnapshot};
use alacer_render::SoftwareRenderer;
use alacer_testing::{TestHarness, pixel_diff_ratio};
use alacer_widgets::{Canvas, Column, Panel};
use std::sync::Arc;

/// The viewport both tests render into (small, deterministic, headless).
const VIEWPORT: Size = Size::new(160.0, 120.0);

/// The column's uniform padding — keeps every damage rect (bounds plus
/// its 1 px margin) strictly inside the viewport, unclamped.
const PADDING: f32 = 4.0;

/// The solid background band behind the foreground element.
const BACKGROUND_BAND: Color = Color::rgb(24, 38, 66);

/// The foreground canvas's repaint-probe color in the mounted scene.
const FOREGROUND: Color = Color::rgb(214, 88, 42);

/// The foreground color the stale framebuffer test repaints *from*.
const STALE_FOREGROUND: Color = Color::rgb(31, 122, 77);

/// The canvas size both tests mount the foreground at.
const FOREGROUND_SIZE: Size = Size::new(64.0, 24.0);

/// A hermetic glyph source: the mounted scene draws quads only, so every
/// glyph query is honestly empty — no font machinery in the test.
struct NoGlyphs;

impl GlyphSource for NoGlyphs {
    fn glyph_bitmap(&mut self, _font_id: u64, _glyph_id: u32, _size: f32) -> Option<GlyphBitmap> {
        None
    }
}

/// The painter a test canvas hangs: one full-bleed quad in `color`.
fn paint_probe(color: Color) -> impl Fn(&mut PaintCtx<'_>) + 'static {
    move |ctx| {
        ctx.push_quad(Quad {
            rect: Rect::new(Point::new(0.0, 0.0), ctx.size()),
            color,
            corner_radius: 0.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// Mounts the scene both tests share: a solid background `Panel` band
/// inside a padded `Column`, then the foreground canvas right under it —
/// a known offset whose damage rect crosses two distinct background
/// fills (band above, surface below), the discipline the scissor must
/// honor. Returns the canvas's node id and the ticked scene snapshot.
fn mount_probe_scene(harness: &mut TestHarness, foreground: Color) -> (NodeId, Arc<SceneSnapshot>) {
    let root = harness.mount(Box::new(Column::new().padding(PADDING)));
    let _band = harness.add_child(
        root,
        Box::new(Panel::new(FOREGROUND_SIZE.height * 3.0, BACKGROUND_BAND)),
    );
    let canvas = harness.add_child(
        root,
        Box::new(
            Canvas::new("The repaint probe")
                .size(FOREGROUND_SIZE.width, FOREGROUND_SIZE.height)
                .painter(paint_probe(foreground)),
        ),
    );
    let report = harness.tick();
    (canvas, report.scene)
}

/// The damage rect the tests repaint: the foreground's laid-out bounds
/// grown by a 1 px margin on every side — the margin exercises the
/// scissor's clipping at a boundary through two background fills.
fn damaged_rect(harness: &TestHarness, canvas: NodeId) -> Rect {
    let bounds = harness.bounds(canvas);
    Rect::new(
        Point::new(bounds.origin.x - 1.0, bounds.origin.y - 1.0),
        Size::new(bounds.size.width + 2.0, bounds.size.height + 2.0),
    )
}

#[test]
fn partial_repaint_of_the_damaged_foreground_is_pixel_identical_to_the_full_render() {
    let mut harness = TestHarness::new(VIEWPORT.width, VIEWPORT.height);
    let (canvas, scene) = mount_probe_scene(&mut harness, FOREGROUND);

    // The probe quad must be in the scene with the mounted color — a
    // silent empty scene would make the equivalence a tautology.
    assert!(
        scene.primitives.iter().any(|primitive| matches!(
            primitive,
            Primitive::Quad(quad) if quad.color == FOREGROUND
        )),
        "the foreground canvas's quad reached the scene snapshot"
    );

    // The full render — and a clone of it playing the persistent
    // framebuffer (the previous frame's rendering, per the contract's
    // premise; here the scene is unchanged between frames).
    let mut renderer = SoftwareRenderer::new();
    let mut glyphs = NoGlyphs;
    let pixmap_full = renderer
        .render(
            &scene,
            f32_to_u32(VIEWPORT.width),
            f32_to_u32(VIEWPORT.height),
            alacer_widgets::theme::SURFACE,
            &mut glyphs,
        )
        .expect("the full render allocates");
    let mut pixmap_partial = pixmap_full.clone();

    // Damage only the foreground's bounds plus the 1 px margin.
    let damaged = damaged_rect(&harness, canvas);
    let mut builder = DamageBuilder::new();
    builder.add(damaged);
    let damage = builder.finish(VIEWPORT);
    assert!(
        !damage.is_full(),
        "the probe's damage stays far under the cover-ratio hatch"
    );
    assert_eq!(
        damage.regions(VIEWPORT),
        vec![damaged],
        "one region, exactly the margin-grown foreground rect"
    );

    renderer
        .render_partial(
            &scene,
            &mut pixmap_partial,
            &damage,
            alacer_widgets::theme::SURFACE,
            &mut glyphs,
        )
        .expect("the partial repaint runs");

    // The Q8 claim, end-to-end: widgets → journal → scene → scissored
    // software raster lands pixel-identical to the full render.
    assert_eq!(
        pixel_diff_ratio(&pixmap_full, &pixmap_partial),
        0.0,
        "the partial repaint of the damaged foreground is pixel-identical to the full render"
    );
}

#[test]
fn partial_repaint_over_a_stale_framebuffer_converges_on_the_new_full_render() {
    // Scene A (stale color) rendered into what the persistent
    // framebuffer holds; scene B (new color) is the frame to paint.
    let mut stale = TestHarness::new(VIEWPORT.width, VIEWPORT.height);
    let (stale_canvas, stale_scene) = mount_probe_scene(&mut stale, STALE_FOREGROUND);
    let mut fresh = TestHarness::new(VIEWPORT.width, VIEWPORT.height);
    let (canvas, scene) = mount_probe_scene(&mut fresh, FOREGROUND);

    let mut renderer = SoftwareRenderer::new();
    let mut glyphs = NoGlyphs;
    let viewport_u = (f32_to_u32(VIEWPORT.width), f32_to_u32(VIEWPORT.height));
    let expected = renderer
        .render(
            &scene,
            viewport_u.0,
            viewport_u.1,
            alacer_widgets::theme::SURFACE,
            &mut glyphs,
        )
        .expect("the new scene's full render allocates");
    let mut framebuffer = renderer
        .render(
            &stale_scene,
            viewport_u.0,
            viewport_u.1,
            alacer_widgets::theme::SURFACE,
            &mut glyphs,
        )
        .expect("the stale scene's full render allocates");

    // The framebuffer really is stale: it disagrees with the new scene's
    // full render. A zero diff here would mean the test cannot see a
    // missed repaint at all.
    assert!(
        pixel_diff_ratio(&framebuffer, &expected) > 0.0,
        "the stale framebuffer differs from the new scene's full render"
    );

    // The geometry is unchanged, so the damage is the same margin-grown
    // rect either scene lays out — the changed pixels it must cover.
    let damaged = damaged_rect(&stale, stale_canvas);
    assert_eq!(
        damaged_rect(&fresh, canvas),
        damaged,
        "both scenes lay the probe out identically"
    );
    let mut builder = DamageBuilder::new();
    builder.add(damaged);
    let damage = builder.finish(VIEWPORT);
    assert!(!damage.is_full());

    renderer
        .render_partial(
            &scene,
            &mut framebuffer,
            &damage,
            alacer_widgets::theme::SURFACE,
            &mut glyphs,
        )
        .expect("the partial repaint runs");

    // The Q8 claim over a genuinely stale frame: the scissored redraw of
    // the damaged rect converges on the new scene's full render.
    assert_eq!(
        pixel_diff_ratio(&framebuffer, &expected),
        0.0,
        "the partial repaint over the stale framebuffer is pixel-identical to the new full render"
    );
}
