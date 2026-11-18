//! M77 exit-criteria integration tests (§7.2 pass 8, ADR-0083): the
//! setting seam grows the motion fact — ADR-0080's first pre-registered
//! re-open rule cashing ("a consumer needs more OS facts than dark
//! mode (reduced motion, high contrast, accent color) → the trait
//! grows the fact beside `dark_mode` (the same one dynamic call per
//! pass, priced by a real reader) — never a second seam per fact").
//! Since M77 every [`alacer_core::inspect::SettingRecord`] carries the
//! reduced-motion fact beside the dark-mode fact, the door's flip
//! check compares both facts, and [`alacer_core::scene::PaintCtx`]
//! resolves the fact beside the palette and dark — the vocabulary
//! consults it for the decisions no role answers (a spinner's spin, a
//! thumbnail's autoplay). The log answers "what does the vocabulary
//! consult now, and did the OS move the motion fact?" from the row
//! the flip already streamed.
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
use alacer_widgets::BoxedWidget as _;

/// The facts the suite flips: both OS facts, one seam — the trait
/// grew the motion fact beside `dark_mode`, never a second seam per
/// fact (ADR-0083).
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

/// A probe that paints its whole bounds from the resolved *motion*
/// fact — deterministic pixels, no palette or glyph dependency (the
/// m74 suite's fact probe, grown beside the dark fact): the pixel
/// evidence that the motion fact reached the paint pass.
struct MotionProbe;

impl Widget for MotionProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("motion probe")
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
            color: if ctx.reduced_motion() {
                alacer_core::Color::rgb(126, 46, 46)
            } else {
                alacer_core::Color::rgb(46, 126, 126)
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

/// The motion flip lands where the dark fact is still — the case the
/// M74 door's dark-only check swallowed: the door answered `false`
/// there, the fact never reached the vocabulary. Since M77 the flip
/// check compares *both* facts, the motion-only flip lands, the
/// broadcast repaints the tree, and the record names both facts beside
/// the reach (the lengths rule, the ids never riding).
#[test]
fn m77_the_motion_flip_lands_where_dark_is_still() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let probe = harness.mount(MotionProbe.boxed());
    settle(&mut harness);

    // The motion-only flip: dark stays light, the OS asks for reduced
    // motion. The door must answer `true` — the M74 check, dark alone,
    // would have answered `false` here.
    assert!(
        harness.pipeline_mut().set_settings(Box::new(FixedSettings {
            dark: false,
            reduced_motion: true,
            high_contrast: false,
        })),
        "the motion-only flip lands — the grown check compares both facts"
    );
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 1, "one fact per flip that landed");
    assert!(
        !facts[0].dark && facts[0].reduced_motion,
        "the record names both facts — dark beside the motion fact"
    );
    assert_eq!(
        facts[0].repainted, 1,
        "the broadcast's own node count: the single-probe tree's reach"
    );

    // The pixel evidence: the probe painted from the resolved fact.
    assert_eq!(
        center_pixel(&mut harness, probe),
        [126, 46, 46, 255],
        "the vocabulary consults the motion fact — the probe repainted reduced"
    );
}

/// The dark flip beside the motion fact lands the same way: one seam,
/// both facts carried, the record naming them beside each other.
#[test]
fn m77_the_record_names_both_facts_beside_each_other() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let _probe = harness.mount(MotionProbe.boxed());
    settle(&mut harness);

    harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: true,
        reduced_motion: true,
        high_contrast: false,
    }));
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 1, "one fact per flip that landed");
    assert!(
        facts[0].dark && facts[0].reduced_motion,
        "the record carries the dark fact beside the motion fact"
    );
    assert_eq!(facts[0].repainted, 1, "the reach rides beside both facts");
}

