//! The gesture benchmark (§13.3, ADR-0052): the value tier's drag
//! driven through the pipeline on a 1,000-node arena with the causal
//! observer installed — a press arms the drag, one hundred dispatched
//! moves stride the thumb, a release lands it, and the tick routes the
//! whole gesture on the shipped composition (harness + AccessKit sink),
//! not a probe stand-in.
//!
//! The published regression gate keeps the trail honest as the log
//! grows: a drag must stay in the cost class of the event route itself
//! — the pointer route's walks, one stamp `Cell` read per routed event,
//! and exactly two bounded-queue pushes per gesture (the phases) —
//! never a per-move observation, a re-layout of the arena, or a
//! re-materialization of its a11y tree (the flood rule the kill
//! criteria pre-registered: the moves note nothing).
//!
//! The bench lives in the harness crate because that is where the whole
//! pipeline is assembled with the real widget set (the DAG's harness
//! layer drives every surface).

use alacer_core::events::{MouseButton, PointerEvent};
use alacer_core::geom::{Point, Size};
use alacer_core::inspect::{InspectEvent, InspectObserver};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label, Panel, Slider};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const MOVES: usize = 100;
const FILLER: usize = 1000;
const PANEL_HEIGHT: f32 = 0.4;

/// The installed observer the trail rides: the drag's phase facts
/// stream into a bounded ring — the bench proves the route's cost
/// *with* the trail on (the A7 idle contract is the m46 suite's own
/// no-observer pin; the devtools path's cost class is this gate).
struct CountingObserver {
    gestures: usize,
}

impl InspectObserver for CountingObserver {
    fn observe(&mut self, event: InspectEvent) {
        if let InspectEvent::Gesture(_) = event {
            self.gestures += 1;
        }
    }
}

fn bench_gesture_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("gestures");
    group.throughput(Throughput::Elements(
        u64::try_from(MOVES).expect("non-negative"),
    ));

    group.bench_function("route_100_drag_moves_1k", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        harness
            .pipeline_mut()
            .set_observer(Box::new(CountingObserver { gestures: 0 }));
        let root = harness.mount(Column::new().boxed());
        let _ = harness.add_child(root, Label::new("The drag's story").boxed());
        for i in 0..FILLER {
            let _ = harness.add_child(
                root,
                Panel::new(PANEL_HEIGHT, alacer_widgets::theme::PANEL).boxed(),
            );
            let _ = i;
        }
        let node = harness.add_child(
            root,
            Slider::new("Volume", 0.0, 100.0)
                .value(0.0)
                .step(1.0)
                .boxed(),
        );
        harness.settle();

        // The slider's track center, read off the laid-out bounds — the
        // position the gesture aims the pointer at.
        let bounds = harness.bounds(node);
        let center = Point::new(
            bounds.min_x() + bounds.width() * 0.5,
            bounds.min_y() + bounds.height() * 0.5,
        );
        let viewport = Size::new(800.0, 600.0);

        // One full drag per iteration: the press arms the gesture (one
        // `Started` record), one hundred dispatched moves stride the
        // thumb (each routed through the capture, none observed), the
        // release lands it (one `Ended` record), and one tick drains
        // the two facts — the gesture's whole story, priced once.
        let mut x = center.x;
        b.iter(|| {
            x = center.x;
            harness
                .pipeline_mut()
                .dispatch_pointer(&PointerEvent::Down {
                    position: center,
                    button: MouseButton::Left,
                    click_count: 1,
                });
            let strides = f32::from(u16::try_from(MOVES).expect("the stride count is small"));
            for step in 0..MOVES {
                x += (bounds.width() - 16.0) / strides;
                let direction = if step % 2 == 0 { 1.0 } else { -1.0 };
                let position = Point::new(
                    (center.x + direction * (x - center.x)).clamp(bounds.min_x(), bounds.max_x()),
                    center.y,
                );
                harness
                    .pipeline_mut()
                    .dispatch_pointer(&PointerEvent::Move { position });
            }
            harness.pipeline_mut().dispatch_pointer(&PointerEvent::Up {
                position: Point::new(x, center.y),
                button: MouseButton::Left,
            });
            black_box(harness.tick());
            let _ = viewport;
        });
    });

    group.finish();
}

criterion_group!(benches, bench_gesture_route);
criterion_main!(benches);
