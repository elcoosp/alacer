//! The announcement benchmark (§11.6, ADR-0024): the announcement route
//! driven through the pipeline on a 1,000-node arena — `announce` sets the
//! reserved region's content and the tick's a11y pass re-emits exactly that
//! node, on the shipped composition (harness + AccessKit sink), not a probe
//! stand-in.
//!
//! The published regression gate keeps the speech act honest as the
//! contract grows: announcing must stay in the cost class of an a11y delta
//! for one node — a string write, one pass run, one `TreeUpdate` — never a
//! re-layout of the arena or a re-materialization of its a11y tree (the
//! region is the root's child from the first frame; announcing never moves
//! the tree's shape).
//!
//! The bench lives in the harness crate because that is where the whole
//! pipeline is assembled with the real widget set (the DAG's harness layer
//! drives every surface).

use alacer_core::announce::Politeness;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label, Panel};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const ROUTES: usize = 100;
const FILLER: usize = 1000;
const PANEL_HEIGHT: f32 = 0.4;

fn bench_announce_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("announce");
    group.throughput(Throughput::Elements(
        u64::try_from(ROUTES).expect("non-negative"),
    ));

    group.bench_function("route_100_announcements_1k", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        let root = harness.mount(Column::new().boxed());
        let _ = harness.add_child(root, Label::new("The announcements").boxed());
        for i in 0..FILLER {
            let _ = harness.add_child(
                root,
                Panel::new(PANEL_HEIGHT, alacer_widgets::theme::PANEL).boxed(),
            );
            let _ = i;
        }
        harness.settle();

        // One hundred announcements, alternating the politeness tiers —
        // each route is `announce` + one tick: the region's delta, the
        // sink's node-only re-emission, and nothing else (no layout, no
        // paint: the widgets never changed).
        let texts: Vec<String> = (0..ROUTES).map(|i| format!("Saved {i} items")).collect();
        b.iter(|| {
            for (i, text) in texts.iter().enumerate() {
                let politeness = if i % 2 == 0 {
                    Politeness::Polite
                } else {
                    Politeness::Assertive
                };
                let accepted = harness.pipeline_mut().announce(text.clone(), politeness);
                black_box(accepted);
                let report = harness.tick();
                black_box(report);
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_announce_route);
criterion_main!(benches);
