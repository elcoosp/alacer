//! M86 exit-criteria integration tests (§7.2 pass 8, ADR-0090): the
//! shaped composition lock — the composition that holds some facts and
//! follows the rest. ADR-0089's own pre-registered re-open rule cashed:
//! "a consumer needs a lock with *shape* (an accent that never moves
//! but a surface that follows dark, a motion-locked kiosk beside an
//! accent-free one) → the provided-composition rule grows the second
//! provided composer beside the lock — never a lock flag the facts
//! interpret." Through M85 the palette the facts could not move
//! followed nothing at all: [`alacer_core::settings::KioskComposer`]
//! answers one palette for every fact permutation, and the app's own
//! composition follows every fact or none by how it is written. The
//! shaped lock is the between: [`alacer_core::settings::ShapedComposer::over`]
//! rides a base composition (the app's own fact → palette mapping) and
//! holds the facts its [`alacer_core::settings::FactShape`] pins — a
//! pinned fact is substituted before the base resolves, so the OS's
//! flip lands (the fact moved, one `Setting` record streams) and the
//! composed answer is the palette the pinned fact already named — the
//! equality check swaps nothing for it, while every followed fact
//! moves the base's answer the way the composition tier always has.
//!
//! Every test runs the real pipeline (A4): real layout, real paint
//! rasterization through the harness, the records through the harness
//! trail door, the pixels through the suite's role probes — one probe
//! the palette's accent role (the fact the beats pin), one the
//! palette's surface role (the fact the beats follow).

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect};
use alacer_core::inspect::{InspectEvent, SettingRecord, ThemeRecord};
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::settings::{
    CanonComposer, FactShape, SettingComposer, SettingSeam, ShapedComposer,
};
use alacer_core::style::{Size2, Style};
use alacer_core::theme::Palette;
use alacer_core::trail::TrailLog;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;

/// The brand accent the beats pin: the look the checkout line ships
/// with, the color the operator's desktop accent must never tint.
const BRAND: Color = Color::rgb(255, 176, 64);

/// The OS accent the beats flip in: the desktop's pick, the fact the
/// shape holds out on purpose.
const OS_RED: Color = Color::rgb(255, 94, 58);

/// The duet's dark palette — the base composition's dark answer, every
/// role resolved (the boarding-gate shape the bench fixture ships).
const DUET_DARK: Palette = Palette {
    surface: Color::rgb(16, 18, 24),
    panel: Color::rgb(28, 31, 40),
    accent: Color::rgb(255, 176, 64),
    accent_pressed: Color::rgb(226, 152, 44),
    selection: Color::rgb(255, 176, 64).with_alpha(72),
    text: Color::rgb(226, 230, 238),
    text_muted: Color::rgb(142, 150, 168),
    border: Color::rgb(56, 62, 80),
    on_accent: Color::rgb(24, 16, 6),
};

/// The base composition's dark answer with `accent` riding its accent
/// roles — the palette the facts resolve to when the OS names an
/// accent (the m81 tint shape, dark half).
fn tinted_dark(accent: Color) -> Palette {
    Palette {
        accent,
        accent_pressed: accent,
        selection: accent.with_alpha(72),
        ..DUET_DARK
    }
}

/// The base composition's light answer with `accent` riding its accent
/// roles — the m81 tint shape's light half, over the canon palette.
fn tinted_light(accent: Color) -> Palette {
    Palette {
        accent,
        accent_pressed: accent,
        selection: accent.with_alpha(72),
        ..Palette::CANON
    }
}

/// The facts the beats flip: the dark fact and the accent fact, the
/// other two riding the honest absences (the m81 suite's seam shape,
/// grown the accent the beats need).
struct FixedSettings {
    dark: bool,
    accent: Option<Color>,
}

