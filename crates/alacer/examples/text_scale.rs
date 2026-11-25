//! The setting seam's text-scale fact driven from the consumer side
//! (§7.2 pass 8, ADR-0159): the OS's preferred text size rides the
//! same seam the dark, motion, contrast, accent, and transparency
//! facts do — ADR-0087's own pre-registered re-open rule cashing ("a
//! consumer needs more OS facts than these → the trait grows the fact
//! beside them, never a second seam per fact"). One
//! [`alacer_core::settings::SettingSeam`], one door — the flip lands
//! when *any* fact moves — and one record: the facts beside each other
//! on the same `InspectEvent::Setting` row, the reach beside them. The
//! fact rides its presence: `None` is the honest absence, never a
//! guessed multiplier.
//!
//! The demo walks the beats before the window opens, simulating the OS
//! fact the adapter pushes: the canon build (light, no read) first,
//! then the OS asks for *1.25× text* while the other five facts are
//! still — the landing the M154 door's five-fact check would have
//! swallowed — then a second ask is the honest no-op (the same
//! multiplier, no fact, an idle frame), and the fact moves back beside
//! a *dark* flip. The fact's *effect* rides with it (ADR-0160, the
//! deferral cashed): the flip re-measures the tree — `LAYOUT` beside
//! `PAINT` — and the text tier multiplies the declared sizes at the
//! seam's one door, so every label on this page renders larger while
//! the ask holds and returns to its declared size when the absence
//! answers. A text-scale probe paints from
//! `ctx.text_scale()` directly: the role-independent decision the
//! vocabulary consults, the fact never munging the palette.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example text_scale
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

/// The multiplier the demo's simulated OS reports when the beats ask —
/// the OS's preferred reading size, the fact's `Some` half.
const OS_SCALE: f32 = 1.25;

/// The facts the demo's simulated OS holds — one seam, all six facts
/// (the shape any adapter rides; the platform's
/// [`alacer_platform::WinitSettings`] answers the winit facts, the
/// demo's is its own so the beats run headless). The text-scale fact
/// rides an atomic the beats toggle, the other five fixed — the
/// scale-only landings are the demo's subject.
struct SimulatedOs {
    scale: Arc<AtomicBool>,
}

impl SettingSeam for SimulatedOs {
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
        None
    }

    fn reduced_transparency(&self) -> bool {
        false
    }

    fn text_scale(&self) -> Option<f32> {
        if self.scale.load(Ordering::Relaxed) {
            Some(OS_SCALE)
        } else {
            None
        }
    }
}

/// A probe that paints its whole bounds from the resolved *text-scale*
/// fact — `PaintCtx::text_scale` read directly off the paint context,
/// the role-independent decision the vocabulary consults (the demo's
/// pixel evidence that the fact reached the paint pass): the
/// declared-size ink the honest absence paints, the emphasized ink the
/// OS's ask pins.
struct TextScaleProbe;

impl Widget for TextScaleProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("the text-scale probe")
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
        // The probe's body: the declared-size ink while the platform
        // reports no read, the emphasized ink the OS's ask pins. The
        // fact resolves once per paint pass — the probe reads the copy
        // its node holds; the labels around it *are* the typography
        // half (ADR-0160): their declared sizes multiply at the seam
        // the ask flipped, the probe's own paint staying the probe.
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), ctx.size()),
            color: if ctx.text_scale().is_some() {
                alacer_core::Color::rgb(16, 122, 87)
            } else {
                alacer_core::Color::rgb(198, 198, 205)
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
/// (light, no read), the OS asks for 1.25× text where the other five
/// facts are still — the landing the five-fact check swallowed — the
/// second ask is the honest no-op, and the fact moves back beside a
/// dark flip.
fn run_beats(pipeline: &mut PassPipeline, scale: &Arc<AtomicBool>) {
    settle(pipeline);

    // Beat one: the OS asks for larger text — dark stays light, motion
    // stays allowed, contrast stays standard, no accent, effects
    // allowed — the grown check lands the flip anyway, the broadcast
    // repaints the tree, the record names all six facts the tick after
    // the door answered.
    scale.store(true, Ordering::Relaxed);
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        scale: Arc::clone(scale),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::text_scale",
        landed,
        "beat 1: the OS asked for 1.25x text — the scale-only flip landed"
    );

    // Beat two: the same facts again — the door answers `false`, no
    // broadcast, no record (the idle-frame property, A1's rule at the
    // setting tier). The honest absence: the log grows by nothing.
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        scale: Arc::clone(scale),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::text_scale",
        landed,
        "beat 2: the no-op flip answered false — the log grew by nothing"
    );

    // Beat three: the OS restores the declared size beside a dark flip
    // — both facts move through the one door, the second record beside
    // the first.
    scale.store(false, Ordering::Relaxed);
    let landed = pipeline.set_settings(Box::new(DarkOs));
    settle(pipeline);
    tracing::info!(
        target: "alacer::text_scale",
        landed,
        "beat 3: the declared size restored beside the dark flip — both facts rode the one door"
    );
}

/// The dark-bearing seam the closing beat rides (the honest absence for
/// the text-scale fact — the dark flip lands beside it).
struct DarkOs;

impl SettingSeam for DarkOs {
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

    fn text_scale(&self) -> Option<f32> {
        None
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
    let _ = pipeline.add_child(
        root,
        Label::new("The setting seam's text-scale fact").boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Label::new(
            "The OS's preferred text size rides the facts' one seam — one door, six facts on one row.",
        )
        .boxed(),
    );
    let _ = pipeline.add_child(root, TextScaleProbe.boxed());
    let _ = pipeline.add_child(root, Button::new("Larger text").boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The trail: every flip that landed, named by its facts").boxed(),
    );
    settle(&mut pipeline);

    // The beats: the OS's flips land through the adapter's door, the
    // records stream — the log's own facts, read back on exit.
    let scale = Arc::new(AtomicBool::new(false));
    run_beats(&mut pipeline, &scale);

    for record in trail.borrow().iter() {
        tracing::info!(
            target: "alacer::text_scale",
            dark = record.dark,
            text_scale = ?record.text_scale,
            repainted = record.repainted,
            "the trail streamed this OS flip — the facts beside the reach they repainted (ADR-0159)"
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
        "Alacer text-scale fact",
        (VIEWPORT.width, VIEWPORT.height),
        BackendPreference::Wgpu.software_env(std::env::var_os("ALACER_SOFTWARE").as_deref()),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
