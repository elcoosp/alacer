//! M89 exit-criteria integration tests (§8.2, ADR-0093): the
//! bring-into-view door grows the axis beside the ask — ADR-0092's own
//! re-open rule's words pre-registered the cut: "a consumer needs the
//! *horizontal* axis — a container that scrolls on both axes — → the
//! door grows the axis beside the ask (the vertical scalar is the
//! scroll vocabulary's only offset today, ADR-0056's own axis rule
//! standing)". The door this cut ships prices both halves from the
//! bounds ledger it already owns — the vertical ask exactly as M88
//! priced it, the horizontal ask beside it by the same edge-pin
//! convention (a rect wider than the viewport pins its left edge) —
//! and hands the one two-axis ask to the container's own clamp
//! arithmetic, which answers each axis by its own declared span. The
//! answer stays data at every landing: the vertical landing the M88
//! shape (unchanged fields, unchanged meaning), the horizontal axis's
//! own answer beside it, and the refusals named per axis — a container
//! whose horizontal span was never declared answers
//! `no_scrollable_content` on the x half while the vertical half
//! lands, the honest wall where the consumer can read it.
//!
//! The record grows the axis beside the action (ADR-0056's own axis
//! rule, cashed): one additive `horizontal` half beside the vertical
//! story — the outcome, the landed offset, the ask, the wall — `None`
//! on the stories with no horizontal half (the AT's unit scrolls ride
//! their vertical vocabulary untouched). The wheel's x run reaches
//! the same clamp arithmetic (positive x scrolls right, the platform's
//! own mapping), so a two-axis container scrolls on both axes from
//! every door the tier already shipped.
//!
//! Every test runs the real pipeline (A4): real layout, real compose
//! translation, the records through the harness trail door, the
//! pixels through the suite's probes — a probe that paints its own
//! halves two different colors, so one pixel's flip names which half
//! of the ask moved it. The compose translation moves the container's
//! clip with its content (the m1 canon's own scrolled-list
//! semantics), so the motion evidence is read inside the band the
//! shipped renderer paints.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::inspect::{InspectEvent, RangeBound, ScrollOutcome, ScrollRecord, ScrollRefusal};
use alacer_core::passes::ScrollIntoViewOutcome;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{CrossAxisAlignment, Size2, Style};
use alacer_core::widget::{A11yAction, LabelSource, Role, ScrollUnit, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};

/// The probe at the top of the list's content — the pixel the
/// vertical beats read.
const TOP_COLOR: Color = Color::rgb(37, 99, 235);

/// The tall wide probe's left half — the pixel the vertical landing
/// reads before the ask moves it.
const FAR_COLOR: Color = Color::rgb(255, 94, 58);

/// The wide probe's right half — the pixel the horizontal landing
/// reads once the x half lands. One probe, two tones: the boundary
/// between them is the offset's own ruler, on either axis.
const MARKER_COLOR: Color = Color::rgb(148, 163, 184);

/// The wheel beat's left tone — the pixel the pure-x run flips.
const WIDE_A: Color = Color::rgb(22, 163, 74);

/// The wheel beat's right tone — the pixel the pure-x run flips to.
const WIDE_B: Color = Color::rgb(250, 204, 21);

/// The wide probe's tones split at half its width — the boundary the
/// pixels read.
const TONE_SPLIT: f32 = 150.0;

/// The wheel beat's probe geometry: 400 px of content in a 200 px
/// viewport, the tones split at 120 — inside the band the shipped
/// renderer still paints after the run's landing.
const WHEEL_PROBE_WIDTH: f32 = 400.0;
const WHEEL_TONE_SPLIT: f32 = 120.0;

/// The stack's geometry: a 64 px top probe, the 192 px far probe, ten
/// 64 px fillers — 896 px of vertical content inside a 200 px
/// viewport, all of it 300 px wide inside the same 200 px viewport
/// width, so the far probe straddles the viewport's bottom edge
/// (64..256) *and* its right edge (0..300).
const TOP_HEIGHT: f32 = 64.0;
const FAR_HEIGHT: f32 = 192.0;
const FILLER_HEIGHT: f32 = 64.0;
const FILLERS: usize = 10;
const VIEWPORT: f32 = 200.0;
const PROBE_WIDTH: f32 = 300.0;

