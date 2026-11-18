//! M91 exit-criteria integration tests (§11.5, ADR-0095): the AT's
//! horizontal vocabulary — ADR-0093's second re-open rule's words
//! pre-registered the cut: "a consumer needs the AT's *horizontal*
//! vocabulary — `ScrollLeft`/`ScrollRight` beside the unit scrolls —
//! → the action enum grows the pair, every variant gaining its
//! consuming widget in the same cut (the enum's own rule)." The AT
//! reader of a wide feed (a screen reader walking a two-axis table
//! column by column, the same real reader the M89 ask priced) has the
//! vertical unit scrolls and only them: the horizontal ask rides no
//! action at all. The enum grows the pair —
//! [`A11yAction::ScrollLeft`]/[`A11yAction::ScrollRight`], the same
//! unit contract the vertical pair travels by (an item is one line, a
//! page is the viewport — the viewport's *width* on the x axis) — and
//! the scroll area consumes them by the same clamp arithmetic every
//! door answers by, the record's action species naming the ask and the
//! horizontal fact carrying its story beside the vertical identity
//! (ADR-0056's axis rule).
//!
//! Every test runs the real pipeline (A4): real layout, real compose
//! translation, the records through the harness trail door, the AT's
//! asks through the a11y action route — the same route the vertical
//! vocabulary rides, the pixels through the suite's probes.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::inspect::{
    InspectEvent, ScrollAction, ScrollOutcome, ScrollRecord, ScrollRefusal,
};
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{CrossAxisAlignment, Size2, Style};
use alacer_core::widget::{A11yAction, LabelSource, Role, ScrollUnit, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, List};

/// The probe at the top of the list's content — the pixel the
/// vertical vocabulary's identity reads.
const TOP_COLOR: Color = Color::rgb(37, 99, 235);

/// The wide probe's left tone — the pixel the x landing reads before
/// the ask moves it.
const LEFT_COLOR: Color = Color::rgb(255, 94, 58);

/// The wide probe's right tone — the pixel the x landing reads once
/// the ask lands. One probe, two tones: the boundary between them is
/// the offset's own ruler on the x axis.
const RIGHT_COLOR: Color = Color::rgb(148, 163, 184);

/// The stack's geometry: a 64 px top probe, the 192 px wide probe,
/// ten 64 px fillers — 896 px of vertical content inside a 200 px
/// viewport, all of it 300 px wide inside the same 200 px viewport
/// width. The x span is 100 px, the y span 696 px; the vertical
/// offset never moves this suite, so every vertical field the beats
/// read rides the identity answer.
const TOP_HEIGHT: f32 = 64.0;
const WIDE_HEIGHT: f32 = 192.0;
const FILLER_HEIGHT: f32 = 64.0;
const FILLERS: usize = 10;
const VIEWPORT: f32 = 200.0;
const PROBE_WIDTH: f32 = 300.0;

/// The wide probe's tones split at 60 px — inside the band the AT's
/// x asks cross: screen x 10 reads content x 10 before the ask (left
/// of the split) and content x 110 after a 100 px ask (right of it).
const TONE_SPLIT: f32 = 60.0;

/// The suite's corpus pin: the coordinated rule's count at the M91
/// cut — the handle door's case (M90) beside the AT horizontal
/// actions' case this cut publishes.
const EXPECTED_BENCH_COUNT: usize = 114;

/// The suite's own read door into the published corpus: the id the
/// AT horizontal actions' bench case publishes (the coordinated
/// rule's bench half, ADR-0095's exit criteria).
const HORIZONTAL_ACTIONS_CASE: &str = "a11y_action/route_100_horizontal_actions_1k";

/// One AT line of scroll, in pixels — the platform's wheel convention
/// (§10.2), the unit contract the vertical pair travels by and the
/// horizontal pair now travels by too.
const A11Y_LINE: f32 = 40.0;

/// A probe painting its bounds one tone left of the split and another
/// right of it — the offset's pixel evidence on the x axis (the
/// m-suite's probe shape, riding the axis this suite's asks move).
struct WideProbe {
    left: Color,
    right: Color,
    name: &'static str,
    height: f32,
    width: f32,
    split: f32,
}

