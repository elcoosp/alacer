//! The shaped composition lock (M86, ADR-0090) driven from the
//! consumer side: the signage display whose surface follows the OS's
//! dark fact while its brand accent never moves — ADR-0089's own
//! pre-registered re-open rule cashed ("a consumer needs a lock with
//! *shape* → the provided-composition rule grows the second provided
//! composer beside the lock"). The lock installs over the app's own
//! duet composition (`ShapedComposer::over`, the shape pinning the
//! accent fact), the OS's dark flips land and the surface follows (one
//! `Theme` record tagged `shaped` riding the flip's own broadcast),
//! and the OS's accent flips land and the accent does not move (the
//! pinned fact the base never sees — the equality check's idle answer,
//! the theme tier silent) — the log's records and the probe's pixels
//! telling the story.
//!
//! Run with `cargo run -p alacer --example shaped_lock`. The example
//! is headless — the real pass pipeline (A4), the probes' pixels read
//! back from the software raster — so the same beats run anywhere the
//! tests do.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect};
use alacer_core::inspect::InspectEvent;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::settings::{FactShape, SettingComposer, SettingSeam, ShapedComposer};
use alacer_core::style::{Size2, Style};
use alacer_core::theme::Palette;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;

/// The brand accent the lock pins: the look the signage ships with,
/// the color the operator's desktop accent must never tint.
const BRAND: Color = Color::rgb(255, 176, 64);

/// The OS accent the demo flips in: the desktop's pick, the fact the
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
/// accent.
fn tinted_dark(accent: Color) -> Palette {
    Palette {
        accent,
        accent_pressed: accent,
        selection: accent.with_alpha(72),
        ..DUET_DARK
    }
}

/// The base composition's light answer with `accent` riding its accent
/// roles — over the canon palette, the m81 tint shape's light half.
fn tinted_light(accent: Color) -> Palette {
    Palette {
        accent,
        accent_pressed: accent,
        selection: accent.with_alpha(72),
        ..Palette::CANON
    }
}

/// The simulated OS: the dark fact and the accent fact the beats flip
/// (the adapter's door, the seam the platform owns in a windowed app).
struct OsSettings {
    dark: bool,
    accent: Option<Color>,
}

impl SettingSeam for OsSettings {
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
    fn reduced_transparency(&self) -> bool {
        false
    }

    fn text_scale(&self) -> Option<f32> {
        None
    }
}

/// The base composition the shape rides: a duet that follows dark and
/// tints its accent roles from the OS accent when the OS reports one —
/// the composition that answers *both* facts, so the shape has
/// something real to hold.
struct DuetAccentComposer;

impl SettingComposer for DuetAccentComposer {
    fn palette(
        &self,
        dark: bool,
        _reduced_motion: bool,
        _high_contrast: bool,
        accent: Option<Color>,
        _reduced_transparency: bool,
        _text_scale: Option<f32>,
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
/// shape's pixel evidence: the surface probe is the fact the demo
/// follows, the accent probe the fact it pins.
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

/// The surface role, as a probe paints it.
fn surface(role: Palette) -> Color {
    role.surface
}

/// The accent role, as a probe paints it.
fn accent(role: Palette) -> Color {
    role.accent
}

/// The pixel at `probe`'s center, as RGBA bytes (the suite's pixel
/// read).
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

/// The theme records the trail streamed since the last drain — the
/// tags the log names the swaps by.
fn theme_tags(trail: &alacer_core::trail::TrailLog) -> Vec<String> {
    trail
        .drain()
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Theme(record) => Some(record.palette.clone()),
            _ => None,
        })
        .collect()
}

fn main() {
    let mut harness = TestHarness::new(220.0, 140.0);
    let root = harness.mount(alacer_widgets::Column::new().padding(8.0).boxed());
    let surface_probe = harness.add_child(
        root,
        Box::new(RoleProbe { role: surface }) as Box<dyn Widget>,
    );
    let accent_probe = harness.add_child(
        root,
        Box::new(RoleProbe { role: accent }) as Box<dyn Widget>,
    );
    let trail = harness.trail();
    harness.settle();

    // The lock installs over the app's own duet composition: the shape
    // pins the accent fact, the base resolves the rest against the
    // facts that are — light surface, the brand accent.
    harness.pipeline_mut().set_settings(Box::new(OsSettings {
        dark: false,
        accent: None,
    }));
    harness
        .pipeline_mut()
        .set_composer(Box::new(ShapedComposer::over(
            Box::new(DuetAccentComposer),
            FactShape::FOLLOW_ALL.pin_accent(BRAND),
        )));
    harness.settle();
    println!(
        "install     : surface {:?}, accent {:?} — the brand accent mounts, the base's light resolves",
        center_pixel(&mut harness, surface_probe),
        center_pixel(&mut harness, accent_probe),
    );
    let _ = trail.drain(); // the install's own record streams once; the beats start clean

    // The OS asks for dark: the flip lands (one `Setting` record) and
    // the surface follows — the dark fact is one the shape left
    // unpinned, the base recomposes, one `Theme` record tagged
    // `shaped` rides the flip's own broadcast. The accent holds.
    let landed = harness.pipeline_mut().set_settings(Box::new(OsSettings {
        dark: true,
        accent: None,
    }));
    harness.settle();
    let tags = theme_tags(&trail);
    println!(
        "dark flip   : landed={landed}, theme records streamed={tags:?} — the surface follows (surface {:?}), the accent holds ({:?})",
        center_pixel(&mut harness, surface_probe),
        center_pixel(&mut harness, accent_probe),
    );
    assert_eq!(
        center_pixel(&mut harness, accent_probe),
        [BRAND.r, BRAND.g, BRAND.b, 0xFF],
        "the pinned accent moved with the flip — the shape is broken"
    );

    // The OS reports its own accent: the flip lands (the fact moved)
    // and the accent does not move — the pinned fact is substituted
    // before the base resolves, the composed answer *is* the palette
    // already installed, the equality check swapping nothing, the
    // theme tier silent. The operator's desktop accent never tints the
    // checkout line.
    let landed = harness.pipeline_mut().set_settings(Box::new(OsSettings {
        dark: true,
        accent: Some(OS_RED),
    }));
    harness.settle();
    let tags = theme_tags(&trail);
    println!(
        "accent flip : landed={landed}, theme records streamed={tags:?} — the OS's red never tints the palette (accent {:?})",
        center_pixel(&mut harness, accent_probe),
    );
    assert!(
        tags.is_empty() && landed,
        "the pinned fact's flip re-swapped the palette — the shape is broken"
    );
    assert_eq!(
        center_pixel(&mut harness, accent_probe),
        [BRAND.r, BRAND.g, BRAND.b, 0xFF],
        "the OS's accent tinted the palette — the shape is broken"
    );
}