/// A probe painting its bounds one tone on the left of the split and
/// another on the right — the offset's pixel evidence on both axes at
/// once (the m-suite's probe shape, grown the axis this cut ships).
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
/// beats can read the offsets and the spans they declared. Both lists
/// start their cross axis (the wide probes keep the width their
/// styles declare, never the stretch the default column would give
/// them) and both fix the viewport's width at 200 px — the
/// vertical-only list's horizontal *span* stays undeclared (the
/// `ScrollState` widths its handle never grew), the layout fact and
/// the clamp's range living apart.
struct Mounted {
    list_node: alacer_core::NodeId,
    far: alacer_core::NodeId,
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
    let far = harness.add_child(
        list_node,
        WideProbe {
            left: FAR_COLOR,
            right: MARKER_COLOR,
            name: "far probe",
            height: FAR_HEIGHT,
            width: PROBE_WIDTH,
            split: TONE_SPLIT,
        }
        .boxed(),
    );
    for index in 0..FILLERS {
        let _ = harness.add_child(
            list_node,
            WideProbe {
                left: MARKER_COLOR,
                right: MARKER_COLOR,
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
        TOP_HEIGHT + FAR_HEIGHT + f32::from(u16::try_from(FILLERS).expect("fits")) * FILLER_HEIGHT;
    handle.set_content_height(content_height);
    if let Some(width) = content_width {
        handle.set_content_width(width);
    }
    harness.settle();
    Mounted {
        list_node,
        far,
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

/// The a11y-stable id of the node a snapshot names — the id the
/// record's stamps carry (the §6.3 id-path hash, read through the
/// snapshot the pipeline publishes rather than recomputed by the test).
fn stable_id(harness: &TestHarness, role: Role, name: &str) -> u64 {
    harness
        .pipeline()
        .a11y_snapshot()
        .expect("the snapshot is built")
        .find(role, name)
        .unwrap_or_else(|| panic!("the {name} is in the snapshot"))
        .id
}

/// The door lands the far probe's ask on both axes: the vertical half
/// exactly the M88 answer, the horizontal half beside it by the same
/// edge-pin convention, the landing data, the record's horizontal half
/// riding beside the vertical story, and the pixels flipping where the
/// shipped renderer paints them.
#[test]
fn m89_the_door_lands_the_far_probe_s_ask_on_both_axes() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, true, Some(PROBE_WIDTH));
    let trail = harness.trail();

    // The far probe straddles both edges: its bottom (256) rides 56 px
    // past the viewport's bottom (200), its right (300) 100 px past
    // the viewport's right (200) — the ask the two-axis door exists
    // for. The two pixels the beat reads: (30, 30) sits on the top
    // probe's fill (content y 30, above the far band), (50, 100) on
    // the far probe's left tone (content 50 < 150).
    assert_eq!(
        probe_pixel(&mut harness, 30, 30),
        painted(TOP_COLOR),
        "the top probe's fill sits at screen (30, 30) before the ask"
    );
    assert_eq!(
        probe_pixel(&mut harness, 50, 100),
        painted(FAR_COLOR),
        "the far probe's left tone sits at screen (50, 100) before the ask"
    );

    // The ask: the door prices both halves from the ledger, the
    // container's clamp answers each by its own span, the landing is
    // data on both axes.
    let landing = match harness.pipeline_mut().scroll_into_view(mounted.far) {
        ScrollIntoViewOutcome::Landed(landing) => landing,
        other => panic!("the far probe's ask must land, got {other:?}"),
    };
    assert_eq!(
        landing.outcome,
        ScrollOutcome::Delivered,
        "the vertical half's clamp delivered the ask"
    );
    assert_eq!(
        landing.offset, 56.0,
        "the minimal vertical ask lands the probe's bottom at the viewport's bottom (256 - 200)"
    );
    assert_eq!(
        landing.pinned, None,
        "a mid-range vertical landing pins no wall — the landing is the ask"
    );
    assert_eq!(
        landing.horizontal.outcome,
        ScrollOutcome::Delivered,
        "the horizontal half's clamp delivered beside the vertical"
    );
    assert_eq!(
        landing.horizontal.offset, 100.0,
        "the minimal horizontal ask lands the probe's right at the viewport's right (300 - 200)"
    );
    assert_eq!(
        landing.horizontal.pinned, None,
        "a mid-range horizontal landing pins no wall either"
    );
    assert_eq!(
        landing.container, mounted.list_node,
        "the landing names the container that moved"
    );
    harness.settle();

    // The offsets moved to the landings, the bounds ledger states the
    // landing on both axes, and the pixels flipped tones where the
    // shipped renderer paints them: the compose translation moves the
    // container's clip with its content (the m1 canon's scrolled-list
    // semantics), so the visible band after the landing is window
    // x 0..100, y 0..144 — and inside it the top probe's slot now
    // holds the far probe's left tone (the y half's motion), the far
    // slot at (50, 100) the probe's right tone (the x half's motion:
    // content 150 is exactly the tone split).
    assert_eq!(
        mounted.handle.offset(),
        56.0,
        "the container holds the vertical landing the door answered"
    );
    assert_eq!(
        mounted.handle.offset_x(),
        100.0,
        "the container holds the horizontal landing beside it"
    );
    let far_bounds = harness.bounds(mounted.far);
    assert_eq!(
        far_bounds.origin.x, -100.0,
        "the bounds ledger states the horizontal landing (0 - 100)"
    );
    assert_eq!(
        far_bounds.origin.y, 8.0,
        "the bounds ledger states the vertical landing (64 - 56)"
    );
    assert_eq!(
        probe_pixel(&mut harness, 30, 30),
        painted(FAR_COLOR),
        "the top probe's old slot now reads the far probe's left tone — the y half moved"
    );
    assert_eq!(
        probe_pixel(&mut harness, 50, 100),
        painted(MARKER_COLOR),
        "the far slot now reads the probe's right tone — the x half moved"
    );

    // The record: one bring_into_view fact, the vertical story the M88
    // shape, the horizontal half beside it (ADR-0056's axis rule
    // cashed), the stamps naming the container and the aimed node.
    let records = drain_scroll_records(&trail);
    m89_assert_two_axis_record(&records, &harness);
}

/// The two-axis landing record's own pin set (the m89 door beat's
/// record half): one fact, the vertical story the M88 shape, the
/// horizontal half beside it, the stamps naming the container and the
/// aimed node.
fn m89_assert_two_axis_record(records: &[ScrollRecord], harness: &TestHarness) {
    assert_eq!(
        records.len(),
        1,
        "one ask, one record — the halves ride one fact"
    );
    let record = records[0];
    assert_eq!(
        record.action,
        alacer_core::inspect::ScrollAction::BringIntoView,
        "the ask rode the geometry-driven species"
    );
    assert_eq!(record.outcome, ScrollOutcome::Delivered);
    assert_eq!(record.offset, 56.0);
    assert_eq!(record.requested, Some(56.0));
    assert_eq!(record.pinned, None);
    let horizontal = record
        .horizontal
        .expect("the two-axis ask's record carries the horizontal half");
    assert_eq!(horizontal.outcome, ScrollOutcome::Delivered);
    assert_eq!(horizontal.offset, 100.0);
    assert_eq!(horizontal.requested, Some(100.0));
    assert_eq!(horizontal.pinned, None);
    assert_eq!(
        record.target,
        Some(stable_id(harness, Role::ScrollArea, "list")),
        "the record names the container the ask rode"
    );
    assert_eq!(
        record.aimed,
        Some(stable_id(harness, Role::Decoration, "far probe")),
        "the record names the node the ask was for"
    );
}

/// The wheel runs the x half: a pure-x run on a two-axis container
/// whose vertical span is empty answers honestly on both axes — the
/// vertical half refuses, the horizontal half delivers by the same
/// clamp arithmetic, the run's pixels flip at the tone boundary, and
/// the run that rides past the horizontal wall is pinned there.
#[test]
fn m89_the_wheel_runs_the_x_half_and_the_wall_pins_the_run() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mut root_column = Column::new();
    root_column.style_mut().align = CrossAxisAlignment::Start;
    let root = harness.mount(root_column.boxed());
    let mut list = List::two_axis(VIEWPORT, VIEWPORT);
    // The same cross-axis start: the 400 px probe keeps its declared
    // width inside the 200 px viewport.
    list.style_mut().align = CrossAxisAlignment::Start;
    let handle = list.handle();
    let list_node = harness.add_child(root, list.boxed());
    let _ = harness.add_child(
        list_node,
        WideProbe {
            left: WIDE_A,
            right: WIDE_B,
            name: "wide probe",
            height: 64.0,
            width: WHEEL_PROBE_WIDTH,
            split: WHEEL_TONE_SPLIT,
        }
        .boxed(),
    );
    handle.set_content_height(64.0);
    handle.set_content_width(WHEEL_PROBE_WIDTH);
    harness.settle();
    let trail = harness.trail();

    // The pixel at (100, 30) reads the left tone (content 100 < 120).
    assert_eq!(
        probe_pixel(&mut harness, 100, 30),
        painted(WIDE_A),
        "the left tone sits at screen (100, 30) before the run"
    );

    // The pure-x run: +80 px scrolls right by the platform's own
    // mapping. The vertical half refuses (the 64 px content fits the
    // 200 px viewport), the horizontal half delivers 80.
    harness.wheel(Point::new(100.0, 30.0), (80.0, 0.0));
    assert_eq!(
        handle.offset_x(),
        80.0,
        "the run moved the x half by the run's own delta"
    );
    assert_eq!(
        handle.offset(),
        0.0,
        "the vertical half holds — the run asked it nothing it could deliver"
    );
    assert_eq!(
        probe_pixel(&mut harness, 100, 30),
        painted(WIDE_B),
        "the slot now reads content 180 — the right tone"
    );

    // The wall: the second run rides past the span (80 + 500 >> 200),
    // the clamp pins it at the span's own edge — the offsets and the
    // bounds ledger are the wall's evidence (the pinned viewport sits
    // past the window's own right edge, the canon's clip-follows-
    // content semantics painting nothing there to read).
    harness.wheel(Point::new(100.0, 30.0), (500.0, 0.0));
    assert_eq!(
        handle.offset_x(),
        200.0,
        "the clamp pinned the run at the horizontal span's edge (400 - 200)"
    );

    // The records: one per routed run, the vertical refusal named on
    // the first, the wall named on the second's horizontal half.
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 2, "one run, one record");
    assert_eq!(
        records[0].action,
        alacer_core::inspect::ScrollAction::Wheel,
        "the run rode the wheel's species"
    );
    assert_eq!(
        records[0].outcome,
        ScrollOutcome::Refused(ScrollRefusal::NoScrollableContent),
        "the vertical half refuses honestly — the content fits the viewport"
    );
    let first = records[0]
        .horizontal
        .expect("the wheel's record carries the x half on a two-axis container");
    assert_eq!(first.outcome, ScrollOutcome::Delivered);
    assert_eq!(first.offset, 80.0);
    assert_eq!(first.requested, Some(80.0));
    assert_eq!(first.pinned, None, "a mid-range landing pins no wall");
    let second = records[1]
        .horizontal
        .expect("the pinned run carries its x half too");
    assert_eq!(second.outcome, ScrollOutcome::Delivered);
    assert_eq!(second.offset, 200.0);
    assert_eq!(
        second.pinned,
        Some(RangeBound::Max),
        "the clamp names the wall the run was turned back at"
    );
}

/// The vertical-only container refuses the x half honestly: the door
/// prices both halves from the geometry (the wide probe rides past
/// the viewport's right edge there too), the container's undeclared
/// horizontal span answers `no_scrollable_content` on the x half
/// while the vertical half lands unchanged from M88.
#[test]
fn m89_the_vertical_only_container_refuses_the_x_half_honestly() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, false, None);
    let trail = harness.trail();

    // The ask lands the vertical half exactly as M88 answered it.
    let landing = match harness.pipeline_mut().scroll_into_view(mounted.far) {
        ScrollIntoViewOutcome::Landed(landing) => landing,
        other => panic!("the far probe's vertical ask must land, got {other:?}"),
    };
    assert_eq!(landing.outcome, ScrollOutcome::Delivered);
    assert_eq!(landing.offset, 56.0);
    assert_eq!(
        landing.horizontal.outcome,
        ScrollOutcome::Refused(ScrollRefusal::NoScrollableContent),
        "the undeclared horizontal span refuses the x half — the wall named"
    );
    assert_eq!(
        landing.horizontal.offset, 0.0,
        "the refusal keeps the offset the container held"
    );
    harness.settle();

    // The vertical landing holds, the horizontal never moved: the
    // pixel the beat reads sits where it started (content x 30, the
    // far probe's left tone) — the y half's own motion is visible one
    // slot up (the top probe's fill replaced by the far probe's).
    assert_eq!(mounted.handle.offset(), 56.0);
    assert_eq!(
        mounted.handle.offset_x(),
        0.0,
        "the refused half never moved the offset it refused to scroll"
    );
    let far_bounds = harness.bounds(mounted.far);
    assert_eq!(far_bounds.origin.y, 8.0, "the vertical landing landed");
    assert_eq!(
        far_bounds.origin.x, 0.0,
        "the bounds ledger states the x half never moved"
    );
    assert_eq!(
        probe_pixel(&mut harness, 30, 100),
        painted(FAR_COLOR),
        "the far probe's left tone holds its screen slot — the refused x half moved nothing"
    );
    assert_eq!(
        probe_pixel(&mut harness, 30, 30),
        painted(FAR_COLOR),
        "the top probe's old slot reads the far probe — the y half landed"
    );

    // The record: the x half rides beside the vertical story with its
    // own refusal, the ask it priced beside it — the geometry priced
    // the ask, the span refused it.
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 1, "one ask, one record");
    let horizontal = records[0]
        .horizontal
        .expect("the refused half rides the record too");
    assert_eq!(
        horizontal.outcome,
        ScrollOutcome::Refused(ScrollRefusal::NoScrollableContent)
    );
    assert_eq!(horizontal.offset, 0.0);
    assert_eq!(
        horizontal.requested,
        Some(100.0),
        "the ask the geometry priced rides beside the span's refusal"
    );
    assert_eq!(horizontal.pinned, None, "a refusal pinned no wall");
}

