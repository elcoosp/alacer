//! The range route's snapshot bench (M97, ADR-0101): the a11y snapshot
//! pass over a 1,000-node slider arena, every node carrying its declared
//! span — the walls every value-tier write funnels through, the step the
//! arrows and the AT's acts snap to — against the no-range 1,000-node
//! toggle arena (`toggle/snapshot_1k_binary`, the M94 binary case). The
//! published `range_snapshot_overhead` ratio budget (≤ 1.25×) keeps the
//! span honest: the two `f64`s and the option ride the snapshot the name
//! rides, the tier's price the copy the node builds beside the label's
//! own — nothing else in the frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Slider};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// Mounts a 1,000-node slider tree under a column root: every node a
/// named slider carrying its declared span — walls and step — laid out
/// and settled before the clock.
fn mount_tree(harness: &mut TestHarness) -> NodeId {
    let root = harness.mount(Column::new().gap(4.0).boxed());
    for index in 0..NODES {
        let slider = Slider::new(format!("level {index}"), 0.0, 100.0).step(5.0);
        let _ = harness.add_child(root, slider.boxed());
    }
    harness.settle();
    root
}

/// One measured frame: the root marked for the a11y pass, the tick
/// rebuilding the snapshot over the whole arena — the pass the AT's
/// tree rides, the same route the toggle route's arenas travel.
fn snapshot_frame(harness: &mut TestHarness, root: NodeId) {
    harness
        .pipeline_mut()
        .arena_mut()
        .merge_mask(root, Change::A11Y);
    black_box(harness.tick());
}

fn bench_range_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("range");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_ranged", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_range_snapshot_overhead);
criterion_main!(benches);
