//! The Masonry side of the cross-framework comparison (M7, ADR-0013).
//!
//! Three shared workloads (architecture §14.2), implemented with Masonry's
//! public API — the retained pass-contract design the Alacer architecture
//! cites as its reference (§7). Everything runs headlessly through
//! `RenderRoot`, the same pipeline the `masonry_winit` app drives:
//!
//! - **W1** (`w1_churn_10k`): 10k `SizedBox` leaves in nested `Flex`
//!   containers under 10%/frame style churn (`SizedBox::set_width` on the
//!   rotated churn window — the WidgetMut discipline masonry requires).
//!   One frame = the edits + `RenderRoot::redraw()`, which runs the real
//!   pass pipeline: update, layout, compose, paint-scene build, and the
//!   accessibility tree update.
//! - **W2** (`w2_text_jitter_1k`): 1k `Label`s with
//!   `LineBreaking::WordWrap`, re-wrapped by a ±5% root-width jitter.
//!   Masonry's parley shaping does its real thing: a changed constraint
//!   re-breaks the lines, an unchanged one re-uses `last_max_advance`.
//! - **W3** (`w3_animation_1k`): 1k nodes whose per-frame translation is a
//!   `WidgetMut::set_transform` — Masonry's compose-level answer to
//!   translation-only updates, the same mechanism class Alacer's W3
//!   exercises.
//!
//! Scope, per the ADR's asymmetry rule (§3): a frame here is the full CPU
//! pass pipeline *including* the vello scene build, but *excluding* vello's
//! rasterization — that step needs a wgpu device, which the declared rig
//! (headless, no GPU) does not carry. The exclusion is labeled in
//! `results.json`, and it is conservative: Masonry's row measures less
//! than a full frame, so a beat won against it is won a fortiori.
//!
//! The no-strawman note (README): Masonry's production answer for long
//! lists is `Portal` (scroll + `virtual_scroll`'s reuse), which sidesteps
//! the tree-wide mechanism under test; like egui's row virtualization, it
//! is documented rather than measured.

use masonry::app::{RenderRoot, RenderRootOptions, WindowSizePolicy};
use masonry::core::{NewWidget, Properties, Widget, WindowEvent};
use masonry::dpi::PhysicalSize;
use masonry::kurbo::Affine;
use masonry::properties::types::Length;
use masonry::properties::LineBreaking;
use masonry::theme::default_property_set;
use masonry::widgets::{Flex, Label, SizedBox};
use std::sync::Arc;

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
pub const CHURN_LEAVES: u64 = 1000;
/// The jitter sweep: ±5% across 21 frames, as §14.2 W2 defines it.
pub const JITTER_STEPS: u64 = 21;

pub const VIEWPORT_W: u32 = 800;
pub const VIEWPORT_H: u32 = 600;
const PARAGRAPH: &str = "The quick brown fox keeps re-wrapping because the \
container cannot make up its mind; every width asks the shaper a slightly \
different question.";

/// A headless `RenderRoot` — no window, no event loop, the real pass
/// pipeline. `use_system_fonts` rides the host font set; the declared rig
/// records what that host carries.
pub fn render_root(root: NewWidget<impl Widget + ?Sized>) -> RenderRoot {
    let mut tree = RenderRoot::new(
        root,
        |_| {},
        RenderRootOptions {
            default_properties: Arc::new(default_property_set()),
            use_system_fonts: true,
            size_policy: WindowSizePolicy::User,
            size: PhysicalSize::new(VIEWPORT_W, VIEWPORT_H),
            scale_factor: 1.0,
            test_font: None,
        },
    );
    // Activate the accessibility tree (a real client would): every measured
    // frame then includes the accessibility pass — the scope the results
    // file documents.
    tree.handle_window_event(WindowEvent::EnableAccessTree);
    tree
}