impl SettingSeam for FixedSettings {
    fn dark_mode(&self) -> bool {
        self.dark
    }
    fn reduced_motion(&self) -> bool {
        false
    }
    fn high_contrast(&self) -> bool {
        false
    }
    fn accent_color(&self) -> Option<Color> {
        self.accent
    }
}

/// The beats' seam, boxed: the dark and accent facts the beats flip,
/// the other two riding the honest absences (the m81 suite's shape).
fn settings(dark: bool, accent: Option<Color>) -> Box<dyn SettingSeam> {
    Box::new(FixedSettings { dark, accent })
}

/// The base composition the shape rides: a duet that follows dark and
/// tints its accent roles from the OS accent when the OS reports one —
/// the composition that answers *both* facts, so the shape has
/// something real to hold. Named, like every composition the records
/// can name.
struct DuetAccentComposer;

impl SettingComposer for DuetAccentComposer {
    fn palette(
        &self,
        dark: bool,
        _reduced_motion: bool,
        _high_contrast: bool,
        accent: Option<Color>,
    ) -> Palette {
        match (dark, accent) {
            (false, None) => Palette::CANON,
            (true, None) => DUET_DARK,
            (false, Some(accent)) => tinted_light(accent),
            (true, Some(accent)) => tinted_dark(accent),
        }
    }

    fn name(&self) -> &'static str {
        "duet-accent"
    }
}

/// A probe painting its whole bounds from one palette role — the
/// composition's pixel evidence: the role the probe paints moves when —
/// and only when — a composition resolves a palette whose role moved,
/// so the probe's pixel is the shape's own answer for that role.
struct RoleProbe {
    role: fn(Palette) -> Color,
}

impl Widget for RoleProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("shaped lock probe")
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
            color: (self.role)(ctx.palette()),
            corner_radius: 0.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// The pixel at `probe`'s center, as RGBA bytes (the m-suite's pixel
/// read — the probe is 64×64, the center 32 in from its origin).
fn center_pixel(harness: &mut TestHarness, probe: alacer_core::NodeId) -> [u8; 4] {
    let bounds = harness.bounds(probe);
    let pixmap = harness.pixels();
    let x = alacer_core::num::f32_to_u32((bounds.origin.x + 32.0).max(0.0));
    let y = alacer_core::num::f32_to_u32((bounds.origin.y + 32.0).max(0.0));
    let px = pixmap
        .pixel(x, y)
        .unwrap_or_else(|| panic!("pixel ({x}, {y}) outside the viewport"));
    [px.red(), px.green(), px.blue(), px.alpha()]
}

/// The pixel an opaque role color paints, as the probes paint it —
/// premultiplied of opaque is the color's own bytes.
fn painted(color: Color) -> [u8; 4] {
    [color.r, color.g, color.b, 0xFF]
}

/// The accent role, as a probe paints it.
fn accent(role: Palette) -> Color {
    role.accent
}

/// The probe's center pixel, asserted against the role color it must
/// paint — the beats' pixel evidence, one call per probe per beat.
fn assert_probe_paints(
    harness: &mut TestHarness,
    probe: alacer_core::NodeId,
    color: Color,
    message: &str,
) {
    assert_eq!(center_pixel(harness, probe), painted(color), "{message}");
}

/// The surface role, as a probe paints it.
fn surface(role: Palette) -> Color {
    role.surface
}

/// The trail's whole window, drained once — the events the beats
/// partition into their tiers (a drain clears whole: a fact streams
/// once, and no beat may re-drain another beat's window).
fn drain_events(trail: &TrailLog) -> Vec<InspectEvent> {
    trail.drain()
}

/// The trail's window, drained and partitioned once: the theme tier's
/// swap records beside the setting tier's landed facts.
fn drained(trail: &TrailLog) -> (Vec<ThemeRecord>, Vec<SettingRecord>) {
    let events = drain_events(trail);
    (theme_records(&events), setting_records(&events))
}

