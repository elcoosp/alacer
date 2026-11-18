//! M80 exit-criteria integration tests (§7.2 pass 8, ADR-0086): the
//! composition follows the facts — ADR-0083's second pre-registered
//! re-open rule, grown by ADR-0085's own cut ("a consumer needs the
//! composition to follow the motion fact → the composer's signature
//! grows beside `palette(dark)` only when the fact exists to hand —
//! the pipeline handing the facts the composition asks for, never the
//! composer reaching into an adapter"). Since M80
//! [`alacer_core::settings::SettingComposer::palette`] answers by the
//! facts the seam holds (`dark`, `reduced_motion`, `high_contrast`):
//! every landed flip re-resolves the composition against the facts
//! that landed — a palette that calms when the OS asks for reduced
//! motion re-resolves on the motion-only landing the M77 cut admitted,
//! riding the flip's own broadcast (one flip, one count, ADR-0084's
//! rule unchanged) — and the install resolves against the facts that
//! are. The wire carries nothing new: the composed swap names itself
//! in the tag its composer answered.
//!
//! Every test runs the real pipeline (A4): real layout, real paint
//! rasterization through the harness, the records through the harness
//! trail door.

use std::path::PathBuf;

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect};
use alacer_core::inspect::{InspectEvent, SettingRecord, ThemeRecord};
use alacer_core::passes::FrameReport;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::settings::{CanonSettings, SettingComposer, SettingSeam};
use alacer_core::style::{Size2, Style};
use alacer_core::theme::Palette;
use alacer_core::trail::TrailLog;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::BoxedWidget as _;

/// The facts the suite flips: all three OS facts, one seam (the m79
/// suite's shape — the suite's subject is which facts the composition
/// resolves by).
struct FixedSettings {
    dark: bool,
    reduced_motion: bool,
    high_contrast: bool,
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

    fn accent_color(&self) -> Option<alacer_core::color::Color> {
        None
    }
}

/// The duet dark palette the fact-following composition answers while
/// motion is allowed (the m75 suite's duet dark, verbatim).
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

/// The duet-calm dark palette the same composition answers when the
/// OS asks for reduced motion — every role resolved, the accent
/// quieted (the palette that calms with the fact; every role distinct
/// from the duet dark's and the canon's, so all three resolutions are
/// tellable apart).
const DUET_CALM: Palette = Palette {
    surface: Color::rgb(18, 20, 26),
    panel: Color::rgb(30, 33, 42),
    accent: Color::rgb(180, 168, 120),
    accent_pressed: Color::rgb(160, 148, 104),
    selection: Color::rgb(180, 168, 120).with_alpha(72),
    text: Color::rgb(220, 224, 232),
    text_muted: Color::rgb(140, 148, 164),
    border: Color::rgb(58, 64, 82),
    on_accent: Color::rgb(26, 22, 10),
};

/// The fact-following composition (ADR-0086): the canon palette for
/// the light fact, the duet dark for the dark one — and the *calm*
/// duet dark when the OS asks for reduced motion, the palette that
/// calms with the fact. The tag names the composition whose answer
/// follows the facts (the M75 rule: one tag over every palette the
/// composition resolves), so the log tells which composition resolved
/// while the palette the tree paints carries the fact's resolution.
struct DuetCalmComposer;

impl SettingComposer for DuetCalmComposer {
    fn palette(
        &self,
        dark: bool,
        reduced_motion: bool,
        _high_contrast: bool,
        _accent: Option<alacer_core::color::Color>,
    ) -> Palette {
        if dark && reduced_motion {
            DUET_CALM
        } else if dark {
            DUET_DARK
        } else {
            Palette::CANON
        }
    }

    fn name(&self) -> &'static str {
        "duet-calm"
    }
}

/// A probe that paints its whole bounds from the resolved *palette*
/// (the m75 suite's role probe, the accent role) — the composition's
/// pixel evidence: the palette the widgets borrow moves when — and
/// only when — the composition resolves a new one by the facts.
struct RoleProbe;

