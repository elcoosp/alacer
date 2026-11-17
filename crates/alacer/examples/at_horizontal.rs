//! The AT's horizontal vocabulary (M91, ADR-0095) driven from the
//! consumer side: the wide feed whose reader walks it column by
//! column — `ScrollRight`/`ScrollLeft` beside the vertical unit
//! scrolls, the same unit contract (an item is one line, a page is
//! the viewport's width), the scroll area's own clamp arithmetic
//! answering on the declared width span, and the log telling the
//! story: one `scroll_right`/`scroll_left` record per routed ask, the
//! horizontal fact riding beside the vertical identity (ADR-0056's
//! axis rule). The beats beside it: the vertical-only container
//! refusing the horizontal ask honestly (`no_scrollable_content`),
//! and the vertical vocabulary riding untouched — the pair grew
//! beside it, never over it.
//!
//! Run with `cargo run -p alacer --example at_horizontal`. The
//! example is headless — the real pass pipeline (A4), the pixels read
//! back from the software raster — so the same beats run anywhere the
//! tests do.

use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::inspect::{InspectEvent, ScrollOutcome, ScrollRecord, ScrollRefusal};
use alacer_core::num::usize_to_f32;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{CrossAxisAlignment, Size2, Style};
use alacer_core::widget::{A11yAction, LabelSource, Role, ScrollUnit, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, List, Slider};

/// The wide row's left tone — the pixel the x landing reads before the
/// ask moves it.
const ROW_LEFT: Color = Color::rgb(255, 94, 58);

/// The wide row's right tone — the pixel the x landing reads once the
/// ask lands. One row, two tones: the boundary between them is the
/// offset's own ruler on the x axis.
const ROW_RIGHT: Color = Color::rgb(148, 163, 184);

/// The feed's geometry: 300 px-wide rows inside a 200 px two-axis
/// viewport — the x span is 100 px, the asks the AT's pair carries.
/// The tones split at 60, so screen x 10 reads content x 10 before the
/// asks and content x 110 after a 100 px walk.
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
/// inside the viewport, the content spans declared through the handle.
fn mount_feed(
    harness: &mut TestHarness,
    two_axis: bool,
) -> (alacer_core::NodeId, alacer_widgets::ListHandle) {
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
    println!("=== the AT's horizontal vocabulary (M91, ADR-0095) ===");
    at_walks_the_x_axis();
    vertical_only_refusal();
    vertical_vocabulary_untouched();
    println!("=== every beat answered — the enum grew the pair, the scroll area consumed it ===");
}

/// Beats 1-3: the AT's horizontal asks landing by their unit (an item
/// is one line, a page is the viewport's width), the record's species
/// naming the ask with the horizontal fact beside the vertical
/// identity, and the pixels flipping where the shipped renderer paints
/// them (the a11y route's PAINT | A11Y merge composing the landing).
fn at_walks_the_x_axis() {
    let mut harness = TestHarness::new(240.0, 300.0);
    let (_list_node, handle) = mount_feed(&mut harness, true);

    // Beat 1 — before the walk: the pixel at (10, 128) sits on the
    // wide row's left tone (content x 10 is left of the 60 split).
    let before = pixel(&mut harness, 10, 128);
    println!("before the walk: (10, 128) = {before:?}");

    // Beat 2 — the AT walks right by its unit: one item = one line =
    // 40 px, the wheel's own convention on the x axis too.
    let trail = harness.trail();
    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollRight(ScrollUnit::Item)
        ),
        "the scroll area consumes the AT's ScrollRight ask"
    );
    assert_eq!(handle.offset_x(), 40.0, "one item = one line = 40 px");
    assert_eq!(handle.offset(), 0.0, "the vertical offset holds");
    let records: Vec<ScrollRecord> = drain(&trail);
    let horizontal = records[0].horizontal.expect("the x half rides");
    println!(
        "the item ask: {} by {:?}, the x half {} at offset {} (ask {:?})",
        records[0].action.name(),
        records[0].unit,
        horizontal.outcome.name(),
        horizontal.offset,
        horizontal.requested,
    );
    assert_eq!(
        records[0].outcome,
        ScrollOutcome::Delivered,
        "the vertical fields ride the identity answer — the axis unasked"
    );

    // Beat 3 — one page rightward: the viewport's width (200 px), the
    // landing clamped at the declared span's edge — and the pixels
    // flipped where the shipped renderer paints them.
    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollRight(ScrollUnit::Page)
        ),
        "the scroll area consumes the AT's page ask"
    );
    assert_eq!(
        handle.offset_x(),
        100.0,
        "one page = the viewport's width, clamped at the span's edge (300 - 200)"
    );
    let after = pixel(&mut harness, 10, 128);
    println!("after the walk:  (10, 128) = {after:?}");
    assert_ne!(before, after, "the x half moved the pixel");
}

/// Beat 4: the vertical-only container refusing the AT's horizontal
/// ask honestly — the span was never declared, the wall is
/// `no_scrollable_content`, and the AT's ask rides beside the refusal
/// where the reader can read it.
fn vertical_only_refusal() {
    let mut harness = TestHarness::new(240.0, 300.0);
    let (_list_node, handle) = mount_feed(&mut harness, false);
    let trail = harness.trail();

    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollRight(ScrollUnit::Item)
        ),
        "the scroll area still consumes the ask — the refusal is the answer"
    );
    let records: Vec<ScrollRecord> = drain(&trail);
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
        "the empty span's wall names itself at the AT's horizontal ask"
    );
}

/// Beat 5: the vertical vocabulary riding untouched — the AT's
/// vertical asks never move the x half, the M89 contract re-pinned at
/// the cut that grows the pair beside them.
fn vertical_vocabulary_untouched() {
    let mut harness = TestHarness::new(240.0, 300.0);
    let (_list_node, handle) = mount_feed(&mut harness, true);
    let trail = harness.trail();

    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollDown(ScrollUnit::Item)
        ),
        "the vertical vocabulary still consumes its asks"
    );
    assert_eq!(handle.offset(), 40.0, "the vertical ask landed by its unit");
    assert_eq!(handle.offset_x(), 0.0, "the x half never moved");
    let records: Vec<ScrollRecord> = drain(&trail);
    assert_eq!(
        records[0].horizontal, None,
        "the vertical story rides its vocabulary untouched — no x half, no guessed zero"
    );

    // The value tier beside it: a slider is not a scroll area — the
    // AT's horizontal ask stays refused there (the tier boundary the
    // log prices).
    let root = harness
        .pipeline()
        .arena()
        .root()
        .expect("the root is mounted");
    let _ = harness.add_child(root, Slider::new("brightness", 0.0, 100.0).boxed());
    harness.settle();
    assert!(
        !harness.act(
            Role::Slider,
            "brightness",
            A11yAction::ScrollRight(ScrollUnit::Item)
        ),
        "the value tier refuses the horizontal pair"
    );
    println!("the value tier: the slider refused the horizontal ask — the tier boundary holds");
}
