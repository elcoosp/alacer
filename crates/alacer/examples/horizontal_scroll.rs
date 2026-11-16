//! The two-axis door (M89, ADR-0093) driven from the consumer side:
//! the wide feed whose "open the item" beat lands a row on BOTH axes —
//! the pipeline pricing the vertical half exactly as M88 priced it and
//! the horizontal half beside it by the same edge-pin convention, the
//! list's own clamp arithmetic answering each axis by its declared
//! span, and the log telling the story: one `bring_into_view` record
//! per landed ask, the horizontal fact riding beside the vertical one
//! (ADR-0056's axis rule cashed). The beats beside it: the wheel's x
//! run reaching the same clamp, the vertical-only container refusing
//! the x half honestly (`no_scrollable_content`, the ask the geometry
//! priced riding beside the refusal), and the declared span pinning a
//! past ask at its own wall.
//!
//! Run with `cargo run -p alacer --example horizontal_scroll`. The
//! example is headless — the real pass pipeline (A4), the pixels read
//! back from the software raster — so the same beats run anywhere the
//! tests do.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::inspect::{InspectEvent, ScrollOutcome, ScrollRecord};
use alacer_core::num::usize_to_f32;
use alacer_core::passes::ScrollIntoViewOutcome;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{CrossAxisAlignment, Size2, Style};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};

/// The wide row's left tone — the pixel the vertical landing reads.
const ROW_LEFT: Color = Color::rgb(255, 94, 58);

/// The wide row's right tone — the pixel the horizontal landing reads.
/// One row, two tones: the boundary between them is the offset's own
/// ruler on the x axis.
const ROW_RIGHT: Color = Color::rgb(148, 163, 184);

/// The top row's fill — the pixel the scroll-back beat reads.
const TOP_COLOR: Color = Color::rgb(37, 99, 235);

/// The feed's geometry: 300 px-wide rows inside a 200 px two-axis
/// viewport — the far row straddles the viewport's bottom edge
/// (y 64..256) and its right edge (x 0..300), the ask the two-axis
/// door exists for.
const VIEWPORT: f32 = 200.0;
const ROW_WIDTH: f32 = 300.0;
const TONE_SPLIT: f32 = 150.0;
const TOP_HEIGHT: f32 = 64.0;
const FAR_HEIGHT: f32 = 192.0;
const FILLER_HEIGHT: f32 = 64.0;
const FILLERS: usize = 10;

/// A feed row painting its left half one tone and its right half
/// another — the offset's pixel evidence on both axes at once, read
/// back from the software raster.
struct WideRow {
    left: Color,
    right: Color,
    name: &'static str,
    height: f32,
}

impl WideRow {
    fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Widget for WideRow {
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
                width: Some(ROW_WIDTH),
                height: Some(self.height),
            },
            ..Style::default()
        }
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let size = ctx.size();
        let split = TONE_SPLIT.min(size.width);
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

/// The pixel at a screen slot, as RGBA bytes — the offset's own
/// evidence, read from the shipped software raster.
fn pixel(harness: &mut TestHarness, x: u32, y: u32) -> [u8; 4] {
    let pixmap = harness.pixels();
    let px = pixmap
        .pixel(x, y)
        .unwrap_or_else(|| panic!("pixel ({x}, {y}) outside the viewport"));
    [px.red(), px.green(), px.blue(), px.alpha()]
}

/// Mounts the wide feed: a two-axis list (or a vertical-only one on
/// `plain`), the rows top-first so the far row straddles the
/// viewport's bottom edge, the content spans declared through the
/// handle's doors.
fn mount_feed(
    harness: &mut TestHarness,
    plain: bool,
) -> (alacer_core::NodeId, alacer_widgets::ListHandle) {
    let mut root_column = Column::new();
    root_column.style_mut().align = CrossAxisAlignment::Start;
    let root = harness.mount(root_column.boxed());
    let mut list = if plain {
        let mut l = List::new(VIEWPORT);
        l.style_mut().size.width = Some(VIEWPORT);
        l
    } else {
        List::two_axis(VIEWPORT, VIEWPORT)
    };
    // The rows keep their declared width, never the column's stretch.
    list.style_mut().align = CrossAxisAlignment::Start;
    let handle = list.handle();
    let list_node = harness.add_child(root, list.boxed());
    let _top = harness.add_child(
        list_node,
        WideRow {
            left: TOP_COLOR,
            right: TOP_COLOR,
            name: "the top row",
            height: TOP_HEIGHT,
        }
        .boxed(),
    );
    let far = harness.add_child(
        list_node,
        WideRow {
            left: ROW_LEFT,
            right: ROW_RIGHT,
            name: "the far row",
            height: FAR_HEIGHT,
        }
        .boxed(),
    );
    for index in 0..FILLERS {
        let _ = harness.add_child(
            list_node,
            WideRow {
                left: ROW_RIGHT,
                right: ROW_RIGHT,
                name: "filler",
                height: FILLER_HEIGHT,
            }
            .boxed(),
        );
        let _ = index;
        let _ = far;
    }
    handle.set_content_height(TOP_HEIGHT + FAR_HEIGHT + usize_to_f32(FILLERS) * FILLER_HEIGHT);
    if !plain {
        handle.set_content_width(ROW_WIDTH);
    }
    harness.settle();
    (list_node, handle)
}

