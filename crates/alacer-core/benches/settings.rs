//! Setting-trail benchmarks (§7.2 pass 8, ADR-0080): the note site is
//! on the door path of every OS setting flip, so the observability
//! cost is pinned by contract, not by hope. The disabled trail costs
//! one branch per note (the A7 idle contract — the published
//! `settings/note_disabled` gate); the enabled trail's bounded push
//! (oldest-first eviction at cap) is priced beside it, the same steady
//! state the theme trail rides (ADR-0079's own corpus). A flip is not
//! rare in a following the OS app — but it is once per door answer,
//! never per frame, and the per-note cost is what a flip pays when one
//! lands, never a frame-budget liability.

use alacer_core::flux::Flux;
use alacer_core::inspect::SettingRecord;
use alacer_core::passes::{A11yDelta, A11ySink, PassPipeline, SceneJournal, TreeSnapshot};
use alacer_core::scene::{Primitive, SceneSnapshot};
use alacer_core::settings::{SETTING_TRAIL_CAP, SettingSeam};
use alacer_core::trail::TrailLog;
use criterion::{Criterion, criterion_group, criterion_main};
use rustc_hash::FxHashMap;
use std::hint::black_box;
use std::sync::Arc;

/// The record one flip carries: the dark-mode fact, verbatim (the
/// payload rule at the setting tier — the bool is the fact).
fn flip() -> SettingRecord {
    SettingRecord { dark: true }
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

/// A setting whose read is already resolved — the steady state the
/// door's equality check rides (the installed seam answers the same
/// fact every frame until a flip lands).
struct FixedSettings {
    dark: bool,
}

impl SettingSeam for FixedSettings {
    fn dark_mode(&self) -> bool {
        self.dark
    }
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

fn bench_settings(c: &mut Criterion) {
    let mut group = c.benchmark_group("settings");

    // The disabled trail: no observer installed — the contract the A7
    // idle rule pins (one branch per note, nothing allocated). Runs
    // first: the enabled case below installs its observer only after.
    group.bench_function("note_disabled", |b| {
        b.iter(|| {
            alacer_core::settings::note_setting(black_box(flip()));
        });
    });

    // The enabled trail at steady state: the observer's installation
    // enabled the thread trail, the queue sits at cap, so every measured
    // note is the bounded push (eviction included), never the cheap
    // fill-up. The trail's records stay undrained on the bench thread:
    // the process exits with them, the pipeline's tick drain is the
    // production reader.
    group.bench_function("note_enabled_bounded", |b| {
        enable_trail_via_observer();
        // Prime the ring to cap so every measured note is the bounded
        // push (eviction included), never the cheap fill-up.
        for _ in 0..SETTING_TRAIL_CAP {
            alacer_core::settings::note_setting(flip());
        }
        b.iter(|| {
            alacer_core::settings::note_setting(black_box(flip()));
        });
    });

    // The door's own steady state, priced beside the note: the
    // installed seam's fact re-read and compared — the no-op flip's
    // whole cost (the idle rule: no broadcast, no record, no facts). An
    // OS-following app pays this on every `set_settings` ask that lands
    // where the fact already is.
    group.bench_function("door_no_op_flip", |b| {
        let mut pipeline = PassPipeline::new(
            Flux::new(),
            Box::new(NoopLayout),
            Box::new(NullJournal),
            Box::new(NullA11y),
        );
        assert!(pipeline.set_settings(Box::new(FixedSettings { dark: true })));
        b.iter(|| {
            black_box(pipeline.set_settings(Box::new(FixedSettings { dark: true })));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_settings);
criterion_main!(benches);
