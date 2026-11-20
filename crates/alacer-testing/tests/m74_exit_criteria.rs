//! M74 exit-criteria integration tests (§7.2 pass 8, ADR-0080): the
//! OS's dark-mode setting joins the theme seam. ADR-0079's
//! pre-registered re-open rule cashes — "a consumer needs the OS's
//! dark-mode fact → the platform grows the setting seam the palette
//! reads, never a poll loop the widget vocabulary guesses at." Since
//! M74 the fact is one [`alacer_core::settings::SettingSeam`] the
//! pipeline owns: the platform adapter installs it (the adapter the
//! platform owns, ADR-0009), the paint pass resolves the fact once per
//! pass beside the palette into [`alacer_core::scene::PaintCtx`], the
//! flip broadcasts `Change::PAINT` on the tree (the currency's own
//! rule — never a private repaint channel), and one
//! [`alacer_core::inspect::SettingRecord`] streams the tick after the
//! door answered.
//!
//! Every test runs the real pipeline (A4): real layout, real paint
//! rasterization through the harness, the records through the harness
//! trail door. The canon setting pins light — the fact the blessed
//! snapshot corpus was blessed with — so a default build never
//! silently re-paints.

use std::path::PathBuf;

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect};
use alacer_core::inspect::{InspectEvent, SettingRecord};
use alacer_core::passes::{FrameReport, PassKind};
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::settings::{CanonSettings, SettingSeam};
use alacer_core::style::{Size2, Style};
use alacer_core::theme::{Palette, ThemeSeam};
use alacer_core::trail::TrailLog;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::{TestHarness, pixel_diff_ratio};
use alacer_widgets::{BoxedWidget as _, Column};

/// The fact the suite flips to: `dark` — the OS fact the platform
/// adapter pushes through the door. A plain `Copy` datum, installed
/// through the same door any adapter rides.
struct FixedSettings {
    dark: bool,
}

impl SettingSeam for FixedSettings {
    fn dark_mode(&self) -> bool {
        self.dark
    }

    /// The m74 suite flips the dark fact alone — the canon absence the
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

/// The midnight palette (the m73 suite's own): every role distinct
/// from the canon's, so a palette-resolution regression cannot pass
/// for the fact's flip.
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

/// A probe that paints its whole bounds from the resolved *fact* —
/// `PaintCtx::dark` read directly off the paint context, the
/// role-independent decision the vocabulary consults, deterministic
/// pixels with no glyph dependency.
struct FactProbe;

impl Widget for FactProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("fact probe")
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
        let fact = ctx.dark();
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), ctx.size()),
            color: if fact {
                Color::rgb(20, 22, 28)
            } else {
                Color::rgb(240, 241, 244)
            },
            corner_radius: 0.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// A probe that paints its whole bounds from the resolved *palette*
/// (the m73 suite's role probe, the accent role) — the contrast the
/// "the fact never munges the palette" test reads.
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

/// The canon setting pins light: the fact the blessed corpus was
/// blessed with — a default build (no seam installed) resolves light,
/// so an OS flip can never silently re-paint an untouched app.
#[test]
fn m74_the_canon_setting_pins_light() {
    let _guard = alacer_core::flux::batch_test_lock();
    assert!(
        !CanonSettings.dark_mode(),
        "the canon setting is light — the corpus's fact"
    );
    // A default build paints the light fact: the fact probe's pixels
    // answer light before any door ask.
    let mut harness = TestHarness::new(200.0, 120.0);
    let probe = harness.mount(FactProbe.boxed());
    settle(&mut harness);
    let bounds = harness.bounds(probe);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [240, 241, 244, 255],
        "the default fact is light — the probe paints the light answer"
    );
}

