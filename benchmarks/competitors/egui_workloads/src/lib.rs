//! The egui side of the cross-framework comparison (M7, ADR-0013).
//!
//! Three shared workloads (architecture §14.2), implemented with egui's
//! everyday API — no exotic knobs, no deliberately handicapped calls:
//!
//! - **W1** (`w1_churn_full_10k`): 10k labels under 10%/frame style churn.
//!   egui rebuilds the whole widget walk every frame — that *is* its
//!   production path, so the workload measures egui honestly at its own
//!   game. One frame = `Context::run_ui` + `Context::tessellate`.
//! - **W2** (`w2_text_jitter_1k`): 1k paragraphs re-wrapped under a ±5%
//!   container-width jitter. egui's galley cache does its real thing:
//!   a changed width re-shapes, an unchanged width re-uses.
//! - **W3** (`w3_animation_1k`): 1k painted rects whose positions advance
//!   per frame, 10% of them fading. Immediate mode repaints all of them;
//!   the workload measures that full-frame cost.
//!
//! Scope, per the ADR's asymmetry rule (§3): a frame here is the widget
//! walk + layout + text shaping + tessellation. Pixel rasterization is
//! outside every competitor row that omits it, and egui's row omits it
//! the same way Masonry's does — the GPU blit is not the mechanism under
//! test. Alacer's W3 row *includes* a damaged software raster, which makes
//! the comparison conservative against Alacer.
//!
//! The no-strawman note (README): egui's production answer for very long
//! lists is `ScrollArea::show_rows` row virtualization. The W1/W2
//! workloads deliberately do not take it — they pin the *tree-wide*
//! mechanism — and the README documents that choice so nobody mistakes a
//! mechanism comparison for a product recommendation.

use egui::{Color32, Context, RawInput, Vec2};
use std::cell::Cell;

/// 10 sections × 1k leaves — the W1 shape (mirrors `w1_churn` on the
/// Alacer side).
pub const SECTIONS: usize = 10;
pub const LEAVES_PER_SECTION: usize = 1000;
/// The W2 paragraph count.
pub const PARAGRAPHS: usize = 1000;
/// The W3 animated node count.
pub const ANIMATED_NODES: usize = 1000;
/// The churn window (10% of leaves, rotated per frame — as the Alacer W1
/// bench rotates its own 10% window).
pub const CHURN_LEAVES: usize = 1000;
/// The jitter sweep: ±5% across 21 frames, as §14.2 W2 defines it.
pub const JITTER_STEPS: u64 = 21;

pub const VIEWPORT: Vec2 = Vec2::new(800.0, 600.0);
/// The W1/W2 workloads paint the *full* tree, so the screen clip rect (which
/// would cull everything below the fold and turn the workload into the
/// virtualization measurement the README explicitly does not take) is
/// enlarged to cover all rows.
fn full_tree_clip() -> egui::Rect {
    egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(VIEWPORT.x, VIEWPORT.y * 1000.0))
}
const PARAGRAPH: &str = "The quick brown fox keeps re-wrapping because the \
container cannot make up its mind; every width asks the shaper a slightly \
different question.";

/// One measured frame: `run_ui` + `tessellate` — egui's real per-frame CPU
/// path, headless. No window, no painter blit (the labeled scope above).
///
/// Returns the tessellated primitives *and* the shape count (one paint op
/// per label/rect — the volume proxy the selfchecks pin; egui merges the
/// meshes themselves into one clipped primitive per clip rect). The bench
/// `black_box`es both, so the optimizer cannot elide either side.
pub fn run_frame(
    ctx: &Context,
    build: impl FnMut(&mut egui::Ui),
) -> (Vec<egui::epaint::ClippedPrimitive>, usize) {
    let mut output = ctx.run_ui(RawInput::default(), build);
    let shape_count = output.shapes.len();
    let primitives = ctx.tessellate(std::mem::take(&mut output.shapes), output.pixels_per_point);
    // Headless: the texture deltas (the font atlas on the first frame)
    // have no painter to be applied to — clear them, which is exactly what
    // egui's `drop_without_applying_deltas` does for backends that skip
    // texture upload.
    output.textures_delta.clear();
    (primitives, shape_count)
}

/// The jitter schedule shared by the W2 benches: a ±5% sweep across
/// `JITTER_STEPS` frames around `base`.
pub fn jitter(base: f32, frame: u64) -> f32 {
    let phase = (frame % JITTER_STEPS) as f32 / (JITTER_STEPS - 1) as f32;
    base * (1.0 + 0.05 * (2.0 * phase - 1.0))
}