/// The wheel on the vertical-only container refuses the x half too:
/// the run's delta reaches the same clamp arithmetic, the undeclared
/// span refuses it, the pixels never move on x — and the vertical
/// half answers its own identity ask.
#[test]
fn m89_the_vertical_only_wheel_refuses_the_x_run() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, false, None);
    let trail = harness.trail();

    // The pixel at (150, 30) rides the top probe's fill (content x
    // 150, inside the 300 px probe and the list's own 200 px clip) —
    // it never moves on x.
    assert_eq!(
        probe_pixel(&mut harness, 150, 30),
        painted(TOP_COLOR),
        "the top probe's fill sits at screen (150, 30) before the run"
    );

    // The pure-x run: the x half refuses, the y half answers its own
    // identity ask (the run asked y nothing — the delta rode all x).
    harness.wheel(Point::new(100.0, 30.0), (80.0, 0.0));
    assert_eq!(
        mounted.handle.offset_x(),
        0.0,
        "the refused run never moved the x offset"
    );
    assert_eq!(
        probe_pixel(&mut harness, 150, 30),
        painted(TOP_COLOR),
        "the pixel holds — the x run moved nothing"
    );

    // The record: the vertical half delivered the identity ask, the
    // horizontal half refused with the ask it priced.
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 1, "one run, one record");
    assert_eq!(
        records[0].outcome,
        ScrollOutcome::Delivered,
        "the y half answers the zero ask it was given"
    );
    assert_eq!(records[0].offset, 0.0);
    let horizontal = records[0]
        .horizontal
        .expect("the refused x run rides the record");
    assert_eq!(
        horizontal.outcome,
        ScrollOutcome::Refused(ScrollRefusal::NoScrollableContent)
    );
    assert_eq!(horizontal.requested, Some(80.0));
}

