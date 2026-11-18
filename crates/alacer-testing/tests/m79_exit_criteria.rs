//! M79 exit-criteria integration tests (§7.2 pass 8, ADR-0085): the
//! setting seam grows the high-contrast fact — ADR-0083's first
//! pre-registered re-open rule cashing again ("a consumer needs more
//! OS facts than these two (high contrast, accent color) → the trait
//! grows the fact beside the two — never a second seam per fact; the
//! record and the wire grow the field beside the facts by this cut's
//! own shape"). Since M79 every [`alacer_core::inspect::SettingRecord`]
//! carries the high-contrast fact beside the dark and reduced-motion
//! facts, the door's flip check compares all three, and
//! [`alacer_core::scene::PaintCtx`] resolves the fact beside the
//! palette, dark, and motion — the vocabulary consults it for the
//! decisions no role answers (a hairline's weight, a focus ring's
//! width). The log answers "what does the vocabulary consult now, and
//! did the OS move the contrast fact?" from the row the flip already
//! streamed.
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

/// The facts the suite flips: all three OS facts, one seam — the trait
/// grew the contrast fact beside `dark_mode` and `reduced_motion`,
/// never a second seam per fact (ADR-0085).
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

/// A probe that paints its whole bounds from the resolved *contrast*
/// fact — deterministic pixels, no palette or glyph dependency (the
/// m77 suite's fact probe, grown beside the contrast fact): the pixel
/// evidence that the contrast fact reached the paint pass. The muted
/// ink the standard-contrast vocabulary paints, the black ink the
/// high-contrast decision raises.
struct ContrastProbe;

impl Widget for ContrastProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("contrast probe")
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
            color: if ctx.high_contrast() {
                alacer_core::Color::rgb(0, 0, 0)
            } else {
                alacer_core::Color::rgb(148, 148, 155)
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

/// The contrast flip lands where the other facts are still — the case
/// the M77 door's two-fact check would have swallowed: the door
/// answered `false` there, the fact never reached the vocabulary.
/// Since M79 the flip check compares all three facts, the
/// contrast-only flip lands, the broadcast repaints the tree, and the
/// record names all three facts beside the reach (the lengths rule,
/// the ids never riding).
#[test]
fn m79_the_contrast_flip_lands_where_the_facts_are_still() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let probe = harness.mount(ContrastProbe.boxed());
    settle(&mut harness);

    // The contrast-only flip: dark stays light, motion stays allowed,
    // the OS asks for high contrast. The door must answer `true` — the
    // M77 check, dark and motion alone, would have answered `false`
    // here.
    assert!(
        harness.pipeline_mut().set_settings(Box::new(FixedSettings {
            dark: false,
            reduced_motion: false,
            high_contrast: true,
        })),
        "the contrast-only flip lands — the grown check compares all three facts"
    );
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 1, "one fact per flip that landed");
    assert!(
        !facts[0].dark && !facts[0].reduced_motion && facts[0].high_contrast,
        "the record names all three facts — contrast beside the facts it sits beside"
    );
    assert_eq!(
        facts[0].repainted, 1,
        "the broadcast's own node count: the single-probe tree's reach"
    );

    // The pixel evidence: the probe painted from the resolved fact.
    assert_eq!(
        center_pixel(&mut harness, probe),
        [0, 0, 0, 255],
        "the vocabulary consults the contrast fact — the probe repainted raised"
    );
}

/// The full flip beside the contrast fact lands the same way: one
/// seam, all three facts carried, the record naming them beside each
/// other — never a second seam or event per fact.
#[test]
fn m79_the_record_names_all_three_facts_beside_each_other() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 140.0);
    let trail = harness.trail();
    let _probe = harness.mount(ContrastProbe.boxed());
    settle(&mut harness);

    harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: true,
        reduced_motion: true,
        high_contrast: true,
    }));
    settle(&mut harness);
    let facts = setting_facts(&trail);
    assert_eq!(facts.len(), 1, "one fact per flip that landed");
    assert!(
        facts[0].dark && facts[0].reduced_motion && facts[0].high_contrast,
        "the record carries the contrast fact beside the dark and motion facts"
    );
    assert_eq!(
        facts[0].repainted, 1,
        "the reach rides beside all three facts"
    );
}

