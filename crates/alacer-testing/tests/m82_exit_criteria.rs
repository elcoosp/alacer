//! M82 exit-criteria integration tests (§7.2 pass 6, the §18 M82 row):
//! the canvas — the app's own paint closure through the paint seam.
//!
//! The row's contract, pinned end to end on the real pipeline (A4 —
//! real layout, real paint rasterization through the harness): `Canvas`
//! mounts a hoisted painter closure, `PaintCtx` handing the resolved
//! facts beside the clip stack, the painter's primitives appending
//! through the paint pass's own commit, and the repaint currency every
//! widget shares queues the canvas's `PAINT` mask at the app's own
//! surface — a theme flip re-runs the painter, the shared broadcast the
//! palette facts ride. The identity rule's reconciler half (a hoisted
//! painter re-hung across runs re-hangs quietly) is pinned at the
//! element's home (`crates/alacer-widgets/src/canvas.rs`
//! `painter_identity_is_the_repaint_contract`, the view crate's patch
//! path); this suite pins the cross-crate halves the harness drives.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect};
use alacer_core::passes::FrameReport;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{Size2, Style};
use alacer_core::theme::{CanonPalette, ThemeSeam};
use alacer_core::widget::{A11yAction, LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::BoxedWidget as _;

/// The canon palette's accent — the ink the painter paints so the
/// resolved-facts half of the contract is visible in the pixels.
const CANON_ACCENT: Color = Color::rgb(37, 99, 235);

/// A palette whose accent the flip resolves differently — the swap the
/// theme door lands, the broadcast every painter rides.
struct Midnight;

impl ThemeSeam for Midnight {
    fn palette(&self) -> alacer_core::theme::Palette {
        alacer_core::theme::Palette {
            accent: Color::rgb(220, 38, 38),
            ..alacer_core::theme::Palette::CANON
        }
    }
    fn name(&self) -> &'static str {
        "midnight"
    }
}

/// How many times the painter ran — the repaint currency's probe.
type PaintCount = std::sync::Arc<std::sync::atomic::AtomicUsize>;

/// A canvas whose painter paints its whole bounds from the resolved
/// palette (the facts the flip hands the painter) and counts its own
/// runs — the pixel evidence and the repaint probe in one closure.
struct CountingCanvas {
    name: &'static str,
    runs: PaintCount,
    accent_seen: std::sync::Arc<std::sync::Mutex<Vec<[u8; 4]>>>,
}

impl CountingCanvas {
    fn new(
        name: &'static str,
    ) -> (
        Self,
        PaintCount,
        std::sync::Arc<std::sync::Mutex<Vec<[u8; 4]>>>,
    ) {
        let runs = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let accent_seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let canvas = Self {
            name,
            runs: std::sync::Arc::clone(&runs),
            accent_seen: std::sync::Arc::clone(&accent_seen),
        };
        (canvas, runs, accent_seen)
    }
}