/// The theme tier's records, partitioned from a drained window (the
/// beats' swap facts — the tags naming the compositions that resolved).
fn theme_records(events: &[InspectEvent]) -> Vec<ThemeRecord> {
    events
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Theme(record) => Some(record.clone()),
            _ => None,
        })
        .collect()
}

/// The setting tier's records, partitioned from a drained window (the
/// beats' landed facts — the bools and the accent riding its presence).
fn setting_records(events: &[InspectEvent]) -> Vec<SettingRecord> {
    events
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Setting(record) => Some(*record),
            _ => None,
        })
        .collect()
}

/// The shaped lock holds the accent and follows the dark: the install
/// resolves the shape over the facts that are (the pinned accent rides
/// the base's light answer), the dark flip lands and the surface
/// follows (the base recomposes — one theme record tagged the shape's
/// own name), the OS's accent flip lands and the accent does not move
/// (the pinned fact the base never sees — the equality check's idle
/// answer), and the next dark flip is where the surface follows again.
#[test]
fn m86_the_shaped_lock_holds_the_accent_and_follows_the_dark() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let root = harness.mount(alacer_widgets::Column::new().padding(8.0).boxed());
    let accent_probe = harness.add_child(
        root,
        Box::new(RoleProbe { role: accent }) as Box<dyn Widget>,
    );
    let surface_probe = harness.add_child(
        root,
        Box::new(RoleProbe { role: surface }) as Box<dyn Widget>,
    );
    let trail = harness.trail();
    harness.settle();

    // The install resolves the shape over the facts that are: light,
    // no OS accent — the pinned brand accent riding the base's answer.
    harness.pipeline_mut().set_settings(settings(false, None));
    harness
        .pipeline_mut()
        .set_composer(Box::new(ShapedComposer::over(
            Box::new(DuetAccentComposer),
            FactShape::FOLLOW_ALL.pin_accent(BRAND),
        )));
    harness.settle();

    assert_probe_paints(
        &mut harness,
        accent_probe,
        tinted_light(BRAND).accent,
        "the install did not mount the pinned accent",
    );
    assert_probe_paints(
        &mut harness,
        surface_probe,
        tinted_light(BRAND).surface,
        "the install did not mount the base's light surface",
    );
    let (themes, _) = drained(&trail);
    assert_eq!(themes.len(), 1, "the install resolved once");
    assert_eq!(themes[0].palette, "shaped", "the record names the shape");

    // The dark flip lands — the fact moved — and the surface follows:
    // the dark fact is one the shape left unpinned, the base recomposes.
    let landed = harness.pipeline_mut().set_settings(settings(true, None));
    assert!(landed, "the dark flip's fact moved — the door lands it");
    harness.settle();
    assert_probe_paints(
        &mut harness,
        accent_probe,
        tinted_dark(BRAND).accent,
        "the pinned accent moved with the flip — the shape is broken",
    );
    assert_probe_paints(
        &mut harness,
        surface_probe,
        tinted_dark(BRAND).surface,
        "the followed surface did not follow the flip",
    );
    let (themes, flips) = drained(&trail);
    assert_eq!(themes.len(), 1, "the followed fact's swap landed once");
    assert_eq!(themes[0].palette, "shaped", "the record names the shape");
    assert_eq!(flips.len(), 1, "one flip, one setting record");
    assert!(flips[0].dark, "the record names the fact that moved");

    // The OS's accent flip lands — the accent fact moved — and the
    // accent does not move: the pinned fact the base never sees, the
    // equality check's idle answer through the shape.
    let landed = harness
        .pipeline_mut()
        .set_settings(settings(true, Some(OS_RED)));
    assert!(landed, "the accent flip's fact moved — the door lands it");
    harness.settle();
    assert_probe_paints(
        &mut harness,
        accent_probe,
        tinted_dark(BRAND).accent,
        "the OS's accent tinted the palette — the shape is broken",
    );
    let (themes, flips) = drained(&trail);
    assert!(
        themes.is_empty(),
        "the pinned fact's flip re-swapped the palette the shape holds: {themes:?}"
    );
    assert_eq!(flips.len(), 1, "one flip, one setting record");
    assert!(
        flips[0].accent.is_some(),
        "the record names the accent fact that moved"
    );

    // The next dark flip: the surface follows again, the accent still
    // held — the shape answering both ways in one composition.
    let landed = harness
        .pipeline_mut()
        .set_settings(settings(false, Some(OS_RED)));
    assert!(landed, "the second dark flip's fact moved");
    harness.settle();
    assert_probe_paints(
        &mut harness,
        accent_probe,
        tinted_light(BRAND).accent,
        "the pinned accent moved with the second flip",
    );
    assert_probe_paints(
        &mut harness,
        surface_probe,
        tinted_light(BRAND).surface,
        "the followed surface did not follow the second flip",
    );
    let events = drain_events(&trail);
    let themes = theme_records(&events);
    assert_eq!(themes.len(), 1, "one swap rode the flip's broadcast");
    assert_eq!(themes[0].palette, "shaped", "the record names the shape");

    harness.assert_all_named();
}

