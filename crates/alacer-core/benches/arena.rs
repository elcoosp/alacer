//! Arena benchmarks: tree build, mask propagation, pre-order walk.

use alacer_core::arena::{Arena, NodeId};
use alacer_core::mask::Change;
use alacer_core::widget::{LabelSource, Role, Widget};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

struct Node;

impl Widget for Node {
    fn role(&self) -> Role {
        Role::Container
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("node")
    }
}

fn build_chain(depth: usize) -> (Arena, Vec<NodeId>) {
    let mut arena = Arena::new();
    let root = arena.set_root(Box::new(Node));
    let mut chain = vec![root];
    for _ in 1..depth {
        chain.push(arena.add_child(*chain.last().expect("non-empty"), Box::new(Node)));
    }
    (arena, chain)
}

fn bench_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena/build");
    for depth in [100_usize, 1_000, 10_000] {
        group.throughput(Throughput::Elements(depth as u64));
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            b.iter(|| {
                black_box(build_chain(depth));
            });
        });
    }
    group.finish();
}

fn bench_mask_propagation(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena/masks");
    for depth in [100_usize, 1_000, 10_000] {
        group.throughput(Throughput::Elements(depth as u64));
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            b.iter_batched(
                || {
                    let (arena, chain) = build_chain(depth);
                    let leaf = *chain.last().expect("non-empty");
                    (arena, chain, leaf)
                },
                |(mut arena, chain, leaf)| {
                    let _ = arena.take_masks(Change::ALL);
                    {
                        let mut guard = arena.mutate(leaf);
                        guard.request(Change::LAYOUT);
                    }
                    black_box(chain);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_pre_order(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena/walk");
    for depth in [100_usize, 1_000, 10_000] {
        group.throughput(Throughput::Elements(depth as u64));
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            b.iter_batched(
                || build_chain(depth).0,
                |arena| {
                    let mut count = 0_usize;
                    arena.for_each_pre_order(&mut |_, _| count += 1);
                    black_box(count);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_build,
    bench_mask_propagation,
    bench_pre_order
);
criterion_main!(benches);
