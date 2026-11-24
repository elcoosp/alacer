//! The kiosk lock (M85, ADR-0089) driven from the consumer side: one
//! palette, every fact, every flip — the boarding-gate app whose look
//! the OS's dark flip, the operator's accent, and the motion ask
//! cannot move. The lock installs over the canon default
//! (`KioskComposer::locked`, the composition the facts resolve to and
//! never overrule), the simulated OS's flips land (the door answers
//! `true` — the facts moved) and the palette holds, the manual
//! `set_theme` ask wins while it holds, and the next flip is where the
//! lock wins it back — the log's records telling the story: one
//! `Theme` record tagged `kiosk` at the install, `Setting` records
//! beside silent theme tiers on the flips, the `manual` tag on the
//! hold, the `kiosk` tag on the win-back.
//!
//! Run with `cargo run -p alacer --example kiosk`. The example is
//! headless — the real pass pipeline (A4), the probe's pixels read
//! back from the software raster — so the same three beats run
//! anywhere the tests do.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect};
use alacer_core::inspect::{InspectEvent, ThemeRecord};
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::settings::{KioskComposer, SettingSeam};
use alacer_core::style::{Size2, Style};
use alacer_core::theme::{Palette, ThemeSeam};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;

/// The kiosk's palette — the lock's answer, every fact: a dark
/// surface, an amber accent, every role resolved.
const KIOSK: Palette = Palette {
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

/// The manual palette the app's own door asks for — the lock's
/// opposite, so the win-back beat is visible in the pixels.
const MANUAL: Palette = Palette {
    surface: Color::rgb(250, 250, 249),
    panel: Color::rgb(255, 255, 255),
    accent: Color::rgb(37, 99, 235),
    accent_pressed: Color::rgb(29, 78, 216),
    selection: Color::rgb(37, 99, 235).with_alpha(72),
    text: Color::rgb(15, 23, 42),
    text_muted: Color::rgb(100, 116, 139),
    border: Color::rgb(203, 213, 225),
    on_accent: Color::rgb(255, 255, 255),
};

/// The simulated OS: the dark fact the beats flip (the adapter's door,
/// the seam the platform owns in a windowed app).
struct OsSettings {
    dark: bool,
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
        None
    }
    fn reduced_transparency(&self) -> bool {
        false
    }

    fn text_scale(&self) -> Option<f32> {
        None
    }
}

/// The manual theme ask: one palette, named (the payload the
/// `set_theme` door holds while it wins).
struct ManualTheme;

impl ThemeSeam for ManualTheme {
    fn palette(&self) -> Palette {
        MANUAL
    }

    fn name(&self) -> &'static str {
        "manual"
    }
}

/// A probe that paints its whole bounds from the resolved palette's
/// accent role — the lock's pixel evidence: the palette the widgets
/// borrow moves when — and only when — a composition resolves a new
/// one, so the probe's pixel is the lock's own answer, read back from
/// the paint pass.
struct RoleProbe;

impl Widget for RoleProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("kiosk probe")
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
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// The probe's center pixel, as RGBA bytes (the suite's pixel read).
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

fn main() {
    let mut harness = TestHarness::new(220.0, 140.0);
    let root = harness.mount(alacer_widgets::Column::new().padding(8.0).boxed());
    let probe = harness.add_child(root, Box::new(RoleProbe) as Box<dyn Widget>);
    let trail = harness.trail();
    harness.settle();

    // The lock installs over the canon default: the composition
    // resolves against the facts that are, the locked palette mounts,
    // and the log names it.
    harness
        .pipeline_mut()
        .set_settings(Box::new(OsSettings { dark: false }));
    harness
        .pipeline_mut()
        .set_composer(Box::new(KioskComposer::locked(KIOSK)));
    harness.settle();
    println!(
        "install     : the lock mounts the kiosk palette (probe {:?})",
        center_pixel(&mut harness, probe)
    );
    let _ = trail.drain(); // the install's own record streams once; the beats start clean

    // The OS asks for dark: the flip lands (the fact moved — the door
    // answers `true`, one `Setting` record streams) and the palette
    // holds — the composed answer *is* the palette already installed,
    // the equality check swapping nothing, the theme tier silent.
    let landed = harness
        .pipeline_mut()
        .set_settings(Box::new(OsSettings { dark: true }));
    harness.settle();
    let themes: Vec<ThemeRecord> = trail
        .drain()
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Theme(record) => Some(record.clone()),
            _ => None,
        })
        .collect();
    println!(
        "dark flip   : landed={landed}, theme records streamed={} — the palette holds (probe {:?})",
        themes.len(),
        center_pixel(&mut harness, probe)
    );
    assert!(
        themes.is_empty() && landed,
        "the kiosk's palette followed the fact — the lock is broken"
    );

    // The app's own ask wins while it holds: the manual door lands its
    // palette, the probe paints it, the log names the ask.
    let landed = harness.pipeline_mut().set_theme(Box::new(ManualTheme));
    harness.settle();
    println!(
        "manual ask  : landed={landed} (probe {:?}) — the set_theme hold, the lock waiting",
        center_pixel(&mut harness, probe)
    );
    let _ = trail.drain(); // the manual swap's own record streams once; the win-back beat starts clean

    // The next flip is where the lock wins it back: the composition
    // resolves on the landing, the swap rides the flip's own
    // broadcast, the record names the composer, and the probe's pixel
    // is the lock's own answer again.
    let landed = harness
        .pipeline_mut()
        .set_settings(Box::new(OsSettings { dark: false }));
    harness.settle();
    let themes: Vec<ThemeRecord> = trail
        .drain()
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Theme(record) => Some(record.clone()),
            _ => None,
        })
        .collect();
    println!(
        "win-back    : landed={landed}, the lock re-resolved ({:?}) — the palette the facts cannot move",
        themes
            .iter()
            .map(|record| record.palette.as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        center_pixel(&mut harness, probe),
        [KIOSK.accent.r, KIOSK.accent.g, KIOSK.accent.b, 0xFF],
        "the lock did not win the palette back on the flip"
    );
}
