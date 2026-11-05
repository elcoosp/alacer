//! Pass-overhead benchmarks: the §7.2 M1 gate workload.
//!
//! 10k nodes, 1% of the tree re-styled per frame through the mutate → style →
//! layout → compose → paint → accessibility contract. The companion test
//! (`pass_overhead_stays_under_the_15_percent_gate_on_10k_node_churn`)
//! asserts the bookkeeping ratio; this bench tracks the frame cost itself as
//! a baseline contract (A6: budgets-as-contracts).

use alacer_core::arena::NodeId;
use alacer_core::flux::Flux;
use alacer_core::geom::Size;
use alacer_core::mask::Change;
use alacer_core::passes::PassPipeline;
use alacer_core::style::{MeasureSpec, Style};
use alacer_core::widget::{LabelSource, Role, Widget};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rustc_hash::FxHashMap;
use std::hint::black_box;
use std::sync::{Arc, Mutex};

/// A container whose style lives in a shared cell (the churn driver).
struct ChurnBox {
    size: Size,
    style: Arc<Mutex<Style>>,
}

impl Widget for ChurnBox {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Container
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("churn")
    }

    fn style(&self) -> Style {
        self.style.lock().expect("style lock").clone()
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(self.size))
    }
}

/// The injected layout engine: stacks children vertically (deterministic
/// geometry, zero Taffy — the pipeline contract is what's under test).
struct StackLayout;

impl alacer_core::passes::LayoutEngine for StackLayout {
    fn sync_and_layout(
        &mut self,
        arena: &mut alacer_core::Arena,
        root: NodeId,
        _dirty: &FxHashMap<NodeId, Change>,
        _viewport: Size,
    ) -> Result<(), alacer_core::passes::LayoutError> {
        fn layout_subtree(
            arena: &mut alacer_core::Arena,
            node: NodeId,
            origin: alacer_core::geom::Point,
        ) {
            let size = match arena.widget(node).measure_spec() {
                Some(MeasureSpec::Fixed(size)) => size,
                _ => Size::zero(),
            };
            arena.update_bounds(node, alacer_core::geom::Rect::new(origin, size));
            let mut y = origin.y;
            for child in arena.children(node).to_vec() {
                layout_subtree(arena, child, alacer_core::geom::Point::new(origin.x, y));
                y = arena.bounds(child).map_or(y, |b| b.max_y());
            }
        }
        layout_subtree(arena, root, alacer_core::geom::Point::zero());
        Ok(())
    }

    fn measure_calls(&self) -> u64 {
        0
    }

    fn reset_measure_calls(&mut self) {}
}

#[derive(Default)]
struct NullJournal {
    layers: Mutex<FxHashMap<NodeId, Vec<alacer_core::scene::Primitive>>>,
}

impl alacer_core::passes::SceneJournal for NullJournal {
    fn commit_layer(&mut self, node: NodeId, primitives: Vec<alacer_core::scene::Primitive>) {
        self.layers
            .lock()
            .expect("journal lock")
            .insert(node, primitives);
    }

    fn drop_layer(&mut self, node: NodeId) {
        self.layers.lock().expect("journal lock").remove(&node);
    }

    fn layer(&self, node: NodeId) -> Option<Vec<alacer_core::scene::Primitive>> {
        self.layers
            .lock()
            .expect("journal lock")
            .get(&node)
            .cloned()
    }

    fn clear(&mut self) {
        self.layers.lock().expect("journal lock").clear();
    }

    fn snapshot(&self, order: &[NodeId]) -> alacer_core::scene::SceneSnapshot {
        let primitives = order
            .iter()
            .filter_map(|node| self.layers.lock().expect("journal lock").get(node).cloned())
            .flatten()
            .collect();
        alacer_core::scene::SceneSnapshot { primitives }
    }

    fn retain(&mut self, order: &[NodeId]) {
        let live: std::collections::HashSet<NodeId> = order.iter().copied().collect();
        self.layers
            .lock()
            .expect("journal lock")
            .retain(|node, _| live.contains(node));
    }
}

#[derive(Default)]
struct NullA11y;

impl alacer_core::passes::A11ySink for NullA11y {
    fn apply(
        &mut self,
        snapshot: std::sync::Arc<alacer_core::passes::TreeSnapshot>,
        delta: &alacer_core::passes::A11yDelta,
    ) {
        let _ = (snapshot, delta);
    }

    fn resolved_tree(&self) -> Option<&alacer_core::passes::TreeSnapshot> {
        None
    }
}

/// One churn driver: the container node and its shared style cell.
type ChurnCell = (NodeId, Arc<Mutex<Style>>);

/// The 10k-node pipeline (100 containers × 100 children) with timings on.
fn churn_pipeline(containers: usize, children: usize) -> (PassPipeline, Vec<ChurnCell>) {
    let mut pipeline = PassPipeline::new(
        Flux::new(),
        Box::new(StackLayout),
        Box::new(NullJournal::default()),
        Box::new(NullA11y),
    );
    pipeline.set_timings(true);
    let root = pipeline.mount_root(Box::new(ChurnBox {
        size: Size::new(400.0, 800.0),
        style: Arc::new(Mutex::new(Style::default())),
    }));
    let mut cells = Vec::new();
    for _ in 0..containers {
        let style = Arc::new(Mutex::new(Style::default()));
        let container = pipeline.add_child(
            root,
            Box::new(ChurnBox {
                size: Size::new(200.0, 4.0),
                style: Arc::clone(&style),
            }),
        );
        cells.push((container, style));
        for _ in 0..children {
            let _leaf = pipeline.add_child(
                container,
                Box::new(ChurnBox {
                    size: Size::new(2.0, 2.0),
                    style: Arc::new(Mutex::new(Style::default())),
                }),
            );
        }
    }
    let viewport = Size::new(400.0, 800.0);
    pipeline.tick(viewport).expect("first frame");
    (pipeline, cells)
}

fn bench_churn_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("pass/overhead");
    for nodes in [1_000_usize, 10_000] {
        let (containers, children) = if nodes == 1_000 {
            (10, 100)
        } else {
            (100, 100)
        };
        group.throughput(Throughput::Elements(
            u64::try_from(nodes).unwrap_or(u64::MAX),
        ));
        group.bench_with_input(BenchmarkId::from_parameter(nodes), &nodes, |b, _| {
            let (mut pipeline, cells) = churn_pipeline(containers, children);
            let viewport = Size::new(400.0, 800.0);
            let mut frame: u32 = 0;
            b.iter(|| {
                frame = frame.wrapping_add(1);
                let gap = (alacer_core::num::u32_to_f32(frame) + 1.0) * 0.1;
                for (node, cell) in &cells {
                    let cell = Arc::clone(cell);
                    let node = *node;
                    pipeline.arena_mut().queue_mutation(
                        node,
                        Box::new(move |_| {
                            cell.lock().expect("style lock").gap = gap;
                        }),
                    );
                }
                black_box(pipeline.tick(viewport).expect("churn frame"))
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_churn_tick);
criterion_main!(benches);
