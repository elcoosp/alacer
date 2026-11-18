//! M88 exit-criteria integration tests (§8.2, ADR-0092): the
//! bring-into-view door — the geometry-driven scroll ask. ADR-0056's
//! third kill rule's own words pre-registered the shape the scroll
//! tier's doors had left open: the wheel, the AT's unit scrolls and
//! value steps, and the app's raw-offset handle all answer asks
//! *priced in offsets*, while the ask consumers actually hold — "this
//! item, this row, this focused control" — is priced in *nodes*. The
//! door this cut ships is the node-priced ask: the pipeline reads the
//! bounds ledger it already owns (pass 6's absolute composed bounds,
//! pass 7's applied offsets), computes the minimal offset that lands
//! the node fully inside the nearest scrollable ancestor's viewport,
//! and hands the ask to the container's own door — the widget's clamp
//! arithmetic (the span walls, the named refusals) answering where the
//! range lives, never the pipeline guessing at a widget-private span.
//!
//! The answer is data at every landing (the M27 rule at the scroll
//! tier): a [`alacer_core::passes::ScrollIntoViewLanding`] beside the
//! outcome, a named [`alacer_core::passes::ScrollIntoViewRefusal`]
//! when the ask never reached a scroll tier (a dead id, an unlaid-out
//! node, no scrollable ancestor, a container without the door), and
//! the honest `Visible` when the node already sits fully inside the
//! viewport — nothing moves, nothing queues (the M51 idle property at
//! the door's own tier). The ask that lands streams one `ScrollRecord`
//! under the new `bring_into_view` action species — the record naming
//! the container it rode (`target`) and, beside it for the first time
//! on the scroll log, the node the ask was *for* (the additive `aimed`
//! field, the M30 attribution rule's shape at the scroll tier: every
//! causal fact names who asked and who answered).
//!
//! Every test runs the real pipeline (A4): real layout, real compose
//! translation, the records through the harness trail door, the pixels
//! through the suite's probes — the probe's pixel rides the shipped
//! renderer's own clipped band (the compose translation moves the
//! container's clip with its content, the m1 canon's own scrolled-list
//! semantics), so the motion evidence is read where the shipped
//! renderer paints it: a probe's screen slot changes when the offset
//! lands, and the bounds ledger states where it landed.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect};
use alacer_core::inspect::{InspectEvent, RangeBound, ScrollRecord};
use alacer_core::passes::{ScrollIntoViewOutcome, ScrollIntoViewRefusal};
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{Size2, Style};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};

/// The probe at the top of the list's content — the pixel the
/// scroll-back beat reads.
const TOP_COLOR: Color = Color::rgb(37, 99, 235);

/// The tall probe straddling the viewport's bottom edge — the pixel
/// the bring-into-view beat reads once the door lands its ask.
const FAR_COLOR: Color = Color::rgb(255, 94, 58);

/// The filler probes below the tall one — the color the moved-away
/// slot reads after the ask lands.
const FILLER_COLOR: Color = Color::rgb(148, 163, 184);

/// The stack: a 64 px probe, the 192 px far probe, ten 64 px fillers —
/// 896 px of content inside a 200 px viewport, so the far probe
/// straddles the window's bottom edge (64..256) and the fillers sit
/// past it (256..896).
const TOP_HEIGHT: f32 = 64.0;
const FAR_HEIGHT: f32 = 192.0;
const FILLER_HEIGHT: f32 = 64.0;
const FILLERS: usize = 10;
const VIEWPORT: f32 = 200.0;
const PROBE_WIDTH: f32 = 160.0;

/// A probe painting its whole bounds one color — the offset's pixel
/// evidence (the m-suite's probe shape, mounted as a list child).
struct ScrollProbe {
    color: Color,
    name: &'static str,
    height: f32,
}

impl ScrollProbe {
    fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Widget for ScrollProbe {
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
                width: Some(PROBE_WIDTH),
                height: Some(self.height),
            },
            ..Style::default()
        }
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), ctx.size()),
            color: self.color,
            corner_radius: 0.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// A widget that claims the scroll container's compose slot but ships
