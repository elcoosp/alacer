//! M75 exit-criteria integration tests (§7.2 pass 8, ADR-0081): the
//! composition door — the palette follows the OS fact. ADR-0080's
//! pre-registered re-open rule cashes — "a consumer needs the palette
//! to *follow* the OS fact → the pipeline grows the composition door
//! (the installed palette re-swapped on the flip's own record), priced
//! by a real reader — never a palette that reaches into the platform's
//! adapter." Since M75 the pipeline owns a
//! [`alacer_core::settings::SettingComposer`] beside the setting seam:
//! the install resolves the composed palette against the fact that is,
//! every landed flip re-resolves it (the theme tier's own swap rules —
//! the equality check, the `Change::PAINT` broadcast riding the flip's
//! own, one [`alacer_core::inspect::ThemeRecord`] tagged with the
//! composer's name), and the manual [`alacer_core::passes::PassPipeline::set_theme`]
//! door wins until the next flip.
//!
//! Every test runs the real pipeline (A4): real layout, real paint
//! rasterization through the harness, the records through the harness
//! trail door. The canon composition pins the canon palette for either
//! fact — the no-op the door answers `false` on where the canon palette
//! is already installed.

use std::path::PathBuf;

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect};
use alacer_core::inspect::{InspectEvent, SettingRecord, ThemeRecord};
use alacer_core::passes::FrameReport;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::settings::{CanonComposer, CanonSettings, SettingComposer, SettingSeam};
use alacer_core::style::{Size2, Style};
use alacer_core::theme::{Palette, ThemeSeam};
use alacer_core::trail::TrailLog;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::{TestHarness, pixel_diff_ratio};
use alacer_widgets::BoxedWidget as _;

/// The fact the suite flips to: `dark` — the OS fact the platform
/// adapter pushes through the door (the m74 suite's own shape).
struct FixedSettings {
    dark: bool,
}

impl SettingSeam for FixedSettings {
    fn dark_mode(&self) -> bool {
        self.dark
    }

    /// The m75 suite flips the dark fact alone — the canon absence the
    /// fact's own growth answers (ADR-0083).
    fn reduced_motion(&self) -> bool {
        false
    }

    fn high_contrast(&self) -> bool {
        false
    }

    fn accent_color(&self) -> Option<alacer_core::color::Color> {
        None
    }
}

/// The composition the suite drives: the canon palette for the light
/// fact, a *duet* dark palette beside it — every dark role distinct
/// from the canon's and from the midnight palette's, so a composed
/// resolution cannot pass for a canon one (the m74 suite's own
/// hygiene, at the composition tier).
struct DuetComposer;

impl SettingComposer for DuetComposer {
    fn palette(
        &self,
        dark: bool,
        _reduced_motion: bool,
        _high_contrast: bool,
        _accent: Option<alacer_core::color::Color>,
    ) -> Palette {
        if dark {
            Palette {
                surface: Color::rgb(16, 18, 24),
                panel: Color::rgb(28, 31, 40),
                accent: Color::rgb(255, 176, 64),
                accent_pressed: Color::rgb(226, 152, 44),
                selection: Color::rgb(255, 176, 64).with_alpha(72),
                text: Color::rgb(226, 230, 238),
                text_muted: Color::rgb(142, 150, 168),
                border: Color::rgb(56, 62, 80),
                on_accent: Color::rgb(24, 16, 6),
            }
        } else {
            Palette::CANON
        }
    }

    fn name(&self) -> &'static str {
        "duet"
    }
}

/// The midnight palette (the m73/m74 suites' own): the manual door's
/// palette in the "the manual door wins" test — its accent distinct
/// from both the canon's and the duet dark's, so all three resolutions
/// are tellable apart on pixels alone.
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

/// A probe that paints its whole bounds from the resolved *palette*
/// (the m73/m74 suites' role probe, the accent role) — the composition
/// door's pixel evidence: the palette the widgets borrow moves when —
/// and only when — the composition resolves a new one.
struct RoleProbe;

impl Widget for RoleProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("role probe")
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

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves")
}

fn read_workspace_file(path: &str) -> String {
    let path = workspace_root().join(path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("required file {} missing: {error}", path.display()))
}

/// The streamed setting facts, oldest first (the m-suite's filter
/// shape: the assertion source is the trail's own records, never
/// internals).
fn setting_facts(trail: &TrailLog) -> Vec<SettingRecord> {
    trail
        .look()
        .into_iter()
        .filter_map(|event| match event {
            InspectEvent::Setting(record) => Some(record),
            _ => None,
        })
        .collect()
}