impl WideProbe {
    fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Widget for WideProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Decoration
    }
    fn label(&self) -> LabelSource {
        LabelSource::text(self.name)
    }
    fn style(&self) -> Style {
        Style {
            size: Size2 {
                width: Some(self.width),
                height: Some(self.height),
            },
            ..Style::default()
        }
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        // The left tone: everything left of the split. The right tone:
        // everything from the split on. One probe, two rulers.
        let size = ctx.size();
        let split = self.split.min(size.width);
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), Size::new(split, size.height)),
            color: self.left,
            corner_radius: 0.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
        ctx.push_quad(Quad {
            rect: Rect::new(
                Point::new(split, 0.0),
                Size::new(size.width - split, size.height),
            ),
            color: self.right,
            corner_radius: 0.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// A scrollable list with the probe stack inside it, mounted through
/// the real pipeline — the handle kept beside the node ids so the
/// beats can read the offsets and the spans they declared.
struct Mounted {
    handle: alacer_widgets::ListHandle,
}

fn mount_probes(harness: &mut TestHarness, two_axis: bool, content_width: Option<f32>) -> Mounted {
    let mut root_column = Column::new();
    root_column.style_mut().align = CrossAxisAlignment::Start;
    let root = harness.mount(root_column.boxed());
    let mut list = if two_axis {
        List::two_axis(VIEWPORT, VIEWPORT)
    } else {
        List::new(VIEWPORT)
    };
    list.style_mut().align = CrossAxisAlignment::Start;
    list.style_mut().size.width = Some(VIEWPORT);
    let handle = list.handle();
    let list_node = harness.add_child(root, list.boxed());
    let _top = harness.add_child(
        list_node,
        WideProbe {
            left: TOP_COLOR,
            right: TOP_COLOR,
            name: "top probe",
            height: TOP_HEIGHT,
            width: PROBE_WIDTH,
            split: TONE_SPLIT,
        }
        .boxed(),
    );
    let _wide = harness.add_child(
        list_node,
        WideProbe {
            left: LEFT_COLOR,
            right: RIGHT_COLOR,
            name: "wide probe",
            height: WIDE_HEIGHT,
            width: PROBE_WIDTH,
            split: TONE_SPLIT,
        }
        .boxed(),
    );
    for index in 0..FILLERS {
        let _ = harness.add_child(
            list_node,
            WideProbe {
                left: RIGHT_COLOR,
                right: RIGHT_COLOR,
                name: "filler probe",
                height: FILLER_HEIGHT,
                width: PROBE_WIDTH,
                split: TONE_SPLIT,
            }
            .boxed(),
        );
        let _ = index;
    }
    let content_height =
        TOP_HEIGHT + WIDE_HEIGHT + f32::from(u16::try_from(FILLERS).expect("fits")) * FILLER_HEIGHT;
    handle.set_content_height(content_height);
    if let Some(width) = content_width {
        handle.set_content_width(width);
    }
    harness.settle();
    Mounted { handle }
}

/// The trail's whole window, drained once — the events the beats
/// partition into their tiers (a drain clears whole: a fact streams
/// once, and no beat may re-drain another beat's window).
fn drain_scroll_records(trail: &alacer_core::trail::TrailLog) -> Vec<ScrollRecord> {
    trail
        .drain()
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Scroll(record) => Some(*record),
            _ => None,
        })
        .collect()
}

/// The pixel at a screen slot, as RGBA bytes — the offset's own
/// evidence, read from the shipped software raster.
fn probe_pixel(harness: &mut TestHarness, x: u32, y: u32) -> [u8; 4] {
    let pixmap = harness.pixels();
    let px = pixmap
        .pixel(x, y)
        .unwrap_or_else(|| panic!("pixel ({x}, {y}) outside the viewport"));
    [px.red(), px.green(), px.blue(), px.alpha()]
}

/// The pixel an opaque probe paints, as the probe paints it —
/// premultiplied of opaque is the color's own bytes.
fn painted(color: Color) -> [u8; 4] {
    [color.r, color.g, color.b, 0xFF]
}