/// no bring-into-view door — the contract gap the door must refuse
/// honestly, never silently skip (the named-refusal rule at the tier
/// whose funnel knows no range). The 100 px fixed height clips its
/// children, so an ask aimed past the window reaches the door slot
/// the widget never shipped.
struct DoorlessScroller;

impl Widget for DoorlessScroller {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::ScrollArea
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("doorless")
    }
    fn is_scroll_container(&self) -> bool {
        true
    }
    fn style(&self) -> Style {
        Style {
            clip_overflow: true,
            size: Size2 {
                width: None,
                height: Some(100.0),
            },
            ..Style::default()
        }
    }
}

/// A scrollable list with the probe stack inside it, mounted through
/// the real pipeline — the handle kept beside the node ids so the
/// beats can read the offset and the content height they declared.
/// The content height is the app's own story (the handle's door):
/// declared one span past the stack, so the fillers' ask lands
/// mid-range.
struct Mounted {
    root: alacer_core::NodeId,
    list_node: alacer_core::NodeId,
    top: alacer_core::NodeId,
    far: alacer_core::NodeId,
    last: alacer_core::NodeId,
    handle: alacer_widgets::ListHandle,
}

fn mount_probes(harness: &mut TestHarness, content_height: f32) -> Mounted {
    let root = harness.mount(Column::new().boxed());
    let list = List::new(VIEWPORT);
    let handle = list.handle();
    let list_node = harness.add_child(root, list.boxed());
    let top = harness.add_child(
        list_node,
        ScrollProbe {
            color: TOP_COLOR,
            name: "top probe",
            height: TOP_HEIGHT,
        }
        .boxed(),
    );
    let far = harness.add_child(
        list_node,
        ScrollProbe {
            color: FAR_COLOR,
            name: "far probe",
            height: FAR_HEIGHT,
        }
        .boxed(),
    );
    let mut tail = far;
    for index in 0..FILLERS {
        let node = harness.add_child(
            list_node,
            ScrollProbe {
                color: FILLER_COLOR,
                name: "filler probe",
                height: FILLER_HEIGHT,
            }
            .boxed(),
        );
        tail = node;
        let _ = index;
    }
    handle.set_content_height(content_height);
    harness.settle();
    Mounted {
        root,
        list_node,
        top,
        far,
        last: tail,
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

/// The pixel at a probe's own origin, as RGBA bytes — the probe is
/// 160 wide, 32 in from the left edge is its own fill (the m-suite's
/// pixel read, at the probe's own geometry).
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

/// The door brings the far probe's ask to its viewport-bottom landing:
/// the landing answers the minimal ask beside the clamp's arithmetic,
/// the record streams one `bring_into_view` fact naming the container
/// it rode and the node it was for, the bounds ledger states where the
/// probe landed, and the probe's screen slot is the offset's own
/// answer.
#[test]
fn m88_the_door_lands_the_far_probe_s_ask_in_the_viewport() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 300.0);
    let mounted = mount_probes(&mut harness, 1000.0);
    let trail = harness.trail();

    // The far probe straddles the viewport's bottom edge: its bottom
    // (256) rides past the window's bottom (200) — the ask the door
    // exists for.
    let list_bounds = harness.bounds(mounted.list_node);
    let far_bounds = harness.bounds(mounted.far);
    assert_eq!(
        far_bounds.max_y(),
        list_bounds.max_y() + 56.0,
        "the far probe's bottom rides 56 px past the viewport before the ask"
    );
    // The screen slot the ask is about to move: the top probe's fill
    // at screen 30 (content 30), the filler's at screen 180 (content
    // 180).
    assert_eq!(
        probe_pixel(&mut harness, 30, 30),
        painted(TOP_COLOR),
        "the top probe's fill sits at screen 30 before the ask"
    );
    assert_eq!(
        probe_pixel(&mut harness, 30, 180),
        painted(FAR_COLOR),
        "the far probe's fill sits at screen 180 before the ask"
    );

    // The ask: the door computes the minimal offset (the probe's bottom
    // edge pinned to the viewport's bottom), the container's clamp
    // answers it mid-range (the declared span covers the ask), the
    // landing is data.
    let landing = match harness.pipeline_mut().scroll_into_view(mounted.far) {
        ScrollIntoViewOutcome::Landed(landing) => landing,
        other => panic!("the far probe's ask must land, got {other:?}"),
    };
    assert_eq!(
        landing.outcome,
        alacer_core::inspect::ScrollOutcome::Delivered,
        "the container's clamp delivered the ask"
    );
    assert_eq!(
        landing.offset, 56.0,
        "the minimal ask lands the probe's bottom at the viewport's bottom (256 - 200)"
    );
    assert_eq!(
        landing.pinned, None,
        "a mid-range landing pins no wall — the landing is the ask"
    );
    assert_eq!(
        landing.container, mounted.list_node,
        "the landing names the container that moved"
    );
    harness.settle();

    // The offset moved to the landing, the bounds ledger states the
    // landing, and the screen slots moved with it: the top probe's
    // slot now holds the far probe's fill, the far probe's old slot the
    // fillers' — the shipped renderer's own band (the compose
    // translation moves the container's clip with its content, the m1
    // canon's scrolled-list semantics).
    assert_eq!(
        mounted.handle.offset(),
        56.0,
        "the container holds the landing the door answered"
    );
    assert_eq!(
        harness.bounds(mounted.far).origin.y,
        8.0,
        "the bounds ledger states the landing (64 - 56)"
    );
    assert_eq!(
        probe_pixel(&mut harness, 30, 30),
        painted(FAR_COLOR),
        "the far probe's fill did not move into the slot the top probe vacated"
    );
    assert_eq!(
        probe_pixel(&mut harness, 30, 104),
        painted(FAR_COLOR),
        "the far probe's own fill paints where its landing puts it (64 - 56 + 96)"
    );

    // The record: one `bring_into_view` fact, the container it rode
    // beside the node it was for (the M30 attribution rule at the
    // scroll tier), the ask beside the landing, no wall.
    let list_id = stable_id(&harness, Role::ScrollArea, "list");
    let far_id = stable_id(&harness, Role::Decoration, "far probe");
    let records = drain_scroll_records(&trail);
    assert_eq!(
        records.len(),
        1,
        "one ask, one record — the scroll tier's fact: {records:?}"
    );
    assert_eq!(
        records[0].action,
        alacer_core::inspect::ScrollAction::BringIntoView,
        "the record names the species the ask rode"
    );
    assert_eq!(
        records[0].target,
        Some(list_id),
        "the record names the container the ask rode"
    );
    assert_eq!(
        records[0].aimed,
        Some(far_id),
        "the record names the node the ask was for — the aimed id, verbatim"
    );
    assert_eq!(
        records[0].requested,
        Some(56.0),
        "the ask beside the landing (the M44 rule at the scroll tier)"
    );
    assert_eq!(records[0].offset, 56.0, "the landing beside the ask");
    assert_eq!(records[0].pinned, None, "a mid-range landing pins no wall");

    harness.assert_all_named();
}