/// W1: 10 sections of 1k labels; the 10% churn window (rotated per frame)
/// carries a style that changes every frame. egui re-walks all of it every
/// frame — the workload measures exactly that.
pub fn build_w1(ui: &mut egui::Ui, frame: u64) {
    ui.set_clip_rect(full_tree_clip());
    let churn_start = (frame % (SECTIONS as u64)) * CHURN_LEAVES as u64;
    let churn_color = Color32::from_rgb(
        (frame * 7 % 255) as u8,
        (frame * 13 % 255) as u8,
        (frame * 29 % 255) as u8,
    );
    for section in 0..SECTIONS {
        ui.heading(format!("Section {section}"));
        for leaf in 0..LEAVES_PER_SECTION {
            let index = (section * LEAVES_PER_SECTION + leaf) as u64;
            if index >= churn_start && index < churn_start + CHURN_LEAVES as u64 {
                ui.label(egui::RichText::new(format!("leaf {index}")).color(churn_color));
            } else {
                ui.label(format!("leaf {index}"));
            }
        }
    }
}

/// W2: 1k paragraphs wrapped at the jittered width. A changed width asks
/// egui's galley cache to re-shape; the sweep makes every paragraph re-wrap
/// once per 21 frames.
pub fn build_w2(ui: &mut egui::Ui, frame: u64) {
    ui.set_clip_rect(full_tree_clip());
    let width = jitter(VIEWPORT.x, frame);
    ui.allocate_ui(Vec2::new(width, VIEWPORT.y), |inner| {
        for index in 0..PARAGRAPHS {
            inner.label(format!("{PARAGRAPH} ({index})"));
        }
    });
}

/// W3: 1k animated rects — positions advance per frame, 10% fade. This is
/// egui's idiomatic per-frame paint path (`ui.painter()`), no retained
/// state to invalidate because there is none.
pub fn build_w3(ui: &mut egui::Ui, time: f32) {
    let painter = ui.painter();
    for index in 0..ANIMATED_NODES {
        let i = index as f32;
        let x = (time * 60.0 + i * 7.0) % (VIEWPORT.x - 24.0);
        let y = (i * 13.0) % (VIEWPORT.y - 18.0);
        let rect = egui::Rect::from_min_size(
            egui::pos2(x, y),
            Vec2::new(24.0, 18.0),
        );
        let color = Color32::from_rgb(61, 90, 128);
        if index % 10 == 0 {
            let fade = 0.5 + 0.5 * (time * 2.0 + i * 0.1).sin();
            painter.rect_filled(rect, 4.0, color.gamma_multiply(fade));
        } else {
            painter.rect_filled(rect, 4.0, color);
        }
    }
}

/// The frames the workload ran (selfchecks use this to assert that a bench
/// frame really produced the claimed paint volume).
#[derive(Default)]
pub struct FrameCount(pub Cell<u64>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn w1_walks_the_full_tree_and_paints_every_leaf() {
        let ctx = Context::default();
        let (_primitives, shapes) = run_frame(&ctx, |ui| build_w1(ui, 0));
        // 10 section headings + 10_000 leaves, one paint op per label: the
        // full walk really happened.
        assert!(
            shapes >= SECTIONS * LEAVES_PER_SECTION,
            "expected the full 10k-label walk, got {shapes} shapes"
        );
    }

    #[test]
    fn w2_rewraps_under_jitter() {
        let ctx = Context::default();
        let (_, shapes_low) = run_frame(&ctx, |ui| build_w2(ui, 0));
        let (_, shapes_high) = run_frame(&ctx, |ui| build_w2(ui, JITTER_STEPS / 2));
        assert!(shapes_low >= PARAGRAPHS, "got {shapes_low} shapes");
        assert!(shapes_high >= PARAGRAPHS, "got {shapes_high} shapes");
        // The jitter schedule really sweeps ±5%.
        assert!((jitter(800.0, 0) - 760.0).abs() < 1e-3);
        assert!((jitter(800.0, JITTER_STEPS / 2) - 800.0).abs() < 1e-3);
    }

    #[test]
    fn w3_paints_all_animated_nodes() {
        let ctx = Context::default();
        let (_primitives, shapes) = run_frame(&ctx, |ui| build_w3(ui, 1.0));
        assert!(
            shapes >= ANIMATED_NODES,
            "expected {ANIMATED_NODES} paint ops, got {shapes}"
        );
    }
}
