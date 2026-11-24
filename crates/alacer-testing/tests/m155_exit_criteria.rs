//! M155 exit-criteria integration tests (§7.2 pass 8, ADR-0159): the
//! setting seam grows the OS's text-scale fact — ADR-0087's own
//! pre-registered re-open rule cashing ("a consumer needs more OS facts
//! than these → the trait grows the fact beside them — never a second
//! seam per fact; the record and the wire grow the field beside the
//! facts by this cut's own shape"). Since M155 every
//! [`alacer_core::inspect::SettingRecord`] carries the text-scale fact
//! beside the dark, reduced-motion, high-contrast, accent, and
//! reduced-transparency facts — the door's flip check compares all six
//! (the facts the door reads once each, the bundle it already holds),
//! [`alacer_core::scene::PaintCtx`] resolves the fact beside the
//! palette and the five facts, and the composition resolves by it at
//! the two points the door already owned: a palette that sizes its
//! typography when the OS asks answers by the fact.
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
use alacer_core::settings::{CanonComposer, CanonSettings, SettingComposer, SettingSeam};
use alacer_core::style::{Size2, Style};
use alacer_core::theme::Palette;
use alacer_core::trail::TrailLog;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::BoxedWidget as _;

/// The ink the scale probe paints when the OS asks for larger text —
/// the emphasized surface the fact names (the decision no palette role
/// answers: typography that follows the OS's preferred reading size).
const SCALED_INK: Color = Color::rgb(16, 122, 87);

/// The ink the scale probe paints while the platform reports no text
/// scale — the declared-size surface the honest absence answers.
const ABSENT_INK: Color = Color::rgb(198, 198, 205);

/// The scale the suite flips: one point two five — the OS's ask the
/// beats land, the multiplier the vocabulary consults.
const OS_SCALE: f32 = 1.25;

/// The facts the suite flips: all six OS facts, one seam — the trait
/// grew the text-scale fact beside `dark_mode`, `reduced_motion`,
/// `high_contrast`, `accent_color`, and `reduced_transparency`, never
/// a second seam per fact (ADR-0159). The scale rides its presence:
/// `None` is the honest absence (the platform reports no read), never
/// a guessed multiplier.
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

/// A probe that paints its whole bounds from the resolved *text-scale*
/// fact — deterministic pixels, no palette or glyph dependency (the
/// m154 suite's fact-probe shape, grown beside the sixth fact): the
/// pixel evidence that the text-scale fact reached the paint pass. The
/// honest absence paints the declared-size ink, the OS's ask paints
/// the emphasized ink — the decision the vocabulary consults.
struct TextScaleProbe;

impl Widget for TextScaleProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("text scale probe")
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
            color: if ctx.text_scale().is_some() {
                SCALED_INK
            } else {
                ABSENT_INK
            },
            corner_radius: 0.0,
            border_color: alacer_core::Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// A probe that paints its whole bounds from the resolved *palette*
/// (the accent role) — the composition's pixel evidence (the m81
/// suite's role probe): the palette the widgets borrow moves when —
/// and only when — the composition resolves a new one by the facts.
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

/// The scaled composition (ADR-0159): the canon palette whose accent
/// role follows the OS's text-size ask when the OS asks for a scale —
/// the palette that answers by the fact at the two points the door
/// already owned (the install, every landed flip). The honest absence
/// resolves the canon palette: the composition that answers `None`
/// with its own. The tag names the composition whose answer follows
/// the fact (the M75 rule: one tag over every palette the composition
/// resolves).
struct ScaledComposer;

impl SettingComposer for ScaledComposer {
    fn palette(
        &self,
        _dark: bool,
        _reduced_motion: bool,
        _high_contrast: bool,
        _accent: Option<Color>,
        _reduced_transparency: bool,
        text_scale: Option<f32>,
    ) -> Palette {
        if text_scale.is_some() {
            Palette {
                accent: SCALED_INK,
                ..Palette::CANON
            }
        } else {
            Palette::CANON
        }
    }

    fn name(&self) -> &'static str {
        "scaled"
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

/// The streamed setting facts, oldest first.
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

/// The text-scale flip lands where the other facts are still — the
/// case the M154 door's five-fact check would have swallowed: the door
/// answered `false` there, the fact never reached the vocabulary.
/// Since M155 the flip check compares all six facts (the facts the
/// door reads once each into the bundle it already holds), the
/// text-scale flip lands, the broadcast repaints the tree, and the
/// record names all six facts beside the reach (the lengths rule, the
/// ids never riding).
#[test]
fn m155_the_text_scale_flip_lands_where_the_facts_are_still() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let probe = harness.mount(TextScaleProbe.boxed());
    settle(&mut harness);

