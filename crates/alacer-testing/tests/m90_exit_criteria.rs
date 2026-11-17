//! M90 exit-criteria integration tests (§13.3, ADR-0094): the app's own
//! handle grows the x half — ADR-0093's third re-open rule's words
//! pre-registered the cut: "a consumer needs the app's own two-axis
//! handle door — `scroll_to_x` beside `scroll_to` — → the handle grows
//! the x half with the same named-wall record, priced by a real reader
//! first." Everything two-axis shipped at M89 but the app door: the
//! wheel runs the x half, the bring-into-view door prices both halves,
//! the content door declares the width — and the app's own programmatic
//! ask still has only the vertical scalar. The handle grows the x half:
//! the same clamp arithmetic ([`alacer_widgets`]’ `answer_axis_ask` on
//! the declared width span), the same named walls, the same record —
//! the x story riding the record's horizontal half (ADR-0056's axis
//! rule) while the vertical fields ride the identity answer (the axis
//! unasked holds the offset it had — the same convention the M89
//! content door answers by).
//!
//! Every test runs the real pipeline (A4): real layout, real compose
//! translation, the records through the harness trail door, the pixels
//! through the suite's probes — one probe, two tones, the boundary
//! between them the x offset's own ruler.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::inspect::{
    InspectEvent, ScrollAction, ScrollOutcome, ScrollRecord, ScrollRefusal,
};
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{CrossAxisAlignment, Size2, Style};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};

/// The probe at the top of the list's content — the pixel the
/// vertical identity reads.
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

/// The wide probe's tones split at 60 px — inside the band the x
/// ask's landing crosses: screen x 10 reads content x 10 before the
/// ask (left of the split) and content x 110 after it (right of it).
const TONE_SPLIT: f32 = 60.0;

/// The suite's beat count never pins the corpus here — that is the
/// coordinated-rule test's own pin below.
const EXPECTED_BENCH_COUNT: usize = 97;

/// The suite's own read door into the published corpus: the id the
/// handle door's bench case publishes (the coordinated rule's bench
/// half, ADR-0094's exit criteria).
const HANDLE_X_CASE: &str = "scroll/route_100_handle_scroll_x_1k";

/// A probe painting its bounds one tone left of the split and another
/// right of it — the offset's pixel evidence on the x axis (the
/// m-suite's probe shape, riding the axis this suite's door moves).
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
/// beats can read the offsets and the spans they declared. The wide
/// probe straddles the viewport's right edge (300 wide inside 200),
/// so the x door exists to move it.
struct Mounted {
    list_node: alacer_core::NodeId,
    wide: alacer_core::NodeId,
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
    let wide = harness.add_child(
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
    Mounted {
        list_node,
        wide,
        handle,
    }
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

/// The door lands the x ask: the offset moves, the record's horizontal
/// half carries the ask's whole story beside the vertical identity —
/// the record half of the door's contract (the M50 suite's own shape
/// for the handle doors: the bare handle routes no event and asks no
/// frame; the visible-motion half is the composition beat below).
#[test]
fn m90_the_handle_door_lands_the_x_ask() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, true, Some(PROBE_WIDTH));
    let trail = harness.trail();

    // The ask: the handle's x door clamps against the declared width
    // span (300 - 200 = 100) and lands the ask mid-range, the wall
    // unpinned (the landing is the ask).
    mounted.handle.scroll_to_x(100.0);
    harness.settle();

    assert_eq!(
        mounted.handle.offset_x(),
        100.0,
        "the container holds the landing the handle's x door answered"
    );
    assert_eq!(
        mounted.handle.offset(),
        0.0,
        "the vertical offset holds where it was — the axis unasked rides the identity"
    );

    // The record: one `scroll_to` fact, the vertical fields riding the
    // identity answer (the axis unasked holds the offset it had — the
    // M89 content door's own convention), the horizontal half beside
    // it carrying the ask's whole story.
    let records = drain_scroll_records(&trail);
    assert_eq!(
        records.len(),
        1,
        "one handle ask queues exactly one scroll fact"
    );
    let record = records[0];
    assert_eq!(
        record.action,
        ScrollAction::ScrollTo,
        "the app door's species"
    );
    assert_eq!(
        record.outcome,
        ScrollOutcome::Delivered,
        "the vertical fields ride the identity answer: delivered, unmoved"
    );
    assert_eq!(
        record.offset, 0.0,
        "the vertical offset the record resolves with is the held one"
    );
    assert_eq!(
        record.requested, None,
        "the vertical half was not this record's ask — no ask beside the identity"
    );
    assert_eq!(
        record.pinned, None,
        "the identity answer pins no wall on the vertical axis"
    );
    let horizontal = record
        .horizontal
        .expect("the x door's story rides the half");
    assert_eq!(
        horizontal.outcome,
        ScrollOutcome::Delivered,
        "the x half's clamp delivered the ask"
    );
    assert_eq!(
        horizontal.offset, 100.0,
        "the x half landed the ask mid-range"
    );
    assert_eq!(
        horizontal.requested,
        Some(100.0),
        "the finite ask rides the fact beside the landing"
    );
    assert_eq!(
        horizontal.pinned, None,
        "a mid-range x landing pins no wall — the landing is the ask"
    );
}

