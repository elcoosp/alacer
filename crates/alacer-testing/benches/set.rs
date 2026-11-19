//! The set tier's snapshot bench (M119, ADR-0123): the a11y snapshot
//! pass over a 1,000-node row arena — every node a data-table row whose
//! constructor declared the place word (`position_in_set` cycling down
//! the rows, the member's index the AT reads beside the depth the
//! level tier carried) and the table declaring the span word
//! (`size_of_set(1_000)`, the collection's own count AccessKit's
//! contract places on the container) — against the same arena bare
//! (`set/snapshot_1k_rows_bare`, this tier's own honest pair: the same
//! shapes, the words absent). The published `set_snapshot_overhead`
//! ratio budget (≤ 1.25×) keeps the words honest: they ride the
//! snapshot the nodes ride, the tier's price the two `Option`s the
//! nodes carry beside them — the same-shaped nodes either way, the
//! words the only difference. The words' semantics (the doors answering
//! unchanged beside them) are pinned functionally by the m119 suite;
//! nothing else in the frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Table, TableRow};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// Mounts a 1,000-node row tree under a column root: every node a
/// data-table row under a table declaring the span word — the words
/// the tree says, the collection facts the AT reads beside the grid
/// positions the M6 slots carried — laid out and settled before the
/// clock. `worded` mounts the bare arena the pair prices the words
/// against (the same shapes the tier's own denominator carries, the
/// words the only difference).
fn mount_tree(harness: &mut TestHarness, worded: bool) -> NodeId {
    let root = harness.mount(Column::new().gap(2.0).boxed());
    let mut table = Table::new("Ledger", 2);
    if worded {
        table = table.size_of_set(NODES);
    }
    let table_id = harness.add_child(root, table.boxed());
    for index in 0..NODES {
        let mut row = TableRow::new();
        if worded {
            row = row.position_in_set(index % 4);
        }
        let _ = harness.add_child(table_id, row.boxed());
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

fn bench_set_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("set");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_rows_set", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, true);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.bench_function("snapshot_1k_rows_bare", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, false);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_set_snapshot_overhead);
criterion_main!(benches);
