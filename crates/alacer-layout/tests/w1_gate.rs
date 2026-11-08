//! The W1 exit gate (§14.2, §18 — ADR-0008): the 10k-node build + 10%/frame
//! style-churn workload. Median incremental relayout must beat a full
//! relayout of the same tree by at least 2× (the design floor; 5× is the
//! milestone target). The §8.4 equivalence oracle guards coherence at scale.

use alacer_core::arena::{Arena, NodeId};
use alacer_core::geom::Size;
use alacer_core::mask::Change;
use alacer_core::passes::LayoutEngine;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_layout::LayoutHost;
use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};

const SECTIONS: usize = 10;
const LEAVES_PER_SECTION: usize = 1000;
const FRAMES: usize = 40;
const VIEWPORT: Size = Size::new(800.0, 600.0);

/// A plain column container (the root).
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

/// A fixed-size section: its box does not depend on its children, so leaf
/// churn inside it never invalidates the outer tree — the canonical list
/// churn (§8.2 item 4's workhorse shape).
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

/// A text leaf whose style churns through a shared cell.
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

/// Fixed-size stand-in measurer: the W1 workload isolates the engine, not
/// the shaper (every leaf measures identically, so the measure cache hits).
struct FixedMeasure;

impl alacer_core::style::MeasureFn for FixedMeasure {
    fn measure(&self, _spec: &TextSpec, _max_width: Option<f32>) -> alacer_core::geom::Size {
        alacer_core::geom::Size::new(40.0, 18.0)
    }
}

/// The built tree: arena, root, sections, leaves, and the leaves' style cells.
type Tree = (
    Arena,
    NodeId,
    Vec<NodeId>,
    Vec<NodeId>,
    Vec<Arc<Mutex<Style>>>,
);

/// Builds the 10k-node tree: root column → 10 sections × 1000 leaves, one
/// style cell per leaf (the churn surface).
fn build_tree() -> Tree {
    let mut arena = Arena::new();
    let root = arena.set_root(Box::new(Column));
    let mut sections = Vec::with_capacity(SECTIONS);
    let mut leaves = Vec::with_capacity(SECTIONS * LEAVES_PER_SECTION);
    let mut cells = Vec::with_capacity(SECTIONS * LEAVES_PER_SECTION);
    for _ in 0..SECTIONS {
        let section = arena.add_child(root, Box::new(Section));
        sections.push(section);
        for _ in 0..LEAVES_PER_SECTION {
            let cell = Arc::new(Mutex::new(Style::default()));
            arena.add_child(
                section,
                Box::new(ChurnLeaf {
                    cell: Arc::clone(&cell),
                }),
            );
            let leaf = *arena.children(section).last().expect("attached");
            leaves.push(leaf);
            cells.push(cell);
        }
    }
    (arena, root, sections, leaves, cells)
}

/// Churns one section's leaf styles and marks them LAYOUT-dirty; returns the
/// dirty map the engine consumes.
fn churn(
    arena: &mut Arena,
    leaves: &[NodeId],
    cells: &[Arc<Mutex<Style>>],
    grow: f32,
) -> FxHashMap<NodeId, Change> {
    let mut dirty = FxHashMap::default();
    for (idx, leaf) in leaves.iter().enumerate() {
        *cells[idx].lock().expect("style cell") = Style {
            grow,
            ..Style::default()
        };
        let mut guard = arena.mutate(*leaf);
        guard.request(Change::LAYOUT);
        dirty.insert(*leaf, Change::LAYOUT);
    }
    dirty
}

fn median(values: &mut [u128]) -> u128 {
    values.sort_unstable();
    values[values.len() / 2]
}

/// The grow value a section carries after the churn run: the style of its
/// last churned frame (later frames never touch it again).
fn final_grow_for_section(section: usize) -> f32 {
    let last_frame = (0..FRAMES)
        .rev()
        .find(|frame| frame % SECTIONS == section)
        .expect("every section churns at least once");
    f32::from(u8::try_from(last_frame % 4).expect("small")) / 2.0
}