/// The no-op flip names nothing: a flip that lands where *all three*
/// facts already are queues no record — the idle-frame property (A1's
/// rule at the setting tier) unchanged by the fact's growth. The canon
/// settings (light, motion allowed, standard contrast) over the
/// default build stay the no-op too: the canon answers the facts the
/// default build already resolves.
#[test]
fn m79_the_no_op_names_nothing() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let trail = harness.trail();
    let probe = harness.mount(ContrastProbe.boxed());
    settle(&mut harness);

    assert!(harness.pipeline_mut().set_settings(Box::new(FixedSettings {
        dark: true,
        reduced_motion: true,
        high_contrast: true,
    })));
    settle(&mut harness);
    let counters = harness.counters();
    assert!(
        !harness.pipeline_mut().set_settings(Box::new(FixedSettings {
            dark: true,
            reduced_motion: true,
            high_contrast: true,
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
        [0, 0, 0, 255],
        "the probe kept the raised-contrast paint"
    );
    // The canon settings over the raised facts are *not* the no-op
    // (the facts move back), and the canon over the canon facts after
    // that is: the default build resolves light, motion allowed,
    // standard contrast.
    assert!(
        harness.pipeline_mut().set_settings(Box::new(CanonSettings)),
        "the canon facts over the raised facts move all three back"
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

/// The wire names all three facts: the setting event's flat record
/// carries `high_contrast` beside `dark`, `reduced_motion`, and
/// `repainted` — the additive field the versioning rule admits at
/// version 1, the ids never riding (the payload rule's shape on the
/// wire).
#[test]
fn m79_the_wire_names_the_contrast_fact() {
    let record = SettingRecord {
        dark: false,
        reduced_motion: false,
        high_contrast: true,
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
        json["reduced_motion"], false,
        "the motion fact rides the wire beside the dark fact"
    );
    assert_eq!(
        json["high_contrast"], true,
        "the contrast fact rides the wire beside the facts — the additive field"
    );
    assert_eq!(
        json["repainted"], 42,
        "the count rides the wire beside the facts — lengths, never ids"
    );
}

/// The observation is priced, not assumed: the disabled trail still
/// lands the contrast flip — the facts computed where the broadcast
/// resolves, the records never queue without a listener (the m77
/// suite's own rule, the fact's growth riding it unchanged).
#[test]
fn m79_the_disabled_trail_still_lands() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(200.0, 120.0);
    let probe = harness.mount(ContrastProbe.boxed());
    settle(&mut harness);
    assert!(
        harness.pipeline_mut().set_settings(Box::new(FixedSettings {
            dark: false,
            reduced_motion: false,
            high_contrast: true,
        })),
        "the contrast flip lands without a listener"
    );
    settle(&mut harness);
    assert_eq!(
        center_pixel(&mut harness, probe),
        [0, 0, 0, 255],
        "the fact resolves regardless of the trail"
    );
    let report = harness.tick();
    assert!(
        report.is_idle(),
        "no listener, no extra paint work — the trail's absence costs nothing"
    );
}

/// The living documents re-agree at the M79 cut: the README's status
/// block names the milestone, the corpus count rides the fact's
/// pricing, the architecture carries the decision, and the protocol
/// document pins the field the wire grew.
#[test]
fn m79_the_living_documents_re_agree() {
    let readme = read_workspace_file("README.md");
    assert!(
        readme.contains("M0 through M115"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("All 134 criterion cases"),
        "the README quotes the baselines.json bench_count the fact tier grew"
    );
    let architecture = read_workspace_file("docs/architecture.md");
    assert!(
        architecture.contains("ADR-0084"),
        "the architecture carries the counted swap's decision"
    );
    assert!(
        architecture.contains("ADR-0085"),
        "the architecture carries the contrast fact's decision"
    );
    assert!(
        architecture.contains("a hundred and twenty decision records"),
        "the register grew the M79 row"
    );
    let baselines = serde_json::from_str::<serde_json::Value>(&read_workspace_file(
        "benchmarks/baselines/baselines.json",
    ))
    .expect("the baselines parse");
    assert_eq!(
        baselines["bench_count"], 134,
        "the published corpus carries the fact tier's pricing"
    );
    assert!(
        baselines["benches"]
            .get("settings/door_flip_contrast")
            .is_some(),
        "the published baseline carries the contrast flip's pricing (the coordinated rule, ADR-0011)"
    );
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("high_contrast"),
        "the protocol document pins the field the wire grew"
    );
    // The sibling suites' pins ride the same cut (one sweep, no drift):
    // the m78 suite's documents pin moved to M79's cut.
    let m78 = read_workspace_file("crates/alacer-testing/tests/m78_exit_criteria.rs");
    assert!(
        m78.contains("M0 through M115"),
        "the m78 suite's pins moved with the documents"
    );
}
