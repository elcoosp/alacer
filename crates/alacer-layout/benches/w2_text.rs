//! The W2 workload (§14.2, ADR-0012): text-dense relayout under width
//! jitter — 1k paragraphs whose available width wobbles ±5% per frame.
//!
//! The gate is *stability* (frame times stay flat; no re-wrap oscillation),
//! enforced by `tests/w2_gate.rs` through the integer wrap rounding (§8.3);
//! this bench measures the cost of the jitter frame. `jitter_incremental`
//! runs the persistent engine (text-node-only dirty marks); `jitter_full`
//! is the clear-and-rebuild reference denominator.

use alacer_core::arena::{Arena, NodeId};
use alacer_core::geom::Size;
use alacer_core::mask::Change;
use alacer_core::passes::LayoutEngine;
use alacer_core::style::{MeasureFn, MeasureSpec, Style, TextSpec};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_layout::LayoutHost;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

const SECTIONS: usize = 10;
const PARAGRAPHS_PER_SECTION: usize = 100;
const VIEWPORT: Size = Size::new(800.0, 600.0);

/// A deterministic wrap-sensitive measurer: text width is characters × 7,
/// height grows a line when it wraps (the same shape the real seam has).
struct ParagraphMeasure;

impl MeasureFn for ParagraphMeasure {
    fn measure(&self, spec: &TextSpec, max: Option<f32>) -> Size {
        let width = f32::from(u16::try_from(spec.text.chars().count()).unwrap_or(u16::MAX)) * 7.0;
        let wraps = max.is_some_and(|max| width > max);
        Size::new(
            width.min(max.unwrap_or(width)),
            if wraps { 28.0 } else { 14.0 },
        )
    }
}

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

struct Paragraph {
    seed: usize,
}

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
        // 60–79 words of filler: a real paragraph's shape.
        let words = 60 + (self.seed % 20);
        Some(MeasureSpec::Text(TextSpec {
            text: "word ".repeat(words).trim_end().to_string(),
            font_id: 0,
            font_size: 14.0,
        }))
    }
}

/// The 1k-paragraph workload: sections carry their own width cell so each
/// frame's jitter is independent per section.
struct Workload {
    arena: Arena,
    root: NodeId,
    sections: Vec<(NodeId, Rc<RefCell<f32>>)>,
    paragraphs: Vec<NodeId>,
}

fn build() -> Workload {
    let mut arena = Arena::new();
    let root = arena.set_root(Box::new(Section0));
    let mut sections = Vec::with_capacity(SECTIONS);
    let mut paragraphs = Vec::with_capacity(SECTIONS * PARAGRAPHS_PER_SECTION);
    for s in 0..SECTIONS {
        let width = Rc::new(RefCell::new(780.0));
        let section = arena.add_child(
            root,
            Box::new(JitterSection {
                width: Rc::clone(&width),
            }),
        );
        for i in 0..PARAGRAPHS_PER_SECTION {
            let paragraph = arena.add_child(
                section,
                Box::new(Paragraph {
                    seed: s * PARAGRAPHS_PER_SECTION + i,
                }),
            );
            paragraphs.push(paragraph);
        }
        sections.push((section, width));
    }
    Workload {
        arena,
        root,
        sections,
        paragraphs,
    }
}

struct Section0;

impl Widget for Section0 {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Container
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("root")
    }
    fn style(&self) -> Style {
        Style::column()
    }
}

/// ±5% width jitter, deterministic per frame and section (a sine-free
/// zig-zag: every frame moves a different sub-pixel distance).
fn jitter(base: f32, frame: usize, section: usize) -> f32 {
    let amplitude = base * 0.05;
    let step = (frame * 7 + section * 13) % 41;
    let offset =
        (alacer_core::num::u32_to_f32(u32::try_from(step).unwrap_or(u32::MAX)) / 40.0 - 0.5) * 2.0;
    base + amplitude * offset
}

fn mark_paragraphs(workload: &mut Workload) -> FxHashMap<NodeId, Change> {
    let mut dirty = FxHashMap::default();
    for (section, width) in &workload.sections {
        dirty.insert(*section, Change::LAYOUT);
        let _ = width;
    }
    for paragraph in &workload.paragraphs {
        dirty.insert(*paragraph, Change::LAYOUT);
    }
    dirty.insert(workload.root, Change::LAYOUT);
    dirty
}

fn bench_w2(c: &mut Criterion) {
    let mut group = c.benchmark_group("w2");
    group.throughput(Throughput::Elements(
        u64::try_from(SECTIONS * PARAGRAPHS_PER_SECTION).expect("fits u64"),
    ));

    // Incremental: the persistent engine under per-frame width jitter —
    // the workload's actual cost.
    {
        let mut workload = build();
        let mut engine = LayoutHost::new(Arc::new(ParagraphMeasure));
        let mut first = FxHashMap::default();
        first.insert(workload.root, Change::STRUCTURE);
        for (section, _) in &workload.sections {
            first.insert(*section, Change::STRUCTURE);
        }
        for paragraph in &workload.paragraphs {
            first.insert(*paragraph, Change::STRUCTURE);
        }
        engine
            .sync_and_layout(&mut workload.arena, workload.root, &first, VIEWPORT)
            .expect("initial layout");

        let mut frame = 0_usize;
        group.bench_function("jitter_incremental_1k", |b| {
            b.iter(|| {
                frame += 1;
                for (idx, (_, width)) in workload.sections.iter().enumerate() {
                    *width.borrow_mut() = jitter(780.0, frame, idx);
                }
                let dirty = mark_paragraphs(&mut workload);
                engine
                    .sync_and_layout(&mut workload.arena, workload.root, &dirty, VIEWPORT)
                    .expect("jitter frame");
            });
        });
    }

    // Full rebuild reference: a fresh engine, everything structural.
    group.bench_function("jitter_full_1k", |b| {
        b.iter(|| {
            let mut workload = build();
            let mut dirty = FxHashMap::default();
            dirty.insert(workload.root, Change::STRUCTURE);
            for (section, _) in &workload.sections {
                dirty.insert(*section, Change::STRUCTURE);
            }
            for paragraph in &workload.paragraphs {
                dirty.insert(*paragraph, Change::STRUCTURE);
            }
            let mut engine = LayoutHost::new(Arc::new(ParagraphMeasure));
            engine
                .sync_and_layout(&mut workload.arena, workload.root, &dirty, VIEWPORT)
                .expect("full relayout");
        });
    });

    group.finish();
}

criterion_group!(benches, bench_w2);
criterion_main!(benches);
