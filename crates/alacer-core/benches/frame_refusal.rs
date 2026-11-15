//! Frame-refusal trail benchmarks (§13.3, ADR-0077): the note site is
//! on the hot path of every refused frame, so the observability cost
//! is pinned by contract, not by hope. The disabled trail costs one
//! branch per note (the A7 idle contract — the published
//! `frame_refusal/note_disabled` gate); the enabled trail's bounded
//! push (oldest-first eviction at cap) is priced beside it, the same
//! steady state the action trail's ring rides. A refusal is rare — the
//! per-note cost is what a frame pays when one lands, never a
//! frame-budget liability.

use alacer_core::flux::Flux;
use alacer_core::inspect::{FrameRefusalRecord, FrameRefusalSpecies};
use alacer_core::passes::{A11yDelta, A11ySink, PassPipeline, SceneJournal, TreeSnapshot};
use alacer_core::scene::{Primitive, SceneSnapshot};
use alacer_core::trail::TrailLog;
use criterion::{Criterion, criterion_group, criterion_main};
use rustc_hash::FxHashMap;
use std::hint::black_box;

/// The record one refused frame carries: the layout stage's species
/// beside the failing node's id (ADR-0023's vocabulary).
fn layout_refused() -> FrameRefusalRecord {
    FrameRefusalRecord {
        species: FrameRefusalSpecies::LayoutRefused,
        node: Some(9),
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
    fn apply(&mut self, _snapshot: std::sync::Arc<TreeSnapshot>, _delta: &A11yDelta) {}
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

fn bench_frame_refusal_note(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_refusal");

    // The disabled trail: no observer installed — the contract the A7
    // idle rule pins (one branch per note, nothing allocated). Runs
    // first: the enabled case below installs its observer only after.
    group.bench_function("note_disabled", |b| {
        b.iter(|| {
            alacer_core::frame_refusal::note_frame_refusal(black_box(layout_refused()));
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
            alacer_core::frame_refusal::note_frame_refusal(black_box(layout_refused()));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_frame_refusal_note);
criterion_main!(benches);