/// The declared horizontal span pins the door's ask: the geometry
/// prices the x half from the probe's real width (100), the
/// app-declared span (240 - 200 = 40) is the range the clamp answers
/// by — the landing at the wall, the ask turned back, the wall named.
#[test]
fn m89_the_declared_span_walls_the_door_s_x_ask() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, true, Some(240.0));
    let trail = harness.trail();

    let landing = match harness.pipeline_mut().scroll_into_view(mounted.far) {
        ScrollIntoViewOutcome::Landed(landing) => landing,
        other => panic!("the far probe's ask must land, got {other:?}"),
    };
    // The vertical half mid-range; the horizontal half pinned at the
    // declared span's wall.
    assert_eq!(landing.outcome, ScrollOutcome::Delivered);
    assert_eq!(landing.offset, 56.0);
    assert_eq!(landing.horizontal.outcome, ScrollOutcome::Delivered);
    assert_eq!(
        landing.horizontal.offset, 40.0,
        "the clamp answered by the declared span (240 - 200), never the geometry's 100"
    );
    assert_eq!(
        landing.horizontal.pinned,
        Some(RangeBound::Max),
        "the wall the clamp turned the ask back at is named"
    );
    harness.settle();

    assert_eq!(mounted.handle.offset_x(), 40.0);
    let far_bounds = harness.bounds(mounted.far);
    assert_eq!(
        far_bounds.origin.x, -40.0,
        "the bounds ledger states the wall's landing (0 - 40)"
    );
    // The pixel flips where the shipped renderer paints: content
    // (120, 100) read the far probe's left tone before (120 < 150);
    // after the wall's landing the slot reads the probe's local
    // (160, 92) — past the tone split, the x half's 40 px ride visible
    // even though the ask never reached the geometry's 100.
    assert_eq!(
        probe_pixel(&mut harness, 120, 100),
        painted(MARKER_COLOR),
        "the slot reads the probe's right tone — the wall's landing moved the 40 px it could"
    );

    // The record: the pinned half rides the wall beside the ask.
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 1, "one ask, one record");
    let horizontal = records[0]
        .horizontal
        .expect("the pinned half rides the record");
    assert_eq!(horizontal.offset, 40.0);
    assert_eq!(horizontal.requested, Some(100.0));
    assert_eq!(horizontal.pinned, Some(RangeBound::Max));
}

