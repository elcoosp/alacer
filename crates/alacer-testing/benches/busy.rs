//! The busy route's snapshot bench (M117, ADR-0121): the a11y snapshot
//! pass over a 1,000-node list arena — every node a scrolling container
//! whose constructor declared the updating word (`busy` cycling
//! truthfully down the stream, the region the AT holds its speech on
//! while the stream is still writing it) — against the same 1,000-node
//! list arena bare (`busy/snapshot_1k_lists_bare`, this tier's own
//! honest pair: the same arena, the word absent). The published
//! `busy_snapshot_overhead` ratio budget (≤ 1.25×) keeps the word
//! honest: the word rides the snapshot the container rides, the tier's
//! price the one bool the node carries beside it — the same-shaped node
//! either way, the word the only difference. The word's semantics (the
//! doors answering unchanged beside it) are pinned functionally by the
//! m117 suite; nothing else in the frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// Mounts a 1,000-node list tree under a column root: every node a
/// scrolling container whose constructor declared the updating word —
/// the word the tree says, the region the AT defers its speech on —
/// laid out and settled before the clock. `worded` mounts the bare
/// arena the pair prices the word against (the same shapes the tier's
/// own denominator carries, the word the only difference).
fn mount_tree(harness: &mut TestHarness, worded: bool) -> NodeId {
    let root = harness.mount(Column::new().gap(2.0).boxed());
    for index in 0..NODES {
        let mut list = List::new(80.0);
        if worded {
            list = list.busy(index % 4 == 0);
        }
        let _ = harness.add_child(root, list.boxed());
    }
    harness.settle();
    root
}

/// One measured frame: the root marked for the a11y pass, the tick
/// rebuilding the snapshot over the whole arena — the pass the AT's
/// tree rides, the same route the bare arena travels.
fn snapshot_frame(harness: &mut TestHarness, root: NodeId) {
    harness
        .pipeline_mut()
        .arena_mut()
        .merge_mask(root, Change::A11Y);
    black_box(harness.tick());
}

fn bench_busy_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("busy");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_lists_busy", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, true);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.bench_function("snapshot_1k_lists_bare", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, false);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_busy_snapshot_overhead);
criterion_main!(benches);
