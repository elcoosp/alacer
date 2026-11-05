//! Keyed reconciliation benchmarks: peel/map/LIS against churn sizes.

use alacer_view::reconcile;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

fn shuffled(n: usize, rotate: usize) -> (Vec<u64>, Vec<u64>) {
    let old: Vec<u64> = (0..n as u64).collect();
    let new: Vec<u64> = old
        .iter()
        .skip(rotate)
        .chain(old.iter().take(rotate))
        .copied()
        .collect();
    (old, new)
}

fn bench_reconcile(c: &mut Criterion) {
    let mut group = c.benchmark_group("view/reconcile");
    for n in [100_usize, 1_000, 10_000] {
        group.throughput(Throughput::Elements(n as u64));
        let (old, new) = shuffled(n, n / 4);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let result = reconcile(&old, &new);
                black_box(result);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_reconcile);
criterion_main!(benches);