impl Widget for CountingCanvas {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Canvas
    }
    fn label(&self) -> LabelSource {
        LabelSource::text(self.name)
    }
    fn style(&self) -> Style {
        Style {
            size: Size2 {
                width: Some(96.0),
                height: Some(64.0),
            },
            ..Style::default()
        }
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        self.runs.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let accent = ctx.palette().accent;
        self.accent_seen
            .lock()
            .expect("accent recorder")
            .push([accent.r, accent.g, accent.b, accent.a]);
        let size = ctx.size();
        ctx.push_quad(Quad {
            rect: Rect::new(
                Point::new(8.0, 8.0),
                alacer_core::geom::Size::new(size.width - 16.0, size.height - 16.0),
            ),
            color: accent,
            corner_radius: 0.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// Ticks until no pass has work (the m-suite's settle shape).
fn settle(harness: &mut TestHarness) -> FrameReport {
    for _ in 0..16 {
        let report = harness.tick();
        if report.is_idle() {
            return report;
        }
    }
    panic!("the suite's frames never settled");
}

/// The pixel at `offset` inside the mounted surface, as RGBA bytes.
fn pixel_at(
    harness: &mut TestHarness,
    origin: alacer_core::geom::Point,
    dx: f32,
    dy: f32,
) -> [u8; 4] {
    let pixmap = harness.pixels();
    let x = alacer_core::num::f32_to_u32(origin.x + dx);
    let y = alacer_core::num::f32_to_u32(origin.y + dy);
    let px = pixmap
        .pixel(x, y)
        .unwrap_or_else(|| panic!("pixel ({x}, {y}) outside the viewport"));
    [px.red(), px.green(), px.blue(), px.alpha()]
}

/// The app's own paint closure paints through the paint pass's own
/// commit: the painter's quad lands in the rasterized frame at the
/// canvas's bounds, and the painter saw the frame's resolved facts —
/// the palette's accent it painted *is* the canon accent.
#[test]
fn the_painter_paints_the_app_surface_with_the_resolved_facts() {
    let mut harness = TestHarness::new(240.0, 160.0);
    let (canvas, runs, accent_seen) = CountingCanvas::new("The waveform");
    let node = harness.mount(canvas.boxed());
    settle(&mut harness);

    let bounds = harness.bounds(node);
    let ink = pixel_at(&mut harness, bounds.origin, 48.0, 32.0);
    assert_eq!(
        [ink[0], ink[1], ink[2]],
        [CANON_ACCENT.r, CANON_ACCENT.g, CANON_ACCENT.b],
        "the painter's quad is in the frame, painted from the resolved facts"
    );
    assert_eq!(
        runs.load(std::sync::atomic::Ordering::Relaxed),
        1,
        "one paint pass, one painter run — the idle contract at the surface"
    );
    let seen = accent_seen.lock().expect("accent recorder").clone();
    assert_eq!(
        seen,
        vec![[
            CANON_ACCENT.r,
            CANON_ACCENT.g,
            CANON_ACCENT.b,
            CANON_ACCENT.a
        ]],
        "the painter saw exactly the canon facts the pass resolved"
    );
    assert!(
        harness.query(Role::Canvas).named("The waveform").exists(),
        "the canvas is a named surface in the a11y tree (ADR-5)"
    );
}

/// The repaint currency every widget shares queues the canvas's `PAINT`
/// mask: a theme flip's broadcast re-runs the painter, the app's own
/// surface repainting beside every widget — and the painter answers by
/// the newly resolved facts.
#[test]
fn the_shared_currency_repaints_the_canvas_surface() {
    let mut harness = TestHarness::new(240.0, 160.0);
    let (canvas, runs, accent_seen) = CountingCanvas::new("The waveform");
    let _node = harness.mount(canvas.boxed());
    settle(&mut harness);
    assert_eq!(
        runs.load(std::sync::atomic::Ordering::Relaxed),
        1,
        "the settled frame ran the painter once"
    );

    // A palette whose accent the flip resolves differently — the swap
    // the theme door lands, the broadcast every painter rides.
    assert!(
        harness.pipeline_mut().set_theme(Box::new(Midnight)),
        "the palette swap lands"
    );
    settle(&mut harness);

    assert_eq!(
        runs.load(std::sync::atomic::Ordering::Relaxed),
        2,
        "the flip's PAINT broadcast re-ran the painter — the shared currency"
    );
    let seen = accent_seen.lock().expect("accent recorder").clone();
    assert_eq!(seen.len(), 2, "one run per paint pass");
    assert_eq!(
        seen[1][0..3],
        [220, 38, 38],
        "the re-run saw the newly resolved facts"
    );
    // The canon palette swaps back through the same door — the provided
    // canon installable by name (the theme door's own rule).
    assert!(
        harness.pipeline_mut().set_theme(Box::new(CanonPalette)),
        "the canon palette installs back"
    );
    settle(&mut harness);
    assert_eq!(
        accent_seen.lock().expect("accent recorder").len(),
        3,
        "the return flip re-ran the painter the same way"
    );
}

/// A canvas has no write door: every AT action is refused honestly (the
/// M11 rule at the canvas tier) — the named surface reads, never writes.
#[test]
fn the_canvas_refuses_every_at_action_honestly() {
    let mut harness = TestHarness::new(240.0, 160.0);
    let (canvas, _runs, _seen) = CountingCanvas::new("The waveform");
    let _node = harness.mount(canvas.boxed());
    settle(&mut harness);

    assert!(
        !harness.act(Role::Canvas, "The waveform", A11yAction::DefaultPress),
        "a canvas has no write door — the press is refused honestly"
    );
}

/// The corpus pin re-agrees: the canvas's pricing door is the testing
/// crate's own bench (`canvas`), already in the published corpus — the
/// suite grows no case, the pin asserting the count this cut publishes.
const EXPECTED_BENCH_COUNT: usize = 207;

#[test]
fn m82_the_corpus_pin_re_agrees_at_the_cut() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let baselines = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("benchmarks/baselines/baselines.json"))
            .expect("the published baseline is readable"),
    )
    .expect("the published baseline is valid JSON");
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the cut"
    );
}
