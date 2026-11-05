//! Flux bridge benchmarks: batch coalescing and the mask drain.

use alacer_core::arena::NodeId;
use alacer_core::flux::Flux;
use alacer_core::mask::Change;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use reactive_graph::traits::Set as _;
use std::hint::black_box;

fn bench_batch_coalescing(c: &mut Criterion) {
    let mut group = c.benchmark_group("flux/batch");
    for writes in [1_u64, 10, 100, 1_000] {
        group.throughput(Throughput::Elements(writes));
        group.bench_with_input(
            BenchmarkId::from_parameter(writes),
            &writes,
            |b, &writes| {
                b.iter_batched(
                    || {
                        let mut flux = Flux::new();
                        let signal = flux.signal(0_i64);
                        flux.bind(signal, NodeId::INVALID, |_: &i64| Change::PAINT);
                        (flux, signal)
                    },
                    |(flux, signal)| {
                        flux.counters().reset();
                        flux.batch(|| {
                            for i in 0..writes {
                                signal.set(i64::try_from(i).unwrap_or_default());
                            }
                        });
                        let drained = flux.take_pending_masks();
                        black_box((flux, drained));
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_mask_drain(c: &mut Criterion) {
    let mut group = c.benchmark_group("flux/drain");
    group.throughput(Throughput::Elements(1_000));
    group.bench_function("1000_nodes_take_pending", |b| {
        let mut flux = Flux::new();
        for i in 0..1_000_i64 {
            let signal = flux.signal(i);
            flux.bind(signal, NodeId::allocate(), |_: &i64| {
                Change::PAINT.union(Change::A11Y)
            });
        }
        b.iter(|| {
            let pending = flux.take_pending_masks();
            black_box(pending);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_batch_coalescing, bench_mask_drain);
criterion_main!(benches);
