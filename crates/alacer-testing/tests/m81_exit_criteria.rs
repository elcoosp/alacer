//! M81 exit-criteria integration tests (§7.2 pass 8, ADR-0087): the
//! setting seam grows the OS's accent-color fact — ADR-0085's own
//! pre-registered re-open rule cashing ("a consumer needs more OS facts
//! than these two (high contrast, accent color) → the trait grows the
//! fact beside the two — never a second seam per fact; the record and
//! the wire grow the field beside the facts by this cut's own shape"),
//! grown by ADR-0086's own rule beside it ("a consumer needs the
//! composition to answer a third fact beside the three (accent color,
//! text scale) → the signature grows the fact beside the three by this
//! cut's own shape, priced where the composition resolves — never the
//! composer reaching into an adapter"). Since M81 every
//! [`alacer_core::inspect::SettingRecord`] carries the accent fact
//! beside the dark, reduced-motion, and high-contrast facts — the fact
//! riding its presence (`None` is the honest absence, never a guess) —
//! the door's flip check compares all four,
//! [`alacer_core::scene::PaintCtx`] resolves the fact beside the
//! palette and the three bool facts, and the composition resolves by it
//! at the two points the door already owned: a palette that tints its
//! accent roles from the OS accent answers by the fact.
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

/// The OS accent the suite's simulated OS reports — the fact's answer
/// an adapter would hand the door (a Linux desktop's picked accent, an
/// orange).
const OS_ACCENT: Color = Color::rgb(255, 94, 58);

/// The facts the suite flips: all four OS facts, one seam — the trait
/// grew the accent fact beside `dark_mode`, `reduced_motion`, and
/// `high_contrast`, never a second seam per fact (ADR-0087). The accent
/// rides its presence: `None` is the honest absence (the platform
/// reports no accent), never a guess.
struct FixedSettings {
    dark: bool,
    reduced_motion: bool,
    high_contrast: bool,
    accent: Option<Color>,
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
}

/// A probe that paints its whole bounds from the resolved *accent*
/// fact — deterministic pixels, no palette or glyph dependency (the
/// m79 suite's fact-probe shape, grown beside the accent fact): the
/// pixel evidence that the accent fact reached the paint pass. The
/// absence ink the honest-absence build paints, the OS accent the
/// consulted decision paints.
struct AccentProbe;

impl Widget for AccentProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("accent probe")
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
            color: ctx.accent_color().unwrap_or(Color::rgb(148, 148, 155)),
            corner_radius: 0.0,
            border_color: alacer_core::Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// A probe that paints its whole bounds from the resolved *palette*
/// (the accent role) — the composition's pixel evidence (the m80
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

/// The accent-following composition (ADR-0087): the canon palette with
/// its accent roles tinted from the OS accent when the fact is
/// present — the palette that answers by the fact at the two points
/// the door already owned (the install, every landed flip). The
/// absence resolves the canon palette: the composition that answers no
/// accent with its own. The tag names the composition whose answer
/// follows the fact (the M75 rule: one tag over every palette the
/// composition resolves).
struct AccentFollowComposer;

impl AccentFollowComposer {
    /// The accent roles the OS accent resolves: the accent itself, a
    /// pressed variant a step darker, the selection riding it. The
    /// roles the palette answers *by the fact* — the tint a following
    /// app borrows.
    fn tinted(accent: Color) -> Palette {
        Palette {
            accent,
            accent_pressed: Color::rgb(
                accent.r.saturating_mul(8) / 10,
                accent.g.saturating_mul(8) / 10,
                accent.b.saturating_mul(8) / 10,
            ),
            selection: accent.with_alpha(64),
            ..Palette::CANON
        }
    }
}

impl SettingComposer for AccentFollowComposer {
    fn palette(
        &self,
        _dark: bool,
        _reduced_motion: bool,
        _high_contrast: bool,
        accent: Option<Color>,
    ) -> Palette {
        accent.map_or(Palette::CANON, Self::tinted)
    }

