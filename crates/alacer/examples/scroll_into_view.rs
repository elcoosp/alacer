//! The bring-into-view door (M88, ADR-0092) driven from the consumer
//! side: the scrolling feed whose "open the item" beat brings a row
//! the viewport never showed into the window — the pipeline pricing
//! the ask from the bounds ledger it already owns, the list's own
//! clamp arithmetic answering it (the span walls, the named
//! refusals), and the log telling the story: one `bring_into_view`
//! record per landed ask, the container it rode beside the node it
//! was for, the ask beside the landing, the wall the clamp pinned.
//! The honest silences beside them: the already-visible ask answers
//! `Visible` and streams nothing, the ask aimed past any scrollable
//! ancestor refuses as data, and the no-op lands nowhere.
//!
//! Run with `cargo run -p alacer --example scroll_into_view`. The
//! example is headless — the real pass pipeline (A4), the pixels read
//! back from the software raster — so the same five beats run
//! anywhere the tests do.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect};
use alacer_core::inspect::{InspectEvent, ScrollRecord};
use alacer_core::passes::ScrollIntoViewOutcome;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{Size2, Style};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};

/// The row the feed opens on — the pixel the beats read.
const ROW_COLOR: Color = Color::rgb(255, 94, 58);

/// The row above it, the scroll-back beat's landing marker.
const TOP_COLOR: Color = Color::rgb(37, 99, 235);

/// The rows between — the fill the moved-away slot reads.
const FILLER_COLOR: Color = Color::rgb(148, 163, 184);

/// One row's geometry: 64 px tall, full width — twenty of them stack
/// to 1280 px of content inside a 300 px viewport.
const ROW_HEIGHT: f32 = 64.0;
const VIEWPORT: f32 = 300.0;
const ROWS: usize = 20;

/// A feed row painting its whole bounds one color — the offset's
/// pixel evidence, read back from the software raster.
struct Row {
    color: Color,
    name: &'static str,
}

