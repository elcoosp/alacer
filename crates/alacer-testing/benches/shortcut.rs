//! The shortcut route's snapshot bench (M121, ADR-0125): the a11y
//! snapshot pass over a 1,000-node button arena — every node a button
//! whose constructor declared the chord word (`keyboard_shortcut`
//! cycling the platform spellings down the stream, the way a toolbar's
//! buttons alternate) — against the current route's published bare
//! arena (`current/snapshot_1k_buttons_bare`, the M114 case the tier
//! reuses by the M111 precedent; the M95 pricing rule — the word is a
//! button word, so the honest comparison is buttons against buttons,
//! the M108/M120 denominator shape). The published
//! `shortcut_snapshot_overhead` ratio budget (≤ 1.25×) keeps the word
//! honest: the word rides the snapshot the button rides, the tier's
//! price the one `Option` the node carries beside it — the same-length
//! node either way, the word the only difference. The word's semantics
//! (the doors answering unchanged beside it) are pinned functionally by
//! the m121 suite; nothing else in the frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// The chords the worded arena cycles — the toolbar spellings the
/// platforms announce, one per flavor the contract speaks, so the
/// snapshot's carried word is every shape the app names, not one.
const CHORDS: [&str; 4] = ["Control+S", "Control+Shift+P", "Mod+P", "Alt+F4"];

/// Mounts a 1,000-node button tree under a column root: every node a
/// button whose constructor declared the chord word — the word the tree
/// says, the chord the AT announces beside the name — laid out and
/// settled before the clock.
fn mount_tree(harness: &mut TestHarness) -> NodeId {
    let root = harness.mount(Column::new().gap(2.0).boxed());
    for index in 0..NODES {
        let _ = harness.add_child(
            root,
            Button::new(format!("Item {index}"))
                .keyboard_shortcut(CHORDS[index % CHORDS.len()])
                .boxed(),
        );
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

fn bench_shortcut_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("shortcut");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_buttons_shortcut", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_shortcut_snapshot_overhead);
criterion_main!(benches);
