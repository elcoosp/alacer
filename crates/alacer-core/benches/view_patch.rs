//! View-patch trail benchmarks (§13.3, ADR-0078): the note site is on
//! the hot path of every re-render's patch, so the observability cost
//! is pinned by contract, not by hope. The disabled trail costs one
//! branch per note (the A7 idle contract — the published
//! `view_patch/note_disabled` gate); the enabled trail's bounded push
//! (oldest-first eviction at cap) is priced beside it, the same steady
//! state the frame-refusal trail's ring rides. A patch is not rare —
//! but it is once per re-render, never per frame, and the per-note
//! cost is what a re-render pays when one lands, never a frame-budget
//! liability.

use alacer_core::flux::Flux;
use alacer_core::inspect::ViewPatchRecord;
use alacer_core::passes::{A11yDelta, A11ySink, PassPipeline, SceneJournal, TreeSnapshot};
use alacer_core::scene::{Primitive, SceneSnapshot};
use alacer_core::trail::TrailLog;
use criterion::{Criterion, criterion_group, criterion_main};
use rustc_hash::FxHashMap;
use std::hint::black_box;
use std::sync::Arc;

/// The record one re-render's patch carries: the reconciler's four
/// counts, verbatim (the `PatchReport` the door still answers, beside
/// it).
fn patch() -> ViewPatchRecord {
    ViewPatchRecord {
        created: 2,
        removed: 1,
        patched: 5,
        moved: 3,
    }
}

struct NullJournal;

impl SceneJournal for NullJournal {
    fn commit_layer(&mut self, _node: alacer_core::NodeId, _primitives: Vec<Primitive>) {}
    fn drop_layer(&mut self, _node: alacer_core::NodeId) {}
    fn layer(&self, _node: alacer_core::NodeId) -> Option<Vec<Primitive>> {
        None
    }
    fn snapshot(&self, _order: &[alacer_core::NodeId]) -> SceneSnapshot {
        SceneSnapshot::default()
    }
    fn retain(&mut self, _order: &[alacer_core::NodeId]) {}
    fn clear(&mut self) {}
    fn stats(&self) -> alacer_core::passes::JournalStats {
        alacer_core::passes::JournalStats::default()
    }
}

struct NullA11y;

impl A11ySink for NullA11y {
    fn apply(&mut self, _snapshot: Arc<TreeSnapshot>, _delta: &A11yDelta) {}
}

/// A layout engine that never runs — the bench never ticks; the
/// pipeline exists only to carry the observer door.
struct NoopLayout;

impl alacer_core::passes::LayoutEngine for NoopLayout {
    fn sync_and_layout(
        &mut self,
        _arena: &mut alacer_core::Arena,
        _root: alacer_core::NodeId,
        _dirty: &FxHashMap<alacer_core::NodeId, alacer_core::Change>,
        _viewport: alacer_core::geom::Size,
    ) -> Result<(), alacer_core::passes::LayoutError> {
        Ok(())
    }

    fn measure_calls(&self) -> u64 {
        0
    }

    fn reset_measure_calls(&mut self) {}
}

/// Installs the trail door the way a devtools consumer does (the
/// observer's installation enables the thread trails — the A7 idle
/// contract's off switch lives at the same door): a minimal pipeline,
/// observed and dropped, the trail enabled on this thread after it.
fn enable_trail_via_observer() {
    let mut pipeline = PassPipeline::new(
        Flux::new(),
        Box::new(NoopLayout),
        Box::new(NullJournal),
        Box::new(NullA11y),
    );
    pipeline.set_observer(Box::new(TrailLog::new()));
}

fn bench_view_patch_note(c: &mut Criterion) {
    let mut group = c.benchmark_group("view_patch");

    // The disabled trail: no observer installed — the contract the A7
    // idle rule pins (one branch per note, nothing allocated). Runs
    // first: the enabled case below installs its observer only after.
    group.bench_function("note_disabled", |b| {
        b.iter(|| {
            alacer_core::view_patch::note_view_patch(black_box(patch()));
        });
    });

    // The enabled trail at steady state: the observer's installation
    // enabled the thread trail, the queue sits at cap, so every note
    // pays the bounded push — the oldest evicts first. The trail's
    // records stay undrained on the bench thread: the process exits
    // with them, the pipeline's tick drain is the production reader.
    group.bench_function("note_enabled_bounded", |b| {
        enable_trail_via_observer();
        b.iter(|| {
            alacer_core::view_patch::note_view_patch(black_box(patch()));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_view_patch_note);
criterion_main!(benches);