/// The streamed theme facts, oldest first.
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

/// The canon composition pins the canon palette for either fact — the
/// composition that follows nothing, installable by name so a consumer
/// that composed away can compose back through the same door it
/// composed in. Installing it where nothing is installed is the no-op
/// the door answers `false` on: the default build already resolves the
/// canon palette.
#[test]
fn m75_the_canon_composer_pins_the_canon_palette() {
    let _guard = alacer_core::flux::batch_test_lock();
    assert_eq!(
        CanonComposer.palette(false, false, false, None),
        Palette::CANON,
        "the canon composition resolves the canon palette for light"
    );
    assert_eq!(
        CanonComposer.palette(true, false, false, None),
        Palette::CANON,
        "the canon composition resolves the canon palette for dark"
    );
    assert_eq!(CanonComposer.name(), "canon");

    // The install over the default build: the composed palette is where
    // the fact resolves already — the no-op, an idle frame.
    let mut harness = TestHarness::new(200.0, 120.0);
    let _probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);
    let landed = harness.pipeline_mut().set_composer(Box::new(CanonComposer));
    assert!(
        !landed,
        "installing the canon composition over the canon default is the no-op"
    );
    assert!(
        settle(&mut harness).is_idle(),
        "no broadcast, an idle frame"
    );
}

/// The install resolves against the fact that *is*: with the dark fact
/// already installed (no palette moved — the M74 door's own rule), the
/// composition's install composes the dark palette immediately — the
/// app that installs mid-session paints the composed answer now, not
/// on the next flip.
#[test]
fn m75_the_install_resolves_against_the_installed_fact() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);
    let bounds = harness.bounds(probe);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [37, 99, 235, 255],
        "the canon accent paints before the composition installs"
    );

    // The dark fact lands first: the flip repaints, the palette the
    // roles borrow does not move (the M74 rule — the fact never munges
    // the palette).
    assert!(
        harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: true }))
    );
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [37, 99, 235, 255],
        "the fact flip never rethemes — the canon accent holds"
    );
    assert!(
        theme_facts(&trail).is_empty(),
        "the fact-only flip streamed no theme fact"
    );

    // The composition installs: the composed *dark* palette lands right
    // away — the door answers `true`, the theme fact streams the
    // composer's tag, the pixels follow.
    let landed = harness.pipeline_mut().set_composer(Box::new(DuetComposer));
    assert!(
        landed,
        "the install composed a palette the fact was not painting"
    );
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [255, 176, 64, 255],
        "the composed dark accent paints after the install"
    );
    let themes = theme_facts(&trail);
    assert_eq!(themes.len(), 1, "one theme fact per composed swap");
    assert_eq!(themes[0].palette, "duet", "the tag is the composer's own");
}

/// The flip composes: with the composition installed, the OS flip
/// re-resolves the palette the widgets borrow — the composed swap
/// riding the flip's own broadcast, the theme tier's equality check
/// and tag rules unchanged.
#[test]
fn m75_the_flip_composes_the_palette() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);
    let bounds = harness.bounds(probe);

    // The composition installs over the light fact: the canon palette
    // is already live — the install is the palette no-op (`false`).
    assert!(
        !harness.pipeline_mut().set_composer(Box::new(DuetComposer)),
        "the composed light palette is the installed one — the install is the no-op"
    );
    settle(&mut harness);

    // The OS flips dark: the flip lands, the composed dark palette
    // lands beside it, the pixels follow — the same door, one broadcast.
    assert!(
        harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: true }))
    );
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [255, 176, 64, 255],
        "the composed dark accent paints after the flip"
    );

    // The flip back: the composition resolves the canon palette again.
    assert!(
        harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: false }))
    );
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [37, 99, 235, 255],
        "the composed light accent paints after the flip back"
    );

    let themes = theme_facts(&trail);
    assert_eq!(themes.len(), 2, "one theme fact per composed swap");
    assert_eq!(
        themes[0].palette, "duet",
        "the dark swap's tag is the composer's own"
    );
    assert_eq!(
        themes[1].palette, "duet",
        "the light swap's tag is the composer's own too"
    );
    assert_eq!(
        setting_facts(&trail).len(),
        2,
        "each landed flip carries its fact"
    );
}