/// The flip lands through the door: the paint pass resolves the fact
/// once per pass beside the palette, the probe's pixels answer dark —
/// the same node, no rebuild, one broadcast.
#[test]
fn m74_the_paint_resolves_the_installed_fact() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let probe = harness.mount(FactProbe.boxed());
    settle(&mut harness);
    let bounds = harness.bounds(probe);

    let landed = harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    assert!(landed, "a fact that differs from the installed one lands");
    // The flip's own frame: the broadcast's masks drain into this
    // tick's dirty map, the paint pass re-emits the layers.
    let report = harness.tick();
    assert!(
        report.pass(PassKind::Paint).is_some_and(|run| run.ran),
        "the flip's broadcast repaints the frame"
    );
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [20, 22, 28, 255],
        "the dark fact paints after the flip"
    );

    // The flip back: the fact re-resolves on its own node.
    let landed = harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: false }));
    assert!(landed, "the flip back lands");
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [240, 241, 244, 255],
        "the light fact paints after the flip back"
    );
}

/// A flip that lands where the fact already is queues no masks and no
/// facts (the idle-frame property, A1's rule at the setting tier) —
/// and installing the canon light where nothing is installed is the
/// no-op too: the default already resolves light.
#[test]
fn m74_the_no_op_flip_queues_nothing() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let probe = harness.mount(FactProbe.boxed());
    settle(&mut harness);
    let bounds = harness.bounds(probe);

    // The canon light over nothing: the default fact is already light.
    let landed = harness.pipeline_mut().set_settings(Box::new(CanonSettings));
    assert!(
        !landed,
        "installing the canon light where nothing is installed is the no-op"
    );
    assert!(
        settle(&mut harness).is_idle(),
        "no broadcast, an idle frame"
    );

    // A real flip lands, then the same fact again is the no-op.
    assert!(
        harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: true }))
    );
    settle(&mut harness);
    let counters = harness.counters();
    let landed = harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    assert!(!landed, "the same fact is a no-op");
    let report = harness.tick();
    assert!(
        report.is_idle(),
        "the no-op flip queues no masks — the idle frame, A1's rule"
    );
    settle(&mut harness);
    assert_eq!(
        harness.counters().paint_runs,
        counters.paint_runs,
        "the no-op flip paints nothing"
    );
    assert_eq!(setting_facts(&trail).len(), 1, "the no-op streamed no fact");
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [20, 22, 28, 255],
        "the probe's pixels are where the flip left them"
    );
}

/// The flip streams one `SettingRecord` the tick after it lands — the
/// fact itself (`dark`), never more than the ask (the payload rule at
/// the setting tier: the bool *is* the fact).
#[test]
fn m74_the_flip_streams_one_setting_record() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let _probe = harness.mount(FactProbe.boxed());
    settle(&mut harness);
    assert!(
        setting_facts(&trail).is_empty(),
        "the frames that flipped nothing stream no setting fact"
    );

    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 1, "one fact per flip that landed");
    assert!(facts[0].dark, "the record names the fact it landed");

    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: false }));
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 2, "one fact per flip, the flip back beside it");
    assert!(
        !facts[1].dark,
        "the flip back's record names the light fact"
    );
}

/// The fact never munges the palette: with a midnight palette
/// installed, the dark flip moves the fact-resolved probe and leaves
/// the palette-resolved probe exactly where it was — the setting is
/// the fact the vocabulary consults, never a palette swap, never a
/// retheme the widget vocabulary guesses at.
#[test]
fn m74_the_fact_never_munges_the_palette() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let role = harness.add_child(root, RoleProbe.boxed());
    let fact = harness.add_child(root, FactProbe.boxed());
    harness.pipeline_mut().set_theme(Box::new(MidnightPalette));
    settle(&mut harness);
    let role_bounds = harness.bounds(role);
    let fact_bounds = harness.bounds(fact);
    let role_before = pixel_at(&mut harness, role_bounds, 32.0, 32.0);
    assert_eq!(
        role_before,
        [240, 180, 41, 255],
        "the midnight accent paints before the flip"
    );

    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, role_bounds, 32.0, 32.0),
        role_before,
        "the palette-resolved probe answers the same midnight accent — \
         the flip never rethemes"
    );
    assert_eq!(
        pixel_at(&mut harness, fact_bounds, 32.0, 32.0),
        [20, 22, 28, 255],
        "the fact-resolved probe answers the dark fact"
    );
}