#[test]
fn w1_incremental_relayout_beats_full_rebuild_by_the_design_floor() {
    // The incremental engine: persistent tree, 10% churn per frame.
    let (mut inc_arena, inc_root, inc_sections, inc_leaves, inc_cells) = build_tree();
    assert_eq!(
        inc_leaves.len(),
        SECTIONS * LEAVES_PER_SECTION,
        "10k leaves"
    );

    let mut inc = LayoutHost::new(Arc::new(FixedMeasure));
    let first: FxHashMap<NodeId, Change> = inc_sections
        .iter()
        .chain(inc_leaves.iter())
        .map(|id| (*id, Change::STRUCTURE))
        .chain(std::iter::once((inc_root, Change::STRUCTURE)))
        .collect();
    inc.sync_and_layout(&mut inc_arena, inc_root, &first, VIEWPORT)
        .expect("initial layout");

    let mut inc_times = Vec::with_capacity(FRAMES);
    for frame in 0..FRAMES {
        let active = frame % SECTIONS;
        let range = active * LEAVES_PER_SECTION..(active + 1) * LEAVES_PER_SECTION;
        let grow = f32::from(u8::try_from(frame % 4).expect("small")) / 2.0;
        let dirty = churn(
            &mut inc_arena,
            &inc_leaves[range.clone()],
            &inc_cells[range],
            grow,
        );
        let began = std::time::Instant::now();
        inc.sync_and_layout(&mut inc_arena, inc_root, &dirty, VIEWPORT)
            .expect("incremental frame");
        inc_times.push(began.elapsed().as_nanos());
    }

    // The full-relayout baseline: per frame, a fresh engine over the same
    // tree shape with everything dirty — the GPUI-style clear-and-rebuild.
    let mut full_times = Vec::with_capacity(FRAMES);
    for frame in 0..FRAMES {
        let (mut full_arena, full_root, full_sections, full_leaves, full_cells) = build_tree();
        let began = std::time::Instant::now();
        for (section_idx, section) in full_sections.iter().enumerate() {
            let range = section_idx * LEAVES_PER_SECTION..(section_idx + 1) * LEAVES_PER_SECTION;
            let grow = f32::from(u8::try_from((frame + section_idx) % 4).expect("small")) / 2.0;
            for cell in &full_cells[range] {
                *cell.lock().expect("style cell") = Style {
                    grow,
                    ..Style::default()
                };
            }
            let _ = section;
        }
        let all_dirty: FxHashMap<NodeId, Change> = full_sections
            .iter()
            .chain(full_leaves.iter())
            .map(|id| (*id, Change::STRUCTURE))
            .chain(std::iter::once((full_root, Change::STRUCTURE)))
            .collect();
        let mut host = LayoutHost::new(Arc::new(FixedMeasure));
        host.sync_and_layout(&mut full_arena, full_root, &all_dirty, VIEWPORT)
            .expect("full relayout");
        full_times.push(began.elapsed().as_nanos());
    }

    // The §8.4 oracle at scale: a full relayout of the same *final* styles
    // must agree exactly with the churned incremental engine.
    let (mut oracle_arena, oracle_root, oracle_sections, oracle_leaves, oracle_cells) =
        build_tree();
    for section_idx in 0..SECTIONS {
        let range = section_idx * LEAVES_PER_SECTION..(section_idx + 1) * LEAVES_PER_SECTION;
        let grow = final_grow_for_section(section_idx);
        for cell in &oracle_cells[range] {
            *cell.lock().expect("style cell") = Style {
                grow,
                ..Style::default()
            };
        }
    }
    let mut oracle = LayoutHost::new(Arc::new(FixedMeasure));
    let oracle_dirty: FxHashMap<NodeId, Change> = oracle_sections
        .iter()
        .chain(oracle_leaves.iter())
        .map(|id| (*id, Change::STRUCTURE))
        .chain(std::iter::once((oracle_root, Change::STRUCTURE)))
        .collect();
    oracle
        .sync_and_layout(&mut oracle_arena, oracle_root, &oracle_dirty, VIEWPORT)
        .expect("oracle layout");
    for (inc_leaf, oracle_leaf) in inc_leaves.iter().zip(oracle_leaves.iter()) {
        assert_eq!(
            inc_arena.bounds(*inc_leaf),
            oracle_arena.bounds(*oracle_leaf),
            "the churned incremental tree equals the full relayout"
        );
    }

    let inc_median = median(&mut inc_times);
    let full_median = median(&mut full_times);
    let ratio = f64::from(u32::try_from(full_median).expect("ns fits u32"))
        / f64::from(u32::try_from(inc_median.max(1)).expect("ns fits u32"));
    println!(
        "W1 gate: inc median {inc_median} ns, full median {full_median} ns, ratio {ratio:.2}×"
    );
    assert!(
        ratio >= 2.0,
        "W1 kill criterion: incremental relayout must be ≥ 2× faster than \
         full rebuild (got {ratio:.2}×; inc {inc_median} ns vs full {full_median} ns)"
    );
}
