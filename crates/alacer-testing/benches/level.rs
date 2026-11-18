//! The level route's snapshot bench (M113, ADR-0117): the a11y snapshot
//! pass over a 1,000-node row arena — every node a data-table row whose
//! constructor declared the depth word (`level` cycling the zero-based
//! depth down the table, the nested-collection contract the AT reads)
//! — against the same 1,000-node row arena bare
//! (`expanded/snapshot_1k_rows_bare`, the M110 case the tier reuses by
//! the M111 precedent; the M95 pricing rule — the word is a row word,
//! so the honest comparison is rows against rows, the M108/M110
//! denominator shape). The published `level_snapshot_overhead` ratio
//! budget (≤ 1.25×) keeps the word honest: the word rides the snapshot
//! the row rides, the tier's price the one `Option` the node carries
//! beside it — the same-length node either way, the word the only
//! difference. The word's semantics (the doors answering unchanged
//! beside it) are pinned functionally by the m113 suite; nothing else
//! in the frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TableRow};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// Mounts a 1,000-node row tree under a column root: every node a
/// data-table row whose constructor declared the depth word — the word
/// the tree says, the nested-collection contract the AT reads beside
/// the grid position the M6 slot carried — laid out and settled before
/// the clock.
fn mount_tree(harness: &mut TestHarness) -> NodeId {
    let root = harness.mount(Column::new().gap(2.0).boxed());
    for index in 0..NODES {
        let row = TableRow::new().level(index % 4);
        let _ = harness.add_child(root, row.boxed());
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

fn bench_level_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("level");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_level", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_level_snapshot_overhead);
criterion_main!(benches);