/// The composed swap streams *behind* the flip's own fact: the tick's
/// start drain reads setting → theme, so a frame's trail answers the
/// fact first and the palette that fact painted into second — the M74
/// drain's own ordering, now carrying the composed swap.
#[test]
fn m75_the_composed_swap_streams_setting_then_theme() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let _probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);

    harness.pipeline_mut().set_composer(Box::new(DuetComposer));
    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    settle(&mut harness);

    let events = trail.look();
    let setting_index = events
        .iter()
        .position(|event| matches!(event, InspectEvent::Setting(_)))
        .expect("the flip streamed its fact");
    let theme_index = events
        .iter()
        .position(|event| matches!(event, InspectEvent::Theme(_)))
        .expect("the composed swap streamed its fact");
    assert!(
        setting_index < theme_index,
        "the drain reads setting → theme: the fact first, the palette it composed second"
    );
}

/// The composition's no-op is honest: a flip that composes the palette
/// already installed answers no theme fact — the setting tier's own
/// idle rule at the theme half. The flip still lands (the fact moved,
/// the broadcast repaints); only the swap is the no-op.
#[test]
fn m75_the_composed_no_op_streams_no_theme_fact() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);
    let bounds = harness.bounds(probe);

    // Walk the composition to the dark palette: install (canon, no-op),
    // flip dark (duet dark lands), flip light (canon lands).
    harness.pipeline_mut().set_composer(Box::new(DuetComposer));
    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    settle(&mut harness);
    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: false }));
    settle(&mut harness);
    let themes_before = theme_facts(&trail).len();

    // The manual door takes the palette where the composition is about
    // to resolve: the dark palette, while the fact is still light.
    assert!(
        harness.pipeline_mut().set_theme(Box::new(DuetDarkTheme)),
        "the manual door lands the composed-dark palette while the fact is light"
    );
    settle(&mut harness);

    // The OS flips dark: the flip lands (the fact moved, the broadcast
    // repaints), but the composition resolves the palette the manual
    // door already installed — no re-swap, no theme fact.
    assert!(
        harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: true }))
    );
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [255, 176, 64, 255],
        "the composed dark accent paints — the same value the manual door left"
    );
    let themes = theme_facts(&trail);
    assert_eq!(
        themes.len(),
        themes_before + 1,
        "only the manual door's swap streamed a theme fact"
    );
    assert_eq!(
        themes.last().map(|record| record.palette.as_str()),
        Some("duet-dark-manual"),
        "the last theme fact is the manual door's, never the flip's"
    );
    assert_eq!(
        setting_facts(&trail).len(),
        3,
        "each landed flip carries its fact"
    );
}

/// A manual theme whose palette is exactly the composition's dark
/// answer — the palette the no-op test needs, under a different tag
/// (the equality check compares palettes, never tags).
struct DuetDarkTheme;

impl ThemeSeam for DuetDarkTheme {
    fn palette(&self) -> Palette {
        DuetComposer.palette(true, false, false, None)
    }

    fn name(&self) -> &'static str {
        "duet-dark-manual"
    }
}

/// The manual door wins until the next flip: a `set_theme` ask lands
/// over a composed palette and holds while the fact is still — the
/// composition owns the palette only when the OS moves.
#[test]
fn m75_the_manual_door_wins_until_the_next_flip() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);
    let bounds = harness.bounds(probe);

    harness.pipeline_mut().set_composer(Box::new(DuetComposer));
    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [255, 176, 64, 255],
        "the composed dark accent paints before the manual ask"
    );

    // The manual door overrides the composed palette: midnight lands.
    assert!(harness.pipeline_mut().set_theme(Box::new(MidnightPalette)));
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [240, 180, 41, 255],
        "the manual midnight accent paints — the door wins while the fact is still"
    );

    // The next flip composes again: the composer is still installed,
    // the light answer lands over midnight.
    assert!(
        harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: false }))
    );
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [37, 99, 235, 255],
        "the composed light accent paints after the flip"
    );

    let themes = theme_facts(&trail);
    let tags: Vec<&str> = themes
        .iter()
        .map(|record| record.palette.as_str())
        .collect();
    assert_eq!(
        tags,
        vec!["duet", "midnight", "duet"],
        "composed → manual → composed: the log reads the ownership it held"
    );
}

