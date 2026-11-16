//! The composition door driven from the consumer side (§7.2 pass 8,
//! ADR-0081): the palette that *follows* the OS's facts is a seam — the
//! app installs one [`alacer_core::settings::SettingComposer`] through
//! `PassPipeline::set_composer`, the door resolves the composed palette
//! against the facts that *are* (the install answers `false` where they
//! already resolve it), and every flip that lands through
//! `PassPipeline::set_settings` re-resolves it — the composed palette
//! mounting as the installed theme the roles borrow, one
//! `InspectEvent::Theme` tagged with the composer's own name streaming
//! beside the flip's `InspectEvent::Setting` (the drain reads setting →
//! theme: the facts first, the palette those facts composed second).
//! The app's own `set_theme` ask wins until the next flip — the
//! composition owns the palette only when the OS moves.
//!
//! Since M80 (ADR-0086, ADR-0083's second pre-registered re-open rule
//! grown by ADR-0085's cut) the composition resolves by *all three*
//! facts — the composer's signature grew beside `dark`: a palette that
//! calms when the OS asks for reduced motion answers by the fact at
//! the same two resolution points the door already owned (the install,
//! every landed flip), never by reaching into the platform's adapter.
//!
//! The demo walks the beats before the window opens, simulating the OS
//! facts the adapter pushes: the *Duet* composition installs over the
//! canon default (the honest no-op — the duet's light answer *is* the
//! canon palette), the OS flips *dark* and the composed dark palette
//! lands beside the fact, the OS then asks for *reduced motion* while
//! dark is still and the composition re-resolves the calm dark palette
//! (the palette that calms with the fact — the landing the dark-only
//! composition could not answer), a repeated ask is the honest no-op
//! (`false`, no facts, an idle frame), and *light* lands — the
//! composition resolving the canon palette again. A role probe paints
//! `ctx.palette().accent` (the palette the widgets borrow — the pixel
//! evidence the composition moved it), a fact probe paints from
//! `ctx.dark()` directly (the fact beside it, never munging the
//! palette). The log's facts read back on exit.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example follow_os
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (named at startup, the
//! `BackendPreference::software_env` door).

use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::inspect::{InspectEvent, InspectObserver, SettingRecord, ThemeRecord};
use alacer_core::passes::PassPipeline;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::settings::{CanonSettings, SettingComposer, SettingSeam};
use alacer_core::style::{Size2, Style};
use alacer_core::theme::Palette;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_layout::LayoutHost;
use alacer_text::ThreadLocalMeasurer;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The viewport the demo's beats settle into (the trail demos' shape).
const VIEWPORT: Size = Size::new(480.0, 340.0);

/// The facts the demo's simulated OS holds — one seam, the shape any
/// adapter rides (the platform's [`alacer_platform::WinitSettings`] is
/// the winit one; the demo's is its own, so the beats run headless).
/// The dark fact rides the beats' asks, the motion fact rides an
/// atomic the beats toggle, the contrast fact stays the honest absence
/// (ADR-0085).
struct SimulatedOs {
    dark: bool,
    motion: Arc<AtomicBool>,
}

impl SettingSeam for SimulatedOs {
    fn dark_mode(&self) -> bool {
        self.dark
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

/// The demo's composition (ADR-0081, ADR-0086): the canon palette for
/// light, a *duet* dark palette beside it — and the *calm* duet dark
/// when the OS asks for reduced motion, the palette that calms with
/// the fact. Every role distinct from the canon's, so a composed swap
/// is pixel-evident. The composer reads no platform state: the
/// pipeline hands it the facts, it answers a palette — the pure
/// function the composition tier owns.
struct DuetComposer;

/// The duet dark palette while motion is allowed (the M75 demo's
/// answer, verbatim).
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

/// The duet-calm dark palette the composition resolves when the OS
/// asks for reduced motion — every role resolved, the accent quieted
/// (the palette that calms with the fact).
const DUET_CALM: Palette = Palette {
    surface: alacer_core::Color::rgb(18, 20, 26),
    panel: alacer_core::Color::rgb(30, 33, 42),
    accent: alacer_core::Color::rgb(180, 168, 120),
    accent_pressed: alacer_core::Color::rgb(160, 148, 104),
    selection: alacer_core::Color::rgb(180, 168, 120).with_alpha(72),
    text: alacer_core::Color::rgb(220, 224, 232),
    text_muted: alacer_core::Color::rgb(140, 148, 164),
    border: alacer_core::Color::rgb(58, 64, 82),
    on_accent: alacer_core::Color::rgb(26, 22, 10),
};

impl SettingComposer for DuetComposer {
    fn palette(
        &self,
        dark: bool,
        reduced_motion: bool,
        _high_contrast: bool,
        _accent: Option<alacer_core::color::Color>,
    ) -> Palette {
        if dark && reduced_motion {
            DUET_CALM
        } else if dark {
            DUET_DARK
        } else {
            Palette::CANON
        }
    }

    fn name(&self) -> &'static str {
        "duet"
    }
}

/// A probe that paints its whole bounds from the resolved *palette*
/// (the accent role) — the composition door's pixel evidence: the
/// palette the widgets borrow moves when — and only when — the
/// composition resolves a new one.
struct RoleProbe;

impl Widget for RoleProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("the palette probe")
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
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), ctx.size()),
            color: ctx.palette().accent,
            corner_radius: 0.0,
            border_color: alacer_core::Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// A probe that paints its whole bounds from the resolved *fact* —
