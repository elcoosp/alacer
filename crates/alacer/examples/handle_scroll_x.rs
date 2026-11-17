//! The app's own two-axis handle door (M90, ADR-0094) driven from the
//! consumer side: the wide feed whose "pan the row" beat asks the x
//! half directly — `ListHandle::scroll_to_x` beside `scroll_to`, the
//! same clamp arithmetic every door answers by, the record's
//! horizontal half carrying the ask's whole story beside the vertical
//! identity (ADR-0056's axis rule), and the walls named where the
//! consumer can read them. The beats beside it: the past run pinned at
//! the range's edge (`max`, the wall that held a no-op against a past
//! ask), and the vertical-only container refusing the x ask honestly
//! (`no_scrollable_content`).
//!
//! Run with `cargo run -p alacer --example handle_scroll_x`. The
//! example is headless — the real pass pipeline (A4), the pixels read
//! back from the software raster — so the same beats run anywhere the
//! tests do.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::inspect::{InspectEvent, ScrollOutcome, ScrollRecord, ScrollRefusal};
use alacer_core::num::usize_to_f32;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{CrossAxisAlignment, Size2, Style};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List, ListHandle};

/// The wide row's left tone — the pixel the x landing reads before the
/// ask moves it.
const ROW_LEFT: Color = Color::rgb(255, 94, 58);

/// The wide row's right tone — the pixel the x landing reads once the
/// ask lands. One row, two tones: the boundary between them is the
/// offset's own ruler on the x axis.
const ROW_RIGHT: Color = Color::rgb(148, 163, 184);

/// The feed's geometry: 300 px-wide rows inside a 200 px two-axis
/// viewport — the x span is 100 px, the ask the handle door carries.
/// The tones split at 60, so screen x 10 reads content x 10 before the
/// ask (left of the split) and content x 110 after it (right of it).
const VIEWPORT: f32 = 200.0;
const ROW_WIDTH: f32 = 300.0;
const TONE_SPLIT: f32 = 60.0;
const ROW_HEIGHT: f32 = 64.0;
const FILLERS: usize = 10;

/// A feed row painting its left half one tone and its right half
/// another — the offset's pixel evidence on the x axis, read back from
/// the software raster.
struct WideRow {
    name: &'static str,
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
                height: Some(ROW_HEIGHT),
            },
            ..Style::default()
        }
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        // The left tone: everything left of the split. The right tone:
        // everything from the split on. One row, two rulers.
        let size = ctx.size();
        let split = TONE_SPLIT.min(size.width);
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), Size::new(split, size.height)),
            color: ROW_LEFT,
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
            color: ROW_RIGHT,
            corner_radius: 0.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// Mounts the two-axis feed (or the vertical-only one): ten wide rows
