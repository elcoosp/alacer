//! The W3 workload (§14.2, ADR-0012): the animation loop — 1k nodes, a
//! per-frame translation advance plus a 10% churn, through the *real*
//! ten-pass pipeline and the software rasterizer with damage scissoring.
//!
//! The bench lives in the harness crate because that is where the whole
//! pipeline is assembled (the DAG's harness layer drives every surface; the
//! wgpu backend measures on the reference rig, not in CI). The mechanism
//! under test: journal replay + translation-only updates (§8.2, §9.1) — a
//! scrolling animation must never re-measure — with the 10% churn riding
//! the compose tier (§14.2's W3 mechanism): the fade is the compose state
//! the pass folds into the recorded alpha at record time, written through
//! the mutate seam, never a per-frame color rewrite.
//!
//! The smoke gate below refuses to price a fake: three animated frames run
//! before the clock, and the frame report plus the recorded scene must
//! show the churn actually reaching the pipeline (compose ran, paint
//! re-recorded, damage produced, no measure call, the folded alpha in the
//! scene) — a mechanism that idles or fakes the fade panics there.
//!
//! The 120 fps @ 1k nodes budget is a reference-rig number (published as
//! data, ADR-0011's honesty rule); CI enforces the same-host regression
//! gate and the zero-measure scrolling test gate.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_core::color::Color;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::geom::{Point, Size, Vector};
use alacer_core::num::f32_to_u8;
use alacer_core::passes::PassKind;
use alacer_core::scene::{Compose, Primitive};
use alacer_core::style::{MeasureSpec, Style};
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::cell::RefCell;
use std::rc::Rc;

const SECTIONS: usize = 10;
const LEAVES_PER_SECTION: usize = 100;
const VIEWPORT: Size = Size::new(800.0, 600.0);
const SCROLL_STEP: f32 = 3.0;
const ROW_HEIGHT: f32 = 16.0;
/// The leaf's paint color, fixed for the whole run: the fade is the
/// compose tier's fact, never a per-frame color rewrite.
const LEAF_COLOR: Color = Color::rgb(37, 99, 235);

/// A scroll container whose offset the animation advances per frame
/// (the List's compose-translation mechanism, in bench-local form).
struct BenchScroller {
    offset: Rc<RefCell<f32>>,
}

impl BenchScroller {
    fn handle(&self) -> Rc<RefCell<f32>> {
        Rc::clone(&self.offset)
    }
}

impl Widget for BenchScroller {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::ScrollArea
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("scroller")
    }
    fn style(&self) -> Style {
        Style {
            clip_overflow: true,
            size: alacer_core::style::Size2 {
                width: None,
                height: Some(VIEWPORT.height),
            },
            ..Style::column()
        }
    }
    fn scroll_offset(&self) -> Vector {
        Vector::new(0.0, *self.offset.borrow())
    }
    fn is_scroll_container(&self) -> bool {
        true
    }
    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, _event: &PointerEvent) -> EventResponse {
        EventResponse::Ignored
    }
}

/// An animation leaf: a fixed-color quad whose fade rides the compose tier
/// (§14.2's W3 mechanism) — the compose state the pass folds into the
/// recorded alpha at record time. One section's leaves churn per frame
/// (10% of the tree); the paint vocabulary never does.
struct AnimationLeaf {
    compose: Compose,
}

impl AnimationLeaf {
    /// The compose write the app's churn rides (the `Label::set_compose`
    /// shape at the leaf): reports whether the state moved, so the caller
    /// requests `PAINT` only when the answer is `true` — the idle-frame
    /// rule at the compose tier.
    fn set_compose(&mut self, compose: Compose) -> bool {
        if self.compose == compose {
            return false;
        }
        self.compose = compose;
        true
    }
}