/// The flip's answer as data: the door answers `bool` the theme door
/// answers, and the setting tier's own record rides the causal log
/// *beside* the door's answer, never instead of it.
#[test]
fn m74_the_door_answers_as_data() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let _probe = harness.mount(FactProbe.boxed());
    settle(&mut harness);

    let landed = harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    assert!(landed, "the first flip lands");
    let landed = harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: false }));
    assert!(landed, "the flip back lands");
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 2, "each landed flip carries its own record");
    assert_ne!(
        facts[0].dark, facts[1].dark,
        "the records never collide — each names its own flip"
    );
}

/// The observation is priced, not assumed: the disabled trail costs
/// one branch per note (no observer installed — the harness never
/// enables it), the records never queue, the flip still lands.
#[test]
fn m74_the_disabled_trail_still_flips() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let probe = harness.mount(FactProbe.boxed());
    settle(&mut harness);
    let bounds = harness.bounds(probe);
    let landed = harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    assert!(landed, "the flip lands without a listener");
    settle(&mut harness);
    assert_eq!(
        pixel_at(&mut harness, bounds, 32.0, 32.0),
        [20, 22, 28, 255],
        "the fact resolves regardless of the trail"
    );
}

/// The living documents re-agree at the M75 cut (the pins ride the
/// living cut, the m75 suite pinning the M75 row itself): the README's
/// status block names the milestone, the corpus count rides the
/// settings tier's pricing and the composition tier's beside it, and
/// the architecture carries the decision.
#[test]
fn m74_the_living_documents_re_agree() {
    let readme = read_workspace_file("README.md");
    assert!(
        readme.contains("M0 through M135"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("All 144 criterion cases"),
        "the README quotes the baselines.json bench_count the composition tier grew"
    );
    let architecture = read_workspace_file("docs/architecture.md");
    assert!(
        architecture.contains("ADR-0080"),
        "the architecture carries the setting seam's decision"
    );
    assert!(
        architecture.contains("a hundred and forty decision records"),
        "the register grew the M75 row the count rides"
    );
    let baselines = serde_json::from_str::<serde_json::Value>(&read_workspace_file(
        "benchmarks/baselines/baselines.json",
    ))
    .expect("the baselines parse");
    assert_eq!(
        baselines["bench_count"], 144,
        "the published corpus carries the composition tier's pricing"
    );
}

/// The demo drives the seam from the consumer side: the dark-mode
/// demo's beats (canon light builds, the OS flips dark, the same fact
/// again is the no-op, the flip back) are pinned here by the same
/// shapes the demo logs — the example compiles under the gates.
#[test]
fn m74_the_demo_shape_drives_the_door() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let _probe = harness.mount(FactProbe.boxed());
    settle(&mut harness);

    // The demo's beat one: the OS pushes dark through the door.
    assert!(
        harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: true }))
    );
    settle(&mut harness);
    // Beat two: the same fact again — the honest no-op.
    assert!(
        !harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: true }))
    );
    settle(&mut harness);
    // Beat three: the OS pushes light back.
    assert!(
        harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: false }))
    );
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
}

/// The probe's fact answer is pixel-distinct from its palette answer —
/// a fact probe and a palette probe can never be confused (the suite's
/// own hygiene: the `pixel_diff_ratio` between the two probes' colors is
/// bounded away from zero).
#[test]
fn m74_the_fact_and_palette_answers_are_distinct() {
    let fact_dark = [20u8, 22, 28];
    let fact_light = [240u8, 241, 244];
    let midnight_accent = [240u8, 180, 41];
    assert_ne!(fact_dark, fact_light);
    assert_ne!(fact_dark, midnight_accent);
    assert_ne!(fact_light, midnight_accent);
    // The probes' colors are distinct enough that a stale resolution
    // cannot pass for the flip (the ratio is a smoke check, the
    // byte-level pins above are the contract).
    let ratio = pixel_diff_ratio(&image_with(fact_dark), &image_with(fact_light));
    assert!(
        ratio > 0.9,
        "the fact's two answers differ across the image"
    );
}

/// Builds a one-color image for the distinctness check.
fn image_with(rgb: [u8; 3]) -> tiny_skia::Pixmap {
    let mut pixmap = tiny_skia::Pixmap::new(8, 8).expect("an 8x8 pixmap");
    pixmap.fill(tiny_skia::Color::from_rgba8(rgb[0], rgb[1], rgb[2], 255));
    pixmap
}