    // The scale-only flip: dark stays light, motion stays allowed,
    // contrast stays standard, no accent, effects allowed. The door
    // must answer `true` — the M154 check, the five facts alone, would
    // have answered `false` here.
    assert!(
        harness.pipeline_mut().set_settings(Box::new(FixedSettings {
            dark: false,
            reduced_motion: false,
            high_contrast: false,
            accent: None,
            reduced_transparency: false,
            text_scale: Some(OS_SCALE),
        })),
        "the text-scale-only flip lands — the grown check compares all six facts"
    );
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 1, "one fact per flip that landed");
    assert!(
        !facts[0].dark
            && !facts[0].reduced_motion
            && !facts[0].high_contrast
            && facts[0].accent.is_none()
            && !facts[0].reduced_transparency,
        "the five facts ride beside the text-scale fact"
    );
    assert_eq!(
        facts[0].text_scale,
        Some(OS_SCALE),
        "the record names the ask the OS landed — the multiplier, not a flag"
    );
    assert_eq!(
        facts[0].repainted, 1,
        "the broadcast's own node count: the single-probe tree's reach"
    );

    // The pixel evidence: the probe painted from the resolved fact.
    assert_eq!(
        center_pixel(&mut harness, probe),
        [16, 122, 87, 255],
        "the vocabulary consults the text-scale fact — the probe repainted emphasized"
    );
}

/// The full flip beside the text-scale fact lands the same way: one
/// seam, all six facts carried, the record naming them beside each
/// other — never a second seam or event per fact.
#[test]
fn m155_the_record_names_all_six_facts_beside_each_other() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let _probe = harness.mount(TextScaleProbe.boxed());
    settle(&mut harness);

    harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: true,
        reduced_motion: true,
        high_contrast: true,
        accent: Some(Color::rgb(255, 94, 58)),
        reduced_transparency: true,
        text_scale: Some(OS_SCALE),
    }));
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 1, "one fact per flip that landed");
    assert!(
        facts[0].dark
            && facts[0].reduced_motion
            && facts[0].high_contrast
            && facts[0].reduced_transparency
            && facts[0].accent.is_some(),
        "the record carries the text-scale fact beside the five facts"
    );
    assert_eq!(
        facts[0].text_scale,
        Some(OS_SCALE),
        "the scale rides beside"
    );
    assert_eq!(
        facts[0].repainted, 1,
        "the reach rides beside all six facts"
    );
}

/// The canon fact is not a guess: the canon settings answer the honest
/// absence — no platform read, no invented multiplier, installable by
/// name — and a flip to the ask over the canon absence lands like any
/// fact move, the probe painting the emphasized ink; the canon absence
/// rides the record's `None` beside the facts it sits beside.
#[test]
fn m155_the_canon_fact_answers_the_honest_absence() {
    assert!(
        CanonSettings.text_scale().is_none(),
        "the canon fact answers the honest absence — no guessed multiplier"
    );
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let probe = harness.mount(TextScaleProbe.boxed());
    settle(&mut harness);
    assert_eq!(
        center_pixel(&mut harness, probe),
        [198, 198, 205, 255],
        "the default build resolves the canon absence — the probe paints the declared size"
    );

    // The ask lands over the canon absence — the fact move the flip
    // door exists for — and the record carries both answers beside
    // each other.
    assert!(
        harness.pipeline_mut().set_settings(Box::new(FixedSettings {
            dark: false,
            reduced_motion: false,
            high_contrast: false,
            accent: None,
            reduced_transparency: false,
            text_scale: Some(OS_SCALE),
        })),
        "the ask over the canon answer is a fact move — the flip lands"
    );
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 1, "the fact move landed once");
    assert_eq!(
        facts[0].text_scale,
        Some(OS_SCALE),
        "the record names the ask that landed"
    );
    assert_eq!(
        center_pixel(&mut harness, probe),
        [16, 122, 87, 255],
        "the probe repainted emphasized — the fact the vocabulary consults moved"
    );
}