/// inside the viewport, the content spans declared through the handle,
/// the cross axis keeping the rows at their own declared width.
fn mount_feed(harness: &mut TestHarness, two_axis: bool) -> (alacer_core::NodeId, ListHandle) {
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
    for index in 0..FILLERS {
        let _ = harness.add_child(list_node, WideRow { name: "row" }.boxed());
        let _ = index;
    }
    handle.set_content_height(usize_to_f32(FILLERS) * ROW_HEIGHT);
    if two_axis {
        handle.set_content_width(ROW_WIDTH);
    }
    harness.settle();
    (list_node, handle)
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

/// The trail's whole window, drained once — the scroll facts the beats
/// read.
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

fn main() {
    println!("=== the app's own two-axis handle door (M90, ADR-0094) ===");
    x_ask_lands();
    wall_pins_the_past_run();
    vertical_only_refusal();
    println!("=== every beat answered — the handle grew the x half ===");
}

/// Beats 1-3: the app's x ask landing (composed through the mutate
/// seam, the pixels flipping where the shipped renderer paints them),
/// the record's horizontal half beside the vertical identity, and the
/// offset the container holds.
fn x_ask_lands() {
    let mut harness = TestHarness::new(240.0, 300.0);
    let (list_node, handle) = mount_feed(&mut harness, true);

    // Beat 1 — before the ask: the pixel at (10, 128) sits on the wide
    // row's left tone (content x 10 is left of the 60 split).
    let before = pixel(&mut harness, 10, 128);
    println!("before the ask: (10, 128) = {before:?}");

    // Beat 2 — the app's own x ask, composed the way apps compose it:
    // the handle inside the mutate-guard `request` seam, the
    // PAINT | A11Y merge surfacing the programmatic scroll as the
    // frame work it is.
    let trail = harness.trail();
    let mut guard = harness.pipeline_mut().arena_mut().mutate(list_node);
    if let Some(list) = guard.widget_mut().as_any_mut().downcast_mut::<List>() {
        list.handle().scroll_to_x(100.0);
    }
    guard.request(alacer_core::mask::Change::PAINT.union(alacer_core::mask::Change::A11Y));
    drop(guard);
    harness.settle();

    let after = pixel(&mut harness, 10, 128);
    println!("after the ask:  (10, 128) = {after:?}");
    assert_ne!(before, after, "the x half moved the pixel");
    assert_eq!(handle.offset_x(), 100.0, "the x landing holds");
    assert_eq!(handle.offset(), 0.0, "the y axis rides the identity");

    // Beat 3 — the record: one `scroll_to` fact, the vertical fields
    // riding the identity answer, the horizontal half beside it
    // carrying the ask's whole story (the additive axis rule).
    let records: Vec<ScrollRecord> = drain(&trail);
    assert_eq!(records.len(), 1, "one ask, one record");
    let horizontal = records[0].horizontal.expect("the x half rides");
    println!(
        "the record: {} at y offset {} (identity), the x half {} at offset {} (ask {:?})",
        records[0].action.name(),
        records[0].offset,
        horizontal.outcome.name(),
        horizontal.offset,
        horizontal.requested,
    );
    assert_eq!(
        horizontal.outcome,
        ScrollOutcome::Delivered,
        "the x half's clamp delivered the ask"
    );
    assert_eq!(
        records[0].requested, None,
        "the vertical half rides the identity — no ask beside it"
    );
}

/// Beat 4: the past run pinned at the range's edge — the ask past the
/// span lands at the wall, the pin names it, and the ask that wanted
/// the wall exactly pins nothing.
fn wall_pins_the_past_run() {
    let mut harness = TestHarness::new(240.0, 300.0);
    let (_list_node, handle) = mount_feed(&mut harness, true);
    let trail = harness.trail();

    handle.scroll_to_x(500.0);
    harness.settle();
    let records = drain(&trail);
    let horizontal = records[0].horizontal.expect("the x half rides");
    println!(
        "the past run: ask 500.0 landed {} at offset {}, pinned {:?}",
        horizontal.outcome.name(),
        horizontal.offset,
        horizontal.pinned,
    );
    assert_eq!(handle.offset_x(), 100.0, "the wall held");
    assert_eq!(
        horizontal.pinned,
        Some(alacer_core::RangeBound::Max),
        "the wall that held a no-op against a past ask names itself"
    );

    handle.scroll_to_x(100.0);
    harness.settle();
    let records = drain(&trail);
    let horizontal = records[0].horizontal.expect("the x half rides");
    println!(
        "the wall ask: landed {} at offset {}, pinned {:?} — the landing is the ask",
        horizontal.outcome.name(),
        horizontal.offset,
        horizontal.pinned,
    );
    assert_eq!(
        horizontal.pinned, None,
        "the ask that wanted the wall exactly pins nothing"
    );
}

/// Beat 5: the vertical-only container refusing the x ask honestly —
/// the span was never declared, the wall is `no_scrollable_content`,
/// and the app's ask rides beside the refusal where the consumer can
/// read it.
fn vertical_only_refusal() {
    let mut harness = TestHarness::new(240.0, 300.0);
    let (_list_node, handle) = mount_feed(&mut harness, false);
    let trail = harness.trail();

    handle.scroll_to_x(50.0);
    harness.settle();
    let records = drain(&trail);
    let horizontal = records[0].horizontal.expect("the refusal rides the half");
    println!(
        "the vertical-only ask: {} (ask {:?}), the offset_x holds at {}",
        horizontal.outcome.name(),
        horizontal.requested,
        handle.offset_x(),
    );
    assert_eq!(
        horizontal.outcome,
        ScrollOutcome::Refused(ScrollRefusal::NoScrollableContent),
        "the empty span's wall names itself at the handle's x door"
    );
    assert_eq!(
        horizontal.requested,
        Some(50.0),
        "the finite ask the app priced rides beside the refusal"
    );
}
