//! The OS setting seam driven from the consumer side (§7.2 pass 8,
//! ADR-0080): the OS's dark-mode fact is a seam — the platform adapter
//! installs one ([`alacer_platform::WinitSettings`], pushed through the
//! door on the window's opening fact and every `ThemeChanged` event),
//! the flip broadcasts `Change::PAINT` on the tree (the mask's own
//! documented semantics), the fact resolves once per paint pass into
//! `PaintCtx` beside the palette, and the flip streams one
//! `InspectEvent::Setting` the tick after the door answered. A flip
//! that lands where the fact already is answers `false` and streams
//! nothing — the idle-frame property at the setting tier.
//!
//! The demo walks the beats before the window opens, simulating the OS
//! fact the adapter pushes: the canon light builds first, the OS flips
//! *dark* and repaints every node, a second *dark* ask is the honest
//! no-op (`false`, no fact, an idle frame), and *light* lands beside
//! them — the log's facts read back on exit. A fact probe paints from
//! `ctx.dark()` directly: the role-independent decision the vocabulary
//! consults, the fact never munging the palette the roles borrow.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example dark_mode
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

/// The viewport the demo's beats settle into (the trail demos' shape).
const VIEWPORT: Size = Size::new(480.0, 340.0);

/// The fact the demo's simulated OS holds — a plain `Copy` datum, the
/// shape any adapter rides (the platform's [`alacer_platform::WinitSettings`]
/// is the winit one; the demo's is its own, so the beats run headless).
struct SimulatedOs {
    dark: bool,
}

impl SettingSeam for SimulatedOs {
    fn dark_mode(&self) -> bool {
        self.dark
    }

    /// The simulated OS reports no motion preference — the demo flips
    /// the dark fact alone, the honest absence the winit adapter
    /// answers too (ADR-0083).
    fn reduced_motion(&self) -> bool {
        false
    }

    fn high_contrast(&self) -> bool {
        false
    }
}

/// A probe that paints its whole bounds from the resolved *fact* —
/// `PaintCtx::dark` read directly off the paint context, the
/// role-independent decision the vocabulary consults (the demo's pixel
/// evidence that the fact reached the paint pass).
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

/// The demo's beats, walked before the window opens: the canon light
/// builds, the OS flips *dark* and repaints every node, the second
/// *dark* ask is the honest no-op, *light* lands beside them.
fn run_beats(pipeline: &mut PassPipeline) {
    settle(pipeline);

    // Beat one: the OS pushes dark through the door — `true`, the
    // broadcast repaints the whole tree, the record streams the tick
    // after the door answered.
    let landed = pipeline.set_settings(Box::new(SimulatedOs { dark: true }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::dark_mode",
        landed,
        "beat 1: the OS flipped dark — the broadcast repainted the tree"
    );

    // Beat two: the same fact again — the door answers `false`, no
    // broadcast, no record (the idle-frame property, A1's rule at the
    // setting tier). The honest absence: the log grows by nothing.
    let landed = pipeline.set_settings(Box::new(SimulatedOs { dark: true }));
    settle(pipeline);
    tracing::info!(
        target: "alacer::dark_mode",
        landed,
        "beat 2: the no-op flip answered false — the log grew by nothing"
    );

    // Beat three: the OS pushes light back — the canon fact lands
    // beside the dark one, the second flip's record on the same log.
    let landed = pipeline.set_settings(Box::new(CanonSettings));
    settle(pipeline);
    tracing::info!(
        target: "alacer::dark_mode",
        landed,
        "beat 3: the OS pushed light back — the canon fact rides the door"
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
    let _ = pipeline.add_child(root, Label::new("The OS setting seam").boxed());
    let _ = pipeline.add_child(
        root,
        Label::new(
            "The OS's dark-mode fact is a seam — the flip repaints the tree, the palette stays the app's own.",
        )
        .boxed(),
    );
    let _ = pipeline.add_child(root, FactProbe.boxed());
    let _ = pipeline.add_child(root, Button::new("Dark").boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The trail: every flip that landed, named by its fact").boxed(),
    );
    settle(&mut pipeline);

    // The beats: the OS's flips land through the adapter's door, the
    // records stream — the log's own facts, read back on exit.
    run_beats(&mut pipeline);

    for record in trail.borrow().iter() {
        tracing::info!(
            target: "alacer::dark_mode",
            dark = record.dark,
            repainted = record.repainted,
            "the trail streamed this OS flip — the fact beside the reach it repainted (ADR-0082)"
        );
    }

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer OS setting flips",
        (VIEWPORT.width, VIEWPORT.height),
        BackendPreference::Wgpu.software_env(std::env::var_os("ALACER_SOFTWARE").as_deref()),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