/// The jitter schedule shared by the W2 benches: a ±5% sweep across
/// `JITTER_STEPS` frames around `base` (the same schedule the egui side
/// and the Alacer W2 bench use).
pub fn jitter(base: f64, frame: u64) -> f64 {
    let phase = (frame % JITTER_STEPS) as f64 / (JITTER_STEPS - 1) as f64;
    base * (1.0 + 0.05 * (2.0 * phase - 1.0))
}

/// The churn window for frame `frame`: the leaf index range it covers.
pub fn churn_window(frame: u64) -> std::ops::Range<u64> {
    let start = (frame % SECTIONS as u64) * CHURN_LEAVES;
    start..start + CHURN_LEAVES
}

/// W1: 10 sections of 1k fixed-size leaves. Built once; the bench measures
/// the churn frames, not the construction.
pub fn make_w1_root() -> RenderRoot {
    let mut root = Flex::column();
    for _ in 0..SECTIONS {
        let mut section = Flex::column();
        for _ in 0..LEAVES_PER_SECTION {
            section = section.with_child(NewWidget::new(SizedBox::empty()));
        }
        root = root.with_child(NewWidget::new(section));
    }
    let mut tree = render_root(NewWidget::new(root));
    // Give every leaf a real size (unmeasured setup, as on the Alacer
    // side: the bench measures churn frames, not construction).
    tree.edit_base_layer(|mut base| {
        let mut root = base.downcast::<Flex>();
        for section in 0..SECTIONS {
            if let Some(mut section_dyn) = Flex::child_mut(&mut root, section) {
                let mut section_flex = section_dyn.downcast::<Flex>();
                for leaf in 0..LEAVES_PER_SECTION {
                    if let Some(mut leaf_dyn) = Flex::child_mut(&mut section_flex, leaf) {
                        let mut sized = leaf_dyn.downcast::<SizedBox>();
                        SizedBox::set_width(&mut sized, Length::px(40.0));
                        SizedBox::set_height(&mut sized, Length::px(18.0));
                    }
                }
            }
        }
    });
    tree.redraw(); // settle the initial layout
    tree
}

/// One W1 churn frame: mutate the 10% window's widths, then run the full
/// pass pipeline through `redraw()`. Returns the emitted vello scene and
/// the accessibility tree update (the frame's outputs — the bench
/// `black_box`es both so the optimizer cannot elide the pipeline).
pub fn w1_frame(tree: &mut RenderRoot, frame: u64) -> (masonry::vello::Scene, Option<TreeUpdate>) {
    let window = churn_window(frame);
    let width = 40.0 + (frame % 8) as f64;
    tree.edit_base_layer(|mut base| {
        let mut root = base.downcast::<Flex>();
        for global in window {
            let section = (global as usize) / LEAVES_PER_SECTION;
            let leaf = (global as usize) % LEAVES_PER_SECTION;
            if let Some(mut section_dyn) = Flex::child_mut(&mut root, section) {
                let mut section_flex = section_dyn.downcast::<Flex>();
                let leaf_dyn = Flex::child_mut(&mut section_flex, leaf);
                if let Some(mut leaf_dyn) = leaf_dyn {
                    let mut sized = leaf_dyn.downcast::<SizedBox>();
                    SizedBox::set_width(&mut sized, Length::px(width));
                }
            }
        }
    });
    tree.redraw()
}

/// W2: 1k word-wrapping labels under a root-width jitter.
pub fn make_w2_root() -> RenderRoot {
    let mut column = Flex::column();
    for index in 0..PARAGRAPHS {
        let label = NewWidget::new_with_props(
            Label::new(format!("{PARAGRAPH} ({index})")),
            Properties::new().with(LineBreaking::WordWrap),
        );
        column = column.with_child(label);
    }
    let wrapped_root = NewWidget::new(SizedBox::new(NewWidget::new(column)));
    let mut tree = render_root(wrapped_root);
    tree.redraw(); // settle the initial layout
    tree
}

/// One W2 jitter frame: re-wrap the whole column at the jittered width.
pub fn w2_frame(tree: &mut RenderRoot, frame: u64) -> (masonry::vello::Scene, Option<TreeUpdate>) {
    let width = jitter(f64::from(VIEWPORT_W), frame);
    tree.edit_base_layer(|mut base| {
        let mut root = base.downcast::<SizedBox>();
        SizedBox::set_width(&mut root, Length::px(width));
    });
    tree.redraw()
}

