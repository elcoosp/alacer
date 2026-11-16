//! Setting-trail benchmarks (§7.2 pass 8, ADR-0080): the note site is
//! on the door path of every OS setting flip, so the observability
//! cost is pinned by contract, not by hope. The disabled trail costs
//! one branch per note (the A7 idle contract — the published
//! `settings/note_disabled` gate); the enabled trail's bounded push
//! (oldest-first eviction at cap) is priced beside it, the same steady
//! state the theme trail rides (ADR-0079's own corpus). A flip is not
//! rare in a following the OS app — but it is once per door answer,
//! never per frame, and the per-note cost is what a flip pays when one
//! lands, never a frame-budget liability. Since M75 the composition
//! door's own costs join the corpus (ADR-0081): the install's
//! resolution (`composer_install_no_op` — the composer call beside the
//! effective-palette equality, the no-op install's whole cost) and the
//! composed flip end to end (`door_flip_composed` — the fact that
//! lands, the composer's resolution, the composed palette's mount).
//! Since M77 the grown fact's flip joins the corpus (ADR-0083,
//! ADR-0080's first re-open rule): `door_flip_motion` — the
//! motion-only flip landing where the dark fact is still, both facts
//! resolved, the record streamed, the real reader pricing the fact
//! tier's growth. Since M79 the second grown fact's flip joins beside
//! it (ADR-0085, the same rule cashed again): `door_flip_contrast` —
//! the contrast-only flip landing where the other facts are still, all
//! three facts resolved, the record streamed, the real reader naming
//! the contrast fact. Since M81 the third grown fact's flip joins
//! beside it (ADR-0087, the same rule cashed a third time):
//! `door_flip_accent` — the accent-only flip landing where the other
//! facts are still, all four facts resolved, the record streamed, the
//! real reader naming the accent fact.

