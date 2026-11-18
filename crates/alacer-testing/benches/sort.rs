//! The sort direction route's snapshot bench (M112, ADR-0116): the a11y
//! snapshot pass over a 1,000-node header arena — every node a column
//! header whose constructor declared the sort word (`sort` alternating
//! the direction down the row, the order the AT reads beside the
//! position the M6 slot already carried) — against the same 1,000-node
//! header arena bare (`sort/snapshot_1k_headers_bare`, this tier's own
//! honest pair: the same arena, the word absent; the M95 pricing rule —
//! the word is a header word, so the honest comparison is headers
//! against headers, the M108/M110 denominator shape). The published
//! `sort_snapshot_overhead` ratio budget (≤ 1.25×) keeps the word
//! honest: the word rides the snapshot the header rides, the tier's
//! price the one `Option` the node carries beside it — the same-length
//! node either way, the word the only difference. The word's semantics
//! (the doors answering unchanged beside it) are pinned functionally by
//! the m112 suite; nothing else in the frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_core::widget::SortDirection;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, ColumnHeader};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// Mounts a 1,000-node header tree under a column root: every node a
/// column header whose constructor declared the sort word — the word
/// the tree says, the order the AT reads beside the grid position the
/// M6 slot carried — laid out and settled before the clock.
/// `worded == false` mounts the same arena bare: the tier's own
/// denominator, the word absent.
fn mount_tree(harness: &mut TestHarness, worded: bool) -> NodeId {
    let root = harness.mount(Column::new().gap(2.0).boxed());
    let kinds = [
        SortDirection::Ascending,
        SortDirection::Descending,
        SortDirection::Other,
    ];
    for index in 0..NODES {
        let mut header = ColumnHeader::new(format!("column {index}"), index);
        if worded {
            header = header.sort(kinds[index % kinds.len()]);
        }
        let _ = harness.add_child(root, header.boxed());
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

fn bench_sort_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_sort", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, true);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.bench_function("snapshot_1k_headers_bare", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, false);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_sort_snapshot_overhead);
criterion_main!(benches);