impl Widget for RoleProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("palette probe")
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
            border_color: alacer_core::Color::TRANSPARENT,
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

/// The streamed theme swaps, oldest first.
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

/// The streamed setting flips, oldest first.
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

/// The composed palette follows the motion fact: a motion-only flip
/// (the landing the M77 cut admitted) re-resolves the composition by
/// the facts that landed — the palette that calms mounts riding the
/// flip's own broadcast, the theme fact naming the resolution the
/// facts landed (`duet-calm` beside the flip's `Setting` record), the
/// probe's pixel moving with it — one flip, one count, the theme
/// record naming the same reach the flip broadcast (ADR-0084's rule
/// unchanged by the signature's growth).
#[test]
fn m80_the_composed_palette_follows_the_motion_fact() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);

    // The composition installs over the canon default: the light fact's
    // answer *is* the canon palette — the honest no-op install.
    harness
        .pipeline_mut()
        .set_composer(Box::new(DuetCalmComposer));
    settle(&mut harness);
    assert!(
        theme_facts(&trail).is_empty(),
        "the canon answer over the canon default composes nothing"
    );

    // The dark flip: the composition resolves the duet dark (motion
    // allowed), the theme fact tagged with the composer's answer.
    assert!(harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: true,
        reduced_motion: false,
        high_contrast: false,
    })));
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(swaps.len(), 1, "the dark flip composed one swap");
    assert_eq!(swaps[0].palette, "duet-calm", "the tag names the composer");
    assert_eq!(
        center_pixel(&mut harness, probe),
        [255, 176, 64, 255],
        "the probe paints the duet dark's accent"
    );

    // The motion-only flip: dark stays dark, the OS asks for reduced
    // motion — the composition re-resolves by the facts that landed,
    // the calm palette mounts riding the flip's own broadcast.
    assert!(harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: true,
        reduced_motion: true,
        high_contrast: false,
    })));
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(swaps.len(), 2, "the motion-only flip composed one swap");
    assert_eq!(
        swaps[1].palette, "duet-calm",
        "the tag names the composer whose answer followed the fact"
    );
    assert_eq!(
        swaps[1].repainted, 1,
        "one flip, one count — the composed swap names the flip's own reach"
    );
    assert!(
        setting_facts(&trail)
            .last()
            .expect("the flip's record")
            .reduced_motion,
        "the flip's own record names the fact the composition followed"
    );
    assert_eq!(
        center_pixel(&mut harness, probe),
        [180, 168, 120, 255],
        "the palette the widgets borrow calmed with the fact"
    );
}

/// The install resolves against the facts that are: a composer
/// installed while the seam already holds the calm facts answers the
/// calm palette on the install's own broadcast — the pipeline handing
/// the facts the composition asks for at the install too, never only
/// on the flips.
#[test]
fn m80_the_install_resolves_against_the_facts_that_are() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);

    // The calm facts land before any composer is installed: the flip
    // lands, the setting fact streams, no theme fact (no composer).
    assert!(harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: true,
        reduced_motion: true,
        high_contrast: false,
    })));
    settle(&mut harness);
    assert_eq!(setting_facts(&trail).len(), 1, "the flip landed");
    assert!(
        theme_facts(&trail).is_empty(),
        "no composer, no composition — the flip streams no theme fact"
    );

    // The install reads the facts that are: the calm answer mounts on
    // the install's own broadcast.
    assert!(
        harness
            .pipeline_mut()
            .set_composer(Box::new(DuetCalmComposer)),
        "the install moves the palette — the calm facts compose the calm answer"
    );
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(swaps.len(), 1, "the install composed one swap");
    assert_eq!(
        swaps[0].palette, "duet-calm",
        "the install resolved by the facts that are"
    );
    assert_eq!(
        center_pixel(&mut harness, probe),
        [180, 168, 120, 255],
        "the calm palette is the palette the widgets borrow from the install on"
    );

    // The canon facts over the calm composition: the palette moves
    // back — one door, the same resolution rules.
    assert!(harness.pipeline_mut().set_settings(Box::new(CanonSettings)));
    settle(&mut harness);
    assert_eq!(
        center_pixel(&mut harness, probe),
        [37, 99, 235, 255],
        "the canon palette re-resolved through the same composer"
    );
}

