//! The runtime theme seam driven from the consumer side (§7.2 pass 8,
//! ADR-0079): the palette every widget borrows for its un-overridden
//! paint roles is a seam — the app's own `PassPipeline::set_theme` door
//! installs one, the swap broadcasts `Change::PAINT` on the tree (the
//! mask's own documented semantics), and the swap streams one
//! `InspectEvent::Theme` the tick after the door answered. A swap that
//! lands where the palette already is answers `false` and streams
//! nothing — the idle-frame property at the theme tier.
//!
//! The demo walks the beats before the window opens: the canon builds
//! first, *Midnight* lands and repaints every node, a second *Midnight*
//! ask is the honest no-op (`false`, no fact, an idle frame), and
//! *High contrast* lands beside them — the log's facts read back on
//! exit.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example theme_swap
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (named at startup, the
//! `BackendPreference::software_env` door).

use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::color::Color;
use alacer_core::geom::Size;
use alacer_core::inspect::{InspectEvent, InspectObserver, ThemeRecord};
use alacer_core::passes::PassPipeline;
use alacer_core::theme::{Palette, ThemeSeam};
use alacer_layout::LayoutHost;
use alacer_text::ThreadLocalMeasurer;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// The viewport the demo's beats settle into (the trail demos' shape).
const VIEWPORT: Size = Size::new(480.0, 340.0);

/// The midnight palette: a dark surface with an amber accent — every
/// role distinct from the canon's, so the swap visibly repaints.
struct MidnightPalette;

impl ThemeSeam for MidnightPalette {
    fn palette(&self) -> Palette {
        Palette {
            surface: Color::rgb(12, 14, 20),
            panel: Color::rgb(22, 26, 36),
            accent: Color::rgb(240, 180, 41),
            accent_pressed: Color::rgb(214, 156, 22),
            selection: Color::rgb(240, 180, 41).with_alpha(72),
            text: Color::rgb(230, 233, 240),
            text_muted: Color::rgb(148, 156, 172),
            border: Color::rgb(58, 64, 82),
            on_accent: Color::rgb(18, 16, 8),
        }
    }

    fn name(&self) -> &'static str {
        "midnight"
    }
}

/// The high-contrast palette: pure black and white.
struct HighContrastPalette;

impl ThemeSeam for HighContrastPalette {
    fn palette(&self) -> Palette {
        Palette {
            surface: Color::rgb(0, 0, 0),
            panel: Color::rgb(24, 24, 24),
            accent: Color::rgb(255, 255, 255),
            accent_pressed: Color::rgb(200, 200, 200),
            selection: Color::rgb(255, 255, 0).with_alpha(160),
            text: Color::rgb(255, 255, 255),
            text_muted: Color::rgb(180, 180, 180),
            border: Color::rgb(255, 255, 255),
            on_accent: Color::rgb(0, 0, 0),
        }
    }

    fn name(&self) -> &'static str {
        "high_contrast"
    }
}

/// The causal trail the pipeline owns: the observer half prices the
/// theme facts into the bounded ring the exit read-back drains.
struct Trail {
    swaps: Rc<RefCell<Vec<ThemeRecord>>>,
}

impl InspectObserver for Trail {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::Theme(record) = event else {
            return;
        };
        self.swaps.borrow_mut().push(record);
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

/// The demo's beats, walked before the window opens: the canon builds,
/// *Midnight* lands and repaints every node, the second *Midnight* ask
/// is the honest no-op, *High contrast* lands beside them.
fn run_beats(pipeline: &mut PassPipeline) {
    settle(pipeline);

    // Beat one: midnight lands — `true`, the broadcast repaints the
    // whole tree, the record streams the tick after the door answered.
    let landed = pipeline.set_theme(Box::new(MidnightPalette));
    settle(pipeline);
    tracing::info!(
        target: "alacer::theme_swap",
        landed,
        "beat 1: midnight landed — the broadcast repainted the tree"
    );

    // Beat two: the same palette again — the door answers `false`, no
    // broadcast, no record (the idle-frame property, A1's rule at the
    // theme tier). The honest absence: the log grows by nothing.
    let landed = pipeline.set_theme(Box::new(MidnightPalette));
    settle(pipeline);
    tracing::info!(
        target: "alacer::theme_swap",
        landed,
        "beat 2: the no-op swap answered false — the log grew by nothing"
    );

    // Beat three: high contrast lands beside the midnight fact — the
    // second swap's tag on the same log.
    let landed = pipeline.set_theme(Box::new(HighContrastPalette));
    settle(pipeline);
    tracing::info!(
        target: "alacer::theme_swap",
        landed,
        "beat 3: high contrast landed"
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

    // The causal trail: the observer half prices the theme facts into
    // the bounded ring the exit read-back drains.
    let trail = Rc::new(RefCell::new(Vec::new()));
    pipeline.set_observer(Box::new(Trail {
        swaps: Rc::clone(&trail),
    }));

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(root, Label::new("The runtime theme seam").boxed());
    let _ = pipeline.add_child(
        root,
        Label::new(
            "Every un-overridden role follows the installed palette — the app's overrides stay put.",
        )
        .boxed(),
    );
    let _ = pipeline.add_child(root, Button::new("Canon").boxed());
    let _ = pipeline.add_child(root, Button::new("Midnight").boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The trail: every swap that landed, named by its palette tag").boxed(),
    );
    settle(&mut pipeline);

    // The beats: the swaps land through the app's own door, the records
    // stream — the log's own facts, read back on exit.
    run_beats(&mut pipeline);

    for record in trail.borrow().iter() {
        tracing::info!(
            target: "alacer::theme_swap",
            palette = %record.palette,
            "the trail streamed this palette's swap"
        );
    }

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer theme swaps",
        (VIEWPORT.width, VIEWPORT.height),
        BackendPreference::Wgpu.software_env(std::env::var_os("ALACER_SOFTWARE").as_deref()),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