/// The x landing composes at the app's own seam: the mutate-guard
/// `request` seam is where a programmatic scroll surfaces its PAINT
/// mask (the M23 shape, ADR-0093's call-site composition rule), so
/// the frame the app asks for carries the x half's translation — the
/// bounds ledger states the landing and the pixels flip where the
/// shipped renderer paints them.
#[test]
fn m90_the_x_landing_composes_at_the_app_seam() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, true, Some(PROBE_WIDTH));

    // The pixel the beat reads: (10, 128) sits on the wide probe's
    // left tone (content x 10 is left of the 60 split) before the ask.
    assert_eq!(
        probe_pixel(&mut harness, 10, 128),
        painted(LEFT_COLOR),
        "the wide probe's left tone sits at screen (10, 128) before the ask"
    );

    // The ask composed: the handle's x door inside the mutate guard —
    // the list node mutated directly, the PAINT | A11Y merge the
    // app's own frame request carries (the M23 shape), the seam every
    // programmatic scroll surfaces through.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(mounted.list_node);
    let list = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<List>()
        .expect("the mutated node is the list");
    list.handle().scroll_to_x(100.0);
    guard.request(alacer_core::mask::Change::PAINT.union(alacer_core::mask::Change::A11Y));
    drop(guard);
    harness.settle();

    assert_eq!(
        mounted.handle.offset_x(),
        100.0,
        "the container holds the landing the composed ask answered"
    );
    let wide_bounds = harness.bounds(mounted.wide);
    assert_eq!(
        wide_bounds.origin.x, -100.0,
        "the bounds ledger states the x landing (0 - 100)"
    );
    assert_eq!(
        wide_bounds.origin.y, 64.0,
        "the vertical bounds hold — the axis unasked rides the identity"
    );
    assert_eq!(
        probe_pixel(&mut harness, 10, 128),
        painted(RIGHT_COLOR),
        "the wide slot now reads the probe's right tone — the x half moved"
    );
}

/// The wall pins a past run: the ask past the range's edge lands at
/// the wall, the pin names which one, and the ask that wanted the
/// wall exactly pins nothing.
#[test]
fn m90_the_wall_pins_the_past_run() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, true, Some(PROBE_WIDTH));
    let trail = harness.trail();

    // A negative ask: the clamp lands it at the range's minimum, the
    // wall that held the no-op named (the M50 sentence at the x half).
    mounted.handle.scroll_to_x(-40.0);
    harness.settle();
    assert_eq!(
        mounted.handle.offset_x(),
        0.0,
        "the negative ask landed at the range's minimum"
    );
    let records = drain_scroll_records(&trail);
    let pinned = records[0].horizontal.expect("the x door's story").pinned;
    assert_eq!(
        pinned,
        Some(alacer_core::RangeBound::Min),
        "the wall that held a no-op against a past ask names itself"
    );

    // A past-the-end ask: the clamp lands it at the maximum, the wall
    // named again — the same arithmetic the vertical door answers by.
    mounted.handle.scroll_to_x(500.0);
    harness.settle();
    assert_eq!(
        mounted.handle.offset_x(),
        100.0,
        "the past-the-end ask landed at the range's maximum (300 - 200)"
    );
    let records = drain_scroll_records(&trail);
    let pinned = records[0].horizontal.expect("the x door's story").pinned;
    assert_eq!(
        pinned,
        Some(alacer_core::RangeBound::Max),
        "the maximum wall names itself beside the landing"
    );

    // The ask that wanted the wall exactly: the landing *is* the ask,
    // no wall held a no-op against it, the pin rides its honest
    // absence (the M50 rule, spoken identically on the x axis).
    mounted.handle.scroll_to_x(100.0);
    harness.settle();
    assert_eq!(
        mounted.handle.offset_x(),
        100.0,
        "the wall ask holds the wall the past run landed at"
    );
    let records = drain_scroll_records(&trail);
    let pinned = records[0].horizontal.expect("the x door's story").pinned;
    assert_eq!(
        pinned, None,
        "the ask that wanted the wall exactly pins nothing"
    );
}

