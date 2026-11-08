//! The W2 gate (§14.2, ADR-0012): text-dense relayout stays stable under
//! width jitter — **no re-wrap oscillation**.
//!
//! The mechanism under test is §8.3's integer wrap rounding: the wrap width
//! floors to whole device pixels before it reaches the measurer (egui's
//! galley-cache lesson, adopted as ALGO). The falsifiable properties:
//!
//! 1. the measurer only ever sees *integral* wrap widths;
//! 2. a width jitter inside the same integer bucket never changes the
//!    measurement (cache hit, no re-wrap);
//! 3. revisiting an already-measured wrap width costs zero measure calls —
//!    the feedback loop between available width and measured width cannot
//!    oscillate on repeat visits.

use alacer_core::arena::{Arena, NodeId};
use alacer_core::geom::Size;
use alacer_core::mask::Change;
use alacer_core::passes::LayoutEngine;
use alacer_core::style::{MeasureFn, MeasureSpec, Style, TextSpec};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_layout::LayoutHost;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// A wrap-sensitive measurer: one line (14 px) while the text fits the wrap
/// width, two lines (28 px) when it wraps. Every wrap width it sees is
/// recorded for the integrality assertion.
#[derive(Default)]
struct WrapRecorder {
    widths: Mutex<Vec<f32>>,
}

impl WrapRecorder {
    fn seen(&self) -> Vec<f32> {
        self.widths.lock().expect("recorder lock").clone()
    }
}

impl MeasureFn for WrapRecorder {
    fn measure(&self, spec: &TextSpec, max: Option<f32>) -> Size {
        if let Some(width) = max {
            self.widths.lock().expect("recorder lock").push(width);
        }
        let text_width =
            f32::from(u16::try_from(spec.text.chars().count()).unwrap_or(u16::MAX)) * 6.0;
        let wraps = max.is_some_and(|max| text_width > max);
        Size::new(text_width, if wraps { 28.0 } else { 14.0 })
    }
}

struct Section;

impl Widget for Section {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Container
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("section")
    }
    fn style(&self) -> Style {
        Style::column()
    }
}

/// A text leaf whose paragraph wraps at the section width.
struct Paragraph;

impl Widget for Paragraph {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Label
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("paragraph")
    }
    fn style(&self) -> Style {
        Style::default()
    }
    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Text(TextSpec {
            text: "word ".repeat(20).trim_end().to_string(),
            font_id: 0,
            font_size: 14.0,
        }))
    }
}

/// A section whose width the workload jitters per frame (the ±5% churn).
struct JitterSection {
    width: Rc<RefCell<f32>>,
}

impl Widget for JitterSection {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Container
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("section")
    }
    fn style(&self) -> Style {
        Style {
            size: alacer_core::style::Size2 {
                width: Some(*self.width.borrow()),
                height: None,
            },
            ..Style::column()
        }
    }
}

fn build(width: Rc<RefCell<f32>>) -> (Arena, NodeId, NodeId, NodeId) {
    let mut arena = Arena::new();
    let root = arena.set_root(Box::new(JitterSection { width }));
    let section = arena.add_child(root, Box::new(Section));
    let paragraph = arena.add_child(section, Box::new(Paragraph));
    (arena, root, section, paragraph)
}

fn viewport() -> Size {
    Size::new(800.0, 600.0)
}

#[test]
fn the_measurer_only_sees_integral_wrap_widths() {
    let recorder = Arc::new(WrapRecorder::default());
    let mut engine = LayoutHost::new(Arc::clone(&recorder) as Arc<dyn MeasureFn>);
    let mut arena = Arena::new();
    let root = arena.set_root(Box::new(Section));
    let paragraph = arena.add_child(root, Box::new(Paragraph));

    // A non-integral container width must arrive floored.
    let mut dirty = FxHashMap::default();
    dirty.insert(root, Change::STRUCTURE);
    engine
        .sync_and_layout(&mut arena, root, &dirty, Size::new(800.3, 600.0))
        .expect("layout");
    for width in recorder.seen() {
        assert_eq!(
            width,
            width.floor(),
            "wrap width {width} reached the measurer un-rounded"
        );
    }
    let _ = paragraph;
}

#[test]
fn sub_pixel_jitter_never_flips_a_line_break() {
    let recorder = Arc::new(WrapRecorder::default());
    let mut engine = LayoutHost::new(Arc::clone(&recorder) as Arc<dyn MeasureFn>);

    // The paragraph's raw text width is 120 px (20 chars × 6): any wrap
    // width ≥ 120 keeps one line. Jitter around 140 — the same integer
    // neighborhood, crossing sub-pixel boundaries every frame.
    let width = Rc::new(RefCell::new(140.0));
    let (mut arena, root, _section, paragraph) = build(Rc::clone(&width));

    let mut dirty = FxHashMap::default();
    dirty.insert(root, Change::STRUCTURE);
    engine
        .sync_and_layout(&mut arena, root, &dirty, viewport())
        .expect("initial layout");

    let heights: Vec<f32> = [140.3, 139.7, 140.9, 139.9, 140.1, 139.6]
        .iter()
        .map(|jitter| {
            *width.borrow_mut() = *jitter;
            let mut dirty = FxHashMap::default();
            dirty.insert(paragraph, Change::LAYOUT);
            engine
                .sync_and_layout(&mut arena, root, &dirty, viewport())
                .expect("jitter frame");
            arena
                .bounds(paragraph)
                .map(|bounds| bounds.size.height)
                .expect("laid out")
        })
        .collect();
    assert!(
        heights
            .windows(2)
            .all(|pair| (pair[0] - pair[1]).abs() < f32::EPSILON),
        "line count oscillated under sub-pixel width jitter: {heights:?}"
    );
}

#[test]
fn revisiting_a_wrap_width_costs_no_measure() {
    let recorder = Arc::new(WrapRecorder::default());
    let mut engine = LayoutHost::new(Arc::clone(&recorder) as Arc<dyn MeasureFn>);

    let width = Rc::new(RefCell::new(140.0));
    let (mut arena, root, section, paragraph) = build(Rc::clone(&width));

    let mut dirty = FxHashMap::default();
    dirty.insert(root, Change::STRUCTURE);
    engine
        .sync_and_layout(&mut arena, root, &dirty, viewport())
        .expect("initial layout");

    // The same width, revisited through the *same compute context* (the
    // section churns, the paragraph re-wraps under it): the integer-bucket
    // cache absorbs it — the measurer itself never runs again.
    let after_initial = recorder.seen().len();
    for _ in 0..4 {
        let mut dirty = FxHashMap::default();
        dirty.insert(section, Change::LAYOUT);
        dirty.insert(paragraph, Change::LAYOUT);
        engine
            .sync_and_layout(&mut arena, root, &dirty, viewport())
            .expect("revisit frame");
    }
    assert_eq!(
        recorder.seen().len(),
        after_initial,
        "a repeated integer wrap width must be a cache hit, not a re-measure; seen={:?}",
        recorder.seen()
    );
}
