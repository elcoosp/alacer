//! The has-popup route's snapshot bench (M115, ADR-0119): the a11y
//! snapshot pass over a 1,000-node button arena — every node a button
//! whose constructor declared the target word (`has_popup` cycling
//! AccessKit's kinds down the toolbar, the opener's target the AT
//! reads) — against the same 1,000-node button arena bare
//! (`current/snapshot_1k_buttons_bare`, the M114 case the M111
//! precedent reuses; the M95 pricing rule — the word is a button word,
//! so the honest comparison is buttons against buttons, the M108/M110
//! denominator shape). The published `has_popup_snapshot_overhead`
//! ratio budget (≤ 1.25×) keeps the word honest: the word rides the
//! snapshot the button rides, the tier's price the one `Option` the
//! node carries beside it — the same-length node either way, the word
//! the only difference. The word's semantics (the doors answering
//! unchanged beside it) are pinned functionally by the m115 suite;
//! nothing else in the frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_core::widget::HasPopupKind;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// The kinds the worded arena cycles — AccessKit's own five values, the
/// menu button and the dialog opener among them, so the snapshot's
/// carried word is every target the contract speaks, not one.
const KINDS: [HasPopupKind; 5] = [
    HasPopupKind::Menu,
    HasPopupKind::Listbox,
    HasPopupKind::Tree,
    HasPopupKind::Grid,
    HasPopupKind::Dialog,
];

/// Mounts a 1,000-node button tree under a column root: every node a
/// button whose constructor declared the target word — the word the
/// tree says, the opener's target the AT reads beside the name — laid
/// out and settled before the clock. `worded` mounts the bare arena
/// the pair prices the word against (the same names the M114 bare
/// arena carries, the word the only difference).
fn mount_tree(harness: &mut TestHarness, worded: bool) -> NodeId {
    let root = harness.mount(Column::new().gap(2.0).boxed());
    for index in 0..NODES {
        let mut button = Button::new(format!("Item {index}"));
        if worded {
            button = button.has_popup(KINDS[index % KINDS.len()]);
        }
        let _ = harness.add_child(root, button.boxed());
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

fn bench_has_popup_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("has_popup");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_has_popups", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, true);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_has_popup_snapshot_overhead);
criterion_main!(benches);
