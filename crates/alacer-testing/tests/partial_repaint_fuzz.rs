//! The Q8 partial-repaint equivalence **fuzz** (§9.3): randomized scenes,
//! randomized damage, zero-diff proof — the universal form of the contract
//! the deterministic `partial_repaint_equivalence` suite pins for one
//! hand-built scene.
//!
//! The plan's demand (README's backends row, quoting §9.3/Q8): "an
//! equivalence fuzz pins partial repaints pixel-equal to full renders." A
//! deterministic test pins one scene; a fuzz generates 128 random, valid
//! [`SceneSnapshot`]s (quads with varying sizes, colors, rounded corners,
//! borders and clips; glyphs with real coverage bitmaps; premultiplied
//! raster images with random source windows, destinations and sampling)
//! and random [`Damage`] regions that always reach into painted pixels,
//! then demands `pixel_diff_ratio(&full, &partial) == 0.0` for every one
//! of them. Seeds are fixed ([`alacer_testing::Lcg`]), so a failure
//! reproduces exactly — a bug report, not a flake.
//!
//! The suite also proves its own sensitivity (a fuzz that cannot fail is
//! a tautology): damage that deliberately misses a changed primitive
//! leaves stale pixels behind (the contract's premise — "the pipeline's
//! damage covers every changed pixel" — is load-bearing), and the
//! full-frame fallback and the `Damage::NONE` no-op round out both ends
//! of the escape hatch.

use alacer_core::color::Color;
use alacer_core::damage::{Damage, DamageBuilder};
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::num::{f32_to_i32, f32_to_u32, usize_to_f32};
use alacer_core::scene::{Primitive, Quad, SceneSnapshot};
use alacer_render::SoftwareRenderer;
use alacer_testing::{FUZZ_VIEWPORT, FuzzGlyphs, SceneGenerator, pixel_diff_ratio};

/// How many random scenes the equivalence fuzz proves (the plan's 100+
/// iterations floor, with headroom).
const FUZZ_ITERATIONS: usize = 128;

/// The opaque background both paths rasterize over (an opaque background
/// is the premise both fills share: the partial path's region clear is a
/// `SourceOver` blend, which replaces exactly when alpha is 255).
const BACKGROUND: Color = Color::rgb(18, 22, 30);

/// The left probe quad's color in both control scenes.
const CONTROL_LEFT: Color = Color::rgb(196, 148, 58);

/// The right probe quad's color in the stale scene.
const CONTROL_STALE: Color = Color::rgb(70, 110, 150);

/// The right probe quad's color after the change the damage misses.
const CONTROL_CHANGED: Color = Color::rgb(214, 88, 42);

/// A sharp, full-height probe quad (the control scenes' primitives).
fn probe_quad(x: f32, width: f32, color: Color) -> Quad {
    Quad {
        rect: Rect::new(Point::new(x, 0.0), Size::new(width, FUZZ_VIEWPORT.height)),
        color,
        corner_radius: 0.0,
        border_color: Color::TRANSPARENT,
        border_width: 0.0,
        clip: None,
    }
}

/// The two-quad control scene the sensitivity test reasons over.
fn control_scene(right: Color) -> SceneSnapshot {
    SceneSnapshot {
        primitives: vec![
            Primitive::Quad(probe_quad(0.0, 64.0, CONTROL_LEFT)),
            Primitive::Quad(probe_quad(80.0, 64.0, right)),
        ],
    }
}

/// Renders `scene` fully into a fresh viewport pixmap.
fn full_render(
    renderer: &mut SoftwareRenderer,
    glyphs: &mut FuzzGlyphs,
    scene: &SceneSnapshot,
) -> tiny_skia::Pixmap {
    renderer
        .render(
            scene,
            f32_to_u32(FUZZ_VIEWPORT.width),
            f32_to_u32(FUZZ_VIEWPORT.height),
            BACKGROUND,
            glyphs,
        )
        .expect("the full render allocates")
}

/// The diff ratio over one region only (the same >8/255 tolerance
/// `pixel_diff_ratio` applies globally) — the targeted probe the
/// sensitivity test reasons with.
fn region_diff_ratio(a: &tiny_skia::Pixmap, b: &tiny_skia::Pixmap, region: &Rect) -> f32 {
    let (width, height) = (a.width(), a.height());
    let x0 = usize::try_from(f32_to_i32(region.min_x().max(0.0))).unwrap_or(0);
    let y0 = usize::try_from(f32_to_i32(region.min_y().max(0.0))).unwrap_or(0);
    let x1 = usize::try_from(f32_to_i32(region.max_x().min(f32::from(u16::MAX))))
        .unwrap_or(usize::from(u16::MAX))
        .min(usize::try_from(width).unwrap_or(0));
    let y1 = usize::try_from(f32_to_i32(region.max_y().min(f32::from(u16::MAX))))
        .unwrap_or(usize::from(u16::MAX))
        .min(usize::try_from(height).unwrap_or(0));
    let stride = usize::try_from(width).unwrap_or(0) * 4;
    let a_bytes = a.data();
    let b_bytes = b.data();
    let mut differing = 0_usize;
    let mut total = 0_usize;
    for y in y0..y1 {
        for x in x0..x1 {
            let index = y * stride + x * 4;
            let (p, q) = (&a_bytes[index..index + 4], &b_bytes[index..index + 4]);
            total += 1;
            if p.iter().zip(q.iter()).any(|(x, y)| x.abs_diff(*y) > 8) {
                differing += 1;
            }
        }
    }
    if total == 0 {
        0.0
    } else {
        usize_to_f32(differing) / usize_to_f32(total)
    }
}