/// The no-op flip names nothing: a flip that lands where *all six*
/// facts already are queues no record — the idle-frame property (A1's
/// rule at the setting tier) unchanged by the fact's growth.
#[test]
fn m155_the_no_op_names_nothing() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let probe = harness.mount(TextScaleProbe.boxed());
    settle(&mut harness);

    assert!(harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: true,
        reduced_motion: true,
        high_contrast: true,
        accent: None,
        reduced_transparency: true,
        text_scale: Some(OS_SCALE),
    })));
    settle(&mut harness);
    let counters = harness.counters();
    assert!(
        !harness.pipeline_mut().set_settings(Box::new(FixedSettings {
            dark: true,
            reduced_motion: true,
            high_contrast: true,
            accent: None,
            reduced_transparency: true,
            text_scale: Some(OS_SCALE),
        })),
        "the same facts are a no-op"
    );
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
        center_pixel(&mut harness, probe),
        [16, 122, 87, 255],
        "the probe kept the emphasized paint"
    );
}

/// The composition follows the text-scale fact: a composition
/// installed over the canon default resolves its scaled palette when
/// the OS asks for larger text and its own palette when it does not —
/// the signature grown beside the five facts (ADR-0086's own rule,
/// cashed a fourth time), the pipeline handing the composition what it
/// asks for, never the composer reaching into an adapter. The install
/// resolves against the facts that are, and the canon composition
/// answers the canon palette for any facts — behavior unchanged,
/// signature grown.
#[test]
fn m155_the_composition_follows_the_text_scale_fact() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);

    // The composition installs over the canon default: the canon's
    // answer *is* the canon palette — the honest no-op install.
    harness
        .pipeline_mut()
        .set_composer(Box::new(ScaledComposer));
    settle(&mut harness);
    assert!(
        theme_facts(&trail).is_empty(),
        "the canon answer over the canon default composes nothing"
    );

    // The ask lands: the composition re-resolves by the fact that
    // landed — the scaled palette mounts riding the flip's own
    // broadcast, the theme fact tagged with the composer's answer.
    assert!(harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: false,
        reduced_motion: false,
        high_contrast: false,
        accent: None,
        reduced_transparency: false,
        text_scale: Some(OS_SCALE),
    })));
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(swaps.len(), 1, "the text-scale flip composed one swap");
    assert_eq!(
        swaps[0].palette, "scaled",
        "the tag names the composer whose answer followed the fact"
    );
    assert_eq!(
        center_pixel(&mut harness, probe),
        [16, 122, 87, 255],
        "the palette the widgets borrow follows the OS's text-size ask"
    );

    // The fact moves back: the composition re-resolves its own palette
    // — the canon answer, the scale leaving with the fact.
    assert!(harness.pipeline_mut().set_settings(Box::new(CanonSettings)));
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(swaps.len(), 2, "the fact's flip composed one swap");
    assert_eq!(
        center_pixel(&mut harness, probe),
        [37, 99, 235, 255],
        "the composition answers the honest absence with its own — the canon palette"
    );

    // The canon composition: grown in signature, unchanged in behavior
    // — the canon palette for any facts, the text-scale fact included.
    assert_eq!(
        CanonComposer.palette(false, false, false, None, false, None),
        Palette::CANON,
        "the canon composition answers the canon palette for the honest absence"
    );
    assert_eq!(
        CanonComposer.palette(false, false, false, None, false, Some(OS_SCALE)),
        Palette::CANON,
        "the canon composition answers the canon palette for the ask too"
    );
}

/// The install resolves against the facts that are: a composer
/// installed while the seam already holds the ask answers the scaled
/// palette on the install's own broadcast — the pipeline handing the
/// facts the composition asks for at the install too, never only on
/// the flips.
#[test]
fn m155_the_install_resolves_against_the_facts_that_are() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);

    // The ask lands before any composer is installed: the flip lands,
    // the setting fact streams, no theme fact (no composer).
    assert!(harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: false,
        reduced_motion: false,
        high_contrast: false,
        accent: None,
        reduced_transparency: false,
        text_scale: Some(OS_SCALE),
    })));
    settle(&mut harness);
    assert_eq!(setting_facts(&trail).len(), 1, "the flip landed");
    assert!(
        theme_facts(&trail).is_empty(),
        "no composer, no composition — the flip streams no theme fact"
    );

    // The install reads the facts that are: the scaled answer mounts on
    // the install's own broadcast.
    assert!(
        harness
            .pipeline_mut()
            .set_composer(Box::new(ScaledComposer)),
        "the install moves the palette — the ask composes the scaled answer"
    );
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(swaps.len(), 1, "the install composed one swap");
    assert_eq!(
        swaps[0].palette, "scaled",
        "the install resolved by the facts that are"
    );
    assert_eq!(
        center_pixel(&mut harness, probe),
        [16, 122, 87, 255],
        "the scaled palette is the palette the widgets borrow from the install on"
    );
}

