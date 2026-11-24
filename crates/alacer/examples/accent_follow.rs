//! The accent fact driven from the consumer side (§7.2 pass 8,
//! ADR-0087): the OS's accent-color preference is a seam now —
//! [`alacer_core::settings::SettingSeam::accent_color`] rides beside
//! the dark, motion, and contrast facts the seam already carries, the
//! fact riding its presence (`None` is the honest absence, never a
//! guessed color), and the composition resolves by it beside the three
//! (ADR-0086's own rule): the app installs one
//! [`alacer_core::settings::SettingComposer`] through
//! `PassPipeline::set_composer`, the door hands it the facts the seam
//! holds at the install and on every flip that lands, and a palette
//! that tints its accent roles from the OS accent answers by the fact
//! — the composed palette mounting as the installed theme the roles
//! borrow, one `InspectEvent::Theme` tagged with the composer's own
//! name streaming beside the flip's `InspectEvent::Setting`.
//!
//! The demo walks the beats before the window opens, simulating the OS
//! facts the adapter pushes: the *accent-follow* composition installs
//! over the canon default (the honest no-op — the absence's answer *is*
//! the canon palette), the OS reports an accent and the accent-only
//! flip lands where the three bool facts are still (the landing the
//! M79 three-fact check would have swallowed) — the tinted palette
//! mounts beside the fact, the OS stops reporting one and the absence
//! moves back — the composition resolving its own answer again, a
//! repeated ask is the honest no-op (`false`, no facts, an idle
//! frame). A role probe paints `ctx.palette().accent` (the palette the
//! widgets borrow — the pixel evidence the composition tinted it), an
//! accent probe paints `ctx.accent_color()` directly (the fact beside
//! it, the absence ink when no accent is reported). The log's facts
//! read back on exit.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example accent_follow
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

/// The accent the demo's simulated OS reports — the fact's answer an
/// adapter would hand the door (a Linux desktop's picked accent, an
/// orange).
const OS_ACCENT: alacer_core::Color = alacer_core::Color::rgb(255, 94, 58);

/// The facts the demo's simulated OS holds — one seam, the shape any
/// adapter rides (the platform's [`alacer_platform::WinitSettings`] is
/// the winit one; the demo's is its own, so the beats run headless).
/// The three bool facts stay light, motion allowed, standard contrast;
/// the accent fact rides its presence on the atomic the beats toggle.
struct SimulatedOs {
    accent: Arc<AtomicBool>,
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

    fn accent_color(&self) -> Option<alacer_core::Color> {
        if self.accent.load(Ordering::Relaxed) {
            Some(OS_ACCENT)
        } else {
            None
        }
    }
    fn reduced_transparency(&self) -> bool {
        false
    }

    fn text_scale(&self) -> Option<f32> {
        None
    }
}

/// The demo's composition (ADR-0087): the canon palette whose accent
/// roles tint from the OS accent when the fact is present — the accent
/// itself, a pressed variant a step darker, the selection riding it —
/// and the canon palette when it is not (the composition that answers
/// the absence with its own). The composer reads no platform state:
/// the pipeline hands it the facts, it answers a palette — the pure
/// function the composition tier owns.
struct AccentFollowComposer;

impl AccentFollowComposer {
    /// The accent roles the OS accent resolves — the tint a following
    /// app borrows, the roles every other palette role leaves canon.
    fn tinted(accent: alacer_core::Color) -> Palette {
        Palette {
            accent,
            accent_pressed: alacer_core::Color::rgb(
                accent.r.saturating_mul(8) / 10,
                accent.g.saturating_mul(8) / 10,
                accent.b.saturating_mul(8) / 10,
            ),
            selection: accent.with_alpha(64),
            ..Palette::CANON
        }
    }
}

impl SettingComposer for AccentFollowComposer {
    fn palette(
        &self,
        _dark: bool,
        _reduced_motion: bool,
        _high_contrast: bool,
        accent: Option<alacer_core::Color>,
        _reduced_transparency: bool,
        _text_scale: Option<f32>,
    ) -> Palette {
        accent.map_or(Palette::CANON, Self::tinted)
    }

    fn name(&self) -> &'static str {
        "accent-follow"
    }
}

/// A probe that paints its whole bounds from the resolved *palette*
/// (the accent role) — the composition door's pixel evidence: the
/// palette the widgets borrow tints when — and only when — the
/// composition resolves it by the fact.
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
/// `PaintCtx::accent_color` read directly off the paint context, the
/// role-independent decision the vocabulary consults, the absence ink
/// where no accent is reported (the fact riding its presence, never a
/// guessed color), riding beside the palette the composition tinted.
struct AccentProbe;