    fn name(&self) -> &'static str {
        "accent-follow"
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

/// The accent flip lands where the other facts are still — the case
/// the M79 door's three-fact check would have swallowed: the door
/// answered `false` there, the fact never reached the vocabulary.
/// Since M81 the flip check compares all four facts, the accent-only
/// flip lands, the broadcast repaints the tree, and the record names
/// all four facts beside the reach (the lengths rule, the ids never
/// riding).
#[test]
fn m81_the_accent_flip_lands_where_the_facts_are_still() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let probe = harness.mount(AccentProbe.boxed());
    settle(&mut harness);

    // The accent-only flip: dark stays light, motion stays allowed,
    // contrast stays standard, the OS reports an accent. The door must
    // answer `true` — the M79 check, the three facts alone, would have
    // answered `false` here.
    assert!(
        harness.pipeline_mut().set_settings(Box::new(FixedSettings {
            dark: false,
            reduced_motion: false,
            high_contrast: false,
            accent: Some(OS_ACCENT),
        })),
        "the accent-only flip lands — the grown check compares all four facts"
    );
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 1, "one fact per flip that landed");
    assert!(
        !facts[0].dark && !facts[0].reduced_motion && !facts[0].high_contrast,
        "the three bool facts ride beside the accent fact"
    );
    assert_eq!(
        facts[0].accent,
        Some(OS_ACCENT),
        "the record names the accent the OS reported"
    );
    assert_eq!(
        facts[0].repainted, 1,
        "the broadcast's own node count: the single-probe tree's reach"
    );

    // The pixel evidence: the probe painted from the resolved fact.
    assert_eq!(
        center_pixel(&mut harness, probe),
        [255, 94, 58, 255],
        "the vocabulary consults the accent fact — the probe repainted tinted"
    );
}

/// The full flip beside the accent fact lands the same way: one seam,
/// all four facts carried, the record naming them beside each other —
/// never a second seam or event per fact.
#[test]
fn m81_the_record_names_all_four_facts_beside_each_other() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let _probe = harness.mount(AccentProbe.boxed());
    settle(&mut harness);

    harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: true,
        reduced_motion: true,
        high_contrast: true,
        accent: Some(OS_ACCENT),
    }));
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 1, "one fact per flip that landed");
    assert!(
        facts[0].dark && facts[0].reduced_motion && facts[0].high_contrast,
        "the record carries the accent fact beside the three bool facts"
    );
    assert_eq!(
        facts[0].accent,
        Some(OS_ACCENT),
        "the accent fact rides beside the facts it sits beside"
    );
    assert_eq!(
        facts[0].repainted, 1,
        "the reach rides beside all four facts"
    );
}

/// The honest absence is not a guess: the canon settings answer `None`
/// — no accent, the fact's own absence, installable by name — and a
/// platform that reports no accent answers `None` through the same
/// door, the probe painting its own absence ink and a flip to the
/// absence landing like any fact move.
#[test]
fn m81_the_honest_absence_is_not_a_guess() {
    assert!(
        CanonSettings.accent_color().is_none(),
        "the canon facts answer the canon absence — no accent"
    );
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let probe = harness.mount(AccentProbe.boxed());
    settle(&mut harness);

    // The accent lands, then the absence moves back — the absence is
    // itself a fact move (the OS stopped reporting one), the flip
    // lands, the probe paints the absence ink.
    harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: false,
        reduced_motion: false,
        high_contrast: false,
        accent: Some(OS_ACCENT),
    }));
    settle(&mut harness);
    assert!(
        harness.pipeline_mut().set_settings(Box::new(CanonSettings)),
        "the absence over the reported accent is a fact move — the flip lands"
    );
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 2, "both fact moves landed");
    assert_eq!(
        facts[1].accent, None,
        "the canon record carries the honest absence"
    );
    assert_eq!(
        center_pixel(&mut harness, probe),
        [148, 148, 155, 255],
        "the probe paints its own absence ink — never a guessed accent"
    );
}

