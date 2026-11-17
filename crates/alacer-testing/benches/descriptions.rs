//! The description route's snapshot bench (M95, ADR-0099): the a11y
//! snapshot pass over the same 1,000-node checkbox arena the toggle
//! route's binary case carries (`toggle/snapshot_1k_binary`), every node
//! described — the help text's cost the measurement. The published
//! `describe_snapshot_overhead` ratio budget (≤ 1.10×) keeps the
//! description honest: the string rides the snapshot the name rides,
//! and the tier's price is the mapping alone — nothing else in the
//! frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Checkbox, Column};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// Mounts a 1,000-node described toggle tree under a column root: every
/// node a named checkbox carrying its description — the help text the
/// AT reads beside the name — laid out and settled before the clock.
fn mount_tree(harness: &mut TestHarness) -> NodeId {
    let root = harness.mount(Column::new().gap(4.0).boxed());
    for index in 0..NODES {
        let checkbox = Checkbox::new(format!("toggle {index}"))
            .description(format!("Controls the bulk run for order {index}"));
        let _ = harness.add_child(root, checkbox.boxed());
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

fn bench_describe_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("describe");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_described", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_describe_snapshot_overhead);
criterion_main!(benches);
