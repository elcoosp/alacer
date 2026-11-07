//! The W3 workload (§14.2, ADR-0012): the animation loop — 1k nodes, a
//! per-frame translation advance plus a 10% paint churn, through the *real*
//! ten-pass pipeline and the software rasterizer with damage scissoring.
//!
//! The bench lives in the harness crate because that is where the whole
//! pipeline is assembled (the DAG's harness layer drives every surface; the
//! wgpu backend measures on the reference rig, not in CI). The mechanism
//! under test: journal replay + translation-only updates (§8.2, §9.1) — a
//! scrolling animation must never re-measure.
//!
//! The 120 fps @ 1k nodes budget is a reference-rig number (published as
//! data, ADR-0011's honesty rule); CI enforces the same-host regression
//! gate and the zero-measure scrolling test gate.

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::geom::{Point, Size, Vector};
use alacer_core::style::{MeasureSpec, Style};
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

const SECTIONS: usize = 10;
const LEAVES_PER_SECTION: usize = 100;
const VIEWPORT: Size = Size::new(800.0, 600.0);
const SCROLL_STEP: f32 = 3.0;
const ROW_HEIGHT: f32 = 16.0;

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

/// An animation leaf: paint color fades per frame (the opacity churn); one
/// section's leaves churn per frame (10% of the tree).
struct AnimationLeaf {
    color: Arc<Mutex<Color>>,
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
    fn paint(&self, ctx: &mut alacer_core::scene::PaintCtx<'_>) {
        let color = *self.color.lock().expect("leaf color");
        ctx.push_quad(alacer_core::scene::Quad::filled(
            alacer_core::geom::Rect::new(Point::zero(), ctx.size()),
            color,
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

/// The animation rig: pipeline + scroller offset + leaf color cells.
struct Rig {
    harness: TestHarness,
    offset: Rc<RefCell<f32>>,
    colors: Vec<Arc<Mutex<Color>>>,
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
    let mut colors = Vec::with_capacity(SECTIONS * LEAVES_PER_SECTION);
    for _ in 0..SECTIONS {
        let section = harness
            .pipeline_mut()
            .add_child(root, Box::new(SectionColumn));
        for _ in 0..LEAVES_PER_SECTION {
            let color = Arc::new(Mutex::new(Color::rgb(37, 99, 235)));
            let _ = harness.pipeline_mut().add_child(
                section,
                Box::new(AnimationLeaf {
                    color: Arc::clone(&color),
                }),
            );
            colors.push(color);
        }
    }
    Rig {
        harness,
        offset: handle,
        colors,
        content_height: content_height(),
    }
}

/// One animation frame: advance the translation, churn one section's paint
/// (10% of the tree), tick the real pipeline, rasterize with damage.
fn animation_frame(rig: &mut Rig, frame: usize) {
    let mut offset = rig.offset.borrow_mut();
    *offset += SCROLL_STEP;
    let max = (rig.content_height - VIEWPORT.height).max(0.0);
    if *offset > max {
        *offset = 0.0;
    }
    drop(offset);

    // The opacity churn: one section's leaves fade (10% of nodes).
    let section = frame % SECTIONS;
    let alpha = u8::try_from((frame * 17) % 255).unwrap_or(255);
    let fading = Color::rgb(37, 99, 235).with_alpha(alpha);
    for color in rig.colors[section * LEAVES_PER_SECTION..(section + 1) * LEAVES_PER_SECTION].iter()
    {
        *color.lock().expect("leaf color") = fading;
    }
    // One tick + one damaged raster per frame (frame_pixels ticks the
    // pipeline itself — the animation frame's whole cost, honestly).
    let _ = rig.harness.frame_pixels();
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
        })
    });
    group.finish();
}

criterion_group!(benches, bench_w3);
criterion_main!(benches);