/// The no-op flip names nothing: a flip that lands where *all four*
/// facts already are queues no record — the idle-frame property (A1's
/// rule at the setting tier) unchanged by the fact's growth.
#[test]
fn m81_the_no_op_names_nothing() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let probe = harness.mount(AccentProbe.boxed());
    settle(&mut harness);

    assert!(harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: true,
        reduced_motion: true,
        high_contrast: true,
        accent: Some(OS_ACCENT),
    })));
    settle(&mut harness);
    let counters = harness.counters();
    assert!(
        !harness.pipeline_mut().set_settings(Box::new(FixedSettings {
            dark: true,
            reduced_motion: true,
            high_contrast: true,
            accent: Some(OS_ACCENT),
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
        [255, 94, 58, 255],
        "the probe kept the tinted paint"
    );
}

/// The composition follows the accent fact: a composition installed
/// over the canon default resolves the tinted palette when the OS
/// reports an accent and its own palette when it does not — the
/// signature grown beside the three bool facts (ADR-0086's own rule),
/// the pipeline handing the composition what it asks for, never the
/// composer reaching into an adapter. The install resolves against the
/// facts that are, and the canon composition answers the canon palette
/// for any facts — behavior unchanged, signature grown.
#[test]
fn m81_the_composition_follows_the_accent_fact() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);

    // The composition installs over the canon default: the absence's
    // answer *is* the canon palette — the honest no-op install.
    harness
        .pipeline_mut()
        .set_composer(Box::new(AccentFollowComposer));
    settle(&mut harness);
    assert!(
        theme_facts(&trail).is_empty(),
        "the canon answer over the canon default composes nothing"
    );

    // The accent lands: the composition re-resolves by the fact that
    // landed — the tinted palette mounts riding the flip's own
    // broadcast, the theme fact tagged with the composer's answer.
    assert!(harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: false,
        reduced_motion: false,
        high_contrast: false,
        accent: Some(OS_ACCENT),
    })));
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(swaps.len(), 1, "the accent flip composed one swap");
    assert_eq!(
        swaps[0].palette, "accent-follow",
        "the tag names the composer whose answer followed the fact"
    );
    assert_eq!(
        center_pixel(&mut harness, probe),
        [255, 94, 58, 255],
        "the palette the widgets borrow tints with the OS accent"
    );

    // The absence moves back: the composition re-resolves its own
    // palette — the canon answer, the tint leaving with the fact.
    assert!(harness.pipeline_mut().set_settings(Box::new(CanonSettings)));
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(swaps.len(), 2, "the absence flip composed one swap");
    assert_eq!(
        center_pixel(&mut harness, probe),
        [37, 99, 235, 255],
        "the composition answers no accent with its own — the canon palette"
    );

    // The canon composition: grown in signature, unchanged in behavior
    // — the canon palette for any facts, the accent fact included.
    assert_eq!(
        CanonComposer.palette(false, false, false, None),
        Palette::CANON,
        "the canon composition answers the canon palette for the absence"
    );
    assert_eq!(
        CanonComposer.palette(false, false, false, Some(OS_ACCENT)),
        Palette::CANON,
        "the canon composition answers the canon palette for the accent too"
    );
}

/// The install resolves against the facts that are: a composer
/// installed while the seam already holds the accent answers the
/// tinted palette on the install's own broadcast — the pipeline
/// handing the facts the composition asks for at the install too,
/// never only on the flips.
#[test]
fn m81_the_install_resolves_against_the_facts_that_are() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);

    // The accent lands before any composer is installed: the flip
    // lands, the setting fact streams, no theme fact (no composer).
    assert!(harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: false,
        reduced_motion: false,
        high_contrast: false,
        accent: Some(OS_ACCENT),
    })));
    settle(&mut harness);
    assert_eq!(setting_facts(&trail).len(), 1, "the flip landed");
    assert!(
        theme_facts(&trail).is_empty(),
        "no composer, no composition — the flip streams no theme fact"
    );

    // The install reads the facts that are: the tinted answer mounts on
    // the install's own broadcast.
    assert!(
        harness
            .pipeline_mut()
            .set_composer(Box::new(AccentFollowComposer)),
        "the install moves the palette — the accent composes the tinted answer"
    );
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(swaps.len(), 1, "the install composed one swap");
    assert_eq!(
        swaps[0].palette, "accent-follow",
        "the install resolved by the facts that are"
    );
    assert_eq!(
        center_pixel(&mut harness, probe),
        [255, 94, 58, 255],
        "the tinted palette is the palette the widgets borrow from the install on"
    );
}

