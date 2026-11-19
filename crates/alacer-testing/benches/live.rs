//! The live route's snapshot bench (M120, ADR-0124): the a11y snapshot
//! pass over a 1,000-node list arena — every node a scrolling container
//! whose constructor declared the region's word (`live` cycling the
//! politeness down the stream, the declared region the AT watches) —
//! against the busy route's published bare arena
//! (`busy/snapshot_1k_lists_bare`, the M117 case the tier reuses by the
//! M111 precedent; the M95 pricing rule — the word is a container
//! word, so the honest comparison is lists against lists, the
//! M108/M109 denominator shape). The published `live_snapshot_overhead`
//! ratio budget (≤ 1.25×) keeps the word honest: the word rides the
//! snapshot the container rides, the tier's price the one `Option` the
//! node carries beside it — the same-shaped node either way, the word
//! the only difference. The word's semantics (the doors answering
//! unchanged beside it) are pinned functionally by the m120 suite;
//! nothing else in the frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::announce::Politeness;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// Mounts a 1,000-node list tree under a column root: every node a
/// scrolling container whose constructor declared the region's word —
/// the word the tree says, the region the AT watches — laid out and
/// settled before the clock. The politeness cycles down the stream the
/// way a feed's regions alternate (polite for the rows that wait,
/// assertive for the ones that interrupt).
fn mount_tree(harness: &mut TestHarness) -> NodeId {
    let root = harness.mount(Column::new().gap(2.0).boxed());
    for index in 0..NODES {
        let politeness = if index % 2 == 0 {
            Politeness::Polite
        } else {
            Politeness::Assertive
        };
        let _ = harness.add_child(root, List::new(80.0).live(politeness).boxed());
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

fn bench_live_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("live");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_lists_live", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_live_snapshot_overhead);
criterion_main!(benches);
