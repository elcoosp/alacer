//! The orientation route's snapshot bench (M98, ADR-0102): the a11y
//! snapshot pass over a 1,000-node vertical list arena, every node
//! carrying its constructor-declared axis — the single-axis word the
//! AT's arrows ride — against the no-fact 1,000-node toggle arena
//! (`toggle/snapshot_1k_binary`, the M94 binary case). The published
//! `orientation_snapshot_overhead` ratio budget (≤ 1.25×) keeps the
//! declaration honest: the enum's option rides the snapshot the name
//! rides, the tier's price the copy the node builds beside the label's
//! own — nothing else in the frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// Mounts a 1,000-node vertical list arena under a column root: every
/// node a scroll container whose constructor declared the y axis its
/// only story, laid out and settled before the clock.
fn mount_tree(harness: &mut TestHarness) -> NodeId {
    let root = harness.mount(Column::new().gap(4.0).boxed());
    for _index in 0..NODES {
        let _ = harness.add_child(root, List::new(48.0).boxed());
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

fn bench_orientation_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("orientation");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_vertical", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_orientation_snapshot_overhead);
criterion_main!(benches);