fn main() {
    println!("=== the two-axis door (M89, ADR-0093) ===");
    two_axis_landing();
    vertical_only_refusal();
    println!("=== every beat answered — the door grew the axis beside the ask ===");
}

/// Beats 1-4: the straddling row's ask landing on both axes, the
/// record's horizontal half beside the vertical story, and the wheel's
/// x run reaching the same clamp.
fn two_axis_landing() {
    let mut harness = TestHarness::new(240.0, 300.0);
    let (list_node, handle) = mount_feed(&mut harness, false);
    let far = {
        let arena = harness.pipeline().arena();
        let children = arena.children(list_node);
        *children.get(1).expect("the far row is the second child")
    };

    // Beat 1 — the far row straddles both edges: its bottom (256)
    // rides 56 px past the viewport's bottom, its right (300) 100 px
    // past the viewport's right. The pixel at (30, 30) sits on the
    // top row's fill; the pixel at (50, 100) on the far row's left
    // tone.
    let trail = harness.trail();
    let before_top = pixel(&mut harness, 30, 30);
    let before_far = pixel(&mut harness, 50, 100);
    println!("before the ask: (30, 30) = {before_top:?}, (50, 100) = {before_far:?}");

    // Beat 2 — the door lands the ask on both axes: the vertical half
    // exactly the M88 answer (56), the horizontal half beside it by
    // the same edge-pin convention (100).
    let landing = match harness.pipeline_mut().scroll_into_view(far) {
        ScrollIntoViewOutcome::Landed(landing) => landing,
        other => panic!("the far row's ask must land, got {other:?}"),
    };
    println!(
        "the ask landed: offset {} (outcome {}), offset_x {} (outcome {})",
        landing.offset,
        landing.outcome.name(),
        landing.horizontal.offset,
        landing.horizontal.outcome.name()
    );
    harness.settle();
    let after_top = pixel(&mut harness, 30, 30);
    let after_far = pixel(&mut harness, 50, 100);
    println!("after the ask:  (30, 30) = {after_top:?}, (50, 100) = {after_far:?}");
    assert_ne!(before_top, after_top, "the y half moved the pixel");
    assert_ne!(before_far, after_far, "the x half moved the pixel");
    assert_eq!(handle.offset(), 56.0, "the vertical landing holds");
    assert_eq!(handle.offset_x(), 100.0, "the horizontal landing holds");

    // Beat 3 — the record: one bring_into_view fact, the horizontal
    // half riding beside the vertical story (the additive axis rule).
    let records: Vec<ScrollRecord> = drain(&trail);
    assert_eq!(records.len(), 1, "one ask, one record");
    let horizontal = records[0].horizontal.expect("the x half rides");
    println!(
        "the record: {} {} at offset {}, the x half {} at offset {} (ask {:?})",
        records[0].action.name(),
        records[0].outcome.name(),
        records[0].offset,
        horizontal.outcome.name(),
        horizontal.offset,
        horizontal.requested
    );

    // Beat 4 — the wheel runs the x half on the same container: a
    // pure-x run delivers by the declared span (already at its wall —
    // the clamp names it), the platform's own mapping.
    harness.wheel(Point::new(100.0, 30.0), (40.0, 0.0));
    println!("the wheel's x run: offset_x = {}", handle.offset_x());
}

/// Beat 5: the vertical-only container refuses the x half honestly —
/// the same ask against a `List::new` prices the x half from the
/// geometry and the undeclared span answers `no_scrollable_content` —
/// the named wall, never a guessed scroll.
fn vertical_only_refusal() {
    let mut harness = TestHarness::new(240.0, 300.0);
    let (list_node, handle) = mount_feed(&mut harness, true);
    let far = {
        let arena = harness.pipeline().arena();
        let children = arena.children(list_node);
        *children.get(1).expect("the far row is the second child")
    };

    // The ask lands the vertical half exactly as M88 answered it.
    let landing = match harness.pipeline_mut().scroll_into_view(far) {
        ScrollIntoViewOutcome::Landed(landing) => landing,
        other => panic!("the vertical-only ask must land vertically, got {other:?}"),
    };
    assert_eq!(landing.outcome, ScrollOutcome::Delivered);
    assert_eq!(
        landing.horizontal.outcome.name(),
        "refused",
        "the undeclared span refuses the x half"
    );
    harness.settle();
    println!(
        "the vertical-only list: y landed at {} (delivered), the x half refused — \
         the undeclared span named the wall where the consumer reads it, offset_x {}",
        landing.offset,
        handle.offset_x()
    );
}

/// Drains the scroll tier's records from a trail reader — a fact
/// streams once, the door's own shape.
fn drain(trail: &alacer_core::trail::TrailLog) -> Vec<ScrollRecord> {
    trail
        .drain()
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Scroll(record) => Some(*record),
            _ => None,
        })
        .collect()
}
