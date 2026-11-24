//! M85 exit-criteria integration tests (§7.2 pass 8, ADR-0089): the
//! kiosk lock — the composition that follows nothing. Through M84 a
//! palette the OS's facts could not move had two half-doors: the
//! manual [`PassPipeline::set_theme`] ask, which wins *until the next
//! flip* (the composition owns the palette when the OS moves, the
//! M75 rule), and [`CanonComposer`], which locks the canon palette —
//! light, motion-allowing — and no other. A kiosk, a signage display,
//! an embedded panel cannot follow the OS: a dark flip at 7pm must not
//! repaint the boarding gate, and the operator's desktop accent must
//! not tint the checkout line. The lock over any palette is the
//! provided composition this cut grows beside the canon one:
//! [`alacer_core::settings::KioskComposer::locked`] answers one
//! [`Palette`] for every fact permutation — the flip lands, the door
//! resolves the composition, the composed answer *is* the palette
//! already installed, and the equality check swaps nothing (the
//! idle-frame property at the theme tier, ADR-0081) — while the app's
//! own `set_theme` ask still wins while it holds, and the next flip is
//! where the lock wins it back.
//!
//! Every test runs the real pipeline (A4): real layout, real paint
//! rasterization through the harness, the records through the harness
//! trail door, the pixels through the suite's role probe.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect};
use alacer_core::inspect::{InspectEvent, SettingRecord, ThemeRecord};
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::settings::{CanonSettings, KioskComposer, SettingSeam};
use alacer_core::style::{Size2, Style};
use alacer_core::theme::{Palette, ThemeSeam};
use alacer_core::trail::TrailLog;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;

/// The kiosk's palette — the palette the lock holds in every beat: a
/// dark surface, an amber accent, every role resolved (the duet-dark
/// shape, the look a boarding gate ships with).
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

/// The manual palette the suite's app door asks for — the look a
/// `set_theme` hold paints, the lock's opposite on purpose.
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

/// The facts the suite flips: the dark fact alone, the accent riding
/// its honest absence (the m81 suite's seam shape, the one fact the
/// beats need).
struct FixedSettings {
    dark: bool,
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
        None
    }
    fn reduced_transparency(&self) -> bool {
        false
    }

    fn text_scale(&self) -> Option<f32> {
        None
    }
}

/// The app's own theme ask (the manual door's payload): one palette,
/// named — the [`ThemeSeam`] the suite installs by hand.
struct ManualTheme;

impl ThemeSeam for ManualTheme {
    fn palette(&self) -> Palette {
        MANUAL
    }

    fn name(&self) -> &'static str {
        "manual"
    }
}

/// A probe that paints its whole bounds from the resolved *palette*
/// (the accent role) — the composition's pixel evidence (the m80
/// suite's role probe): the palette the widgets borrow moves when —
/// and only when — a composition resolves a new one, so the probe's
/// pixel is the lock's own answer, read back from the paint pass.
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

/// The pixel at `probe`'s center, as RGBA bytes (the m-suite's pixel
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

/// The accent the palette resolves, as the pixels see it — the probe
/// paints premultiplied, the palette names straight, so the suite
/// compares through the same premultiplied read both sides share (the
/// m-suite's pixel read at one probe).
fn accent_pixel(palette: Palette) -> [u8; 4] {
    let pixmap = tiny_skia::Pixmap::new(1, 1).expect("one pixel");
    let mut probe_pixmap = pixmap;
    probe_pixmap.fill(tiny_skia::Color::from_rgba8(
        palette.accent.r,
        palette.accent.g,
        palette.accent.b,
        0xFF,
    ));
    let px = probe_pixmap.pixel(0, 0).expect("one pixel");
    [px.red(), px.green(), px.blue(), px.alpha()]
}

/// The trail's whole window, drained once — the events the beats
/// partition into their tiers (a drain clears whole: a fact streams
/// once, and no beat may re-drain another beat's window).
fn drain_events(trail: &TrailLog) -> Vec<InspectEvent> {
    trail.drain()
}