/// The visible half answers its own identity ask: the door prices
/// both halves from the geometry, and the axis that already holds the
/// aimed node asks the offset it holds — the clamp delivering it
/// unchanged (the landing is the ask, no wall), the vertical half
/// landing beside it.
#[test]
fn m89_the_visible_half_answers_its_own_identity_ask() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, true, Some(PROBE_WIDTH));
    let trail = harness.trail();

    // The beat's own probe: a narrow row mounted after the fillers —
    // fully inside on x (0..200 in the 200 px viewport), below the
    // window on y (content y 896..960, past the 200 px viewport). The
    // declared content height grows with it (the handle's door).
    let narrow = harness.add_child(
        mounted.list_node,
        WideProbe {
            left: WIDE_A,
            right: WIDE_A,
            name: "narrow probe",
            height: 64.0,
            width: VIEWPORT,
            split: TONE_SPLIT,
        }
        .boxed(),
    );
    mounted.handle.set_content_height(896.0 + 64.0);
    harness.settle();

    // The door asks y (the minimal ask pins the probe's bottom to the
    // viewport's bottom), and x answers its own identity.
    let landing = match harness.pipeline_mut().scroll_into_view(narrow) {
        ScrollIntoViewOutcome::Landed(landing) => landing,
        other => panic!("the narrow probe's ask must land, got {other:?}"),
    };
    assert_eq!(landing.outcome, ScrollOutcome::Delivered);
    assert_eq!(
        landing.offset, 760.0,
        "the vertical ask lands the narrow probe's bottom at the viewport's bottom (960 - 200)"
    );
    assert_eq!(
        landing.horizontal.outcome,
        ScrollOutcome::Delivered,
        "the visible half's clamp delivers the identity ask"
    );
    assert_eq!(
        landing.horizontal.offset, 0.0,
        "the identity answer holds the offset the container already had"
    );
    assert_eq!(
        landing.horizontal.pinned, None,
        "the landing is the ask — no wall"
    );
    harness.settle();
    assert_eq!(mounted.handle.offset_x(), 0.0, "the x half never moved");
    assert_eq!(
        harness.bounds(narrow).origin.y,
        136.0,
        "the bounds ledger states the vertical landing (896 - 760)"
    );

    // The record: the identity half rides beside the vertical landing,
    // the ask it answered beside both.
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 1, "one ask, one record");
    let horizontal = records[0]
        .horizontal
        .expect("the identity half rides the record too");
    assert_eq!(horizontal.outcome, ScrollOutcome::Delivered);
    assert_eq!(horizontal.offset, 0.0);
    assert_eq!(
        horizontal.requested,
        Some(0.0),
        "the identity ask is the offset the container held"
    );
}