/// The door scrolls back up to the top: from the far landing, the
/// top probe's ask rides the same arithmetic upward — the landed
/// offset the ask itself (zero is the range's floor, and the ask
/// wanting the floor exactly is the landing, no wall).
#[test]
fn m88_the_door_scrolls_back_up_to_the_top() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 300.0);
    let mounted = mount_probes(&mut harness, 1000.0);
    let trail = harness.trail();

    // Down to the far probe first (the previous beat's landing).
    let down = match harness.pipeline_mut().scroll_into_view(mounted.far) {
        ScrollIntoViewOutcome::Landed(landing) => landing,
        other => panic!("the far probe's ask must land, got {other:?}"),
    };
    assert_eq!(down.offset, 56.0, "the down beat lands at the far probe");
    harness.settle();
    let _ = drain_scroll_records(&trail);

    // Up to the top probe: the ask is the probe's content-space top
    // (zero), the clamp delivers it exactly — the landing is the ask.
    let landing = match harness.pipeline_mut().scroll_into_view(mounted.top) {
        ScrollIntoViewOutcome::Landed(landing) => landing,
        other => panic!("the top probe's ask must land, got {other:?}"),
    };
    assert_eq!(
        landing.offset, 0.0,
        "the top probe's ask lands at the range's floor"
    );
    assert_eq!(
        landing.pinned, None,
        "the ask wanting the wall exactly is the landing — no wall held it"
    );
    harness.settle();

    assert_eq!(
        mounted.handle.offset(),
        0.0,
        "the container holds the landing the door answered"
    );
    assert_eq!(
        harness.bounds(mounted.top).origin.y,
        0.0,
        "the bounds ledger states the landing"
    );
    assert_eq!(
        probe_pixel(&mut harness, 30, 30),
        painted(TOP_COLOR),
        "the top probe's fill is back in the slot the down beat moved it out of"
    );
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 1, "one ask, one record");
    assert_eq!(
        records[0].requested,
        Some(0.0),
        "the ask the door computed rides beside the landing"
    );
    assert_eq!(records[0].offset, 0.0, "the landing beside the ask");
}

