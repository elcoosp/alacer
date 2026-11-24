//! The setting seam's motion fact driven from the consumer side (§7.2
//! pass 8, ADR-0083): the OS's reduced-motion fact rides the same seam
//! the dark fact does — ADR-0080's first pre-registered re-open rule
//! cashing ("a consumer needs more OS facts than dark mode → the trait
//! grows the fact beside `dark_mode`, never a second seam per fact").
//! One [`alacer_core::settings::SettingSeam`], one door — the flip
//! lands when *either* fact moves — and one record: the facts beside
//! each other on the same `InspectEvent::Setting` row, the reach
//! beside them.
//!
//! The demo walks the beats before the window opens, simulating the OS
//! fact the adapter pushes: the canon build (light, motion allowed)
//! first, then the OS asks for *reduced motion* while the dark fact is
//! still — the landing the M74 door's dark-only check would have
//! swallowed — then a second ask is the honest no-op (`false`, no
//! fact, an idle frame), and the fact moves back beside a *dark* flip.
//! A motion probe paints from `ctx.reduced_motion()` directly: the
//! role-independent decision the vocabulary consults (a spinner's
//! spin, a thumbnail's autoplay), the fact never munging the palette.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example reduced_motion
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (named at startup, the
//! `BackendPreference::software_env` door).

use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::inspect::{InspectEvent, InspectObserver, SettingRecord};
use alacer_core::passes::PassPipeline;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::settings::{CanonSettings, SettingSeam};
use alacer_core::style::{Size2, Style};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_layout::LayoutHost;
use alacer_text::ThreadLocalMeasurer;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The viewport the demo's beats settle into (the trail demos' shape).
const VIEWPORT: Size = Size::new(480.0, 340.0);

/// The facts the demo's simulated OS holds — one seam, both facts (the
/// shape any adapter rides; the platform's
/// [`alacer_platform::WinitSettings`] answers the winit facts, the
/// demo's is its own so the beats run headless). The motion fact rides
/// an atomic the beats toggle, the dark fact fixed — the motion-only
/// landings are the demo's subject.
struct SimulatedOs {
    motion: Arc<AtomicBool>,
}

impl SettingSeam for SimulatedOs {
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
    fn reduced_transparency(&self) -> bool {
        false
    }
}

/// A probe that paints its whole bounds from the resolved *motion*
/// fact — `PaintCtx::reduced_motion` read directly off the paint
/// context, the role-independent decision the vocabulary consults (the
/// demo's pixel evidence that the fact reached the paint pass): the
/// striped bar the animation paints while motion is allowed, the
/// static bar the reduced fact pins.
struct MotionProbe;

impl Widget for MotionProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("the motion probe")
    }
    fn style(&self) -> Style {
        Style {
            size: Size2 {
                width: Some(64.0),
                height: Some(64.0),
            },
            ..Style::default()
        }
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        // The animated bar: full-width while motion is allowed, the
        // static bar the reduced fact pins. The fact resolves once per
        // paint pass — the probe reads the copy its node holds.
        let width = if ctx.reduced_motion() {
            ctx.size().width
        } else {
            ctx.size().width / 2.0
        };
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), Size::new(width, ctx.size().height)),
            color: if ctx.reduced_motion() {
                alacer_core::Color::rgb(126, 46, 46)
            } else {
                alacer_core::Color::rgb(46, 126, 126)
            },
            corner_radius: 0.0,
            border_color: alacer_core::Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// The causal trail the pipeline owns: the observer half prices the
/// setting facts into the bounded ring the exit read-back drains.
struct Trail {
    flips: Rc<RefCell<Vec<SettingRecord>>>,
}

impl InspectObserver for Trail {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::Setting(record) = event else {
            return;
        };
        self.flips.borrow_mut().push(record);
    }
}

/// Ticks until no pass has work (the idle contract's demo shape, A7).
fn settle(pipeline: &mut PassPipeline) {
    for _ in 0..16 {
        let report = pipeline.tick(VIEWPORT).expect("the demo's frames lay out");
        if report.is_idle() {
            return;
        }
    }
}