impl Widget for AccentProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("the OS accent probe")
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
            color: ctx
                .accent_color()
                .unwrap_or(alacer_core::Color::rgb(148, 148, 155)),
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

/// The demo's beats, walked before the window opens: the
/// *accent-follow* composition installs over the canon default (the
/// honest no-op), the OS reports an accent and the accent-only flip
/// lands — the tinted palette mounts beside the fact, the OS stops
/// reporting one and the absence moves back, and the repeated ask is
/// the honest no-op.
fn run_beats(pipeline: &mut PassPipeline, accent: &Arc<AtomicBool>) {
    settle(pipeline);

    // Beat one: the app installs the composition door — the absence's
    // answer *is* the canon palette the default build resolves, so the
    // install is the honest no-op (`false`, no facts).
    let landed = pipeline.set_composer(Box::new(AccentFollowComposer));
    settle(pipeline);
    tracing::info!(
        target: "alacer::accent_follow",
        landed,
        "beat 1: the accent-follow composition installed — the canon default already resolved the absence's answer"
    );

    // Beat two: the OS reports an accent — the accent-only flip lands
    // where the three bool facts are still (the landing the M79 check
    // would have swallowed), the tinted palette mounts beside the fact
    // (the theme fact tagged "accent-follow" streaming behind the
    // setting fact), the whole tree repaints once.
    accent.store(true, Ordering::Relaxed);
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        accent: Arc::clone(accent),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::accent_follow",
        landed,
        "beat 2: the OS reported an accent — the tinted palette landed beside the fact"
    );

    // Beat three: the OS stops reporting one — the absence is itself a
    // fact move, the flip lands, and the composition re-resolves its
    // own answer (the canon palette, the tint leaving with the fact).
    accent.store(false, Ordering::Relaxed);
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        accent: Arc::clone(accent),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::accent_follow",
        landed,
        "beat 3: the OS stopped reporting an accent — the composition resolved its own answer"
    );

    // Beat four: the same facts again — the door answers `false`, no
    // broadcast, no facts (the idle-frame property, A1's rule at the
    // setting tier). The honest absence: the log grows by nothing.
    let landed = pipeline.set_settings(Box::new(SimulatedOs {
        accent: Arc::clone(accent),
    }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::accent_follow",
        landed,
        "beat 4: the no-op flip answered false — the log grew by nothing"
    );

    // Beat five: the canon settings install by name — the canon facts
    // where the canon facts already are: the no-op, the same door a
    // consumer that swapped away swaps back through.
    let landed = pipeline.set_settings(Box::new(CanonSettings));
    settle(pipeline);
    tracing::info!(
        target: "alacer::accent_follow",
        landed,
        "beat 5: the canon facts installed where the canon facts already are — the no-op"
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
    let _ = pipeline.add_child(root, Label::new("The accent fact").boxed());
    let _ = pipeline.add_child(
        root,
        Label::new(
            "The palette tints from the OS's accent — the app composes it, the pipeline swaps it.",
        )
        .boxed(),
    );
    let _ = pipeline.add_child(root, RoleProbe.boxed());
    let _ = pipeline.add_child(root, AccentProbe.boxed());
    let _ = pipeline.add_child(root, Button::new("Follow the OS").boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The trail: every flip and the palette it composed, tagged").boxed(),
    );
    settle(&mut pipeline);

    // The beats: the OS's flips land through the adapter's door, the
    // composition resolves beside each, the records stream — the log's
    // own facts, read back on exit.
    let accent = Arc::new(AtomicBool::new(false));
    run_beats(&mut pipeline, &accent);

    for record in flips.borrow().iter() {
        tracing::info!(
            target: "alacer::accent_follow",
            dark = record.dark,
            reduced_motion = record.reduced_motion,
            high_contrast = record.high_contrast,
            accent = ?record.accent,
            "the trail streamed this OS flip"
        );
    }
    for record in swaps.borrow().iter() {
        tracing::info!(
            target: "alacer::accent_follow",
            palette = %record.palette,
            "the trail streamed this composed swap"
        );
    }

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer palette follows the OS accent",
        (VIEWPORT.width, VIEWPORT.height),
        BackendPreference::Wgpu.software_env(std::env::var_os("ALACER_SOFTWARE").as_deref()),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
