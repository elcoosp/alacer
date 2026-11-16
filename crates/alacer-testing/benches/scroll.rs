//! The scroll benchmark (§8.2, ADR-0092): the bring-into-view door
//! driven through the pipeline on a 1,000-node list — one hundred asks
//! alternating between the stack's two ends, each ask riding one tick
//! of the real frame currency (the container's `PAINT | A11Y` merge,
//! compose translating the moved descendants, the paint pass
//! re-emitting their layers, the a11y pass re-announcing their bounds).
//!
//! The published regression gate keeps the door honest as the tier
//! grows: a node-priced ask must stay in the cost class of an
//! offset-priced one — the bounds ledger's own difference pricing the
//! ask, the container's clamp arithmetic answering it, the compose
//! translation doing the moving — never a re-layout of the arena (the
//! translation-only workhorse, §8.2: a scroll frame measures nothing).
//!
//! The bench lives in the harness crate because that is where the
//! whole pipeline is assembled with the real widget set (the DAG's
//! harness layer drives every surface).

use alacer_core::num::usize_to_f32;
use alacer_testing::TestHarness;
use alacer_widgets::{List, Panel};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const ROUTES: usize = 100;
const FILLER: usize = 1000;
const ROW_HEIGHT: f32 = 32.0;
const VIEWPORT: f32 = 600.0;

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

    group.finish();
}

criterion_group!(benches, bench_scroll_route);
criterion_main!(benches);