/// The no-op flip names nothing: a flip that lands where *both* facts
/// already are queues no record — the idle-frame property (A1's rule
/// at the setting tier) unchanged by the fact's growth. The canon
/// settings (light, no motion preference) over the default build stay
/// the no-op too: the canon answers the facts the default build
/// already resolves.
#[test]
fn m77_the_no_op_names_nothing() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let probe = harness.mount(MotionProbe.boxed());
    settle(&mut harness);

    assert!(harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: true,
        reduced_motion: true,
        high_contrast: false,
    })));
    settle(&mut harness);
    let counters = harness.counters();
    assert!(
        !harness.pipeline_mut().set_settings(Box::new(FixedSettings {
            dark: true,
            reduced_motion: true,
            high_contrast: false,
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
    // The pixel stays the fact's answer: the no-op repainted nothing.
    assert_eq!(
        center_pixel(&mut harness, probe),
        [126, 46, 46, 255],
        "the probe kept the reduced-motion paint"
    );
    // The canon settings over the reduced-motion fact are *not* the
    // no-op (the fact moves back), and the canon over the canon facts
    // after that is: the default build resolves light, no motion
    // preference.
    assert!(
        harness.pipeline_mut().set_settings(Box::new(CanonSettings)),
        "canon light beside no motion moves the motion fact back"
    );
    settle(&mut harness);
    assert!(
        !harness.pipeline_mut().set_settings(Box::new(CanonSettings)),
        "the canon facts where the canon facts already are: the no-op"
    );
    assert_eq!(
        setting_facts(&trail).len(),
        2,
        "the canon no-op streamed none"
    );
}

/// The wire names both facts: the setting event's flat record carries
/// `reduced_motion` beside `dark` and `repainted` — the additive field
/// the versioning rule admits at version 1, the ids never riding (the
/// payload rule's shape on the wire).
#[test]
fn m77_the_wire_names_the_motion_fact() {
    let record = SettingRecord {
        dark: false,
        reduced_motion: true,
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
        json["dark"], false,
        "the dark fact rides the wire as the stable, documented field"
    );
    assert_eq!(
        json["reduced_motion"], true,
        "the motion fact rides the wire beside the dark fact — the additive field"
    );
    assert_eq!(
        json["repainted"], 42,
        "the count rides the wire beside the facts — lengths, never ids"
    );
}

/// The observation is priced, not assumed: the disabled trail still
/// lands the motion flip — the facts computed where the broadcast
/// resolves, the records never queue without a listener (the m74
/// suite's own rule, the fact's growth riding it unchanged).
#[test]
fn m77_the_disabled_trail_still_lands() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let probe = harness.mount(MotionProbe.boxed());
    settle(&mut harness);
    assert!(
        harness.pipeline_mut().set_settings(Box::new(FixedSettings {
            dark: false,
            reduced_motion: true,
            high_contrast: false,
        })),
        "the motion flip lands without a listener"
    );
    settle(&mut harness);
    assert_eq!(
        center_pixel(&mut harness, probe),
        [126, 46, 46, 255],
        "the fact resolves regardless of the trail"
    );
    let report = harness.tick();
    assert!(
        report.is_idle(),
        "no listener, no extra paint work — the trail's absence costs nothing"
    );
}

/// The living documents re-agree at the M77 cut: the README's status
/// block names the milestone, the corpus count rides the fact's
/// pricing, and the architecture carries the decision.
#[test]
fn m77_the_living_documents_re_agree() {
    let readme = read_workspace_file("README.md");
    assert!(
        readme.contains("M0 through M111"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("All 128 criterion cases"),
        "the README quotes the baselines.json bench_count the fact tier grew"
    );
    let architecture = read_workspace_file("docs/architecture.md");
    assert!(
        architecture.contains("ADR-0083"),
        "the architecture carries the motion fact's decision"
    );
    assert!(
        architecture.contains("a hundred and sixteen decision records"),
        "the register grew the M77 row"
    );
    let baselines = serde_json::from_str::<serde_json::Value>(&read_workspace_file(
        "benchmarks/baselines/baselines.json",
    ))
    .expect("the baselines parse");
    assert_eq!(
        baselines["bench_count"], 128,
        "the published corpus carries the fact tier's pricing"
    );
}
