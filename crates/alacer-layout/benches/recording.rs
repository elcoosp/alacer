//! The recording wrapper's honesty bench (M93, ADR-0097): the wrapper
//! against the bare engine on the same incremental route — a
//! 1,000-node arena, ten percent of it dirty per frame, the persistent
//! tree re-measuring from the closest common ancestor. The published
//! `recording_wrapper_overhead` ratio budget keeps the instrument
//! honest: one record per call is the ledger's own price, the layout
//! work dominates the measurement the way the route tier's routes do.

use alacer_core::geom::Size;
use alacer_core::passes::PassPipeline;
use alacer_core::passes::{A11ySink, SceneJournal};
use alacer_core::style::{MeasureSpec, Style};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_layout::{LayoutHost, RecordingLayout};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::Arc;

const NODES: usize = 1_000;
const DIRTY: usize = 100;
const VIEWPORT: Size = Size::new(800.0, 600.0);

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

/// Assembles the route's pipeline over the given engine: 1,000 fixed
/// boxes under a column root, laid out and settled before the clock —
/// the leaf ids beside it (the dirty set's suppliers).
fn build(
    engine: Box<dyn alacer_core::passes::LayoutEngine>,
) -> (PassPipeline, Vec<alacer_core::arena::NodeId>) {
    let mut pipeline = PassPipeline::new(
        alacer_core::Flux::new(),
        engine,
        Box::new(NullJournal),
        Box::new(NullA11y),
    );
    let root = pipeline.mount_root(Box::new(TestBox {
        style: Style::column(),
    }));
    let mut ids = Vec::with_capacity(NODES);
    for _ in 0..NODES {
        ids.push(pipeline.add_child(
            root,
            Box::new(TestBox {
                style: Style::default(),
            }),
        ));
    }
    pipeline.tick(VIEWPORT).expect("layout");
    (pipeline, ids)
}

/// One measured frame: ten percent of the leaves dirty, the tick
/// driving the sync-and-layout call the ledger prices.
fn frame(pipeline: &mut PassPipeline, ids: &[alacer_core::arena::NodeId]) {
    for id in &ids[..DIRTY] {
        pipeline
            .arena_mut()
            .merge_mask(*id, alacer_core::Change::LAYOUT);
    }
    black_box(pipeline.tick(VIEWPORT).expect("layout"));
}

fn bench_recording_wrapper(c: &mut Criterion) {
    let mut group = c.benchmark_group("recording");
    group.throughput(Throughput::Elements(
        u64::try_from(DIRTY).expect("non-negative"),
    ));

    // The bare engine: the route's own cost, the instrument absent.
    group.bench_function("bare_wrapper_1k_dirty_10pct", |b| {
        let (mut pipeline, ids) = build(Box::new(LayoutHost::new(Arc::new(Measurer))));
        b.iter(|| frame(&mut pipeline, &ids));
    });

    // The wrapped engine: the same route, one record landing per call.
    group.bench_function("wrapper_1k_dirty_10pct", |b| {
        let (mut pipeline, ids) = build(Box::new(RecordingLayout::new(Box::new(LayoutHost::new(
            Arc::new(Measurer),
        )))));
        b.iter(|| frame(&mut pipeline, &ids));
    });

    group.finish();
}

criterion_group!(benches, bench_recording_wrapper);
criterion_main!(benches);