/// The AT's horizontal ask lands by its unit: an item is one line (40
/// px), a page is the viewport's width — the same unit contract the
/// vertical pair travels by, spoken on the x axis — the record's
/// species naming the ask, the horizontal fact carrying its story
/// beside the vertical identity, and the pixels flipping where the
/// shipped renderer paints them (the a11y route's PAINT | A11Y merge
/// composing the landing).
#[test]
fn m91_the_at_s_horizontal_ask_lands_by_its_unit() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, true, Some(PROBE_WIDTH));
    let trail = harness.trail();

    // The pixel the beat reads: (10, 128) sits on the wide probe's
    // left tone (content x 10 is left of the 60 split) before the ask.
    assert_eq!(
        probe_pixel(&mut harness, 10, 128),
        painted(LEFT_COLOR),
        "the wide probe's left tone sits at screen (10, 128) before the ask"
    );

    // One item rightward: one line = 40 px, the wheel's own
    // convention — "an AT's scroll one item lands where one wheel
    // notch does", on the x axis too.
    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollRight(ScrollUnit::Item)
        ),
        "the scroll area consumes the AT's ScrollRight ask"
    );
    assert_eq!(
        mounted.handle.offset_x(),
        A11Y_LINE,
        "one item = one line = 40 px, the unit contract on the x axis"
    );
    assert_eq!(
        mounted.handle.offset(),
        0.0,
        "the vertical offset holds — the horizontal ask never touches it"
    );
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 1, "one AT ask, one scroll fact");
    let record = records[0];
    assert_eq!(
        record.action,
        ScrollAction::ScrollRight,
        "the AT's ask names its own species on the record"
    );
    assert_eq!(
        record.unit,
        Some(ScrollUnit::Item),
        "the ask's unit rides the record, attributed the same way the vertical pair's is"
    );
    let horizontal = record.horizontal.expect("the x half's story rides");
    assert_eq!(
        horizontal.outcome,
        ScrollOutcome::Delivered,
        "the x half's clamp delivered the ask"
    );
    assert_eq!(
        horizontal.offset, 40.0,
        "the landing beside the unit the ask traveled by"
    );
    assert_eq!(
        record.outcome,
        ScrollOutcome::Delivered,
        "the vertical fields ride the identity answer — the axis unasked"
    );
    assert_eq!(
        record.requested, None,
        "the vertical half was not this record's ask"
    );

    // One page rightward: the viewport's width (200 px) — the page
    // contract on the x axis, the landing clamped to the span's edge.
    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollRight(ScrollUnit::Page)
        ),
        "the scroll area consumes the AT's page ask"
    );
    assert_eq!(
        mounted.handle.offset_x(),
        100.0,
        "one page = the viewport's width, clamped at the declared span's edge (300 - 200)"
    );

    // The pixels flipped where the shipped renderer paints them: the
    // a11y route's PAINT | A11Y merge composed the landings.
    assert_eq!(
        probe_pixel(&mut harness, 10, 128),
        painted(RIGHT_COLOR),
        "the wide slot now reads the probe's right tone — the x half moved"
    );
}

/// The leftward ask walks back and the wall pins the past run: the
/// same named walls the every door answers by, spoken per axis — the
/// ask below the range's floor lands at `min`, the ask past the
/// ceiling at `max`, and the AT's ask riding beside the pin.
#[test]
fn m91_the_leftward_ask_walks_back_and_the_walls_pin() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, true, Some(PROBE_WIDTH));
    let trail = harness.trail();

    // Walk right to the wall: two page asks land at the span's edge.
    assert!(harness.act(
        Role::ScrollArea,
        "list",
        A11yAction::ScrollRight(ScrollUnit::Page)
    ));
    assert!(harness.act(
        Role::ScrollArea,
        "list",
        A11yAction::ScrollRight(ScrollUnit::Page)
    ));
    assert_eq!(
        mounted.handle.offset_x(),
        100.0,
        "the second page ask landed at the ceiling — the clamp held"
    );
    let records = drain_scroll_records(&trail);
    let pinned = records[1].horizontal.expect("the x half's story").pinned;
    assert_eq!(
        pinned,
        Some(alacer_core::RangeBound::Max),
        "the ceiling wall names itself beside the landing"
    );

    // One item leftward walks the offset back by the line convention.
    assert!(harness.act(
        Role::ScrollArea,
        "list",
        A11yAction::ScrollLeft(ScrollUnit::Item)
    ));
    assert_eq!(
        mounted.handle.offset_x(),
        60.0,
        "one item leftward = 40 px back from the wall (100 - 40)"
    );

    // A leftward run past the floor lands at zero, the wall named.
    assert!(harness.act(
        Role::ScrollArea,
        "list",
        A11yAction::ScrollLeft(ScrollUnit::Page)
    ));
    assert_eq!(
        mounted.handle.offset_x(),
        0.0,
        "the leftward page ask landed at the range's floor"
    );
    // Both asks ride the window: the item ask mid-range (no wall), the
    // page ask at the floor (the wall named).
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 2, "two leftward asks, two facts");
    assert_eq!(
        records[0].horizontal.expect("the x half's story").pinned,
        None,
        "the mid-range item ask pins no wall — the landing is the ask"
    );
    assert_eq!(
        records[1].horizontal.expect("the x half's story").pinned,
        Some(alacer_core::RangeBound::Min),
        "the floor wall names itself beside the landing"
    );
}