impl Widget for AnimationLeaf {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("leaf")
    }
    fn style(&self) -> Style {
        Style {
            size: alacer_core::style::Size2 {
                width: Some(60.0),
                height: Some(ROW_HEIGHT),
            },
            ..Style::default()
        }
    }
    /// The compose tier's ride: the leaf's fade joins the pass's
    /// inheritance walk, the fold landing in the recorded alpha.
    fn compose(&self) -> Compose {
        self.compose
    }
    fn paint(&self, ctx: &mut alacer_core::scene::PaintCtx<'_>) {
        ctx.push_quad(alacer_core::scene::Quad::filled(
            alacer_core::geom::Rect::new(Point::zero(), ctx.size()),
            LEAF_COLOR,
        ));
    }
    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(alacer_core::geom::Size::new(
            60.0, ROW_HEIGHT,
        )))
    }
}

struct SectionColumn;

impl Widget for SectionColumn {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Container
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("section")
    }
    fn style(&self) -> Style {
        Style::column()
    }
}

/// The animation rig: pipeline + scroller node + the leaf nodes the churn
/// writes through the mutate seam.
struct Rig {
    harness: TestHarness,
    scroller: NodeId,
    offset: Rc<RefCell<f32>>,
    leaves: Vec<NodeId>,
    content_height: f32,
}

fn content_height() -> f32 {
    alacer_core::num::u32_to_f32(u32::try_from(SECTIONS * LEAVES_PER_SECTION).unwrap_or(u32::MAX))
        * ROW_HEIGHT
}

fn build() -> Rig {
    let mut harness = TestHarness::new(VIEWPORT.width, VIEWPORT.height);
    let offset = Rc::new(RefCell::new(0.0));
    let scroller = BenchScroller {
        offset: Rc::clone(&offset),
    };
    let handle = scroller.handle();
    let root = harness.mount(Box::new(scroller));
    let mut leaves = Vec::with_capacity(SECTIONS * LEAVES_PER_SECTION);
    for _ in 0..SECTIONS {
        let section = harness
            .pipeline_mut()
            .add_child(root, Box::new(SectionColumn));
        for _ in 0..LEAVES_PER_SECTION {
            let leaf = harness.pipeline_mut().add_child(
                section,
                Box::new(AnimationLeaf {
                    compose: Compose::IDENTITY,
                }),
            );
            leaves.push(leaf);
        }
    }
    let mut rig = Rig {
        harness,
        scroller: root,
        offset: handle,
        leaves,
        content_height: content_height(),
    };
    smoke_check(&mut rig);
    rig
}

/// The churn half of one animation frame: the translation advance plus the
/// 10% fade write — everything the app side does before the frame presents.
fn advance_and_churn(rig: &mut Rig, frame: usize) {
    let mut offset = rig.offset.borrow_mut();
    *offset += SCROLL_STEP;
    let max = (rig.content_height - VIEWPORT.height).max(0.0);
    if *offset > max {
        *offset = 0.0;
    }
    drop(offset);

    // The programmatic scroll surfaces as a PAINT mask — the seam pass 7
    // documents ("paint must see translated bounds", §8.2): the compose
    // pass polls the offset, the bounds translate without a measure call.
    rig.harness
        .pipeline_mut()
        .arena_mut()
        .merge_mask(rig.scroller, Change::PAINT);

    // The opacity churn: one section's leaves fade (10% of nodes) — the
    // compose tier's own fact, written through the mutate seam the app's
    // code rides, the paint vocabulary never touched. The fade's alpha
    // arithmetic matches the old color path exactly (the same u8 alpha,
    // the same 10% of nodes), so the workload is unchanged; the tier is.
    let section = frame % SECTIONS;
    let alpha = u8::try_from((frame * 17) % 255).unwrap_or(255);
    let fade = Compose::faded(fade_opacity(alpha));
    let section_leaves =
        &rig.leaves[section * LEAVES_PER_SECTION..(section + 1) * LEAVES_PER_SECTION];
    for leaf in section_leaves {
        let mut guard = rig.harness.pipeline_mut().arena_mut().mutate(*leaf);
        let moved = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<AnimationLeaf>()
            .is_some_and(|leaf| leaf.set_compose(fade));
        if moved {
            guard.request(Change::PAINT);
        }
    }
}

/// One animation frame: churn, then tick the real pipeline and rasterize
/// (`frame_pixels` runs the tick itself — the animation frame's whole cost).
fn animation_frame(rig: &mut Rig, frame: usize) {
    advance_and_churn(rig, frame);
    let _ = rig.harness.frame_pixels();
}