/// The install resolves the shape over the facts that are (the M75
/// rule at the shape's door): with the dark fact already held and the
/// OS accent already reported, the shape's install still mounts its own
/// answer — the pinned accent over the base's dark resolution — not a
/// fact-driven answer the shape then abandons.
#[test]
fn m86_the_install_resolves_the_shape_over_the_facts_that_are() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let root = harness.mount(alacer_widgets::Column::new().padding(8.0).boxed());
    let accent_probe = harness.add_child(
        root,
        Box::new(RoleProbe { role: accent }) as Box<dyn Widget>,
    );
    let surface_probe = harness.add_child(
        root,
        Box::new(RoleProbe { role: surface }) as Box<dyn Widget>,
    );
    let trail = harness.trail();
    harness.settle();

    harness
        .pipeline_mut()
        .set_settings(settings(true, Some(OS_RED)));
    harness
        .pipeline_mut()
        .set_composer(Box::new(ShapedComposer::over(
            Box::new(DuetAccentComposer),
            FactShape::FOLLOW_ALL.pin_accent(BRAND),
        )));
    harness.settle();

    assert_probe_paints(
        &mut harness,
        accent_probe,
        tinted_dark(BRAND).accent,
        "the install resolved a fact-driven accent instead of the pinned one",
    );
    assert_probe_paints(
        &mut harness,
        surface_probe,
        tinted_dark(BRAND).surface,
        "the install did not resolve the dark fact that is",
    );
    let (themes, _) = drained(&trail);
    assert_eq!(themes.len(), 1, "the install resolved once");
    assert_eq!(themes[0].palette, "shaped");
}

/// The shape that holds nothing is the composition it rides: over the
/// canon composition and the canon facts, the follow-all shape's
/// install is the honest no-op the door answers `false` on (the M75
/// rule at the shape's own door — installing the palette already
/// installed swaps nothing, streams nothing).
#[test]
fn m86_the_follow_all_shape_over_the_canon_composition_installs_as_a_no_op() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let root = harness.mount(alacer_widgets::Column::new().padding(8.0).boxed());
    let _probe = harness.add_child(
        root,
        Box::new(RoleProbe { role: accent }) as Box<dyn Widget>,
    );
    let trail = harness.trail();
    harness.settle();

    harness
        .pipeline_mut()
        .set_settings(Box::new(alacer_core::settings::CanonSettings));
    let landed = harness
        .pipeline_mut()
        .set_composer(Box::new(ShapedComposer::over(
            Box::new(CanonComposer),
            FactShape::FOLLOW_ALL,
        )));
    assert!(
        !landed,
        "the follow-all shape over the canon composition re-swapped the canon default"
    );
    let (themes, _) = drained(&trail);
    assert!(
        themes.is_empty(),
        "the no-op install streamed a swap record: {themes:?}"
    );
}