/// The fact-agnostic composer stays the no-op: a composer whose
/// answer does not read the facts composes nothing on a motion-only
/// flip — the setting fact streams (the flip landed), the theme tier
/// stays silent (the composed palette is unchanged, the equality's
/// idle rule), the A1 property guarding the swap half exactly as it
/// did before the signature grew.
#[test]
fn m80_the_fact_agnostic_composer_stays_the_no_op() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let _probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);

    harness
        .pipeline_mut()
        .set_composer(Box::new(DuetCalmComposer));
    harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: true,
        reduced_motion: false,
        high_contrast: false,
    }));
    settle(&mut harness);

    // The fact-agnostic composition: the canon composer answers the
    // canon palette for any facts — the install moves the palette
    // back (the duet dark is installed), and the answers that follow
    // never read the facts.
    assert!(
        harness
            .pipeline_mut()
            .set_composer(Box::new(alacer_core::settings::CanonComposer)),
        "the canon install moves the duet dark back to the canon palette"
    );
    settle(&mut harness);
    let swaps_before = theme_facts(&trail).len();
    let counters = harness.counters();
    let flips_before = setting_facts(&trail).len();

    // The motion-only flip over the fact-agnostic composition: the
    // flip lands (the setting fact streams), the composition resolves
    // the palette already installed (no theme fact, no paint).
    assert!(harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: true,
        reduced_motion: true,
        high_contrast: false,
    })));
    settle(&mut harness);
    assert_eq!(
        setting_facts(&trail).len(),
        flips_before + 1,
        "the motion-only flip landed — the setting tier's own record"
    );
    assert_eq!(
        theme_facts(&trail).len(),
        swaps_before,
        "the composition resolved the palette already installed — no theme fact"
    );
    // The flip's own broadcast still repaints the tree — the setting
    // tier's fact moved (the A1 rule at the setting tier), even though
    // the theme tier's swap half stayed silent. The theme tier's own
    // idle property is the absence of the theme fact and the composed
    // swap, not the flip's own broadcast.
    settle(&mut harness);
    assert_eq!(
        harness.counters().paint_runs,
        counters.paint_runs + 1,
        "the flip's own broadcast painted once; the composition's swap half painted nothing beyond it"
    );
}

/// The living documents re-agree at the M80 cut: the README's status
/// block names the milestone, the corpus count carries the
/// composition's pricing (the route the corpus already prices), the
/// architecture carries the decision, and the M79 suite's pins moved
/// with the documents (one sweep, no drift).
#[test]
fn m80_the_living_documents_re_agree() {
    let readme = read_workspace_file("README.md");
    assert!(
        readme.contains("M0 through M113"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("All 131 criterion cases"),
        "the README quotes the baselines.json bench_count the composition's pricing rides"
    );
    let architecture = read_workspace_file("docs/architecture.md");
    assert!(
        architecture.contains("ADR-0085"),
        "the architecture carries the contrast fact's decision"
    );
    assert!(
        architecture.contains("ADR-0086"),
        "the architecture carries the fact-following composition's decision"
    );
    assert!(
        architecture.contains("a hundred and eighteen decision records"),
        "the register grew the M80 row"
    );
    let baselines = serde_json::from_str::<serde_json::Value>(&read_workspace_file(
        "benchmarks/baselines/baselines.json",
    ))
    .expect("the baselines parse");
    assert_eq!(
        baselines["bench_count"], 131,
        "the published corpus carries the composition's pricing — the route is not new"
    );
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("high_contrast"),
        "the protocol document pins the field the M79 cut grew"
    );
    let m79 = read_workspace_file("crates/alacer-testing/tests/m79_exit_criteria.rs");
    assert!(
        m79.contains("M0 through M113"),
        "the m79 suite's pins moved with the documents"
    );
}
