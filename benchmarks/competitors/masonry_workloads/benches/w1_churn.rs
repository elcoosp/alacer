//! W1 — 10k-leaf tree under 10%/frame style churn (§14.2), the Masonry
//! side of the beat (ADR-0013). Case id: `masonry/w1_churn_10k`.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use masonry_workloads::*;
use std::time::Duration;

fn churn_10k(c: &mut Criterion) {
    let mut tree = make_w1_root();
    let mut frame: u64 = 0;
    let mut group = c.benchmark_group("masonry");
    group.sample_size(30);
    group.measurement_time(Duration::from_millis(3000));
    group.warm_up_time(Duration::from_millis(500));
    group.throughput(Throughput::Elements((SECTIONS * LEAVES_PER_SECTION) as u64));
    group.bench_function("w1_churn_10k", |b| {
        b.iter(|| {
            frame += 1;
            let (scene, tree_update) = w1_frame(&mut tree, frame);
            std::hint::black_box(scene);
            std::hint::black_box(tree_update)
        })
    });
    group.finish();
}

criterion_group!(benches, churn_10k);
criterion_main!(benches);
