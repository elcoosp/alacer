//! M76 exit-criteria integration tests (§7.2 pass 8, ADR-0082): the
//! flip names its repaint count — ADR-0080's second pre-registered
//! re-open rule cashing ("a consumer prices *which nodes* the flip
//! repainted → the record grows the broadcast's node count beside the
//! fact (the lengths rule — the ids never ride), priced by a real
//! reader"). Since M76 every [`alacer_core::inspect::SettingRecord`]
//! carries the broadcast's node count beside the dark-mode fact: the
//! log answers "what does the vocabulary consult now, and how much did
//! the flip repaint?" from one row, the ids never riding (the payload
//! rule's shape — lengths, never identities).
//!
//! Every test runs the real pipeline (A4): real layout, real paint
//! rasterization through the harness, the records through the harness
//! trail door.

use std::path::PathBuf;

use alacer_core::geom::{Point, Rect};
use alacer_core::inspect::{InspectEvent, SettingRecord};
use alacer_core::passes::FrameReport;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::settings::{CanonSettings, SettingSeam};
use alacer_core::style::{Size2, Style};
use alacer_core::trail::TrailLog;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column};

/// The fact the suite flips to: `dark` (the m74 suite's own shape).
struct FixedSettings {
    dark: bool,
}

impl SettingSeam for FixedSettings {
    fn dark_mode(&self) -> bool {
        self.dark
    }

    /// The m76 suite flips the dark fact alone — the canon absence the
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

/// A probe that paints its whole bounds from the resolved *fact* —
/// deterministic pixels, no palette or glyph dependency (the m74
/// suite's fact probe).
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
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), ctx.size()),
            color: if ctx.dark() {
                alacer_core::Color::rgb(20, 22, 28)
            } else {
                alacer_core::Color::rgb(240, 241, 244)
            },
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

/// The flip names the broadcast's node count beside the fact: a flip
/// over a tree of N nodes streams `repainted == N` — the lengths rule
/// (the count is the reach, the ids never ride), the count computed
/// where the broadcast resolves (`PassPipeline::set_settings`'s own
/// walk, never re-counted at the reader).
#[test]
fn m76_the_flip_names_its_repaint_count() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let _a = harness.add_child(root, FactProbe.boxed());
    let _b = harness.add_child(root, FactProbe.boxed());
    let _c = harness.add_child(root, FactProbe.boxed());
    settle(&mut harness);

    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 1, "one fact per flip that landed");
    assert!(facts[0].dark, "the record names the fact it landed");
    assert_eq!(
        facts[0].repainted, 4,
        "root + three probes: the broadcast's own node count, the ids never riding"
    );

    // The flip back names the same reach: the tree did not move.
    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: false }));
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 2, "one fact per flip, the flip back beside it");
    assert_eq!(
        facts[1].repainted, 4,
        "the flip back's record names the same broadcast reach"
    );
}

/// The count is the flip's own fact: the record carries the reach the
/// broadcast had at the flip's moment — a tree that grows later never
/// rewrites a queued record (the record is the fact of its own flip,
/// never a live window into the tree).
#[test]
fn m76_the_count_names_the_flips_own_moment() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let _probe = harness.add_child(root, FactProbe.boxed());
    settle(&mut harness);

    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    settle(&mut harness);

    // The tree grows after the flip: the queued record is unchanged.
    let _second = harness.add_child(root, FactProbe.boxed());
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 1, "the growth queued no setting fact");
    assert_eq!(
        facts[0].repainted, 2,
        "the record still names the flip's own reach — root + one probe"
    );

    // The next flip names the grown tree's reach.
    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: false }));
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 2, "the second flip carries its own record");
    assert_eq!(
        facts[1].repainted, 3,
        "root + two probes: the grown tree's broadcast reach"
    );
}

/// The no-op flip names nothing: a flip that lands where the fact
/// already is queues no record — there is no count to name, the
/// idle-frame property (A1's rule at the setting tier) unchanged by
/// the record's growth.
#[test]
fn m76_the_no_op_names_nothing() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let _probe = harness.mount(FactProbe.boxed());
    settle(&mut harness);

    assert!(
        harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: true }))
    );
    settle(&mut harness);
    let counters = harness.counters();
    assert!(
        !harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: true })),
        "the same fact is a no-op"
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
    // The flip back lands, and the canon light over the light fact is
    // the no-op too: the default build already resolves light.
    assert!(
        harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: false }))
    );
    settle(&mut harness);
    assert!(!harness.pipeline_mut().set_settings(Box::new(CanonSettings)));
    settle(&mut harness);
    assert_eq!(
        setting_facts(&trail).len(),
        2,
        "the canon no-op streamed none"
    );
}