#[test]
fn every_random_scene_repaints_partially_pixel_identical_to_the_full_render() {
    let mut renderer = SoftwareRenderer::new();
    let mut glyphs = FuzzGlyphs::new();
    for seed in 0..FUZZ_ITERATIONS {
        let mut generator = SceneGenerator::new(u64::try_from(seed).unwrap_or(0), FUZZ_VIEWPORT);
        let scene = generator.generate_scene();
        assert!(
            !scene.primitives.is_empty(),
            "seed {seed}: the generator produced a non-empty scene"
        );

        let damage = generator.generate_damage(&scene);
        let pixmap_full = full_render(&mut renderer, &mut glyphs, &scene);
        let mut pixmap_partial = pixmap_full.clone();
        renderer
            .render_partial(
                &scene,
                &mut pixmap_partial,
                &damage,
                BACKGROUND,
                &mut glyphs,
            )
            .expect("the partial repaint runs");

        assert_eq!(
            pixel_diff_ratio(&pixmap_full, &pixmap_partial),
            0.0,
            "seed {seed}: the partial repaint of {damage:?} is pixel-identical \
             to the full render of a {}-primitive random scene",
            scene.primitives.len()
        );
    }
}

#[test]
fn damage_that_misses_a_changed_primitive_leaves_stale_pixels() {
    // The sensitivity control: the same two-quad scene, the right quad's
    // color changed, the damage covering the *left* quad only. The
    // contract's premise (the pipeline's damage covers every changed
    // pixel) is what makes partial paint correct; a region that skips the
    // change must leave the stale color behind — a fuzz that converged
    // anyway would prove nothing.
    let mut renderer = SoftwareRenderer::new();
    let mut glyphs = FuzzGlyphs::new();
    let stale = control_scene(CONTROL_STALE);
    let changed = control_scene(CONTROL_CHANGED);

    let mut framebuffer = full_render(&mut renderer, &mut glyphs, &stale);
    let expected = full_render(&mut renderer, &mut glyphs, &changed);
    assert!(
        pixel_diff_ratio(&framebuffer, &expected) > 0.0,
        "the stale framebuffer genuinely disagrees with the changed scene"
    );

    let left_region = Rect::new(Point::new(0.0, 0.0), Size::new(72.0, FUZZ_VIEWPORT.height));
    let mut builder = DamageBuilder::new();
    builder.add(left_region);
    let damage = builder.finish(FUZZ_VIEWPORT);
    assert_eq!(
        damage.regions(FUZZ_VIEWPORT),
        vec![left_region],
        "one region over the left quad alone"
    );

    renderer
        .render_partial(&changed, &mut framebuffer, &damage, BACKGROUND, &mut glyphs)
        .expect("the partial repaint runs");

    let right_region = Rect::new(Point::new(76.0, 0.0), Size::new(72.0, FUZZ_VIEWPORT.height));
    assert!(
        region_diff_ratio(&framebuffer, &expected, &right_region) > 0.0,
        "the uncovered change stays stale: the right quad keeps its old color"
    );
    assert_eq!(
        region_diff_ratio(&framebuffer, &expected, &left_region),
        0.0,
        "the covered region repainted pixel-identically"
    );
    assert!(
        pixel_diff_ratio(&framebuffer, &expected) > 0.0,
        "the whole frame is NOT equal — the missed change is real damage the \
         pipeline must never emit"
    );
}

#[test]
fn the_full_frame_fallback_repaints_everything_pixel_identical() {
    // The escape hatch (§9.3): when damage outgrows the region budget's
    // cover share, the whole frame repaints — and the scissored full
    // redraw must land exactly where the full render lands.
    let mut renderer = SoftwareRenderer::new();
    let mut glyphs = FuzzGlyphs::new();
    for seed in [0_u64, 7, 19, 63] {
        let mut generator = SceneGenerator::new(seed, FUZZ_VIEWPORT);
        let scene = generator.generate_scene();
        let mut builder = DamageBuilder::new();
        builder.add(Rect::new(Point::new(0.0, 0.0), Size::new(160.0, 60.0)));
        builder.add(Rect::new(Point::new(0.0, 40.0), Size::new(160.0, 60.0)));
        builder.add(Rect::new(Point::new(0.0, 80.0), Size::new(160.0, 40.0)));
        let damage = builder.finish(FUZZ_VIEWPORT);
        assert!(damage.is_full(), "seed {seed}: the hatch fires");

        let pixmap_full = full_render(&mut renderer, &mut glyphs, &scene);
        let mut pixmap_partial = pixmap_full.clone();
        renderer
            .render_partial(
                &scene,
                &mut pixmap_partial,
                &damage,
                BACKGROUND,
                &mut glyphs,
            )
            .expect("the partial repaint runs");
        assert_eq!(
            pixel_diff_ratio(&pixmap_full, &pixmap_partial),
            0.0,
            "seed {seed}: the full-frame fallback is pixel-identical"
        );
    }
}

#[test]
fn no_damage_repaints_nothing() {
    // The other hatch end (§9.3): `Damage::NONE` skips the present — the
    // persistent framebuffer must be left untouched, bit for bit.
    let mut renderer = SoftwareRenderer::new();
    let mut glyphs = FuzzGlyphs::new();
    let mut generator = SceneGenerator::new(11, FUZZ_VIEWPORT);
    let scene = generator.generate_scene();
    let pixmap_full = full_render(&mut renderer, &mut glyphs, &scene);
    let mut pixmap_partial = pixmap_full.clone();
    renderer
        .render_partial(
            &scene,
            &mut pixmap_partial,
            &Damage::NONE,
            BACKGROUND,
            &mut glyphs,
        )
        .expect("the no-op repaint runs");
    assert_eq!(
        pixel_diff_ratio(&pixmap_full, &pixmap_partial),
        0.0,
        "no damage, no pixels touched"
    );
}