/// The non-finite ask refuses before the arithmetic: the corruption
/// the clamp would have spread into the state named at the door, the
/// offset keeping what it held, the ask riding the absence a
/// non-finite scalar leaves behind.
#[test]
fn m90_the_non_finite_ask_refuses_before_the_arithmetic() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, true, Some(PROBE_WIDTH));
    let trail = harness.trail();

    mounted.handle.scroll_to_x(f32::NAN);
    harness.settle();
    assert_eq!(
        mounted.handle.offset_x(),
        0.0,
        "the non-finite ask moved nothing — the state keeps what it held"
    );
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 1, "the refusal streams: one fact, one ask");
    let record = records[0];
    let horizontal = record
        .horizontal
        .expect("the x door's story rides the half");
    assert_eq!(
        horizontal.outcome,
        ScrollOutcome::Refused(ScrollRefusal::NonFiniteAsk),
        "the non-finite ask is named at the x door before the arithmetic"
    );
    assert_eq!(
        horizontal.requested, None,
        "a non-finite scalar prices no ask beside the refusal"
    );
    assert_eq!(
        horizontal.offset, 0.0,
        "the offset the refusal resolves with is the held one"
    );
    assert_eq!(
        record.outcome,
        ScrollOutcome::Delivered,
        "the vertical fields ride the identity answer — the axis unasked"
    );
}

/// The vertical-only container refuses the x ask honestly: the span
/// was never declared, the wall is `no_scrollable_content`, and the
/// app's ask rides beside the refusal where the consumer can read it
/// (the M89 wall, met at the handle's own door).
#[test]
fn m90_the_vertical_only_container_refuses_the_x_ask_honestly() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, false, None);
    let trail = harness.trail();

    mounted.handle.scroll_to_x(50.0);
    harness.settle();
    assert_eq!(
        mounted.handle.offset_x(),
        0.0,
        "the undeclared span has no scroll to deliver — the offset holds"
    );
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 1, "one ask, one fact — refused or landed");
    let record = records[0];
    let horizontal = record.horizontal.expect("the refusal rides the half");
    assert_eq!(
        horizontal.outcome,
        ScrollOutcome::Refused(ScrollRefusal::NoScrollableContent),
        "the empty span's wall names itself at the handle's x door"
    );
    assert_eq!(
        horizontal.requested,
        Some(50.0),
        "the finite ask the geometry priced rides beside the refusal"
    );
    assert_eq!(
        record.outcome,
        ScrollOutcome::Delivered,
        "the vertical fields ride the identity answer beside the refusal"
    );
}

/// The app door rides no route: the record's stamps carry the honest
/// absence — the handle knows no node id at all (the M43 rule at the
/// x half, the vertical door's own stamp rule spoken identically).
#[test]
fn m90_the_x_door_rides_no_route_stamp() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, true, Some(PROBE_WIDTH));
    let trail = harness.trail();

    mounted.handle.scroll_to_x(30.0);
    harness.settle();
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 1, "one handle ask, one fact");
    let record = records[0];
    assert_eq!(
        record.target, None,
        "the app door knows no container id — the stamp's honest absence"
    );
    assert_eq!(
        record.aimed, None,
        "the app door aims at no node — the stamp's honest absence"
    );
    assert_eq!(
        record.unit, None,
        "the handle door is a scalar ask, not a unit scroll"
    );
    assert_eq!(
        record.lines, None,
        "the handle door prices no wheel run — the count is the wheel's story"
    );
    let _ = mounted.wide;
}

/// The corpus grew by the coordinated rule: the bench case the handle
/// door prices joins the published corpus, its same-host gate joins
/// the contract set, and the M5/M6 pins re-agree at the cut — the
/// coordinated bench + contracts.json + baseline-regeneration rule,
/// reviewed here.
#[test]
fn m90_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the handle door's case.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(HANDLE_X_CASE),
        "the handle door's bench case is published ({HANDLE_X_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M90 cut"
    );

    // The contract set gates the case, same-host like its scroll-tier
    // siblings (absolute nanoseconds across silicon is noise theatre).
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    let gate = gates
        .iter()
        .find(|gate| gate["baseline"].as_str() == Some(HANDLE_X_CASE))
        .expect("the handle door's case carries a regression gate");
    assert_eq!(
        gate["scope"].as_str(),
        Some("same-host"),
        "the gate is same-host scoped like the scroll tier's own"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 97;"),
        "the m5 corpus pin re-agrees at the M90 cut"
    );
}
