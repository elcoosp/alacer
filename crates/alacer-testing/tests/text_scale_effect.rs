//! The text-scale effect (ADR-0160): ADR-0159's own deferral cashes —
//! *"Not in this milestone: a text-scaling **effect** in the text tier
//! (the fact is the decision's input, the typography a widget's own
//! vocabulary — priced first when a widget needs it)."* A widget needs
//! it now: typography is the one consumer the M155 cut left reading the
//! fact as a probe instead of following it as a size.
//!
//! This suite pins the effect end to end on the real pipeline (A4 —
//! real layout host, real text seam, real rasterization): a landed
//! text-scale flip re-measures the tree (`LAYOUT` beside `PAINT` — the
//! one flip whose reach changes sizes, not only inks), the text tier
//! multiplies the declared sizes at the seam's one door (measure,
//! paint, and the caret's mapping see the same scaled geometry), the
//! same-scale ask is the honest no-op (the idle contract at the effect
//! tier), and the absence returns the declared sizes (the scale rides
//! its presence, `None` the base — never a guessed multiplier).

use std::sync::{Arc, Mutex};

use alacer_core::color::Color;
use alacer_core::geom::Point;
use alacer_core::passes::FrameReport;
use alacer_core::scene::{Glyph, PaintCtx};
use alacer_core::settings::SettingSeam;
use alacer_core::style::{Size2, Style};
use alacer_core::text_seam;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::BoxedWidget as _;

/// The scale the suite flips: one point two five — the OS's ask the
/// beats land, the multiplier the vocabulary multiplies by.
const OS_SCALE: f32 = 1.25;

/// The declared size every probe starts at — the base the multiplier
/// applies to.
const DECLARED: f32 = 16.0;

/// The facts the suite flips: all six OS facts, one seam (the m155
/// suite's simulated OS — the shape the cut itself shipped).
#[expect(
    clippy::struct_excessive_bools,
    reason = "the suite's simulated OS holds the seam's facts the beats flip — the bools *are* the facts (ADR-0080's rule), one field per fact the sixth cut grew beside the five"
)]
struct FixedSettings {
    dark: bool,
    reduced_motion: bool,
    high_contrast: bool,
    accent: Option<Color>,
    reduced_transparency: bool,
    text_scale: Option<f32>,
}

impl SettingSeam for FixedSettings {
    fn dark_mode(&self) -> bool {
        self.dark
    }
    fn reduced_motion(&self) -> bool {
        self.reduced_motion
    }
    fn high_contrast(&self) -> bool {
        self.high_contrast
    }
    fn accent_color(&self) -> Option<Color> {
        self.accent
    }
    fn reduced_transparency(&self) -> bool {
        self.reduced_transparency
    }
    fn text_scale(&self) -> Option<f32> {
        self.text_scale
    }
}

/// The scale ask that lands: every other fact still, the multiplier the
/// only mover (the landing the five-fact check would have swallowed).
fn scaled_ask() -> FixedSettings {
    FixedSettings {
        dark: false,
        reduced_motion: false,
        high_contrast: false,
        accent: None,
        reduced_transparency: false,
        text_scale: Some(OS_SCALE),
    }
}

/// The honest absence: every fact still, `None` where the ask rode —
/// the platform reports no read, the declared sizes return.
fn absent_ask() -> FixedSettings {
    FixedSettings {
        dark: false,
        reduced_motion: false,
        high_contrast: false,
        accent: None,
        reduced_transparency: false,
        text_scale: None,
    }
}

/// A probe that paints real text through the seam's one door and
/// records the effective sizes the painted glyphs carry — the pixel
/// side of the effect: whatever the seam multiplies is what paint
/// draws, the same geometry measure sized the node with.
#[derive(Clone)]
struct GlyphProbe {
    font_size: f32,
    painted: Arc<Mutex<Vec<f32>>>,
}

impl GlyphProbe {
    fn clear(&self) {
        self.painted.lock().expect("probe recorder").clear();
    }
}

