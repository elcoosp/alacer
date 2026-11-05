//! The W1 workload at scale (§14.2 — ADR-0008): a 10k-node tree, 10%/frame
//! style churn. `incremental` relayouts through the persistent engine's root
//! candidate; `full_rebuild` is the GPUI-style clear-and-rebuild baseline.

use alacer_core::arena::{Arena, NodeId};
use alacer_core::geom::Size;
use alacer_core::mask::Change;
use alacer_core::passes::LayoutEngine;
use alacer_core::style::{MeasureFn, MeasureSpec, Style, TextSpec};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_layout::LayoutHost;
use criterion::{Criterion, criterion_group, criterion_main};
use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};

const SECTIONS: usize = 10;
const LEAVES_PER_SECTION: usize = 1000;
const VIEWPORT: Size = Size::new(800.0, 600.0);

struct FixedMeasure;

impl MeasureFn for FixedMeasure {
    fn measure(&self, _spec: &TextSpec, _max: Option<f32>) -> Size {
        Size::new(40.0, 18.0)
    }
}

struct Column;

impl Widget for Column {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Container
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("column")
    }
    fn style(&self) -> Style {
        Style::column()
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
        Style {
            size: alacer_core::style::Size2 {
                width: Some(780.0),
                height: Some(580.0),
            },
            ..Style::column()
        }
    }
}

struct ChurnLeaf {
    cell: Arc<Mutex<Style>>,
}

impl Widget for ChurnLeaf {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Label
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("leaf")
    }
    fn style(&self) -> Style {
        self.cell.lock().expect("style cell").clone()
    }
    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Text(TextSpec {
            text: "Item".to_string(),
            font_id: 0,
            font_size: 14.0,
        }))
    }
}

/// The 10k-node tree with per-leaf style cells.
struct Workload {
    arena: Arena,
    root: NodeId,
    leaves: Vec<NodeId>,
    cells: Vec<Arc<Mutex<Style>>>,
}

fn build() -> Workload {
    let mut arena = Arena::new();
    let root = arena.set_root(Box::new(Column));
    let mut leaves = Vec::with_capacity(SECTIONS * LEAVES_PER_SECTION);
    let mut cells = Vec::with_capacity(SECTIONS * LEAVES_PER_SECTION);
    for _ in 0..SECTIONS {
        let section = arena.add_child(root, Box::new(Section));
        for _ in 0..LEAVES_PER_SECTION {
            let cell = Arc::new(Mutex::new(Style::default()));
            arena.add_child(
                section,
                Box::new(ChurnLeaf {
                    cell: Arc::clone(&cell),
                }),
            );
            leaves.push(*arena.children(section).last().expect("attached"));
            cells.push(cell);
        }
    }
    Workload {
        arena,
        root,
        leaves,
        cells,
    }
}

/// Marks one section's leaves (10% of the tree) style-churned and dirty.
fn churn(workload: &mut Workload, section_idx: usize, frame: usize) -> FxHashMap<NodeId, Change> {
    let grow = ((frame % 4) as f32) / 2.0;
    let range = section_idx * LEAVES_PER_SECTION..(section_idx + 1) * LEAVES_PER_SECTION;
    let mut dirty = FxHashMap::default();
    for idx in range {
        *workload.cells[idx].lock().expect("style cell") = Style {
            grow,
            ..Style::default()
        };
        let mut guard = workload.arena.mutate(workload.leaves[idx]);
        guard.request(Change::LAYOUT);
        dirty.insert(workload.leaves[idx], Change::LAYOUT);
    }
    dirty
}

fn everything_dirty(workload: &Workload) -> FxHashMap<NodeId, Change> {
    let sections: Vec<NodeId> = workload.arena.children(workload.root).to_vec();
    sections
        .iter()
        .chain(workload.leaves.iter())
        .map(|id| (*id, Change::STRUCTURE))
        .chain(std::iter::once((workload.root, Change::STRUCTURE)))
        .collect()
}

fn bench_w1(c: &mut Criterion) {
    let mut group = c.benchmark_group("w1");
    group.throughput(criterion::Throughput::Elements(
        u64::try_from(SECTIONS * LEAVES_PER_SECTION).expect("fits u64"),
    ));

    // Incremental: the persistent engine; each iteration churns one section.
    {
        let mut workload = build();
        let mut engine = LayoutHost::new(Arc::new(FixedMeasure));
        let first = everything_dirty(&workload);
        engine
            .sync_and_layout(&mut workload.arena, workload.root, &first, VIEWPORT)
            .expect("initial");
        let mut frame = 0_usize;
        group.bench_function("incremental_churn_10pct", |b| {
            b.iter(|| {
                let dirty = churn(&mut workload, frame % SECTIONS, frame);
                frame += 1;
                engine
                    .sync_and_layout(&mut workload.arena, workload.root, &dirty, VIEWPORT)
                    .expect("incremental frame");
            })
        });
    }

    // Full rebuild: a fresh engine + everything dirty, every iteration.
    group.bench_function("full_rebuild", |b| {
        b.iter(|| {
            let mut workload = build();
            let dirty = everything_dirty(&workload);
            let mut engine = LayoutHost::new(Arc::new(FixedMeasure));
            engine
                .sync_and_layout(&mut workload.arena, workload.root, &dirty, VIEWPORT)
                .expect("full relayout");
        })
    });

    group.finish();
}

criterion_group!(benches, bench_w1);
criterion_main!(benches);
