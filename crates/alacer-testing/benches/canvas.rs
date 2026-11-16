//! The canvas benchmark (M82): the custom-paint route driven through the
//! pipeline on 100 canvases — one repaint mark per surface, each frame
//! re-recording every painter's primitives through the paint seam into the
//! scene journal on the shipped composition (harness + AccessKit sink), not
//! a probe stand-in.
//!
//! The published regression gate keeps the route honest as the capability
//! grows: a canvas frame costs the paint pass's own record — the closure
//! runs, the primitives land in the journal, and the software renderer
//! materializes them; nothing else moves (the idle-frame property the
//! hoisted painter's identity rule leans on).
//!
//! The bench lives in the harness crate because that is where the whole
//! pipeline is assembled with the real widget set (the DAG's harness
//! layer drives every surface).

use alacer_core::Change;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Canvas, Column};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const CANVASES: usize = 100;
const SURFACE_W: f32 = 60.0;
const SURFACE_H: f32 = 40.0;

fn bench_canvas_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("canvas");
    group.throughput(Throughput::Elements(
        u64::try_from(CANVASES).expect("non-negative"),
    ));

    group.bench_function("paint_100_canvases", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        let root = harness.mount(Column::new().gap(4.0).boxed());
        let mut surfaces = Vec::with_capacity(CANVASES);
        for _ in 0..CANVASES {
            let node = harness.add_child(
                root,
                Canvas::new("The canvas bench")
                    .size(SURFACE_W, SURFACE_H)
                    .painter(|ctx| {
                        // One quad per surface: the route's own record —
                        // the closure runs, the primitive lands in the
                        // journal, nothing else moves.
                        ctx.push_quad(alacer_core::scene::Quad::filled(
                            alacer_core::geom::Rect::new(
                                alacer_core::geom::Point::new(0.0, 0.0),
                                ctx.size(),
                            ),
                            ctx.palette().accent,
                        ));
                    })
                    .boxed(),
            );
            surfaces.push(node);
        }
        harness.settle();

        // Honesty before the clock: a repaint mark on every surface
        // re-records exactly one quad each.
        for node in &surfaces {
            harness
                .pipeline_mut()
                .arena_mut()
                .merge_mask(*node, Change::PAINT);
        }
        let report = harness.tick();
        assert!(
            report.scene.primitives.len() >= CANVASES,
            "every canvas re-recorded its quad (got {})",
            report.scene.primitives.len()
        );

        b.iter(|| {
            for node in &surfaces {
                harness
                    .pipeline_mut()
                    .arena_mut()
                    .merge_mask(*node, Change::PAINT);
            }
            black_box(harness.tick());
        });
    });

    group.finish();
}

criterion_group!(benches, bench_canvas_route);
criterion_main!(benches);