/// The vertical-only container refuses the AT's horizontal ask
/// honestly: the span was never declared, the wall is
/// `no_scrollable_content`, and the AT's ask rides beside the refusal
/// where the reader can read it — while the vertical vocabulary
/// beside it lands untouched (the M89 contract, re-pinned at the cut
/// that grows the pair).
#[test]
fn m91_the_vertical_only_container_refuses_the_horizontal_ask_honestly() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, false, None);
    let trail = harness.trail();

    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollRight(ScrollUnit::Item)
        ),
        "the scroll area still consumes the ask — the refusal is the answer, not a dropped route"
    );
    assert_eq!(
        mounted.handle.offset_x(),
        0.0,
        "the undeclared span has no scroll to deliver — the offset holds"
    );
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 1, "one AT ask, one fact — refused or landed");
    let record = records[0];
    let horizontal = record.horizontal.expect("the refusal rides the half");
    assert_eq!(
        horizontal.outcome,
        ScrollOutcome::Refused(ScrollRefusal::NoScrollableContent),
        "the empty span's wall names itself at the AT's horizontal ask"
    );
    assert_eq!(
        record.unit,
        Some(ScrollUnit::Item),
        "the ask's unit rides beside the refusal too"
    );

    // The vertical vocabulary beside it lands untouched: the same
    // container's y span scrolls by the same route (the M89 rule —
    // the pair grew beside the vertical vocabulary, never over it).
    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollDown(ScrollUnit::Item)
        ),
        "the vertical vocabulary still consumes its asks"
    );
    assert_eq!(
        mounted.handle.offset(),
        40.0,
        "the vertical ask landed by its unit on the declared height span"
    );
}

/// The vertical vocabulary rides untouched by the growth: the AT's
/// vertical asks never move the x half, the M89 suite's own beat
/// re-pinned at the cut that grows the pair beside them.
#[test]
fn m91_the_vertical_vocabulary_rides_untouched() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, true, Some(PROBE_WIDTH));
    let trail = harness.trail();

    assert!(harness.act(
        Role::ScrollArea,
        "list",
        A11yAction::ScrollDown(ScrollUnit::Page)
    ));
    assert!(harness.act(
        Role::ScrollArea,
        "list",
        A11yAction::ScrollUp(ScrollUnit::Item)
    ));
    assert_eq!(
        mounted.handle.offset_x(),
        0.0,
        "the vertical vocabulary never moves the x half"
    );
    let records = drain_scroll_records(&trail);
    for record in &records {
        assert_eq!(
            record.horizontal, None,
            "the vertical stories ride their vocabulary untouched — no x half, no guessed zero"
        );
    }
}

/// The route refuses what the tier does not speak: the AT's horizontal
/// ask routed at a widget outside the scroll tier stays refused and
/// silent — a question outside the tier is not a tier fact (the
/// ADR-0046 rule the scroll tier rides), and the value-tier widgets'
/// refusal arms grew the pair in the same cut (the enum's own rule).
#[test]
fn m91_the_route_refuses_the_pair_outside_the_scroll_tier() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let _mounted = mount_probes(&mut harness, true, Some(PROBE_WIDTH));

    // A slider is not a scroll area: the AT's horizontal ask stays
    // refused there — never munged into a value write. The slider
    // mounts beside the list, a name of its own (ADR-5's mandatory
    // name at the value tier), settled before the ask rides the route.
    let root = harness
        .pipeline()
        .arena()
        .root()
        .expect("the root is mounted");
    let _ = harness.add_child(
        root,
        alacer_widgets::Slider::new("brightness", 0.0, 100.0).boxed(),
    );
    harness.settle();

    assert!(
        !harness.act(
            Role::Slider,
            "brightness",
            A11yAction::ScrollRight(ScrollUnit::Item)
        ),
        "the value tier refuses the horizontal pair"
    );
}

/// The corpus grew by the coordinated rule: the bench case the AT's
/// horizontal pair prices joins the published corpus, its same-host
/// gate joins the contract set, and the M5 pin re-agrees at the cut —
/// the coordinated bench + contracts.json + baseline-regeneration
/// rule, reviewed here.
#[test]
fn m91_the_corpus_grew_by_the_coordinated_rule() {
    let _guard = alacer_core::flux::batch_test_lock();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let baselines = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("benchmarks/baselines/baselines.json"))
            .expect("the published baseline is readable"),
    )
    .expect("the published baseline is valid JSON");
    let contracts = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("benchmarks/contracts.json"))
            .expect("the contract set is readable"),
    )
    .expect("the contract set is valid JSON");

    // The published corpus carries the AT horizontal pair's case.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(HORIZONTAL_ACTIONS_CASE),
        "the AT horizontal pair's bench case is published ({HORIZONTAL_ACTIONS_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M91 cut"
    );

    // The contract set gates the case, same-host like its route-tier
    // siblings (absolute nanoseconds across silicon is noise theatre).
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    let gate = gates
        .iter()
        .find(|gate| gate["baseline"].as_str() == Some(HORIZONTAL_ACTIONS_CASE))
        .expect("the AT horizontal pair's case carries a regression gate");
    assert_eq!(
        gate["scope"].as_str(),
        Some("same-host"),
        "the gate is same-host scoped like the route tier's own"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 114;"),
        "the m5 corpus pin re-agrees at the M91 cut"
    );
}
