//! Theme-trail benchmarks (§7.2 pass 8, ADR-0079): the note site is on
//! the door path of every theme swap, so the observability cost is
//! pinned by contract, not by hope. The disabled trail costs one
//! branch per note (the A7 idle contract — the published
//! `theme/note_disabled` gate); the enabled trail's bounded push
//! (oldest-first eviction at cap) is priced beside it, the same steady
//! state the frame-refusal and view-patch trails ride. A swap is not
//! rare in a theming app — but it is once per door answer, never per
//! frame, and the per-note cost is what a swap pays when one lands,
//! never a frame-budget liability. Since M78 the counted swap joins
//! the corpus (ADR-0084, ADR-0082's second re-open rule reaching the
//! theme tier): `door_swap_counted` — the swap that lands with its
//! broadcast count, the tick's start drain streaming it into the trail
//! reader, the reader naming the count beside the tag.

use alacer_core::flux::Flux;
use alacer_core::inspect::{InspectEvent, ThemeRecord};
use alacer_core::passes::{A11yDelta, A11ySink, PassPipeline, SceneJournal, TreeSnapshot};
use alacer_core::scene::{Primitive, SceneSnapshot};
use alacer_core::theme::{THEME_TRAIL_CAP, ThemeSeam};
use alacer_core::trail::TrailLog;
use criterion::{Criterion, criterion_group, criterion_main};
use rustc_hash::FxHashMap;
use std::hint::black_box;
use std::sync::Arc;

/// The record one swap carries: the palette's tag beside its reach
/// (the wire vocabulary's naming rule — the log names the palette, the
/// paint pass resolves it; the lengths rule names the reach).
fn swap() -> ThemeRecord {
    ThemeRecord {
        palette: "midnight".to_string(),
        repainted: 1,
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

/// A palette whose reads are already resolved — the steady state the
/// door's equality check rides (the installed seam answers the same
/// roles every frame until a swap lands).
struct FixedPalette;

impl ThemeSeam for FixedPalette {
    fn palette(&self) -> alacer_core::theme::Palette {
        alacer_core::theme::Palette::CANON
    }

    fn name(&self) -> &'static str {
        "canon"
    }
}

/// The palette the counted swap installs — every role distinct from
/// the canon's, so the swap lands and broadcasts every iteration (the
/// m73 suite's midnight shape, minimal).
struct MidnightPalette;

impl ThemeSeam for MidnightPalette {
    fn palette(&self) -> alacer_core::theme::Palette {
        alacer_core::theme::Palette {
            surface: alacer_core::Color::rgb(12, 14, 20),
            panel: alacer_core::Color::rgb(22, 26, 36),
            accent: alacer_core::Color::rgb(240, 180, 41),
            accent_pressed: alacer_core::Color::rgb(214, 156, 22),
            selection: alacer_core::Color::rgb(240, 180, 41).with_alpha(72),
            text: alacer_core::Color::rgb(226, 230, 238),
            text_muted: alacer_core::Color::rgb(142, 150, 168),
            border: alacer_core::Color::rgb(56, 62, 80),
            on_accent: alacer_core::Color::rgb(24, 16, 6),
        }
    }

    fn name(&self) -> &'static str {
        "midnight"
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

fn bench_theme_note(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme");

    // The disabled trail: no observer installed — the contract the A7
    // idle rule pins (one branch per note, nothing allocated). Runs
    // first: the enabled case below installs its observer only after.
    group.bench_function("note_disabled", |b| {
        b.iter(|| {
            alacer_core::theme::note_theme(black_box(swap()));
        });
    });

    // The enabled trail at steady state: the observer's installation
    // enabled the thread trail, the queue sits at cap, so every note
    // pays the bounded push — the oldest evicts first. The trail's
    // records stay undrained on the bench thread: the process exits
    // with them, the pipeline's tick drain is the production reader.
    group.bench_function("note_enabled_bounded", |b| {
        enable_trail_via_observer();
        // Prime the ring to cap so every measured note is the bounded
        // push (eviction included), never the cheap fill-up.
        for _ in 0..THEME_TRAIL_CAP {
            alacer_core::theme::note_theme(swap());
        }
        b.iter(|| {
            alacer_core::theme::note_theme(black_box(swap()));
        });
    });

    // The door's own steady state, priced beside the note: the
    // installed seam's palette re-resolved and compared — the no-op
    // swap's whole cost (the idle rule: no broadcast, no record, no
    // facts). A theming app pays this on every `set_theme` ask that
    // lands where the palette already is.
    group.bench_function("door_no_op_swap", |b| {
        let mut pipeline = PassPipeline::new(
            Flux::new(),
            Box::new(NoopLayout),
            Box::new(NullJournal),
            Box::new(NullA11y),
        );
        assert!(pipeline.set_theme(Box::new(FixedPalette)));
        b.iter(|| {
            black_box(pipeline.set_theme(Box::new(FixedPalette)));
        });
    });

    // The counted swap, read end to end (ADR-0084): the swap that
    // lands with its broadcast count, the tick's start drain streaming
    // it into the trail reader, the reader filtering the theme tier
    // and naming the count beside the tag — the real reader the
    // re-open rule priced. Each measured iteration toggles between the
    // two palettes, so every swap lands and streams; a midnight
    // install over the canon default is the swap the m78 suite pins.
    group.bench_function("door_swap_counted", |b| {
        let mut pipeline = PassPipeline::new(
            Flux::new(),
            Box::new(NoopLayout),
            Box::new(NullJournal),
            Box::new(NullA11y),
        );
        let reader = TrailLog::new();
        pipeline.set_observer(Box::new(reader.clone()));
        b.iter(|| {
            black_box(pipeline.set_theme(Box::new(MidnightPalette)));
            pipeline
                .tick(alacer_core::geom::Size::new(320.0, 240.0))
                .expect("the counted-swap bench's frames lay out");
            black_box(pipeline.set_theme(Box::new(FixedPalette)));
            pipeline
                .tick(alacer_core::geom::Size::new(320.0, 240.0))
                .expect("the counted-swap bench's frames lay out");
            let count = reader
                .look()
                .into_iter()
                .filter_map(|event| match event {
                    InspectEvent::Theme(record) => Some(record.repainted),
                    _ => None,
                })
                .next_back();
            black_box(count);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_theme_note);
criterion_main!(benches);
