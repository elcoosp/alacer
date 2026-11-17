//! The placeholder route's honesty bench (M96, ADR-0100): the a11y
//! snapshot pass over a 1,000-node input arena carrying its hint against
//! the same 1,000-node arena bare — the same roles, the same names, the
//! same trees, the same route through the real pipeline (A4). The
//! published `placeholder_snapshot_overhead` ratio budget (≤ 1.25×)
//! keeps the hint honest: the string rides the snapshot the name rides,
//! the tier's price the second bounded copy the node builds beside the
//! label's own — nothing else in the frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// Mounts a 1,000-node input tree under a column root: every node a
/// named text input; `hinted` gives each its placeholder — the hint the
/// AT reads beside the name — laid out and settled before the clock.
fn mount_tree(harness: &mut TestHarness, hinted: bool) -> NodeId {
    let root = harness.mount(Column::new().gap(4.0).boxed());
    for index in 0..NODES {
        let mut input = TextInput::new(format!("field {index}"));
        if hinted {
            input = input.placeholder(format!("Enter the part number for {index}"));
        }
        let _ = harness.add_child(root, input.boxed());
    }
    harness.settle();
    root
}

/// One measured frame: the root marked for the a11y pass, the tick
/// rebuilding the snapshot over the whole arena — the pass the AT's
/// tree rides, the same route both arenas travel.
fn snapshot_frame(harness: &mut TestHarness, root: NodeId) {
    harness
        .pipeline_mut()
        .arena_mut()
        .merge_mask(root, Change::A11Y);
    black_box(harness.tick());
}

fn bench_placeholder_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("placeholder");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_placeholdered", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, true);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.bench_function("snapshot_1k_bare", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, false);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_placeholder_snapshot_overhead);
criterion_main!(benches);
