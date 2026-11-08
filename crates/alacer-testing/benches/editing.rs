//! The pointer-editing benchmark (§10.3, ADR-0018): one hundred caret
//! placements routed through the pipeline on a 1,000-node arena — each a
//! queued Down/Up pair the coalescer hands the tick at its start, the
//! field answers with a seam hit-test and a caret move. The published
//! `editing/route_100_caret_clicks_1k` regression gate keeps the pointer
//! tier honest as the editing contract grows: placing a caret must stay
//! in the cost class of routing a pointer event — the full pre-order
//! walk, the resolved frame, the hit-test inversion — never a re-layout
//! of the arena or a re-materialization of its a11y tree.
//!
//! The bench lives in the harness crate because that is where the whole
//! pipeline is assembled with the real widget set (the DAG's harness
//! layer drives every surface): the measured widget is the shipped
//! `TextInput`, not a probe stand-in.

use alacer_core::events::{MouseButton, PointerEvent};
use alacer_core::geom::Point;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Panel, TextInput};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const CLICKS: usize = 100;
const FILLER: usize = 1000;
const PANEL_HEIGHT: f32 = 0.4;

/// The input's bounds, read through the a11y tree (§11.5 — the same query
/// a headless test targets through), then the click x positions spread
/// across the field's inner width so every placement moves the caret.
fn click_xs(harness: &TestHarness) -> (Vec<f32>, f32) {
    let bounds = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, "Search"))
        .and_then(|node| node.bounds)
        .expect("the input has resolved bounds");
    let y = bounds.origin.y + bounds.size.height * 0.5;
    let inner = bounds.size.width - 8.0;
    let step = inner / (alacer_core::num::usize_to_f32(CLICKS) - 1.0);
    let xs = (0..CLICKS)
        .map(|i| bounds.origin.x + 4.0 + step * alacer_core::num::usize_to_f32(i))
        .collect();
    (xs, y)
}

fn bench_editing_caret_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("editing");
    group.throughput(Throughput::Elements(
        u64::try_from(CLICKS).expect("non-negative"),
    ));

    group.bench_function("route_100_caret_clicks_1k", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        let root = harness.mount(Column::new().boxed());
        // The input is the first child; the 1,000-node filler follows it —
        // the routing walk pays for the whole arena on every queued event.
        let _ = harness.add_child(root, TextInput::new("Search").boxed());
        for _ in 0..FILLER {
            let _ = harness.add_child(
                root,
                Panel::new(PANEL_HEIGHT, alacer_widgets::theme::PANEL).boxed(),
            );
        }
        harness.settle();
        let (xs, y) = click_xs(&harness);

        b.iter(|| {
            for x in &xs {
                harness.queue_pointer(PointerEvent::Down {
                    position: Point::new(*x, y),
                    button: MouseButton::Left,
                });
                harness.queue_pointer(PointerEvent::Up {
                    position: Point::new(*x, y),
                    button: MouseButton::Left,
                });
            }
            let report = harness.tick();
            black_box(report);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_editing_caret_route);
criterion_main!(benches);