/// The lock holds through the facts: the install resolves the lock over
/// the facts that are, the dark flip lands (the door answers `true` —
/// the facts moved) and the palette does not follow, the setting record
/// streams with no theme record beside it (the composed answer *is* the
/// palette already installed — the equality check's idle answer), the
/// manual `set_theme` ask wins while it holds, and the next flip is
/// where the lock wins it back (the record tagged the composer's own
/// name, the probe's pixel the lock's own answer again).
#[test]
fn m85_the_kiosk_lock_holds_through_the_facts() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let root = harness.mount(alacer_widgets::Column::new().padding(8.0).boxed());
    let probe = harness.add_child(root, Box::new(RoleProbe) as Box<dyn Widget>);
    let trail = harness.trail();
    harness.settle();

    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: false }));
    harness
        .pipeline_mut()
        .set_composer(Box::new(KioskComposer::locked(KIOSK)));
    harness.settle();

    // The install resolved the lock over the facts that are: the probe
    // paints the locked accent, the log names the composition.
    assert_eq!(
        center_pixel(&mut harness, probe),
        accent_pixel(KIOSK),
        "the install did not mount the locked palette"
    );
    let events = drain_events(&trail);
    let themes: Vec<ThemeRecord> = events
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Theme(record) => Some(record.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(themes.len(), 1, "the install resolved once");
    assert_eq!(themes[0].palette, "kiosk", "the record names the composer");

    // The dark flip lands — the fact moved, the door answers `true` —
    // and the palette does not follow: the composed answer is the
    // palette already installed, the equality check swaps nothing, the
    // theme tier streams no record.
    let landed = harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    assert!(landed, "the dark flip's fact moved — the door lands it");
    harness.settle();
    assert_eq!(
        center_pixel(&mut harness, probe),
        accent_pixel(KIOSK),
        "the kiosk's palette followed the fact — the lock is broken"
    );
    let events = drain_events(&trail);
    let themes: Vec<&ThemeRecord> = events
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Theme(record) => Some(record),
            _ => None,
        })
        .collect();
    assert!(
        themes.is_empty(),
        "the flip re-swapped the palette the lock already holds: {themes:?}"
    );
    let settings: Vec<&SettingRecord> = events
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Setting(record) => Some(record),
            _ => None,
        })
        .collect();
    assert_eq!(settings.len(), 1, "one flip, one setting record");
    assert!(settings[0].dark, "the record names the fact that moved");

    // The manual door wins while it holds: the app's own ask lands over
    // the composed answer, the probe paints it.
    let landed = harness.pipeline_mut().set_theme(Box::new(ManualTheme));
    assert!(landed, "the manual ask moved the palette");
    harness.settle();
    assert_eq!(
        center_pixel(&mut harness, probe),
        accent_pixel(MANUAL),
        "the manual ask did not land"
    );
    let events = drain_events(&trail);
    let themes: Vec<&ThemeRecord> = events
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Theme(record) => Some(record),
            _ => None,
        })
        .collect();
    assert_eq!(themes.len(), 1, "the manual swap names itself");
    assert_eq!(themes[0].palette, "manual", "the record names the ask");

    // The next flip is where the lock wins it back: the composition
    // resolves on the landing, the answer is the locked palette, the
    // swap rides the flip's own broadcast, the record names the
    // composer, and the probe's pixel is the lock's own answer again.
    let landed = harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: false }));
    assert!(landed, "the second flip's fact moved — the door lands it");
    harness.settle();
    assert_eq!(
        center_pixel(&mut harness, probe),
        accent_pixel(KIOSK),
        "the lock did not win the palette back on the flip"
    );
    let events = drain_events(&trail);
    let themes: Vec<&ThemeRecord> = events
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Theme(record) => Some(record),
            _ => None,
        })
        .collect();
    assert_eq!(themes.len(), 1, "one swap rode the flip's broadcast");
    assert_eq!(themes[0].palette, "kiosk", "the record names the lock");

    harness.assert_all_named();
}

/// The install resolves against the facts that are (the M75 rule): with
/// the dark fact already held, the lock still mounts its own answer —
/// the palette the lock holds is the palette the facts resolve to, not
/// a fact-driven answer the lock then abandons.
#[test]
fn m85_the_install_resolves_the_lock_over_the_facts_that_are() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let root = harness.mount(alacer_widgets::Column::new().padding(8.0).boxed());
    let probe = harness.add_child(root, Box::new(RoleProbe) as Box<dyn Widget>);
    let trail = harness.trail();
    harness.settle();

    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    harness
        .pipeline_mut()
        .set_composer(Box::new(KioskComposer::locked(KIOSK)));
    harness.settle();

    assert_eq!(
        center_pixel(&mut harness, probe),
        accent_pixel(KIOSK),
        "the install resolved a fact-driven answer instead of the lock"
    );
    let events = drain_events(&trail);
    let themes: Vec<&ThemeRecord> = events
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Theme(record) => Some(record),
            _ => None,
        })
        .collect();
    assert_eq!(themes.len(), 1, "the install resolved once");
    assert_eq!(themes[0].palette, "kiosk");
}

/// The canon composition's own shape, re-agreed beside the lock: the
/// canon answers the canon palette for any facts — and the lock's
/// install over the canon default is the honest no-op only when the
/// lock holds the canon palette itself (the M75 rule at the lock's
/// door: installing the palette already installed swaps nothing).
#[test]
fn m85_the_lock_over_the_canon_palette_installs_as_a_no_op() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let root = harness.mount(alacer_widgets::Column::new().padding(8.0).boxed());
    let _probe = harness.add_child(root, Box::new(RoleProbe) as Box<dyn Widget>);
    let trail = harness.trail();
    harness.settle();

    harness.pipeline_mut().set_settings(Box::new(CanonSettings));
    let landed = harness
        .pipeline_mut()
        .set_composer(Box::new(KioskComposer::locked(Palette::CANON)));
    assert!(
        !landed,
        "the lock over the canon palette re-swapped the canon default"
    );
    let events = drain_events(&trail);
    let themes: Vec<&ThemeRecord> = events
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Theme(record) => Some(record),
            _ => None,
        })
        .collect();
    assert!(
        themes.is_empty(),
        "the no-op install streamed a swap record: {themes:?}"
    );
}
