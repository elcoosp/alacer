//! The scroll position route's snapshot bench (M101, ADR-0105): the
//! a11y snapshot pass over a 1,000-node scrolled list arena — every
//! node a scroll container whose declared span is live and whose
//! offset sits mid-range, the tree resolving the position-and-extent
//! pair (`ScrollY`/`ScrollYMin`/`ScrollYMax` beside the x halves where
//! declared) — against the same 1,000-node list arena carrying only
//! the axis word (`orientation/snapshot_1k_vertical`, the M98 case).
//! The published `scroll_position_snapshot_overhead` ratio budget
//! (≤ 1.25×) keeps the position honest: the two axis options ride the
//! snapshot the name and the orientation ride, the tier's price the
//! pair the node builds beside the axis word's own — nothing else in
//! the frame pays for the tree's truth. The position's tracking
//! behavior (the doors' landings and walls re-announcing through the
//! a11y currency) is pinned functionally by the m101 suite; nothing
//! else in the frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);
/// Each container's declared content height — the live span (300 − 48)
/// the position's wall names, never an empty one.
const CONTENT_HEIGHT: f32 = 300.0;
/// The offset every container sits at: mid-range of its span, so the
/// resolved position carries a real landing and a real wall.
const MID_RANGE: f32 = 126.0;

/// Mounts a 1,000-node scrolled list arena under a column root: every
/// node a scroll container with a declared, live span and an offset
/// parked mid-range — the state the doors resolve and the tree says —
/// laid out and settled before the clock.
fn mount_tree(harness: &mut TestHarness) -> NodeId {
    let root = harness.mount(Column::new().gap(4.0).boxed());
    for _index in 0..NODES {
        let list = List::new(48.0);
        let handle = list.handle();
        handle.set_content_height(CONTENT_HEIGHT);
        handle.scroll_to(MID_RANGE);
        let _ = harness.add_child(root, list.boxed());
    }
    harness.settle();
    root
}

/// One measured frame: the root marked for the a11y pass, the tick
/// rebuilding the snapshot over the whole arena — the pass the AT's
/// tree rides, the same route the axis-word arena travels.
fn snapshot_frame(harness: &mut TestHarness, root: NodeId) {
    harness
        .pipeline_mut()
        .arena_mut()
        .merge_mask(root, Change::A11Y);
    black_box(harness.tick());
}

fn bench_scroll_position_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("scroll_position");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_scrolled", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_scroll_position_snapshot_overhead);
criterion_main!(benches);