/// W3: 1k animatable nodes under per-frame translation.
pub fn make_w3_root() -> RenderRoot {
    let mut root = Flex::column();
    for _ in 0..ANIMATED_NODES {
        root = root.with_child(NewWidget::new(SizedBox::empty()));
    }
    let mut tree = render_root(NewWidget::new(root));
    tree.edit_base_layer(|mut base| {
        let mut flex = base.downcast::<Flex>();
        for index in 0..ANIMATED_NODES {
            if let Some(mut leaf_mut) = Flex::child_mut(&mut flex, index) {
                let mut sized = leaf_mut.downcast::<SizedBox>();
                SizedBox::set_width(&mut sized, Length::px(24.0));
                SizedBox::set_height(&mut sized, Length::px(18.0));
            }
        }
    });
    tree.redraw(); // settle the initial layout
    tree
}

/// One W3 animation frame: advance every node's translation (compose-level
/// transform updates, Masonry's translation-only path).
pub fn w3_frame(tree: &mut RenderRoot, frame: u64) -> (masonry::vello::Scene, Option<TreeUpdate>) {
    let time = frame as f64 / 60.0;
    tree.edit_base_layer(|mut base| {
        let mut flex = base.downcast::<Flex>();
        for index in 0..ANIMATED_NODES {
            if let Some(mut leaf_mut) = Flex::child_mut(&mut flex, index) {
                let mut sized = leaf_mut.downcast::<SizedBox>();
                let x = (time * 60.0 + index as f64 * 7.0) % (f64::from(VIEWPORT_W) - 24.0);
                sized.set_transform(Affine::translate((x, 0.0)));
            }
        }
    });
    tree.redraw()
}

/// The accessibility tree update type `redraw()` hands back (re-exported
/// here so the benches do not name masonry internals).
pub type TreeUpdate = masonry::accesskit::TreeUpdate;

#[cfg(test)]
mod tests {
    use super::*;
    use masonry::core::WidgetMut;

    fn count_leaves(root: &mut WidgetMut<'_, Flex>) -> usize {
        let mut total = 0;
        for section in 0..SECTIONS {
            if let Some(mut section_dyn) = Flex::child_mut(root, section) {
                let mut section_flex = section_dyn.downcast::<Flex>();
                for leaf in 0..LEAVES_PER_SECTION {
                    if Flex::child_mut(&mut section_flex, leaf).is_some() {
                        total += 1;
                    }
                }
            }
        }
        total
    }

    #[test]
    fn w1_holds_ten_thousand_leaves_and_runs_the_pipeline() {
        let mut tree = make_w1_root();
        tree.edit_base_layer(|mut base| {
            let mut root = base.downcast::<Flex>();
            assert_eq!(
                count_leaves(&mut root),
                SECTIONS * LEAVES_PER_SECTION,
                "the full 10k tree is really there"
            );
        });
        let (scene, tree_update) = w1_frame(&mut tree, 0);
        assert!(tree_update.is_some(), "the a11y pass ran on the churn frame");
        std::hint::black_box(scene);
    }

    #[test]
    fn w2_rewraps_under_jitter() {
        let mut tree = make_w2_root();
        let (scene, tree_update) = w2_frame(&mut tree, 0);
        assert!(tree_update.is_some());
        std::hint::black_box(scene);
        // The jitter schedule really sweeps ±5%.
        assert!((jitter(800.0, 0) - 760.0).abs() < 1e-9);
        assert!((jitter(800.0, JITTER_STEPS / 2) - 800.0).abs() < 1e-9);
    }

    #[test]
    fn w3_translates_every_node() {
        let mut tree = make_w3_root();
        let (scene, tree_update) = w3_frame(&mut tree, 0);
        assert!(tree_update.is_some());
        std::hint::black_box(scene);
    }
}
