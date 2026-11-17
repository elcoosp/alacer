//! The numeric route's snapshot bench (M99, ADR-0103): the a11y
//! snapshot pass over a 1,000-node value arena whose every node answers
//! the number beside the text — the scalar the value tier's funnel
//! holds, the `aria-valuenow` riding the `aria-valuetext` — against the
//! no-fact 1,000-node toggle arena (`toggle/snapshot_1k_binary`, the
//! M94 binary case). The published `numeric_snapshot_overhead` ratio
//! budget (≤ 1.25×) keeps the number honest: the `f64` option rides the
//! snapshot the text rides, the tier's price the copy the node builds
//! beside the label's own — nothing else in the frame pays for the
//! tree's truth. The arena is the range route's own (the M97 corpus's
//! slider shape), the numeric fact the one fact richer the M95 pricing
//! rule measures.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Slider};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// Mounts a 1,000-node value arena under a column root: every node a
/// named slider whose funnel holds a scalar — the number the AT reads
/// beside the text — laid out and settled before the clock. The shape
/// is the range route's own: the walls, the step, and now the number.
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

fn bench_numeric_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("numeric");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_valued", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_numeric_snapshot_overhead);
criterion_main!(benches);
