//! M78 exit-criteria integration tests (§7.2 pass 8, ADR-0084): the
//! swap names its repaint count — ADR-0082's second pre-registered
//! re-open rule cashing on the theme tier ("a consumer needs the count
//! on other tiers' records → each tier's record grows its own lengths
//! beside its fact by its own coordinated cut, never a shared
//! side-channel"). Since M78 every [`alacer_core::inspect::ThemeRecord`]
//! carries the broadcast's node count beside the palette's tag: the log
//! answers "which palette is live now, when did it move, and how much
//! did the swap repaint?" from one row — the setting tier's own count
//! the M76 cut grew, the theme tier's lengths beside its fact, the ids
//! never riding (the payload rule's shape).
//!
//! Every test runs the real pipeline (A4): real layout, real paint
//! rasterization through the harness, the records through the harness
//! trail door.

use std::path::PathBuf;

use alacer_core::color::Color;
use alacer_core::inspect::{InspectEvent, ThemeRecord};
use alacer_core::passes::FrameReport;
use alacer_core::scene::PaintCtx;
use alacer_core::settings::{SettingComposer, SettingSeam};
use alacer_core::theme::{Palette, ThemeSeam};
use alacer_core::trail::TrailLog;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::BoxedWidget as _;

/// The palette the suite swaps to: every role distinct from the canon's,
/// so a palette-resolution regression cannot pass for the swap's own
/// fact (the m73 suite's midnight shape, minimal).
struct MidnightPalette;

impl ThemeSeam for MidnightPalette {
    fn palette(&self) -> Palette {
        Palette {
            surface: Color::rgb(12, 14, 20),
            panel: Color::rgb(22, 26, 36),
            accent: Color::rgb(240, 180, 41),
            accent_pressed: Color::rgb(214, 156, 22),
            selection: Color::rgb(240, 180, 41).with_alpha(72),
            text: Color::rgb(226, 230, 238),
            text_muted: Color::rgb(142, 150, 168),
            border: Color::rgb(56, 62, 80),
            on_accent: Color::rgb(24, 16, 6),
        }
    }

    fn name(&self) -> &'static str {
        "midnight"
    }
}

/// The composition the suite drives (the m75 suite's duet shape): the
/// canon palette for light, the midnight-dark beside it — so a flip's
/// composed swap resolves a palette that moved.
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

/// The facts the suite flips (the m77 suite's shape): the canon
/// absence for the motion fact — the dark flip is the suite's subject.
struct FixedSettings;

impl SettingSeam for FixedSettings {
    fn dark_mode(&self) -> bool {
        true
    }

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

/// A probe that paints its whole bounds from the resolved palette —
/// deterministic pixels, no glyph dependency: the pixel evidence that
/// the swap landed (the m73 suite's role probe, minimal).
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
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let _ = ctx.palette().accent;
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
fn setting_facts(trail: &TrailLog) -> Vec<alacer_core::inspect::SettingRecord> {
    trail
        .look()
        .into_iter()
        .filter_map(|event| match event {
            InspectEvent::Setting(record) => Some(record),
            _ => None,
        })
        .collect()
}

/// The swap names the broadcast's node count beside the tag: a swap
/// over a tree of N nodes streams `repainted == N` — the lengths rule
/// (the count is the reach, the ids never ride), the count computed
/// where the broadcast resolves (`PassPipeline::set_theme`'s own walk,
/// never re-counted at the reader), the theme tier's own coordinated
/// cut of the M76 rule.
#[test]
fn m78_the_swap_names_its_repaint_count() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let root = harness.mount(RoleProbe.boxed());
    let _a = harness.add_child(root, RoleProbe.boxed());
    let _b = harness.add_child(root, RoleProbe.boxed());
    let _c = harness.add_child(root, RoleProbe.boxed());
    settle(&mut harness);

    assert!(
        harness.pipeline_mut().set_theme(Box::new(MidnightPalette)),
        "the swap lands over the canon default"
    );
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(swaps.len(), 1, "one fact per swap that landed");
    assert_eq!(
        swaps[0].palette, "midnight",
        "the record names the palette it installed"
    );
    assert_eq!(
        swaps[0].repainted, 4,
        "root + three probes: the broadcast's own node count, the ids never riding"
    );

    // The swap back names the same reach: the tree did not move.
    assert!(harness.pipeline_mut().set_theme(Box::new(CanonTheme)));
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(swaps.len(), 2, "one fact per swap, the swap back beside it");
    assert_eq!(
        swaps[1].repainted, 4,
        "the swap back's record names the same broadcast reach"
    );
}

/// The provided canon theme, installable by name (the m73 suite's
/// swap-back shape).
struct CanonTheme;

impl ThemeSeam for CanonTheme {
    fn palette(&self) -> Palette {
        Palette::CANON
    }

    fn name(&self) -> &'static str {
        "canon"
    }
}