impl Row {
    fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Widget for Row {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::ListItem
    }
    fn label(&self) -> LabelSource {
        LabelSource::text(self.name)
    }
    fn style(&self) -> Style {
        Style {
            size: Size2 {
                width: Some(200.0),
                height: Some(ROW_HEIGHT),
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

/// The pixel at a probe's own origin, as RGBA bytes — the rows are
/// 200 wide, 32 in from the left edge is their own fill.
fn probe_pixel(harness: &mut TestHarness, x: u32, y: u32) -> [u8; 4] {
    let pixmap = harness.pixels();
    let px = pixmap
        .pixel(x, y)
        .unwrap_or_else(|| panic!("pixel ({x}, {y}) outside the viewport"));
    [px.red(), px.green(), px.blue(), px.alpha()]
}

/// The pixel an opaque row paints, as the row paints it —
/// premultiplied of opaque is the color's own bytes.
fn painted(color: Color) -> [u8; 4] {
    [color.r, color.g, color.b, 0xFF]
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

fn main() {
    let mut harness = TestHarness::new(260.0, 380.0);
    let root = harness.mount(Column::new().boxed());
    let list = List::new(VIEWPORT);
    let handle = list.handle();
    let list_node = harness.add_child(root, list.boxed());
    let mut first = None;
    let mut target = None;
    for index in 0..ROWS {
        let (color, name) = match index {
            0 => (TOP_COLOR, "first row"),
            4 => (ROW_COLOR, "target row"),
            _ => (FILLER_COLOR, "filler row"),
        };
        let node = harness.add_child(list_node, Row { color, name }.boxed());
        first.get_or_insert(node);
        if index == 4 {
            target = Some(node);
        }
    }
    let (first, target) = (
        first.expect("the first row is mounted"),
        target.expect("the target row is mounted"),
    );
    // The content height is the app's own story (the handle's door):
    // declared one span past the stack, so the fillers' asks land
    // mid-range.
    handle.set_content_height(alacer_core::num::usize_to_f32(ROWS) * ROW_HEIGHT + 100.0);
    let trail = harness.trail();
    harness.settle();

    println!("== the bring-into-view door (M88, ADR-0092) ==");
    println!(
        "the feed mounts {} rows ({} px of content) in a {} px viewport",
        ROWS,
        alacer_core::num::usize_to_u32(ROWS) * alacer_core::num::f32_to_u32(ROW_HEIGHT),
        alacer_core::num::f32_to_u32(VIEWPORT),
    );

    // Beat 1 — the already-visible ask: the honest no-op. The first
    // row sits fully inside the viewport at offset zero; the door
    // answers `Visible`, nothing moves, nothing queues.
    let answer = harness.pipeline_mut().scroll_into_view(first);
    harness.settle();
    println!("beat 1: the visible ask answers {answer:?} — no offset, no record, the idle frame");
    assert!(matches!(answer, ScrollIntoViewOutcome::Visible));
    assert!(drain_scroll_records(&trail).is_empty(), "no ask, no record");

    // Beat 2 — the node-priced ask: the target row straddles the
    // window's bottom edge (content 256..320 against the 300 px
    // viewport), the ask the window only half showed. The pipeline
    // prices it from the bounds ledger, the list's clamp answers it
    // mid-range.
    let answer = harness.pipeline_mut().scroll_into_view(target);
    harness.settle();
    match answer {
        ScrollIntoViewOutcome::Landed(landing) => {
            println!(
                "beat 2: the target row's ask landed — offset {} pinned {:?}, the container {:?}",
                landing.offset, landing.pinned, landing.container
            );
            assert_eq!(landing.offset, 20.0, "the target row lands at its ask");
        }
        other => panic!("the target row's ask must land, got {other:?}"),
    }
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 1, "one ask, one record");
    println!(
        "         the log: action {}, requested {:?}, offset {}, aimed {:?}",
        records[0].action.name(),
        records[0].requested,
        records[0].offset,
        records[0].aimed
    );

    // Beat 3 — the scroll-back: from the target row, the first row's
    // ask rides the same arithmetic upward, the landing the ask
    // itself.
    let answer = harness.pipeline_mut().scroll_into_view(first);
    harness.settle();
    match answer {
        ScrollIntoViewOutcome::Landed(landing) => {
            println!(
                "beat 3: the first row's ask landed — offset {} back at the range's floor",
                landing.offset
            );
            assert_eq!(landing.offset, 0.0);
        }
        other => panic!("the first row's ask must land, got {other:?}"),
    }
    assert_eq!(handle.offset(), 0.0, "the container holds the landing");
    let records = drain_scroll_records(&trail);
    assert_eq!(records.len(), 1, "one ask, one record on the way back too");

    // Beat 4 — the pixel evidence: the top row's fill is back where
    // the first frame put it, the offset's own answer read from the
    // software raster.
    let pixel = probe_pixel(&mut harness, 30, 30);
    assert_eq!(pixel, painted(TOP_COLOR), "the first row's fill is home");
    println!("beat 4: the pixel at the first row's slot is its own fill again");

    // Beat 5 — the honest refusal: the ask aimed at a node no tree
    // node carries refuses as data, outside the scroll tier, and
    // streams no record (the ADR-0046 rule the scroll tier rides).
    let answer = harness
        .pipeline_mut()
        .scroll_into_view(alacer_core::NodeId::INVALID);
    println!("beat 5: the dead id refuses as data — {answer:?}, no record");
    assert!(matches!(
        answer,
        ScrollIntoViewOutcome::Refused(alacer_core::passes::ScrollIntoViewRefusal::UnknownNode)
    ));
    assert_eq!(
        drain_scroll_records(&trail).len(),
        0,
        "the dead id's refusal streams no record"
    );

    harness.assert_all_named();
    println!("== all five beats landed ==");
}