/// The already-visible ask is the honest no-op: the door answers
/// `Visible`, nothing moves, and nothing queues — no record, no
/// repaint, the idle frame the content clamp's own rule rides (the
/// M51 idle property at the door's tier).
#[test]
fn m88_the_already_visible_ask_is_the_honest_no_op() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 300.0);
    let mounted = mount_probes(&mut harness, 1000.0);
    let trail = harness.trail();
    let bounds_before = harness.bounds(mounted.top);
    let paint_runs_before = harness.counters().paint_runs;

    let answer = harness.pipeline_mut().scroll_into_view(mounted.top);
    assert_eq!(
        answer,
        ScrollIntoViewOutcome::Visible,
        "the top probe sits fully inside the viewport at offset zero"
    );
    harness.settle();

    assert_eq!(
        mounted.handle.offset(),
        0.0,
        "the visible ask moved no offset"
    );
    assert_eq!(
        harness.bounds(mounted.top),
        bounds_before,
        "the visible ask moved no bounds"
    );
    assert_eq!(
        harness.counters().paint_runs,
        paint_runs_before,
        "the visible ask queued no repaint — the idle frame's own rule"
    );
    let records = drain_scroll_records(&trail);
    assert!(
        records.is_empty(),
        "the visible ask streamed no record — {records:?}"
    );
}

/// The refusals are named, every one, and none of them streams a
/// record: a dead id, a node the layout has not answered yet, an ask
/// with no scrollable ancestor — each answered by the door itself,
/// never a munged landing, never a tier fact for a question the scroll
/// tier never answered (the ADR-0046 rule the scroll tier rides).
#[test]
fn m88_the_refusals_are_named_and_none_of_them_streams() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 300.0);
    let mounted = mount_probes(&mut harness, 1000.0);
    let trail = harness.trail();

    // The dead id: the sentinel no live node ever holds.
    let dead = harness
        .pipeline_mut()
        .scroll_into_view(alacer_core::NodeId::INVALID);
    assert_eq!(
        dead,
        ScrollIntoViewOutcome::Refused(ScrollIntoViewRefusal::UnknownNode),
        "the sentinel id refuses as the dead node it is"
    );

    // The unlaid-out node: mounted, never ticked — the bounds ledger
    // holds no answer for it yet.
    let unlaid = harness.add_child(
        mounted.list_node,
        ScrollProbe {
            color: TOP_COLOR,
            name: "unlaid probe",
            height: TOP_HEIGHT,
        }
        .boxed(),
    );
    let unlaid_answer = harness.pipeline_mut().scroll_into_view(unlaid);
    assert_eq!(
        unlaid_answer,
        ScrollIntoViewOutcome::Refused(ScrollIntoViewRefusal::NotLaidOut),
        "a node the layout has not answered refuses before any geometry is guessed"
    );

    // No scrollable ancestor: the probe mounted beside the list, on
    // the root column — no scroll container above it. The settle first:
    // the bounds ledger must hold the node's answer, or the ask would
    // refuse NotLaidOut before the ancestor walk ever ran.
    let outside = harness.add_child(
        mounted.root,
        ScrollProbe {
            color: TOP_COLOR,
            name: "outside probe",
            height: TOP_HEIGHT,
        }
        .boxed(),
    );
    harness.settle();
    let outside_answer = harness.pipeline_mut().scroll_into_view(outside);
    assert_eq!(
        outside_answer,
        ScrollIntoViewOutcome::Refused(ScrollIntoViewRefusal::NoScrollableAncestor),
        "an ask with no scrollable ancestor above it refuses before any container is guessed"
    );
    harness.settle();
    let records = drain_scroll_records(&trail);
    assert!(
        records.is_empty(),
        "the refusals so far streamed no record — {records:?}"
    );

    harness.assert_all_named();
}

