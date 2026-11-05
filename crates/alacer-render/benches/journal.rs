//! Scene journal benchmarks: commit, replay materialization, batching.

use alacer_core::arena::NodeId;
use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::passes::SceneJournal;
use alacer_core::scene::{Primitive, Quad};
use alacer_render::{SceneJournalImpl, partition};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

fn quad(i: usize) -> Primitive {
    Primitive::Quad(Quad::filled(
        Rect::new(
            Point::new((i % 50) as f32, (i / 50) as f32),
            Size::new(20.0, 20.0),
        ),
        Color::rgb(200, 200, 200),
    ))
}

fn bench_commit_and_snapshot(c: &mut Criterion) {
    let mut group = c.benchmark_group("journal");
    for ops in [100_usize, 1_000, 10_000] {
        group.throughput(Throughput::Elements(ops as u64));
        group.bench_with_input(
            BenchmarkId::new("snapshot_1000_nodes", ops),
            &ops,
            |b, &ops| {
                b.iter_batched(
                    || {
                        let mut journal = SceneJournalImpl::new();
                        let order: Vec<NodeId> = (0..1_000).map(|_| NodeId::allocate()).collect();
                        for node in &order {
                            journal.commit_layer(*node, (0..ops / 1_000).map(quad).collect());
                        }
                        (journal, order)
                    },
                    |(journal, order)| {
                        let scene = journal.snapshot(&order);
                        let batches = partition(&scene);
                        black_box(batches);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_dirty_recommit(c: &mut Criterion) {
    let mut group = c.benchmark_group("journal");
    group.throughput(Throughput::Elements(100));
    group.bench_function("recommit_100_of_1000", |b| {
        let mut journal = SceneJournalImpl::new();
        let order: Vec<NodeId> = (0..1_000).map(|_| NodeId::allocate()).collect();
        for node in &order {
            journal.commit_layer(*node, vec![quad(0)]);
        }
        b.iter(|| {
            // The paint pass re-emits only dirty layers.
            for node in order.iter().take(100) {
                journal.commit_layer(*node, vec![quad(1)]);
            }
            let scene = journal.snapshot(&order);
            black_box(scene);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_commit_and_snapshot, bench_dirty_recommit);
criterion_main!(benches);