/// The observation is priced, not assumed: the composition is
/// paint-affecting with no listener installed — the door's answer and
/// the pixels never depend on the trail (the m74 suite's own rule).
/// The install itself is the no-op where the fact already resolves
/// (the duet's light answer *is* the canon default), the flip lands
/// and composes regardless of the trail.
#[test]
fn m75_the_disabled_trail_still_composes() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);
    let bounds = harness.bounds(probe);
    assert!(
        !harness.pipeline_mut().set_composer(Box::new(DuetComposer)),
        "the install is the no-op where the fact already resolves"
    );
    assert!(
        harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: true })),
        "the flip lands without a listener"
    );
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [255, 176, 64, 255],
        "the composition resolves regardless of the trail"
    );
}

/// The living documents re-agree at the M75 cut: the README's status
/// block names the milestone, the corpus count rides the composition
/// tier's pricing, and the architecture carries the decision.
#[test]
fn m75_the_living_documents_re_agree() {
    let readme = read_workspace_file("README.md");
    assert!(
        readme.contains("M0 through M89"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("All 95 criterion cases"),
        "the README quotes the baselines.json bench_count the composition tier grew"
    );
    let architecture = read_workspace_file("docs/architecture.md");
    assert!(
        architecture.contains("ADR-0081"),
        "the architecture carries the composition door's decision"
    );
    assert!(
        architecture.contains("94 decision records"),
        "the register grew the M75 row"
    );
    let baselines = serde_json::from_str::<serde_json::Value>(&read_workspace_file(
        "benchmarks/baselines/baselines.json",
    ))
    .expect("the baselines parse");
    assert_eq!(
        baselines["bench_count"], 95,
        "the published corpus carries the composition tier's pricing"
    );
}

/// The demo drives the door from the consumer side: the follow-OS
/// demo's beats (the install is the no-op where the fact already
/// resolves, the flip composes, the same fact again is the honest
/// no-op, the flip back composes back) are pinned here by the same
/// shapes the demo logs — the example compiles under the gates.
#[test]
fn m75_the_demo_shape_drives_the_door() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let _probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);

    // Beat one: the app installs the composition door — the canon
    // palette is already where the light fact resolves, the honest no-op.
    assert!(
        !harness.pipeline_mut().set_composer(Box::new(DuetComposer)),
        "the install is the no-op where the fact already resolves"
    );
    settle(&mut harness);
    // Beat two: the OS flips dark — the composed dark palette lands.
    assert!(
        harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: true }))
    );
    settle(&mut harness);
    // Beat three: the same fact again — the honest no-op, nothing lands.
    assert!(
        !harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: true }))
    );
    settle(&mut harness);
    // Beat four: the OS pushes light back — the composition resolves
    // the canon palette again.
    assert!(harness.pipeline_mut().set_settings(Box::new(CanonSettings)));
    settle(&mut harness);

    let facts = setting_facts(&trail);
    assert_eq!(
        facts.len(),
        2,
        "the demo's log carries the two flips that landed, never the no-op"
    );
    assert!(
        facts[0].dark && !facts[1].dark,
        "the log reads dark → light"
    );
    let themes = theme_facts(&trail);
    assert_eq!(
        themes.len(),
        2,
        "the demo's log carries the two composed swaps, never the no-op install"
    );
    assert_eq!(
        themes[0].palette, "duet",
        "the dark swap's tag rides the composer's name"
    );
    assert_eq!(
        themes[1].palette, "duet",
        "the light swap's tag rides the composer's name too"
    );
}

/// The composition's three answers are pixel-distinct — the canon
/// accent, the duet dark accent, and the midnight accent can never be
/// confused (the suite's own hygiene: a stale resolution cannot pass
/// for the composed swap, the byte-level pins above are the contract).
#[test]
fn m75_the_three_answers_are_distinct() {
    let canon = [37u8, 99, 235];
    let duet_dark = [255u8, 176, 64];
    let midnight = [240u8, 180, 41];
    assert_ne!(canon, duet_dark);
    assert_ne!(canon, midnight);
    assert_ne!(duet_dark, midnight);
    // The palettes' accents are distinct enough that a stale resolution
    // cannot pass for the flip (the ratio is a smoke check, the
    // byte-level pins above are the contract).
    let ratio = pixel_diff_ratio(&image_with(duet_dark), &image_with(canon));
    assert!(
        ratio > 0.9,
        "the composed swap's answers differ across the image"
    );
}

/// Builds a one-color image for the distinctness check.
fn image_with(rgb: [u8; 3]) -> tiny_skia::Pixmap {
    let mut pixmap = tiny_skia::Pixmap::new(8, 8).expect("an 8x8 pixmap");
    pixmap.fill(tiny_skia::Color::from_rgba8(rgb[0], rgb[1], rgb[2], 255));
    pixmap
}