/// A container that claims the scroll slot without the door refuses
/// honestly: the ask aimed past its clipped window reaches the door
/// slot the widget never shipped — the ask never lands, never moves a
/// pixel, and the refusal names the missing door instead of skipping
/// the ask silently.
#[test]
fn m88_the_doorless_container_refuses_honestly() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 300.0);
    let root = harness.mount(Column::new().boxed());
    let scroller = harness.add_child(root, Box::new(DoorlessScroller));
    let near = harness.add_child(
        scroller,
        ScrollProbe {
            color: TOP_COLOR,
            name: "near probe",
            height: TOP_HEIGHT,
        }
        .boxed(),
    );
    let deep = harness.add_child(
        scroller,
        ScrollProbe {
            color: FAR_COLOR,
            name: "deep probe",
            height: FAR_HEIGHT,
        }
        .boxed(),
    );
    harness.settle();
    let trail = harness.trail();

    // The visible child answers before any container's door: the
    // honest no-op needs no door to answer it.
    let visible = harness.pipeline_mut().scroll_into_view(near);
    assert_eq!(
        visible,
        ScrollIntoViewOutcome::Visible,
        "the near probe sits fully inside the doorless container's window"
    );

    // The child past the window reaches the door slot — and names the
    // door the container never shipped.
    let answer = harness.pipeline_mut().scroll_into_view(deep);
    assert_eq!(
        answer,
        ScrollIntoViewOutcome::Refused(ScrollIntoViewRefusal::NoScrollDoor),
        "the container that claims the compose slot without the door names the gap"
    );
    let records = drain_scroll_records(&trail);
    assert!(
        records.is_empty(),
        "the refusal streamed no record — the ask never reached a scroll answer: {records:?}"
    );

    harness.assert_all_named();
}

/// The clamp's wall still holds the door's ask: a content height the
/// app declared short of the children's stack pins the last filler's
/// ask at the span's own edge — the wall named on the landing and on
/// the record alike (the clamp arithmetic is the container's own, the
/// pipeline never bypasses it).
#[test]
fn m88_the_span_wall_pins_the_door_s_ask() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(220.0, 300.0);
    // Declared 700 px of content under the 896 px stack: the span is
    // 500 px, and the last filler's minimal ask (696) rides past it.
    let mounted = mount_probes(&mut harness, 700.0);
    let trail = harness.trail();

    let landing = match harness.pipeline_mut().scroll_into_view(mounted.last) {
        ScrollIntoViewOutcome::Landed(landing) => landing,
        other => panic!("the last filler's ask must land, got {other:?}"),
    };
    assert_eq!(
        landing.offset, 500.0,
        "the clamp held the ask at the span's edge (700 - 200)"
    );
    assert_eq!(
        landing.pinned,
        Some(RangeBound::Max),
        "the wall the clamp pinned is named on the landing"
    );
    harness.settle();

    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 1, "one ask, one record");
    assert_eq!(
        records[0].requested,
        Some(696.0),
        "the ask beside the landing — the wall held a past ask"
    );
    assert_eq!(records[0].offset, 500.0, "the landing at the wall");
    assert_eq!(
        records[0].pinned,
        Some(RangeBound::Max),
        "the wall named on the record (the clamp arithmetic is the container's own)"
    );
}

/// The corpus grew 93 → 94 by the coordinated rule (ADR-0011): the new
/// bench case is published with its baseline and its same-host gate,
/// and the m5 pin re-agrees with the grown corpus.
#[test]
fn m88_the_corpus_grew_by_the_coordinated_rule() {
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

    let case = "scroll/route_100_bring_into_view_1k";
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
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 110;"),
        "the m5 suite's corpus pin re-agrees with the grown corpus"
    );
}