impl Widget for GlyphProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("glyph probe")
    }
    fn style(&self) -> Style {
        Style {
            size: Size2 {
                width: Some(128.0),
                height: Some(32.0),
            },
            ..Style::default()
        }
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        // The widget's own paint site, unchanged from before the effect:
        // the *declared* size goes in, the seam's multiplier comes out —
        // the one door every text path shares (ADR-9).
        text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs("Ag", self.font_size, None);
            let mut painted = self.painted.lock().expect("probe recorder");
            for glyph in &layout.glyphs {
                painted.push(glyph.size);
                ctx.push_glyph(Glyph {
                    position: Point::new(glyph.x, glyph.y),
                    size: glyph.size,
                    color: ctx.palette().text,
                    font_id: glyph.font_id,
                    glyph_id: glyph.glyph_id,
                    clip: None,
                });
            }
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

/// The distinct effective sizes a paint pass recorded, sorted.
fn recorded_sizes(painted: &[f32]) -> Vec<f32> {
    let mut sizes = painted.to_vec();
    sizes.sort_by(f32::total_cmp);
    sizes.dedup();
    sizes
}

/// The headline beat: the flip re-measures the tree and the typography
/// follows the ask. A plain `Label` (the measure path — its bounds are
/// the measured text's) and a `GlyphProbe` (the paint path) both ride
/// the same seam door: bounds grow by the multiplier, the painted
/// glyphs carry the scaled size, and the flip's reach is the
/// re-measure the sizes need (`measure_calls` grows across the settle —
/// the `LAYOUT` reach a palette-only flip never queues).
#[test]
fn the_flip_re_measures_and_the_typography_follows_the_ask() {
    // The measure side: a plain label's bounds are its measured text.
    // The root stretches to the viewport, so the label rides under a
    // column — its bounds shrink-wrap to the text the measure pass
    // resolved, exactly what the multiplier changes.
    let mut harness = TestHarness::new(320.0, 240.0);
    let column = harness.mount(alacer_widgets::Column::new().boxed());
    let label = harness.add_child(
        column,
        alacer_widgets::Label::new("the quick brown fox").boxed(),
    );
    settle(&mut harness);
    let before = harness.bounds(label);
    assert!(before.size.height > 0.0, "the label measured its text");

    let measures_before = harness.measure_calls();
    assert!(
        harness.pipeline_mut().set_settings(Box::new(scaled_ask())),
        "the scale ask lands"
    );
    settle(&mut harness);
    let after = harness.bounds(label);

    // The declared size is the base the multiplier applies to: 16 → 20.
    let (h0, h1) = (before.size.height, after.size.height);
    let height_ratio = h1 / h0;
    assert!(
        (height_ratio - OS_SCALE).abs() < 0.05,
        "the label's height follows the ask: {h0} → {h1} \
         (ratio {height_ratio}, expected ≈ {OS_SCALE})"
    );
    let measures_after = harness.measure_calls();
    assert!(
        measures_after > measures_before,
        "the flip re-measured the tree ({measures_before} → {measures_after}): \
         the sizes the multiplier changes need the LAYOUT reach"
    );
}

/// The paint side: the same multiplier at the paint site's own door —
/// a widget whose paint shapes through the seam with the *declared*
/// size draws the *effective* one, the geometry measure sized the
/// node with. The absence returns the declared sizes: the scale rides
/// its presence, `None` the base (never a remembered multiplier).
#[test]
fn the_painted_glyphs_carry_the_ask_and_the_absence_restores() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let probe = GlyphProbe {
        font_size: DECLARED,
        painted: Arc::new(Mutex::new(Vec::new())),
    };
    let painted = Arc::clone(&probe.painted);
    let _probe_node = harness.mount(probe.clone().boxed());
    settle(&mut harness);
    assert_eq!(
        recorded_sizes(&painted.lock().expect("probe recorder")),
        vec![DECLARED],
        "the absence paints the declared size"
    );

    probe.clear();
    assert!(harness.pipeline_mut().set_settings(Box::new(scaled_ask())));
    settle(&mut harness);
    let scaled = DECLARED * OS_SCALE;
    assert_eq!(
        recorded_sizes(&painted.lock().expect("probe recorder")),
        vec![scaled],
        "the ask multiplies the declared size at the paint site's own door"
    );

    probe.clear();
    assert!(harness.pipeline_mut().set_settings(Box::new(absent_ask())));
    settle(&mut harness);
    assert_eq!(
        recorded_sizes(&painted.lock().expect("probe recorder")),
        vec![DECLARED],
        "the absence returns the declared sizes"
    );
}

/// The idle contract at the effect tier: an ask that lands where the
/// multiplier already is answers `false`, queues nothing, and the
/// settled tree re-measures nothing — the flip's equality check prices
/// the ask before the reach walks.
#[test]
fn the_same_scale_ask_is_the_honest_no_op() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let column = harness.mount(alacer_widgets::Column::new().boxed());
    let label = harness.add_child(column, alacer_widgets::Label::new("steady").boxed());
    settle(&mut harness);
    assert!(harness.pipeline_mut().set_settings(Box::new(scaled_ask())));
    settle(&mut harness);
    let bounds = harness.bounds(label);
    let measures = harness.measure_calls();

    assert!(
        !harness.pipeline_mut().set_settings(Box::new(scaled_ask())),
        "the same scale is the honest no-op"
    );
    settle(&mut harness);
    assert_eq!(harness.bounds(label), bounds, "no sizes moved");
    assert_eq!(
        harness.measure_calls(),
        measures,
        "the no-op re-measured nothing — the idle contract at the effect tier"
    );
}

/// The absence returns the declared sizes at the measure side, too:
/// the label's bounds come back exactly what they were before the ask
/// landed — the multiplier leaves no residue once the fact rides its
/// `None` again.
#[test]
fn the_absence_returns_the_declared_bounds() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let column = harness.mount(alacer_widgets::Column::new().boxed());
    let label = harness.add_child(
        column,
        alacer_widgets::Label::new("the quick brown fox").boxed(),
    );
    settle(&mut harness);
    let before = harness.bounds(label);

    assert!(harness.pipeline_mut().set_settings(Box::new(scaled_ask())));
    settle(&mut harness);
    let scaled = harness.bounds(label);
    assert!(
        scaled.size.height > before.size.height,
        "the ask grew the bounds before the absence restores them"
    );

    assert!(harness.pipeline_mut().set_settings(Box::new(absent_ask())));
    settle(&mut harness);
    let restored = harness.bounds(label);
    assert_eq!(
        (restored.size.width, restored.size.height),
        (before.size.width, before.size.height),
        "the absence returns the declared bounds exactly"
    );
}