/// The composition's swaps stream their own counts beside theirs
/// (ADR-0081's door riding the grown record): the composed flip's
/// `Theme` record stays the tag alone, the `Setting` record beside it
/// naming the same reach the flip broadcast.
#[test]
fn m76_the_composed_flip_names_one_count() {
    use alacer_core::settings::SettingComposer;
    use alacer_core::theme::Palette;

    /// A composer whose two answers are distinct palettes (the m75
    /// suite's duet shape, minimal).
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
                    surface: alacer_core::Color::rgb(16, 18, 24),
                    panel: alacer_core::Color::rgb(28, 31, 40),
                    accent: alacer_core::Color::rgb(255, 176, 64),
                    accent_pressed: alacer_core::Color::rgb(226, 152, 44),
                    selection: alacer_core::Color::rgb(255, 176, 64).with_alpha(72),
                    text: alacer_core::Color::rgb(226, 230, 238),
                    text_muted: alacer_core::Color::rgb(142, 150, 168),
                    border: alacer_core::Color::rgb(56, 62, 80),
                    on_accent: alacer_core::Color::rgb(24, 16, 6),
                }
            } else {
                Palette::CANON
            }
        }
        fn name(&self) -> &'static str {
            "duet"
        }
    }

    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let _probe = harness.mount(FactProbe.boxed());
    settle(&mut harness);
    harness.pipeline_mut().set_composer(Box::new(DuetComposer));
    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 1, "one flip, one record");
    assert_eq!(
        facts[0].repainted, 1,
        "the composed flip names the same broadcast the bare flip names"
    );
}

/// The wire names the count: the setting event's flat record carries
/// `repainted` beside `dark` — the additive field the versioning rule
/// admits at version 1, the ids never riding (the payload rule's
/// shape on the wire).
#[test]
fn m76_the_wire_names_the_count() {
    let record = SettingRecord {
        dark: true,
        reduced_motion: false,
        high_contrast: false,
        accent: None,
        repainted: 42,
    };
    let event = alacer_inspect::wire::wire_setting(&record);
    let json = serde_json::to_value(&event).expect("the wire event serializes");
    assert_eq!(
        json["kind"], "setting",
        "the setting event's kind is the stable, documented value"
    );
    assert_eq!(
        json["dark"], true,
        "the fact rides the wire as the stable, documented field"
    );
    assert_eq!(
        json["repainted"], 42,
        "the count rides the wire beside the fact — lengths, never ids"
    );
}

/// The demo drives the door from the consumer side: the dark-mode
/// demo's exit read-back prints the count beside the fact — the shape
/// pinned here by the same fields the demo logs.
#[test]
fn m76_the_demo_shape_drives_the_door() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let _probe = harness.mount(FactProbe.boxed());
    settle(&mut harness);
    harness
        .pipeline_mut()
        .set_settings(Box::new(FixedSettings { dark: true }));
    settle(&mut harness);
    harness.pipeline_mut().set_settings(Box::new(CanonSettings));
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 2, "the demo's log carries the two flips");
    assert!(
        facts[0].dark && !facts[1].dark,
        "the log reads dark → light"
    );
    assert_eq!(
        facts[0].repainted, facts[1].repainted,
        "the two flips broadcast the same tree — the counts agree"
    );
    assert!(
        facts.iter().all(|record| record.repainted == 1),
        "the demo's single-probe tree repaints one node per flip"
    );
}

/// The observation is priced, not assumed: the disabled trail still
/// flips — the count computed where the broadcast resolves, the
/// records never queue without a listener (the m74 suite's own rule).
#[test]
fn m76_the_disabled_trail_still_counts() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let probe = harness.mount(FactProbe.boxed());
    settle(&mut harness);
    let bounds = harness.bounds(probe);
    assert!(
        harness
            .pipeline_mut()
            .set_settings(Box::new(FixedSettings { dark: true }))
    );
    settle(&mut harness);
    let pixel = {
        let pixmap = harness.pixels();
        let x = alacer_core::num::f32_to_u32((bounds.origin.x + 32.0).max(0.0));
        let y = alacer_core::num::f32_to_u32((bounds.origin.y + 32.0).max(0.0));
        let px = pixmap
            .pixel(x, y)
            .unwrap_or_else(|| panic!("pixel ({x}, {y}) outside the viewport"));
        [px.red(), px.green(), px.blue(), px.alpha()]
    };
    assert_eq!(
        pixel,
        [20, 22, 28, 255],
        "the fact resolves regardless of the trail — the count rides beside it"
    );
    let report = harness.tick();
    assert!(
        report.is_idle(),
        "no listener, no extra paint work — the trail's absence costs nothing"
    );
}

/// The living documents re-agree at the M76 cut: the README's status
/// block names the milestone, the corpus count rides the count tier's
/// pricing, and the architecture carries the decision.
#[test]
fn m76_the_living_documents_re_agree() {
    let readme = read_workspace_file("README.md");
    assert!(
        readme.contains("M0 through M98"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("All 110 criterion cases"),
        "the README quotes the baselines.json bench_count the count tier grew"
    );
    let architecture = read_workspace_file("docs/architecture.md");
    assert!(
        architecture.contains("ADR-0082"),
        "the architecture carries the count's decision"
    );
    assert!(
        architecture.contains("a hundred and three decision records"),
        "the register grew the M76 row"
    );
    let baselines = serde_json::from_str::<serde_json::Value>(&read_workspace_file(
        "benchmarks/baselines/baselines.json",
    ))
    .expect("the baselines parse");
    assert_eq!(
        baselines["bench_count"], 110,
        "the published corpus carries the count tier's pricing"
    );
}
