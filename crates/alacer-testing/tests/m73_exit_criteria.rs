//! M73 exit-criteria integration tests (§13.3, ADR-0079): the runtime
//! theme seam. The palette every widget borrowed for its defaults was
//! ten compile-time constants — "make it dark" meant rebuilding every
//! widget, and no door answered the ask at all. Since M73 the palette
//! is a seam: the app installs any [`alacer_core::theme::ThemeSeam`],
//! the paint pass resolves the roles once per pass, the widgets paint
//! the roles their app did not override, and the swap — a write that
//! repaints every node — streams one
//! [`alacer_core::inspect::ThemeRecord`] on the causal log.
//!
//! Every test runs the real pipeline (A4): real layout, real paint
//! rasterization through the harness, the records through the harness
//! trail door. The canon palette pins the constants the corpus's
//! blessed snapshots were blessed with — a retheme must not silently
//! re-paint a default build.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect};
use alacer_core::inspect::{InspectEvent, ThemeRecord};
use alacer_core::passes::{FrameReport, PassKind};
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{Size2, Style};
use alacer_core::theme::{CanonPalette, Palette, ThemeSeam};
use alacer_core::trail::TrailLog;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::{TestHarness, pixel_diff_ratio};
use alacer_widgets::theme;
use alacer_widgets::{BoxedWidget as _, Button, Label};

/// The custom palette the suite swaps to: a midnight surface with an
/// amber accent — every role distinct from the canon's, so a stale
/// resolution cannot pass for the swap.
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

/// A probe that paints its whole bounds in one palette role — the
/// seam's resolution read directly off the paint context,
/// deterministic pixels with no glyph dependency.
struct RoleProbe {
    role_name: &'static str,
}

impl RoleProbe {
    /// A probe painted in the accent role.
    fn accent() -> Self {
        Self {
            role_name: "accent probe",
        }
    }

    /// A probe painted in the muted-text role.
    fn muted() -> Self {
        Self {
            role_name: "muted probe",
        }
    }

    fn color(&self, palette: &Palette) -> Color {
        match self.role_name {
            "accent probe" => palette.accent,
            _ => palette.text_muted,
        }
    }
}

impl Widget for RoleProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text(self.role_name)
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
        let palette = ctx.palette();
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), ctx.size()),
            color: self.color(&palette),
            corner_radius: 0.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// Reads one pixel inside `bounds`, inset past the corner radius.
fn pixel_at(harness: &mut TestHarness, bounds: Rect, dx: f32, dy: f32) -> [u8; 4] {
    let pixmap = harness.pixels();
    let x = alacer_core::num::f32_to_u32((bounds.origin.x + dx).max(0.0));
    let y = alacer_core::num::f32_to_u32((bounds.origin.y + dy).max(0.0));
    let px = pixmap
        .pixel(x, y)
        .unwrap_or_else(|| panic!("pixel ({x}, {y}) outside the viewport"));
    [px.red(), px.green(), px.blue(), px.alpha()]
}

/// Ticks until no pass has work (the m-suite's settle shape): the last
/// report answers, so the beat can assert on the frame the work rode.
fn settle(harness: &mut TestHarness) -> FrameReport {
    for _ in 0..16 {
        let report = harness.tick();
        if report.is_idle() {
            return report;
        }
    }
    panic!("the suite's frames never settled");
}

/// The streamed theme facts, oldest first (the m-suite's filter shape:
/// the assertion source is the trail's own records, never internals).
fn theme_facts(trail: &TrailLog) -> Vec<ThemeRecord> {
    trail
        .look()
        .into_iter()
        .filter_map(|event| match event {
            InspectEvent::Theme(record) => Some(record),
            _ => None,
        })
        .collect()
}

/// The canon palette pins the constants the blessed corpus was blessed
/// with: a default build paints exactly what 0.16 painted — the pin
/// the snapshots' green depends on.
#[test]
fn m73_the_canon_palette_pins_the_constants() {
    let _guard = alacer_core::flux::batch_test_lock();
    let canon = CanonPalette.palette();
    assert_eq!(canon.surface, theme::SURFACE);
    assert_eq!(canon.panel, theme::PANEL);
    assert_eq!(canon.accent, theme::ACCENT);
    assert_eq!(canon.accent_pressed, theme::ACCENT_PRESSED);
    assert_eq!(canon.selection, theme::SELECTION);
    assert_eq!(canon.text, theme::TEXT);
    assert_eq!(canon.text_muted, theme::TEXT_MUTED);
    assert_eq!(canon.border, theme::BORDER);
    assert_eq!(canon.on_accent, Color::WHITE, "the button's text default");
}