/// The wire names the text-scale fact: the setting event's flat
/// record carries `text_scale` beside `dark`, `reduced_motion`,
/// `high_contrast`, `accent`, `reduced_transparency`, and `repainted`
/// — the additive field the versioning rule admits at version 1, the
/// ask as the number the OS reported, the honest absence as `null`
/// (never a guessed multiplier), the ids never riding (the payload
/// rule's shape).
#[test]
fn m155_the_wire_names_the_text_scale_fact() {
    let asked = SettingRecord {
        dark: false,
        reduced_motion: false,
        high_contrast: false,
        accent: None,
        reduced_transparency: false,
        text_scale: Some(OS_SCALE),
        repainted: 42,
    };
    let event = alacer_inspect::wire::wire_setting(&asked);
    let json = serde_json::to_value(&event).expect("the wire event serializes");
    assert_eq!(
        json["kind"], "setting",
        "the setting event's kind is the stable, documented value"
    );
    assert_eq!(
        json["text_scale"], OS_SCALE,
        "the text-scale fact rides the wire beside the facts — the additive field"
    );
    assert_eq!(
        json["repainted"], 42,
        "the count rides the wire beside the facts — lengths, never ids"
    );

    let absent = SettingRecord {
        dark: false,
        reduced_motion: false,
        high_contrast: false,
        accent: None,
        reduced_transparency: false,
        text_scale: None,
        repainted: 0,
    };
    let event = alacer_inspect::wire::wire_setting(&absent);
    let json = serde_json::to_value(&event).expect("the wire event serializes");
    assert!(
        json["text_scale"].is_null(),
        "the honest absence rides the wire as `null` — never a guessed multiplier"
    );
}

/// The observation is priced, not assumed: the disabled trail still
/// lands the text-scale flip — the facts computed where the broadcast
/// resolves, the records never queue without a listener (the m154
/// suite's own rule, the fact's growth riding it unchanged).
#[test]
fn m155_the_disabled_trail_still_lands() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let probe = harness.mount(TextScaleProbe.boxed());
    settle(&mut harness);
    assert!(
        harness.pipeline_mut().set_settings(Box::new(FixedSettings {
            dark: false,
            reduced_motion: false,
            high_contrast: false,
            accent: None,
            reduced_transparency: false,
            text_scale: Some(OS_SCALE),
        })),
        "the text-scale flip lands without a listener"
    );
    settle(&mut harness);
    assert_eq!(
        center_pixel(&mut harness, probe),
        [16, 122, 87, 255],
        "the fact resolves regardless of the trail"
    );
    let report = harness.tick();
    assert!(
        report.is_idle(),
        "no listener, no extra paint work — the trail's absence costs nothing"
    );
}

/// The living documents re-agree at the M155 cut: the README's status
/// block names the milestone, the corpus count carries the fact's
/// pricing, the architecture carries the decision, and the protocol
/// document pins the field the wire grew.
#[test]
fn m155_the_living_documents_re_agree() {
    let readme = read_workspace_file("README.md");
    assert!(
        readme.contains("M0 through M155"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("All 204 criterion cases"),
        "the README quotes the baselines.json bench_count the fact tier grew"
    );
    let architecture = read_workspace_file("docs/architecture.md");
    assert!(
        architecture.contains("ADR-0159"),
        "the architecture carries the text-scale fact's decision"
    );
    assert!(
        architecture.contains("a hundred and sixty decision records"),
        "the register grew the M155 row"
    );
    let baselines = serde_json::from_str::<serde_json::Value>(&read_workspace_file(
        "benchmarks/baselines/baselines.json",
    ))
    .expect("the baselines parse");
    assert_eq!(
        baselines["bench_count"], 204,
        "the published corpus carries the text-scale flip's pricing"
    );
    assert!(
        baselines["benches"]
            .get("settings/door_flip_text_scale")
            .is_some(),
        "the published baseline carries the text-scale flip's pricing (the coordinated rule, ADR-0011)"
    );
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("text_scale"),
        "the protocol document pins the field the wire grew"
    );
    // The sibling suites' pins ride the same cut (one sweep, no drift):
    // the m154 suite's documents pin moved to M155's cut.
    let m154 = read_workspace_file("crates/alacer-testing/tests/m154_exit_criteria.rs");
    assert!(
        m154.contains("M0 through M155"),
        "the m154 suite's pins moved with the documents"
    );
}
