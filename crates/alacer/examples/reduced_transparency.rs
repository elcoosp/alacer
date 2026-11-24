//! The setting seam's transparency fact driven from the consumer side
//! (§7.2 pass 8, ADR-0158): the OS's reduced-transparency fact rides
//! the same seam the dark, motion, contrast, and accent facts do —
//! ADR-0087's own pre-registered re-open rule cashing ("a consumer
//! needs more OS facts than these → the trait grows the fact beside
//! them, never a second seam per fact"). One
//! [`alacer_core::settings::SettingSeam`], one door — the flip lands
//! when *any* fact moves — and one record: the facts beside each other
//! on the same `InspectEvent::Setting` row, the reach beside them.
//!
//! The demo walks the beats before the window opens, simulating the OS
//! fact the adapter pushes: the canon build (light, effects allowed)
//! first, then the OS asks for *reduced transparency* while the other
//! four facts are still — the landing the M81 door's four-fact check
//! would have swallowed — then a second ask is the honest no-op
//! (`false`, no fact, an idle frame), and the fact moves back beside a
//! *dark* flip. A transparency probe paints from
//! `ctx.reduced_transparency()` directly: the role-independent decision
//! the vocabulary consults (a backdrop that mutes its glass, a
//! selection that holds its fill), the fact never munging the palette.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example reduced_transparency
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

/// The facts the demo's simulated OS holds — one seam, all five facts
/// (the shape any adapter rides; the platform's
/// [`alacer_platform::WinitSettings`] answers the winit facts, the
/// demo's is its own so the beats run headless). The transparency fact
/// rides an atomic the beats toggle, the other four fixed — the
/// transparency-only landings are the demo's subject.
struct SimulatedOs {
    transparency: Arc<AtomicBool>,
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
        self.transparency.load(Ordering::Relaxed)
    }

    fn text_scale(&self) -> Option<f32> {
        None
    }
}

/// A probe that paints its whole bounds from the resolved
/// *transparency* fact — `PaintCtx::reduced_transparency` read directly
/// off the paint context, the role-independent decision the vocabulary
/// consults (the demo's pixel evidence that the fact reached the paint
/// pass): the glass tint the overlay paints while effects are allowed,
/// the calm solid the reduced fact pins.
struct TransparencyProbe;

impl Widget for TransparencyProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("the transparency probe")
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
        // The overlay's body: the glass tint while effects are allowed,
        // the calm solid the reduced fact pins. The fact resolves once
        // per paint pass — the probe reads the copy its node holds.
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), ctx.size()),
            color: if ctx.reduced_transparency() {
                alacer_core::Color::rgb(58, 110, 165)
            } else {
                alacer_core::Color::rgb(150, 190, 255)
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
/// (light, effects allowed), the OS asks for reduced transparency
/// where the other four facts are still — the landing the four-fact
/// check swallowed — the second ask is the honest no-op, and the fact
/// moves back beside a dark flip.
fn run_beats(pipeline: &mut PassPipeline, transparency: &Arc<AtomicBool>) {
    settle(pipeline);

    // Beat one: the OS asks for reduced transparency — dark stays
    // light, motion stays allowed, contrast stays standard, no accent —
    // the grown check lands the flip anyway, the broadcast repaints the
    // tree, the record names all five facts the tick after the door
    // answered.
    transparency.store(true, Ordering::Relaxed);
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        transparency: Arc::clone(transparency),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::reduced_transparency",
        landed,
        "beat 1: the OS asked for reduced transparency — the transparency-only flip landed"
    );

    // Beat two: the same facts again — the door answers `false`, no
    // broadcast, no record (the idle-frame property, A1's rule at the
    // setting tier). The honest absence: the log grows by nothing.
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        transparency: Arc::clone(transparency),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::reduced_transparency",
        landed,
        "beat 2: the no-op flip answered false — the log grew by nothing"
    );

    // Beat three: the OS restores effects beside a dark flip — both
    // facts move through the one door, the second record beside the
    // first.
    transparency.store(false, Ordering::Relaxed);
    let landed = pipeline.set_settings(Box::new(DarkTransparencyOs));
    settle(pipeline);
    tracing::info!(
        target: "alacer::reduced_transparency",
        landed,
        "beat 3: effects restored beside the dark flip — both facts rode the one door"
    );
}

/// The dark-bearing seam the closing beat rides (the canon answers for
/// the transparency fact — the dark flip lands beside it).
struct DarkTransparencyOs;

impl SettingSeam for DarkTransparencyOs {
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
        Label::new("The setting seam's transparency fact").boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Label::new(
            "The OS's reduced-transparency fact rides the facts' one seam — one door, five facts on one row.",
        )
        .boxed(),
    );
    let _ = pipeline.add_child(root, TransparencyProbe.boxed());
    let _ = pipeline.add_child(root, Button::new("Blur the backdrop").boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The trail: every flip that landed, named by its facts").boxed(),
    );
    settle(&mut pipeline);

    // The beats: the OS's flips land through the adapter's door, the
    // records stream — the log's own facts, read back on exit.
    let transparency = Arc::new(AtomicBool::new(false));
    run_beats(&mut pipeline, &transparency);

    for record in trail.borrow().iter() {
        tracing::info!(
            target: "alacer::reduced_transparency",
            dark = record.dark,
            reduced_transparency = record.reduced_transparency,
            repainted = record.repainted,
            "the trail streamed this OS flip — the facts beside the reach they repainted (ADR-0158)"
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
        "Alacer reduced-transparency fact",
        (VIEWPORT.width, VIEWPORT.height),
        BackendPreference::Wgpu.software_env(std::env::var_os("ALACER_SOFTWARE").as_deref()),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
