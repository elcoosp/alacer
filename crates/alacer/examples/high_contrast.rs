//! The setting seam's high-contrast fact driven from the consumer side
//! (§7.2 pass 8, ADR-0085): the OS's high-contrast preference rides the
//! same seam the dark and motion facts do — ADR-0083's first
//! pre-registered re-open rule cashed a second time ("a consumer needs
//! more OS facts than these two → the trait grows the fact beside the
//! two, never a second seam per fact"). One
//! [`alacer_core::settings::SettingSeam`], one door — the flip lands
//! when *any* fact moves — and one record: the three facts beside each
//! other on the same `InspectEvent::Setting` row, the reach beside
//! them.
//!
//! The demo walks the beats before the window opens, simulating the OS
//! fact the adapter pushes: the canon build (light, motion allowed,
//! standard contrast) first, then the OS asks for *high contrast* while
//! the other facts are still — the landing the M77 door's two-fact
//! check would have swallowed — then a second ask is the honest no-op
//! (`false`, no fact, an idle frame), and the fact moves back beside a
//! *dark* flip. A contrast probe paints from `ctx.high_contrast()`
//! directly: the role-independent decision the vocabulary consults (a
//! hairline's weight, a focus ring's width), the fact never munging the
//! palette.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example high_contrast
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

/// The facts the demo's simulated OS holds — one seam, all three facts
/// (the shape any adapter rides; the platform's
/// [`alacer_platform::WinitSettings`] answers the winit facts, the
/// demo's is its own so the beats run headless). The contrast fact
/// rides an atomic the beats toggle, the dark and motion facts fixed —
/// the contrast-only landings are the demo's subject.
struct SimulatedOs {
    contrast: Arc<AtomicBool>,
}

impl SettingSeam for SimulatedOs {
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
    fn reduced_transparency(&self) -> bool {
        false
    }

    fn text_scale(&self) -> Option<f32> {
        None
    }
}

/// A probe that paints its whole bounds from the resolved *contrast*
/// fact — `PaintCtx::high_contrast` read directly off the paint
/// context, the role-independent decision the vocabulary consults (the
/// demo's pixel evidence that the fact reached the paint pass): the
/// muted ink the standard-contrast vocabulary paints, the black ink the
/// high-contrast decision raises.
struct ContrastProbe;

impl Widget for ContrastProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("the contrast probe")
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
        // The hairline the contrast fact raises: a muted fill at the
        // standard preference, the black ink the high-contrast decision
        // paints. The fact resolves once per paint pass — the probe
        // reads the copy its node holds.
        let (fill, ring) = if ctx.high_contrast() {
            (
                alacer_core::Color::rgb(0, 0, 0),
                alacer_core::Color::rgb(255, 255, 255),
            )
        } else {
            (
                alacer_core::Color::rgb(148, 148, 155),
                alacer_core::Color::TRANSPARENT,
            )
        };
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), ctx.size()),
            color: fill,
            corner_radius: 0.0,
            border_color: ring,
            border_width: if ctx.high_contrast() { 4.0 } else { 0.0 },
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

/// The dark-bearing seam the closing beat rides (the canon answers for
/// the motion and contrast facts — the dark flip lands beside them).
struct DarkContrastOs;

impl SettingSeam for DarkContrastOs {
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

/// The demo's beats, walked before the window opens: the canon build
/// (light, motion allowed, standard contrast), the OS asks for high
/// contrast where the other facts are still — the landing the two-fact
/// check swallowed — the second ask is the honest no-op, and the fact
/// moves back beside a dark flip.
fn run_beats(pipeline: &mut PassPipeline, contrast: &Arc<AtomicBool>) {
    settle(pipeline);

    // Beat one: the OS asks for high contrast — dark and motion stay,
    // the grown check lands the flip anyway, the broadcast repaints the
    // tree, the record names all three facts the tick after the door
    // answered.
    contrast.store(true, Ordering::Relaxed);
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        contrast: Arc::clone(contrast),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::high_contrast",
        landed,
        "beat 1: the OS asked for high contrast — the contrast-only flip landed"
    );

    // Beat two: the same facts again — the door answers `false`, no
    // broadcast, no record (the idle-frame property, A1's rule at the
    // setting tier). The honest absence: the log grows by nothing.
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        contrast: Arc::clone(contrast),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::high_contrast",
        landed,
        "beat 2: the no-op flip answered false — the log grew by nothing"
    );

    // Beat three: the OS restores standard contrast beside a dark flip
    // — all three facts move through the one door, the second record
    // beside the first.
    contrast.store(false, Ordering::Relaxed);
    let landed = pipeline.set_settings(Box::new(DarkContrastOs));
    settle(pipeline);
    tracing::info!(
        target: "alacer::high_contrast",
        landed,
        "beat 3: contrast restored beside the dark flip — all three facts rode the one door"
    );
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
        Label::new("The setting seam's high-contrast fact").boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Label::new(
            "The OS's contrast preference rides the facts' seam — one door, three facts on one row.",
        )
        .boxed(),
    );
    let _ = pipeline.add_child(root, ContrastProbe.boxed());
    let _ = pipeline.add_child(root, Button::new("Raise contrast").boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The trail: every flip that landed, named by its facts").boxed(),
    );
    settle(&mut pipeline);

    // The beats: the OS's flips land through the adapter's door, the
    // records stream — the log's own facts, read back on exit.
    let contrast = Arc::new(AtomicBool::new(false));
    run_beats(&mut pipeline, &contrast);

    for record in trail.borrow().iter() {
        tracing::info!(
            target: "alacer::high_contrast",
            dark = record.dark,
            reduced_motion = record.reduced_motion,
            high_contrast = record.high_contrast,
            repainted = record.repainted,
            "the trail streamed this OS flip — the facts beside the reach they repainted (ADR-0085)"
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
        "Alacer high-contrast fact",
        (VIEWPORT.width, VIEWPORT.height),
        BackendPreference::Wgpu.software_env(std::env::var_os("ALACER_SOFTWARE").as_deref()),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