/// `PaintCtx::dark` read directly off the paint context, the
/// role-independent decision the vocabulary consults, riding beside
/// the palette the composition composed (the fact never munging it).
struct FactProbe;

impl Widget for FactProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("the OS fact probe")
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
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), ctx.size()),
            color: if ctx.dark() {
                alacer_core::Color::rgb(20, 22, 28)
            } else {
                alacer_core::Color::rgb(240, 241, 244)
            },
            corner_radius: 0.0,
            border_color: alacer_core::Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// The causal trail the pipeline owns: the observer half prices the
/// setting flips and the composed swaps into the bounded rings the
/// exit read-back drains.
struct Trail {
    flips: Rc<RefCell<Vec<SettingRecord>>>,
    swaps: Rc<RefCell<Vec<ThemeRecord>>>,
}

impl InspectObserver for Trail {
    fn observe(&mut self, event: InspectEvent) {
        match event {
            InspectEvent::Setting(record) => self.flips.borrow_mut().push(record),
            InspectEvent::Theme(record) => self.swaps.borrow_mut().push(record),
            _ => {}
        }
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

/// The demo's beats, walked before the window opens: the *Duet*
/// composition installs over the canon default (the honest no-op), the
/// OS flips *dark* and the composed palette lands beside the fact, the
/// OS asks for *reduced motion* where dark is still — the composition
/// re-resolving the calm palette by the facts that landed — the
/// repeated ask is the honest no-op, and *light* lands — the
/// composition resolving the canon palette again.
fn run_beats(pipeline: &mut PassPipeline, motion: &Arc<AtomicBool>) {
    settle(pipeline);

    // Beat one: the app installs the composition door — the duet's
    // light answer *is* the canon palette the default build resolves,
    // so the install is the honest no-op (`false`, no facts).
    let landed = pipeline.set_composer(Box::new(DuetComposer));
    settle(pipeline);
    tracing::info!(
        target: "alacer::follow_os",
        landed,
        "beat 1: the Duet composition installed — the canon default already resolved light"
    );

    // Beat two: the OS flips dark — the flip lands, the composed dark
    // palette lands beside it (the theme fact tagged "duet" streaming
    // behind the setting fact), the whole tree repaints once.
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        dark: true,
        motion: Arc::clone(motion),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::follow_os",
        landed,
        "beat 2: the OS flipped dark — the composed dark palette landed beside the fact"
    );

    // Beat three: the OS asks for reduced motion where dark is still —
    // the motion-only flip lands, and the composition re-resolves by
    // the facts that landed: the calm palette mounts riding the flip's
    // own broadcast (ADR-0086 — the palette that calms with the fact).
    motion.store(true, Ordering::Relaxed);
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        dark: true,
        motion: Arc::clone(motion),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::follow_os",
        landed,
        "beat 3: the OS asked for reduced motion — the composition calmed the dark palette"
    );

    // Beat four: the same facts again — the door answers `false`, no
    // broadcast, no facts (the idle-frame property, A1's rule at the
    // setting tier). The honest absence: the log grows by nothing.
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        dark: true,
        motion: Arc::clone(motion),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::follow_os",
        landed,
        "beat 4: the no-op flip answered false — the log grew by nothing"
    );

    // Beat five: the OS pushes light back — the composition resolves
    // the canon palette again, the composed swaps' third row on the
    // same log.
    motion.store(false, Ordering::Relaxed);
    let landed = pipeline.set_settings(Box::new(CanonSettings));
    settle(pipeline);
    tracing::info!(
        target: "alacer::follow_os",
        landed,
        "beat 5: the OS pushed light back — the composition resolved the canon palette"
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

    // The causal trail: the observer half prices the setting flips and
    // the composed swaps into the bounded rings the exit read-back
    // drains.
    let flips = Rc::new(RefCell::new(Vec::new()));
    let swaps = Rc::new(RefCell::new(Vec::new()));
    pipeline.set_observer(Box::new(Trail {
        flips: Rc::clone(&flips),
        swaps: Rc::clone(&swaps),
    }));

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(root, Label::new("The composition door").boxed());
    let _ = pipeline.add_child(
        root,
        Label::new(
            "The palette follows the OS's facts — the app composes it, the pipeline swaps it.",
        )
        .boxed(),
    );
    let _ = pipeline.add_child(root, RoleProbe.boxed());
    let _ = pipeline.add_child(root, FactProbe.boxed());
    let _ = pipeline.add_child(root, Button::new("Dark").boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The trail: every flip and the palette it composed, tagged").boxed(),
    );
    settle(&mut pipeline);

    // The beats: the OS's flips land through the adapter's door, the
    // composition resolves beside each, the records stream — the log's
    // own facts, read back on exit.
    let motion = Arc::new(AtomicBool::new(false));
    run_beats(&mut pipeline, &motion);

    for record in flips.borrow().iter() {
        tracing::info!(
            target: "alacer::follow_os",
            dark = record.dark,
            reduced_motion = record.reduced_motion,
            "the trail streamed this OS flip"
        );
    }
    for record in swaps.borrow().iter() {
        tracing::info!(
            target: "alacer::follow_os",
            palette = %record.palette,
            "the trail streamed this composed swap"
        );
    }

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer palette follows the OS",
        (VIEWPORT.width, VIEWPORT.height),
        BackendPreference::Wgpu.software_env(std::env::var_os("ALACER_SOFTWARE").as_deref()),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