/// The count is the swap's own fact: the record carries the reach the
/// broadcast had at the swap's moment — a tree that grows later never
/// rewrites a queued record (the M76 rule at the theme tier).
#[test]
fn m78_the_count_names_the_swaps_own_moment() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let root = harness.mount(RoleProbe.boxed());
    let _probe = harness.add_child(root, RoleProbe.boxed());
    settle(&mut harness);

    harness.pipeline_mut().set_theme(Box::new(MidnightPalette));
    settle(&mut harness);

    // The tree grows after the swap: the queued record is unchanged.
    let _second = harness.add_child(root, RoleProbe.boxed());
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(swaps.len(), 1, "the growth queued no theme fact");
    assert_eq!(
        swaps[0].repainted, 2,
        "the record still names the swap's own reach — root + one probe"
    );

    // The next swap names the grown tree's reach.
    harness.pipeline_mut().set_theme(Box::new(CanonTheme));
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(swaps.len(), 2, "the second swap carries its own record");
    assert_eq!(
        swaps[1].repainted, 3,
        "root + two probes: the grown tree's broadcast reach"
    );
}

/// The no-op swap names nothing: a swap that lands where the palette
/// already is queues no record — there is no count to name, the
/// idle-frame property (A1's rule at the theme tier) unchanged by the
/// record's growth.
#[test]
fn m78_the_no_op_streams_none() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let _probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);

    assert!(harness.pipeline_mut().set_theme(Box::new(MidnightPalette)));
    settle(&mut harness);
    let counters = harness.counters();
    assert!(
        !harness.pipeline_mut().set_theme(Box::new(MidnightPalette)),
        "the same palette is a no-op"
    );
    let report = harness.tick();
    assert!(
        report.is_idle(),
        "the no-op swap queues no masks — the idle frame, A1's rule"
    );
    settle(&mut harness);
    assert_eq!(
        harness.counters().paint_runs,
        counters.paint_runs,
        "the no-op swap paints nothing"
    );
    assert_eq!(theme_facts(&trail).len(), 1, "the no-op streamed no fact");
}

/// One flip, one count: the composed swap's record names the same
/// reach the flip's own broadcast resolved — the composed `Theme`
/// record riding the flip's count (ADR-0082's companion rule: "one
/// flip, one count, however many palettes the composition resolved"),
/// never a second walk's length beside it.
#[test]
fn m78_the_composed_swap_names_the_flips_own_count() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let _probe = harness.mount(RoleProbe.boxed());
    settle(&mut harness);
    harness.pipeline_mut().set_composer(Box::new(DuetComposer));
    settle(&mut harness);
    let swaps_before = theme_facts(&trail).len();

    // The flip lands: the composed dark palette mounts beside the
    // fact — one broadcast, one count, two records.
    harness.pipeline_mut().set_settings(Box::new(FixedSettings));
    settle(&mut harness);
    let swaps = theme_facts(&trail);
    assert_eq!(
        swaps.len(),
        swaps_before + 1,
        "the composed swap streamed one theme fact"
    );
    let composed = swaps.last().expect("the composed swap's record");
    assert_eq!(
        composed.palette, "duet",
        "the composed swap names the composer's tag"
    );
    assert_eq!(
        composed.repainted, 1,
        "the composed swap names the flip's own broadcast reach — one flip, one count"
    );
    let facts = setting_facts(&trail);
    assert_eq!(
        facts.last().expect("the flip's record").repainted,
        composed.repainted,
        "the setting and theme records name the same broadcast — one flip, one count"
    );
}

/// The wire names the count: the theme event's flat record carries
/// `repainted` beside `palette` — the additive field the versioning
/// rule admits at version 1, the colors never riding (the payload
/// rule's shape on the wire).
#[test]
fn m78_the_wire_names_the_count() {
    let record = ThemeRecord {
        palette: "midnight".to_string(),
        repainted: 42,
    };
    let event = alacer_inspect::wire::wire_theme(&record);
    let json = serde_json::to_value(&event).expect("the wire event serializes");
    assert_eq!(
        json["kind"], "theme",
        "the theme event's kind is the stable, documented value"
    );
    assert_eq!(
        json["palette"], "midnight",
        "the tag rides the wire as the stable, documented field"
    );
    assert_eq!(
        json["repainted"], 42,
        "the count rides the wire beside the tag — lengths, never ids, never colors"
    );
}

/// The living documents re-agree at the M78 cut: the README's status
/// block names the milestone, the corpus count rides the count tier's
/// pricing, and the architecture carries the decision.
#[test]
fn m78_the_living_documents_re_agree() {
    let readme = read_workspace_file("README.md");
    assert!(
        readme.contains("M0 through M115"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("All 134 criterion cases"),
        "the README quotes the baselines.json bench_count the count tier grew"
    );
    let architecture = read_workspace_file("docs/architecture.md");
    assert!(
        architecture.contains("ADR-0083"),
        "the architecture carries the motion fact's decision"
    );
    assert!(
        architecture.contains("ADR-0084"),
        "the architecture carries the counted swap's decision"
    );
    assert!(
        architecture.contains("a hundred and twenty decision records"),
        "the register grew the M77 and M78 rows"
    );
    let baselines = serde_json::from_str::<serde_json::Value>(&read_workspace_file(
        "benchmarks/baselines/baselines.json",
    ))
    .expect("the baselines parse");
    assert_eq!(
        baselines["bench_count"], 134,
        "the published corpus carries the count tier's pricing"
    );
}