/// The demo's beats, walked before the window opens: the canon build
/// (light, motion allowed), the OS asks for reduced motion where the
/// dark fact is still — the landing the dark-only check swallowed —
/// the second ask is the honest no-op, and the fact moves back beside
/// a dark flip.
fn run_beats(pipeline: &mut PassPipeline, motion: &Arc<AtomicBool>) {
    settle(pipeline);

    // Beat one: the OS asks for reduced motion — dark stays light, the
    // grown check lands the flip anyway, the broadcast repaints the
    // tree, the record names both facts the tick after the door
    // answered.
    motion.store(true, Ordering::Relaxed);
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        motion: Arc::clone(motion),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::reduced_motion",
        landed,
        "beat 1: the OS asked for reduced motion — the motion-only flip landed"
    );

    // Beat two: the same facts again — the door answers `false`, no
    // broadcast, no record (the idle-frame property, A1's rule at the
    // setting tier). The honest absence: the log grows by nothing.
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        motion: Arc::clone(motion),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::reduced_motion",
        landed,
        "beat 2: the no-op flip answered false — the log grew by nothing"
    );

    // Beat three: the OS restores motion beside a dark flip — both
    // facts move through the one door, the second record beside the
    // first.
    motion.store(false, Ordering::Relaxed);
    let landed = pipeline.set_settings(Box::new(DarkMotionOs));
    settle(pipeline);
    tracing::info!(
        target: "alacer::reduced_motion",
        landed,
        "beat 3: motion restored beside the dark flip — both facts rode the one door"
    );
}

/// The dark-bearing seam the closing beat rides (the canon answers for
/// the motion fact — the dark flip lands beside it).
struct DarkMotionOs;

impl SettingSeam for DarkMotionOs {
    fn dark_mode(&self) -> bool {
        true
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
    fn reduced_transparency(&self) -> bool {
        false
    }
}

fn main() {
    // The pipeline assembled like `pipeline_default` — the production
    // pieces the facade installs (the trail demos' shape).
    alacer_core::text_seam::install(Rc::new(alacer_text::ThreadLocalTextSeam));
    let clipboard = Rc::new(alacer_platform::clipboard::ArboardClipboard::new());
    alacer_core::clipboard::install(clipboard);
    let mut pipeline = PassPipeline::new(
        alacer_core::Flux::new(),
        Box::new(LayoutHost::new(Arc::new(ThreadLocalMeasurer))),
        Box::new(alacer_render::SceneJournalImpl::new()),
        Box::new(alacer_a11y::AccessKitSink::new()),
    );

    // The causal trail: the observer half prices the setting facts into
    // the bounded ring the exit read-back drains.
    let trail = Rc::new(RefCell::new(Vec::new()));
    pipeline.set_observer(Box::new(Trail {
        flips: Rc::clone(&trail),
    }));

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(root, Label::new("The setting seam's motion fact").boxed());
    let _ = pipeline.add_child(
        root,
        Label::new(
            "The OS's reduced-motion fact rides the dark fact's seam — one door, both facts on one row.",
        )
        .boxed(),
    );
    let _ = pipeline.add_child(root, MotionProbe.boxed());
    let _ = pipeline.add_child(root, Button::new("Animate").boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The trail: every flip that landed, named by its facts").boxed(),
    );
    settle(&mut pipeline);

    // The beats: the OS's flips land through the adapter's door, the
    // records stream — the log's own facts, read back on exit.
    let motion = Arc::new(AtomicBool::new(false));
    run_beats(&mut pipeline, &motion);

    for record in trail.borrow().iter() {
        tracing::info!(
            target: "alacer::reduced_motion",
            dark = record.dark,
            reduced_motion = record.reduced_motion,
            repainted = record.repainted,
            "the trail streamed this OS flip — the facts beside the reach they repainted (ADR-0083)"
        );
    }

    // The canon facts close the demo — the dark fact moves back, the
    // door lands the flip one last time, the trail streaming it beside
    // the others (the canon facts over the canon facts would be the
    // no-op; the demo ends where it started).
    let _ = pipeline.set_settings(Box::new(CanonSettings));
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer reduced-motion fact",
        (VIEWPORT.width, VIEWPORT.height),
        BackendPreference::Wgpu.software_env(std::env::var_os("ALACER_SOFTWARE").as_deref()),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