/// The AT's vertical vocabulary rides untouched on a two-axis
/// container: the unit scroll moves the vertical half by its unit,
/// the x half holds where it is, and the record carries the honest
/// absence — `None`, the AT's ask rode no horizontal half.
#[test]
fn m89_the_at_s_vertical_vocabulary_rides_untouched() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(240.0, 300.0);
    let mounted = mount_probes(&mut harness, true, Some(PROBE_WIDTH));
    let trail = harness.trail();

    // A wheel run first, so the x half holds a real offset the AT's
    // ask must not disturb.
    harness.wheel(Point::new(100.0, 30.0), (60.0, 0.0));
    assert_eq!(mounted.handle.offset_x(), 60.0);
    harness.settle();

    // The AT's page scroll: the vertical half moves by the viewport,
    // the x half untouched.
    let answered = harness.act(
        Role::ScrollArea,
        "list",
        A11yAction::ScrollDown(ScrollUnit::Page),
    );
    assert!(answered, "the scroll area answers the AT's page scroll");
    harness.settle();

    assert_eq!(
        mounted.handle.offset(),
        VIEWPORT,
        "the page scroll moved the vertical half by the viewport"
    );
    assert_eq!(
        mounted.handle.offset_x(),
        60.0,
        "the AT's vertical ask never moved the x half"
    );

    // The records: the wheel run carries its horizontal half; the
    // AT's page scroll rides the honest absence.
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 2, "one run, one ask, one record each");
    assert!(
        records[0].horizontal.is_some(),
        "the wheel's record carries its x half"
    );
    assert_eq!(
        records[1].action,
        alacer_core::inspect::ScrollAction::ScrollDown,
        "the AT's ask rode the unit-scroll species"
    );
    assert!(
        records[1].horizontal.is_none(),
        "the AT's vertical vocabulary rides no horizontal half — the honest absence"
    );
}

/// The corpus grew by the coordinated rule (ADR-0011): the horizontal
/// case is published with its baseline and its same-host gate, the
/// baseline's own count agrees with its entries, and the m5 suite's
/// corpus pin re-agrees at 95.
#[test]
fn m89_the_corpus_grew_by_the_coordinated_rule() {
    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let baselines: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(workspace.join("benchmarks/baselines/baselines.json"))
            .expect("the published baseline exists"),
    )
    .expect("the baseline is valid JSON");
    let contracts: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(workspace.join("benchmarks/contracts.json"))
            .expect("the contracts exist"),
    )
    .expect("the contracts are valid JSON");

    let case = "scroll/route_100_horizontal_bring_into_view_1k";
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(case),
        "the new case is published with its baseline: {case}"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(benches.len() as u64),
        "the baseline's own count agrees with its entries"
    );
    assert!(
        contracts["regressions"]
            .as_array()
            .expect("the regressions are an array")
            .iter()
            .any(|gate| gate["baseline"].as_str() == Some(case)),
        "the new case carries its same-host gate"
    );
    let m5 =
        std::fs::read_to_string(workspace.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
            .expect("the m5 suite exists");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 207;"),
        "the m5 suite's corpus pin re-agrees with the grown corpus"
    );
}
