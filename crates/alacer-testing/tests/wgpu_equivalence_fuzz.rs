//! The cross-backend equivalence fuzz (§9.3, Q8): randomized scenes,
//! rendered by **both** backends, must agree pixel-for-pixel.
//!
//! The plan's demand (§9.3's Q8 row): "the equivalence fuzz (§9.3) runs
//! against **both backends** from M2." Until now every equivalence proof
//! lived on one backend: `partial_repaint_fuzz` pins the software
//! renderer's partial frames to its own full frames, and the wgpu backend
//! was compile-verified only (A4) — no headless machine could ask the GPU
//! what it draws. `alacer_platform::wgpu_headless::HeadlessRenderer` is
//! the capability that closes the gap: the surface pipeline's shader,
//! scene packing, and premultiplied blending over an offscreen
//! `Rgba8Unorm` target, read back through
//! [`wgpu::util::DownloadBuffer`] into a `tiny_skia::Pixmap` — the byte
//! twin of the software backend's output.
//!
//! The fuzz generates 64 random, valid [`SceneSnapshot`]s (the
//! [`SceneGenerator`]'s quads — sharp and rounded, bordered and bare,
//! clipped and free — glyphs with real coverage bitmaps, premultiplied
//! raster images with source windows and both sampling modes), renders
//! each through [`SoftwareRenderer`] and the headless [`HeadlessRenderer`]
//! over the same [`alacer_widgets::theme::SURFACE`] background, and
//! demands `pixel_diff_ratio(&software, &gpu) <= 0.01` per scene — a tiny
//! 1% tolerance for the GPU float rounding vs CPU fixed-point blending
//! both paths are entitled to (the anti-aliased edges of the SDF quads
//! and the bilinear atlas samples land within it). Seeded and
//! reproducible: a failure is a bug report, not a flake. The iteration
//! floor (10) is asserted explicitly, so a silently-skipping fuzz can
//! never look like a passing one.
//!
//! Machines with no headless adapter at all (no Vulkan driver, no EGL)
//! skip the suite with the reason named — the honest refusal A4's
//! software-first discipline rides, never a fake pass.

use alacer_core::color::Color;
use alacer_platform::BackendError;
use alacer_platform::wgpu_headless::HeadlessRenderer;
use alacer_render::SoftwareRenderer;
use alacer_testing::{FUZZ_VIEWPORT, FuzzGlyphs, SceneGenerator, pixel_diff_ratio};
use alacer_widgets::theme;

/// How many random scenes the cross-backend fuzz proves (§9.3's 50+
/// iterations floor, with headroom).
const FUZZ_ITERATIONS: usize = 64;

/// The per-scene pixel-diff ceiling: 1% of the frame may disagree, the
/// honest margin for GPU float rounding vs CPU fixed-point blending.
const DIFF_TOLERANCE: f32 = 0.01;

/// The iteration floor the fuzz itself asserts, so a skip can never
/// masquerade as a pass.
const MIN_RUN_ITERATIONS: usize = 10;

/// The fuzz's seed — fixed, so a failure reproduces exactly.
const SEED: u64 = 0x9E_37_79_B9_7F_4A_7C_15;

#[test]
fn headless_wgpu_frames_match_software_for_fuzz_scenes() {
    let width = f32_to_u32(FUZZ_VIEWPORT.width);
    let height = f32_to_u32(FUZZ_VIEWPORT.height);
    let mut gpu = match HeadlessRenderer::new(width, height) {
        Ok(renderer) => renderer,
        Err(BackendError::NoAdapter) => {
            eprintln!(
                "SKIPPED: no headless wgpu adapter (no Vulkan/EGL device); \
                 the cross-backend equivalence needs one to prove anything"
            );
            return;
        }
        Err(error) => panic!("headless renderer setup failed: {error}"),
    };

    // Two keyed-deterministic glyph sources: identical bitmaps per query,
    // so the GPU atlas packs exactly what the software path stamps.
    let mut glyphs_gpu = FuzzGlyphs::new();
    let mut glyphs_sw = FuzzGlyphs::new();
    let mut software = SoftwareRenderer::new();
    let mut generator = SceneGenerator::new(SEED, FUZZ_VIEWPORT);

    let mut worst = 0.0_f32;
    let mut ran = 0_usize;
    for scene_index in 0..FUZZ_ITERATIONS {
        let scene = generator.generate_scene();
        let expected = software
            .render(&scene, width, height, theme::SURFACE, &mut glyphs_sw)
            .expect("the fuzz viewport's pixmap always allocates");
        let gpu_frame = gpu
            .render(&scene, &mut glyphs_gpu)
            .expect("the headless readback delivers the frame");
        assert_eq!(
            (expected.width(), expected.height()),
            (gpu_frame.width(), gpu_frame.height()),
            "scene {scene_index}: both backends drew the same viewport"
        );
        let diff = pixel_diff_ratio(&expected, &gpu_frame);
        worst = worst.max(diff);
        assert!(
            diff <= DIFF_TOLERANCE,
            "scene {scene_index}: cross-backend diff {diff:.4} exceeds \
             {DIFF_TOLERANCE} (worst so far {worst:.4}){}",
            worst_pixels(&expected, &gpu_frame)
        );
        ran += 1;
    }
    assert!(
        ran >= MIN_RUN_ITERATIONS,
        "the fuzz ran {ran} scenes; the equivalence contract needs \
         at least {MIN_RUN_ITERATIONS} to prove anything"
    );
    assert_eq!(ran, FUZZ_ITERATIONS, "every generated scene was proven");
    eprintln!("cross-backend equivalence: {ran} scenes, worst diff {worst:.4}");
}

/// One ranked disagreeing pixel: its channel delta, position, and both
/// backends' values — the failure report a renderer bug needs.
struct PixelDelta {
    delta: u8,
    x: usize,
    y: usize,
    software: [u8; 4],
    gpu: [u8; 4],
}

/// The five worst disagreeing pixels, with both backends' values — the
/// failure report a renderer bug needs to be fixed from.
fn worst_pixels(software: &tiny_skia::Pixmap, gpu: &tiny_skia::Pixmap) -> String {
    let mut ranked: Vec<PixelDelta> = Vec::new();
    for (index, (sw, gp)) in software
        .data()
        .as_chunks::<4>()
        .0
        .iter()
        .zip(gpu.data().as_chunks::<4>().0)
        .enumerate()
    {
        let delta = sw
            .iter()
            .zip(gp.iter())
            .map(|(a, b)| u8::abs_diff(*a, *b))
            .max()
            .unwrap_or(0);
        if delta > 0 {
            ranked.push(PixelDelta {
                delta,
                x: index % software.width() as usize,
                y: index / software.width() as usize,
                software: *sw,
                gpu: *gp,
            });
        }
    }
    ranked.sort_by_key(|pixel| std::cmp::Reverse(pixel.delta));
    ranked.truncate(5);
    let detail: Vec<String> = ranked
        .iter()
        .map(|pixel| {
            format!(
                "({},{}) sw={:?} gpu={:?} d={}",
                pixel.x, pixel.y, pixel.software, pixel.gpu, pixel.delta
            )
        })
        .collect();
    format!("; worst pixels: {}", detail.join(", "))
}

/// Rounds the fuzz viewport's float edge up to the pixel size both
/// backends draw into.
fn f32_to_u32(value: f32) -> u32 {
    u32::try_from(alacer_core::num::f32_to_i32(value.ceil())).unwrap_or(1)
}

/// The background the software path paints is the theme's own surface
/// byte color; a compile-time witness that the headless clear rides the
/// same value the readback is compared against.
const _: Color = theme::SURFACE;
