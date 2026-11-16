//! The scroll benchmark (§8.2, ADR-0092; the horizontal cut since M89,
//! ADR-0093): the bring-into-view door driven through the pipeline on
//! 1,000-node lists — one hundred asks alternating between the stack's
//! two ends, each ask riding one tick of the real frame currency (the
//! container's `PAINT | A11Y` merge, compose translating the moved
//! descendants, the paint pass re-emitting their layers, the a11y pass
//! re-announcing their bounds).
//!
//! The published regression gates keep the door honest as the tier
//! grows: a node-priced ask must stay in the cost class of an
//! offset-priced one — the bounds ledger's own difference pricing the
//! ask, the container's clamp arithmetic answering it, the compose
//! translation doing the moving — never a re-layout of the arena (the
//! translation-only workhorse, §8.2: a scroll frame measures nothing).
//! The horizontal case (M89) holds the two-axis ask to the same class:
//! the wide rows re-pin the x half on every landing — a row wider than
//! the viewport always leaves the other edge sticking out — so both
//! halves price, clamp, and compose each ask.
//!
//! The bench lives in the harness crate because that is where the
//! whole pipeline is assembled with the real widget set (the DAG's
//! harness layer drives every surface).

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect};
use alacer_core::num::usize_to_f32;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{Size2, Style};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::{List, Panel};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const ROUTES: usize = 100;
const FILLER: usize = 1000;
const ROW_HEIGHT: f32 = 32.0;
const VIEWPORT: f32 = 600.0;

/// The horizontal case's geometry: 1,000 px rows inside an 800 px
/// viewport — every ask leaves one x edge sticking out, so the x half
/// re-pins each landing (right edge in, left edge out, and back).
const ROW_WIDTH: f32 = 1000.0;
const TWO_AXIS_VIEWPORT_WIDTH: f32 = 800.0;

/// A solid row of its own declared width — the horizontal case's
/// content: wider than the viewport, so the door prices a real x half
/// on every ask.
struct WideRow {
    height: f32,
    width: f32,
}

impl WideRow {
    fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Widget for WideRow {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("row")
    }
    fn style(&self) -> Style {
        Style {
            size: Size2 {
                width: Some(self.width),
                height: Some(self.height),
            },
            ..Style::default()
        }
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), ctx.size()),
            color: Color::rgb(37, 99, 235),
            corner_radius: 0.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

fn bench_scroll_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("scroll");
    group.throughput(Throughput::Elements(
        u64::try_from(ROUTES).expect("non-negative"),
    ));

    group.bench_function("route_100_bring_into_view_1k", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        let list = List::new(VIEWPORT);
        let handle = list.handle();
        let list_node = harness.mount(list.boxed());
        for _ in 0..FILLER {
            let _ = harness.add_child(
                list_node,
                Panel::new(ROW_HEIGHT, alacer_widgets::theme::PANEL).boxed(),
            );
        }
        handle.set_content_height(usize_to_f32(FILLER) * ROW_HEIGHT);
        harness.settle();

        // The stack's two ends: the asks alternate between them, so
        // every landing moves the whole visible band — the compose
        // translation, the paint re-emission, and the a11y bounds
        // refresh priced at their real magnitude each tick.
        let (top, bottom) = {
            let arena = harness.pipeline().arena();
            let children = arena.children(list_node);
            (
                *children.first().expect("the top row is mounted"),
                *children.last().expect("the bottom row is mounted"),
            )
        };
        b.iter(|| {
            for index in 0..ROUTES {
                let target = if index % 2 == 0 { bottom } else { top };
                black_box(harness.pipeline_mut().scroll_into_view(target));
                black_box(harness.tick());
            }
        });
    });

    // The horizontal cut (M89, ADR-0093): the same 1,000-node stack,
    // the rows wider than the two-axis viewport. Every ask prices
    // both halves from the ledger — the y half alternates the stack's
    // two ends, the x half re-pins on every landing (a row wider than
    // the viewport always leaves the other edge sticking out) — so
    // the two-axis door's own price is the scrolled frame's, and the
    // gate keeps it in the offset-priced class as the axis grows.
    group.bench_function("route_100_horizontal_bring_into_view_1k", |b| {
        let mut harness = TestHarness::new(900.0, 600.0);
        let mut list = List::two_axis(TWO_AXIS_VIEWPORT_WIDTH, VIEWPORT);
        // The rows keep the width their styles declare, never the
        // stretch the default column would give them.
        list.style_mut().align = alacer_core::style::CrossAxisAlignment::Start;
        let handle = list.handle();
        let list_node = harness.mount(list.boxed());
        for _ in 0..FILLER {
            let _ = harness.add_child(
                list_node,
                WideRow {
                    height: ROW_HEIGHT,
                    width: ROW_WIDTH,
                }
                .boxed(),
            );
        }
        handle.set_content_height(usize_to_f32(FILLER) * ROW_HEIGHT);
        handle.set_content_width(ROW_WIDTH);
        harness.settle();

        let (top, bottom) = {
            let arena = harness.pipeline().arena();
            let children = arena.children(list_node);
            (
                *children.first().expect("the top row is mounted"),
                *children.last().expect("the bottom row is mounted"),
            )
        };
        b.iter(|| {
            for index in 0..ROUTES {
                let target = if index % 2 == 0 { bottom } else { top };
                black_box(harness.pipeline_mut().scroll_into_view(target));
                black_box(harness.tick());
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_scroll_route);
criterion_main!(benches);