use alacer_core::flux::Flux;
use alacer_core::inspect::SettingRecord;
use alacer_core::passes::{A11yDelta, A11ySink, PassPipeline, SceneJournal, TreeSnapshot};
use alacer_core::scene::{Primitive, SceneSnapshot};
use alacer_core::settings::{SETTING_TRAIL_CAP, SettingComposer, SettingSeam};
use alacer_core::theme::Palette;
use alacer_core::trail::TrailLog;
use criterion::{Criterion, criterion_group, criterion_main};
use rustc_hash::FxHashMap;
use std::hint::black_box;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The record one flip carries: the OS facts, verbatim (the payload
/// rule at the setting tier — the bools are the facts, the accent
/// riding its presence).
fn flip() -> SettingRecord {
    SettingRecord {
        dark: true,
        reduced_motion: false,
        high_contrast: false,
        accent: None,
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

    fn reduced_motion(&self) -> bool {
        false
    }

    fn high_contrast(&self) -> bool {
        false
    }

    fn accent_color(&self) -> Option<alacer_core::color::Color> {
        None
    }
}

/// The composed dark palette the bench composer answers — every role
/// distinct from the canon's, so the flip's composition resolves a
/// swap every measured iteration (never the idle equality).
const DUET_DARK: Palette = Palette {
    surface: alacer_core::Color::rgb(16, 18, 24),
    panel: alacer_core::Color::rgb(28, 31, 40),
    accent: alacer_core::Color::rgb(255, 176, 64),
    accent_pressed: alacer_core::Color::rgb(226, 152, 44),
    selection: alacer_core::Color::rgb(255, 176, 64).with_alpha(72),
    text: alacer_core::Color::rgb(226, 230, 238),
    text_muted: alacer_core::Color::rgb(142, 150, 168),
    border: alacer_core::Color::rgb(56, 62, 80),
    on_accent: alacer_core::Color::rgb(24, 16, 6),
};

/// The bench composer (ADR-0081): the canon palette for light, the
/// duet dark beside it — the shape any app composition rides, two
/// palettes resolved by the fact.
struct DuetComposer;

impl SettingComposer for DuetComposer {
    fn palette(
        &self,
        dark: bool,
        _reduced_motion: bool,
        _high_contrast: bool,
        _accent: Option<alacer_core::color::Color>,
    ) -> Palette {
        if dark { DUET_DARK } else { Palette::CANON }
    }

    fn name(&self) -> &'static str {
        "duet"
    }
}

/// The bench's motion-bearing seam (ADR-0083): the dark fact stays
/// fixed, the motion fact answers an atomic the bench handle toggles —
/// every measured flip is the motion-only landing the grown door check
/// admits where the dark fact is still, the case the M74 check would
/// have swallowed.
struct MotionSettings {
    motion: std::sync::Arc<AtomicBool>,
}

impl SettingSeam for MotionSettings {
    fn dark_mode(&self) -> bool {
        false
    }

    fn reduced_motion(&self) -> bool {
        self.motion.load(Ordering::Relaxed)
    }

    fn high_contrast(&self) -> bool {
        false
    }

    fn accent_color(&self) -> Option<alacer_core::color::Color> {
        None
    }
}

/// The bench's contrast-bearing seam (ADR-0085): the dark and motion
/// facts stay fixed, the contrast fact answers an atomic the bench
/// handle toggles — every measured flip is the contrast-only landing
/// the grown door check admits where the other facts are still, the
/// case the M77 check would have swallowed.
struct ContrastSettings {
    contrast: std::sync::Arc<AtomicBool>,
}

impl SettingSeam for ContrastSettings {
    fn dark_mode(&self) -> bool {
        false
    }

    fn reduced_motion(&self) -> bool {
        false
    }

    fn high_contrast(&self) -> bool {
        self.contrast.load(Ordering::Relaxed)
    }

    fn accent_color(&self) -> Option<alacer_core::color::Color> {
        None
    }
}

/// The bench's accent-bearing seam (ADR-0087): the dark, motion, and
/// contrast facts stay fixed, the accent fact answers an atomic the
/// bench handle toggles — every measured flip is the accent-only
/// landing the grown door check admits where the other facts are
/// still, the case the M79 check would have swallowed.
struct AccentSettings {
    accent: std::sync::Arc<AtomicBool>,
}

impl SettingSeam for AccentSettings {
    fn dark_mode(&self) -> bool {
        false
    }

    fn reduced_motion(&self) -> bool {
        false
    }

    fn high_contrast(&self) -> bool {
        false
    }

    fn accent_color(&self) -> Option<alacer_core::color::Color> {
        // The fact rides its presence: the toggled atomic says whether
        // the OS reports one, the reported color the bench's fixed
        // answer.
        if self.accent.load(Ordering::Relaxed) {
            Some(alacer_core::Color::rgb(255, 94, 58))
        } else {
            None
        }
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

    group.finish();
}

/// The setting door's own costs (ADR-0080, ADR-0081, ADR-0082, ADR-0083): the
/// flip's equality check, the composition's resolution, the counted and
/// motion-bearing flips read end to end — the real readers the lengths rule
/// priced.
fn bench_settings_door(c: &mut Criterion) {
    let mut group = c.benchmark_group("settings");

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

/// The composition tier's own costs (ADR-0081): the install's resolution
/// beside the effective-palette equality, the composed flip end to end.
fn bench_settings_composition(c: &mut Criterion) {
    let mut group = c.benchmark_group("settings");

    // The composition install's own steady state, priced beside the
    // door (ADR-0081): the composer's resolution and the effective-
    // palette equality — the no-op install's whole cost (the canon
    // composition over the canon default resolves nothing). An app
    // installing its composition mid-session pays this once, every
    // later flip riding `door_flip_composed` instead.
    group.bench_function("composer_install_no_op", |b| {
        let mut pipeline = PassPipeline::new(
            Flux::new(),
            Box::new(NoopLayout),
            Box::new(NullJournal),
            Box::new(NullA11y),
        );
        b.iter(|| {
            black_box(pipeline.set_composer(Box::new(DuetComposer)));
        });
    });

    // The composed flip, priced beside the door (ADR-0081): the fact
    // that lands, the composition's resolution, the composed palette's
    // mount — the flip that moves the palette end to end, the real
    // reader the re-open rule priced. Each measured iteration toggles
    // the fact, so every flip lands and composes the other palette —
    // the swap half never idles here, the equality check answered once
    // per flip against the palette the previous flip composed.
    group.bench_function("door_flip_composed", |b| {
        let mut pipeline = PassPipeline::new(
            Flux::new(),
            Box::new(NoopLayout),
            Box::new(NullJournal),
            Box::new(NullA11y),
        );
        pipeline.set_composer(Box::new(DuetComposer));
        let dark = AtomicBool::new(false);
        b.iter(|| {
            let fact = dark.fetch_xor(true, Ordering::Relaxed);
            black_box(pipeline.set_settings(Box::new(FixedSettings { dark: fact })));
        });
    });

    group.finish();
}

/// The counted flips' own costs (ADR-0082, ADR-0083, ADR-0085): the
/// flips whose records name their reach and their facts, each read end
/// to end through the real reader the lengths rule priced — the tick's
/// start drain streaming the record into the trail reader, the reader
/// naming the count or the fact beside it.
fn bench_settings_counted(c: &mut Criterion) {
    let mut group = c.benchmark_group("settings");

    // The counted flip, read end to end (ADR-0082): the fact that
    // lands with its broadcast count, the tick's start drain streaming
    // it into the trail reader, the reader filtering the setting tier
    // and naming the count — the real reader the lengths rule priced.
    // Each measured iteration toggles the fact so every flip lands and
    // streams.
    group.bench_function("door_flip_counted", |b| {
        let mut pipeline = PassPipeline::new(
            Flux::new(),
            Box::new(NoopLayout),
            Box::new(NullJournal),
            Box::new(NullA11y),
        );
        let reader = TrailLog::new();
        pipeline.set_observer(Box::new(reader.clone()));
        pipeline.set_settings(Box::new(FixedSettings { dark: false }));
        let dark = AtomicBool::new(false);
        b.iter(|| {
            let fact = dark.fetch_xor(true, Ordering::Relaxed);
            black_box(pipeline.set_settings(Box::new(FixedSettings { dark: fact })));
            pipeline
                .tick(alacer_core::geom::Size::new(320.0, 240.0))
                .expect("the counted-flip bench's frames lay out");
            let count = reader
                .look()
                .into_iter()
                .filter_map(|event| match event {
                    alacer_core::inspect::InspectEvent::Setting(record) => Some(record.repainted),
                    _ => None,
                })
                .next_back();
            black_box(count);
        });
    });

    // The motion-only flip, read end to end (ADR-0083): the fact the
    // M74 door's dark-only check swallowed lands now — both facts
    // resolved where the broadcast resolves, the record carrying them
    // beside the count, the tick's start drain streaming it into the
    // trail reader, the reader naming the motion fact. Each measured
    // iteration toggles the dark fact so every flip lands (the motion
    // fact rides beside it unchanged); the motion-only landing itself
    // is the shape the m77 suite pins.
    group.bench_function("door_flip_motion", |b| {
        let mut pipeline = PassPipeline::new(
            Flux::new(),
            Box::new(NoopLayout),
            Box::new(NullJournal),
            Box::new(NullA11y),
        );
        let reader = TrailLog::new();
        pipeline.set_observer(Box::new(reader.clone()));
        let motion = Arc::new(AtomicBool::new(false));
        pipeline.set_settings(Box::new(MotionSettings {
            motion: Arc::clone(&motion),
        }));
        b.iter(|| {
            let _previous = motion.fetch_xor(true, Ordering::Relaxed);
            black_box(pipeline.set_settings(Box::new(MotionSettings {
                motion: Arc::clone(&motion),
            })));
            pipeline
                .tick(alacer_core::geom::Size::new(320.0, 240.0))
                .expect("the motion-flip bench's frames lay out");
            let landed = reader
                .look()
                .into_iter()
                .filter_map(|event| match event {
                    alacer_core::inspect::InspectEvent::Setting(record) => {
                        Some(record.reduced_motion)
                    }
                    _ => None,
                })
                .next_back();
            black_box(landed);
        });
    });

    // The contrast-only flip, read end to end (ADR-0085): the fact the
    // M77 door's two-fact check swallowed lands now — all three facts
    // resolved where the broadcast resolves, the record carrying them
    // beside the count, the tick's start drain streaming it into the
    // trail reader, the reader naming the contrast fact. Each measured
    // iteration toggles the contrast fact so every flip lands (the
    // dark and motion facts ride beside it unchanged); the
    // contrast-only landing itself is the shape the m79 suite pins.
    group.bench_function("door_flip_contrast", |b| {
        let mut pipeline = PassPipeline::new(
            Flux::new(),
            Box::new(NoopLayout),
            Box::new(NullJournal),
            Box::new(NullA11y),
        );
        let reader = TrailLog::new();
        pipeline.set_observer(Box::new(reader.clone()));
        let contrast = Arc::new(AtomicBool::new(false));
        pipeline.set_settings(Box::new(ContrastSettings {
            contrast: Arc::clone(&contrast),
        }));
        b.iter(|| {
            let _previous = contrast.fetch_xor(true, Ordering::Relaxed);
            black_box(pipeline.set_settings(Box::new(ContrastSettings {
                contrast: Arc::clone(&contrast),
            })));
            pipeline
                .tick(alacer_core::geom::Size::new(320.0, 240.0))
                .expect("the contrast-flip bench's frames lay out");
            let landed = reader
                .look()
                .into_iter()
                .filter_map(|event| match event {
                    alacer_core::inspect::InspectEvent::Setting(record) => {
                        Some(record.high_contrast)
                    }
                    _ => None,
                })
                .next_back();
            black_box(landed);
        });
    });

    group.finish();
}

/// The accent flip's own cost (ADR-0087): the third grown fact's flip
/// read end to end through the real reader the lengths rule priced —
/// the tick's start drain streaming the record into the trail reader,
/// the reader naming the accent fact.
fn bench_settings_accent(c: &mut Criterion) {
    let mut group = c.benchmark_group("settings");

    // The accent-only flip, read end to end (ADR-0087): the fact the
    // M79 door's three-fact check swallowed lands now — all four facts
    // resolved where the broadcast resolves, the record carrying them
    // beside the count, the tick's start drain streaming it into the
    // trail reader, the reader naming the accent fact. Each measured
    // iteration toggles the accent fact so every flip lands (the dark,
    // motion, and contrast facts ride beside it unchanged); the
    // accent-only landing itself is the shape the m81 suite pins.
    group.bench_function("door_flip_accent", |b| {
        let mut pipeline = PassPipeline::new(
            Flux::new(),
            Box::new(NoopLayout),
            Box::new(NullJournal),
            Box::new(NullA11y),
        );
        let reader = TrailLog::new();
        pipeline.set_observer(Box::new(reader.clone()));
        let accent = Arc::new(AtomicBool::new(false));
        pipeline.set_settings(Box::new(AccentSettings {
            accent: Arc::clone(&accent),
        }));
        b.iter(|| {
            let _previous = accent.fetch_xor(true, Ordering::Relaxed);
            black_box(pipeline.set_settings(Box::new(AccentSettings {
                accent: Arc::clone(&accent),
            })));
            pipeline
                .tick(alacer_core::geom::Size::new(320.0, 240.0))
                .expect("the accent-flip bench's frames lay out");
            let landed = reader
                .look()
                .into_iter()
                .filter_map(|event| match event {
                    alacer_core::inspect::InspectEvent::Setting(record) => {
                        Some(record.accent.is_some())
                    }
                    _ => None,
                })
                .next_back();
            black_box(landed);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_settings,
    bench_settings_door,
    bench_settings_composition,
    bench_settings_counted,
    bench_settings_accent
);
criterion_main!(benches);