/// The compose state a leaf's fade names: the base color's own alpha
/// carried as the compose opacity, so the fold lands the same u8 alpha the
/// color path wrote — the same workload, the new tier.
fn fade_opacity(alpha: u8) -> f32 {
    f32::from(alpha) / f32::from(u8::MAX)
}

/// The compose fold's recorded alpha for a base 255 quad faded by `opacity`
/// — the same arithmetic the paint context applies at record time
/// (`f32_to_u8(f32::from(255) * opacity)`), computed here so the smoke gate
/// asserts the fold, not a constant.
fn folded_alpha(opacity: f32) -> u8 {
    f32_to_u8(f32::from(u8::MAX) * opacity)
}

/// The smoke gate the bench refuses to start without: three animated frames
/// through the real pipeline, then the animation frame's contract asserted
/// on the `FrameReport` and the recorded scene. A mechanism that fakes the
/// churn — idle ticks, or writes the pipeline never re-records — panics
/// here, before the clock starts.
fn smoke_check(rig: &mut Rig) {
    rig.harness.settle();
    let measures_before = rig.harness.measure_calls();
    for frame in 0..3 {
        advance_and_churn(rig, frame);
        let report = rig.harness.tick();
        assert!(
            !report.is_idle(),
            "animation frame {frame} idled — the churn never reached the pipeline"
        );
        assert!(
            report.pass(PassKind::Compose).is_some_and(|run| run.ran),
            "animation frame {frame} skipped the compose pass — the translation/fade tier never ran"
        );
        assert!(
            report.paint_ran && !report.damage.is_empty(),
            "animation frame {frame} replayed a cached scene — nothing re-recorded, no damage"
        );
        assert!(
            !report.layout_ran,
            "animation frame {frame} re-measured — the compose tier's own contract forbids it"
        );
        assert_eq!(
            rig.harness.measure_calls(),
            measures_before,
            "animated frames grew measure calls — a layout ride the compose tier forbids"
        );
        // The fold is in the recorded scene: every section the smoke has
        // faded so far records its fade folded into the quad alpha (the
        // same u8 the color path wrote), every untouched section records
        // the base color untouched.
        let active = frame + 1;
        let mut base = 0_usize;
        let mut faded = vec![0_usize; usize::from(u8::MAX) + 1];
        for primitive in &report.scene.primitives {
            if let Primitive::Quad(quad) = primitive {
                match quad.color.a {
                    u8::MAX => base += 1,
                    a => faded[usize::from(a)] += 1,
                }
            }
        }
        let expected_total = SECTIONS * LEAVES_PER_SECTION;
        let total = base + faded.iter().sum::<usize>();
        assert_eq!(
            total, expected_total,
            "the recorded scene holds {total} leaf quads, expected {expected_total}"
        );
        for section in 0..active {
            let alpha = u8::try_from((section * 17) % 255).unwrap_or(255);
            let expected = folded_alpha(fade_opacity(alpha));
            assert_eq!(
                faded[usize::from(expected)],
                LEAVES_PER_SECTION,
                "section {section} records {} quads at the folded alpha {expected}, found {}",
                LEAVES_PER_SECTION,
                faded[usize::from(expected)]
            );
        }
        let expected_base = expected_total - active * LEAVES_PER_SECTION;
        assert_eq!(
            base, expected_base,
            "the untouched sections record the base alpha untouched ({base} quads, expected {expected_base})"
        );
    }
    // The exact path the loop times runs end to end.
    animation_frame(rig, 3);
}

fn bench_w3(c: &mut Criterion) {
    let mut group = c.benchmark_group("w3");
    group.throughput(Throughput::Elements(
        u64::try_from(SECTIONS * LEAVES_PER_SECTION).expect("fits u64"),
    ));
    group.bench_function("animation_frame_1k", |b| {
        let mut rig = build();
        let mut frame = 0_usize;
        b.iter(|| {
            animation_frame(&mut rig, frame);
            frame += 1;
        });
    });
    group.finish();
}

criterion_group!(benches, bench_w3);
criterion_main!(benches);