/// The wire names the accent fact: the setting event's flat record
/// carries `accent` beside `dark`, `reduced_motion`, `high_contrast`,
/// and `repainted` — the additive field the versioning rule admits at
/// version 1, the fact riding its presence (`null` is the honest
/// absence on the wire), the ids never riding (the payload rule's
/// shape).
#[test]
fn m81_the_wire_names_the_accent_fact() {
    let reported = SettingRecord {
        dark: false,
        reduced_motion: false,
        high_contrast: false,
        accent: Some(OS_ACCENT),
        repainted: 42,
    };
    let event = alacer_inspect::wire::wire_setting(&reported);
    let json = serde_json::to_value(&event).expect("the wire event serializes");
    assert_eq!(
        json["kind"], "setting",
        "the setting event's kind is the stable, documented value"
    );
    assert_eq!(
        json["accent"], "#FF5E3AFF",
        "the accent fact rides the wire as the documented hex — the additive field"
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
        repainted: 0,
    };
    let event = alacer_inspect::wire::wire_setting(&absent);
    let json = serde_json::to_value(&event).expect("the wire event serializes");
    assert!(
        json["accent"].is_null(),
        "the absence rides the wire as null — the honest absence, never a guess"
    );
}

/// The observation is priced, not assumed: the disabled trail still
/// lands the accent flip — the facts computed where the broadcast
/// resolves, the records never queue without a listener (the m79
/// suite's own rule, the fact's growth riding it unchanged).
#[test]
fn m81_the_disabled_trail_still_lands() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let probe = harness.mount(AccentProbe.boxed());
    settle(&mut harness);
    assert!(
        harness.pipeline_mut().set_settings(Box::new(FixedSettings {
            dark: false,
            reduced_motion: false,
            high_contrast: false,
            accent: Some(OS_ACCENT),
        })),
        "the accent flip lands without a listener"
    );
    settle(&mut harness);
    assert_eq!(
        center_pixel(&mut harness, probe),
        [255, 94, 58, 255],
        "the fact resolves regardless of the trail"
    );
    let report = harness.tick();
    assert!(
        report.is_idle(),
        "no listener, no extra paint work — the trail's absence costs nothing"
    );
}

/// The living documents re-agree at the M81 cut: the README's status
/// block names the milestone, the corpus count carries the fact's
/// pricing, the architecture carries the decision, and the protocol
/// document pins the field the wire grew.
#[test]
fn m81_the_living_documents_re_agree() {
    let readme = read_workspace_file("README.md");
    assert!(
        readme.contains("M0 through M120"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("All 140 criterion cases"),
        "the README quotes the baselines.json bench_count the fact tier grew"
    );
    let architecture = read_workspace_file("docs/architecture.md");
    assert!(
        architecture.contains("ADR-0086"),
        "the architecture carries the fact-following composition's decision"
    );
    assert!(
        architecture.contains("ADR-0087"),
        "the architecture carries the accent fact's decision"
    );
    assert!(
        architecture.contains("a hundred and twenty-five decision records"),
        "the register grew the M81 row"
    );
    let baselines = serde_json::from_str::<serde_json::Value>(&read_workspace_file(
        "benchmarks/baselines/baselines.json",
    ))
    .expect("the baselines parse");
    assert_eq!(
        baselines["bench_count"], 140,
        "the published corpus carries the accent flip's pricing"
    );
    assert!(
        baselines["benches"]
            .get("settings/door_flip_accent")
            .is_some(),
        "the published baseline carries the accent flip's pricing (the coordinated rule, ADR-0011)"
    );
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("accent"),
        "the protocol document pins the field the wire grew"
    );
    // The sibling suites' pins ride the same cut (one sweep, no drift):
    // the m80 suite's documents pin moved to M81's cut.
    let m80 = read_workspace_file("crates/alacer-testing/tests/m80_exit_criteria.rs");
    assert!(
        m80.contains("M0 through M120"),
        "the m80 suite's pins moved with the documents"
    );
}