/// The paint pass resolves the installed palette: the probe's pixels
/// answer the canon accent before the swap and the midnight accent
/// after — the same node, no rebuild, one broadcast.
#[test]
fn m73_the_paint_resolves_the_installed_palette() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let probe = harness.mount(RoleProbe::accent().boxed());
    settle(&mut harness);
    let bounds = harness.bounds(probe);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [37, 99, 235, 255],
        "the canon accent paints before any swap"
    );

    let landed = harness.pipeline_mut().set_theme(Box::new(MidnightPalette));
    assert!(landed, "a distinct palette lands");
    // The swap's own frame: the broadcast's masks drain into this
    // tick's dirty map, the paint pass re-emits the layers.
    let report = harness.tick();
    assert!(
        report.pass(PassKind::Paint).is_some_and(|run| run.ran),
        "the swap's broadcast repaints the frame"
    );
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [240, 180, 41, 255],
        "the midnight accent paints after the swap"
    );
}

/// The roles resolve independently: the muted role answers midnight's
/// muted color, never the text's — one palette, nine named roles.
#[test]
fn m73_the_roles_resolve_independently() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let muted = harness.mount(RoleProbe::muted().boxed());
    settle(&mut harness);
    let bounds = harness.bounds(muted);
    harness.pipeline_mut().set_theme(Box::new(MidnightPalette));
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [148, 156, 172, 255],
        "the muted role resolved midnight's text_muted"
    );
}

/// The swap streams one `ThemeRecord` the tick after it lands — the
/// palette's tag beside the door, never the palette's colors (the
/// payload rule: the log names the palette, `PaintCtx` carries it).
#[test]
fn m73_the_swap_streams_one_theme_record() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let _probe = harness.mount(RoleProbe::accent().boxed());
    settle(&mut harness);
    assert!(
        theme_facts(&trail).is_empty(),
        "the frames that swapped nothing stream no theme fact"
    );

    harness.pipeline_mut().set_theme(Box::new(MidnightPalette));
    settle(&mut harness);
    let facts = theme_facts(&trail);
    assert_eq!(facts.len(), 1, "one fact per swap that landed");
    assert_eq!(facts[0].palette, "midnight");
}

/// A swap that lands where the palette already is queues no masks and
/// no facts (the idle-frame property, A1's rule at the theme tier).
#[test]
fn m73_the_no_op_swap_queues_nothing() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let _probe = harness.mount(RoleProbe::accent().boxed());
    settle(&mut harness);
    harness.pipeline_mut().set_theme(Box::new(MidnightPalette));
    settle(&mut harness);
    let before = theme_facts(&trail).len();
    let counters = harness.counters();

    let landed = harness.pipeline_mut().set_theme(Box::new(MidnightPalette));
    assert!(!landed, "the same palette is a no-op");
    let report = harness.tick();
    assert!(report.is_idle(), "the no-op repaints nothing");
    assert_eq!(
        harness.counters().paint_runs,
        counters.paint_runs,
        "the no-op's frame re-committed no layer"
    );
    assert_eq!(
        theme_facts(&trail).len(),
        before,
        "the no-op streams no fact"
    );
}

/// An app-overridden color stays overridden: the swap re-resolves the
/// roles the widgets borrowed, never the props the app set.
#[test]
fn m73_the_overrides_stay_explicit() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(260.0, 120.0);
    let pinned = Color::rgb(180, 40, 90);
    let button = harness.mount(Button::new("pinned").colors(pinned, pinned).boxed());
    settle(&mut harness);
    let bounds = harness.bounds(button);
    harness.pipeline_mut().set_theme(Box::new(MidnightPalette));
    settle(&mut harness);
    let painted = pixel_at(&mut harness, bounds, 12.0, bounds.size.height * 0.5);
    assert_eq!(
        [painted[0], painted[1], painted[2]],
        [180, 40, 90],
        "the explicit fill survives the retheme"
    );
}

/// The default-built vocabulary follows the palette: a plain label
/// paints canon text before the swap and midnight text after — the
/// role the widget borrowed, re-resolved per paint pass.
#[test]
fn m73_the_default_vocabulary_follows_the_palette() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(260.0, 120.0);
    let _label = harness.mount(Label::new("follows").boxed());
    settle(&mut harness);
    let before = harness.pixels();

    harness.pipeline_mut().set_theme(Box::new(MidnightPalette));
    settle(&mut harness);
    let after = harness.pixels();
    assert!(
        pixel_diff_ratio(&before, &after) > 0.0,
        "the default label repainted in the new palette"
    );
}
