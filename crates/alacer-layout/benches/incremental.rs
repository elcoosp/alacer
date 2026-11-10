//! Mini-W1: incremental relayout vs full relayout on style churn (§14.2 W1;
//! the ≥2× gate is M2's — this bench exists from day one).

use alacer_core::flux::Flux;
use alacer_core::geom::Size;
use alacer_core::mask::Change;
use alacer_core::passes::PassPipeline;
use alacer_core::passes::{A11ySink, SceneJournal};
use alacer_core::style::{MeasureSpec, Style};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_layout::LayoutHost;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rustc_hash::FxHashMap;
use std::hint::black_box;
use std::sync::Arc;

struct Measurer;

impl alacer_core::style::MeasureFn for Measurer {
    fn measure(
        &self,
        spec: &alacer_core::style::TextSpec,
        _max: Option<f32>,
    ) -> alacer_core::geom::Size {
        alacer_core::geom::Size::new(
            f32::from(u16::try_from(spec.text.len()).unwrap_or(1)) * 6.0,
            14.0,
        )
    }
}

struct TestBox {
    style: Style,
}

impl Widget for TestBox {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Container
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("box")
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(alacer_core::geom::Size::new(24.0, 16.0)))
    }
}

struct NullJournal;
impl SceneJournal for NullJournal {
    fn commit_layer(
        &mut self,
        _n: alacer_core::arena::NodeId,
        _p: Vec<alacer_core::scene::Primitive>,
    ) {
    }
    fn drop_layer(&mut self, _n: alacer_core::arena::NodeId) {}
    fn layer(&self, _n: alacer_core::arena::NodeId) -> Option<Vec<alacer_core::scene::Primitive>> {
        None
    }
    fn snapshot(&self, _o: &[alacer_core::arena::NodeId]) -> alacer_core::scene::SceneSnapshot {
        alacer_core::scene::SceneSnapshot::default()
    }
    fn retain(&mut self, _o: &[alacer_core::arena::NodeId]) {}
    fn clear(&mut self) {}
    fn stats(&self) -> alacer_core::passes::JournalStats {
        alacer_core::passes::JournalStats::default()
    }
}

struct NullA11y;
impl A11ySink for NullA11y {
    fn apply(
        &mut self,
        _s: std::sync::Arc<alacer_core::passes::TreeSnapshot>,
        _d: &alacer_core::passes::A11yDelta,
    ) {
    }
}

fn viewport() -> Size {
    Size::new(800.0, 600.0)
}

fn build(nodes: usize) -> (PassPipeline, Vec<alacer_core::arena::NodeId>) {
    let mut pipeline = PassPipeline::new(
        Flux::new(),
        Box::new(LayoutHost::new(Arc::new(Measurer))),
        Box::new(NullJournal),
        Box::new(NullA11y),
    );
    let root = pipeline.mount_root(Box::new(TestBox {
        style: Style::column(),
    }));
    let mut ids = Vec::new();
    for _ in 0..nodes {
        ids.push(pipeline.add_child(
            root,
            Box::new(TestBox {
                style: Style::default(),
            }),
        ));
    }
    pipeline.tick(viewport()).expect("layout");
    (pipeline, ids)
}

fn bench_incremental_vs_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout/mini_w1");
    for nodes in [100_usize, 1_000, 10_000] {
        group.throughput(Throughput::Elements(nodes as u64));

        // Full: every node dirty each frame.
        group.bench_with_input(BenchmarkId::new("full", nodes), &nodes, |b, &nodes| {
            b.iter_batched(
                || build(nodes),
                |(mut pipeline, ids)| {
                    let mut dirty: FxHashMap<alacer_core::arena::NodeId, Change> =
                        ids.iter().map(|id| (*id, Change::LAYOUT)).collect();
                    dirty.insert(pipeline.arena().root().expect("root"), Change::LAYOUT);
                    pipeline.tick(viewport()).expect("layout");
                    black_box(dirty);
                },
                criterion::BatchSize::SmallInput,
            );
        });

        // Incremental: one node dirty each frame (the persistent-tree win).
        group.bench_with_input(
            BenchmarkId::new("incremental", nodes),
            &nodes,
            |b, &nodes| {
                b.iter_batched(
                    || build(nodes),
                    |(mut pipeline, ids)| {
                        for id in &ids {
                            let mut dirty = FxHashMap::default();
                            dirty.insert(*id, Change::LAYOUT);
                            pipeline.tick(viewport()).expect("layout");
                        }
                        black_box(ids);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_incremental_vs_full);
criterion_main!(benches);
