//! W2 — 1k word-wrapping labels under a ±5% width jitter (§14.2), the
//! Masonry side of the beat (ADR-0013). Case id: `masonry/w2_text_jitter_1k`.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use masonry_workloads::*;
use std::time::Duration;

fn text_jitter_1k(c: &mut Criterion) {
    let mut tree = make_w2_root();
    let mut frame: u64 = 0;
    let mut group = c.benchmark_group("masonry");
    group.sample_size(30);
    group.measurement_time(Duration::from_millis(3000));
    group.warm_up_time(Duration::from_millis(500));
    group.throughput(Throughput::Elements(PARAGRAPHS as u64));
    group.bench_function("w2_text_jitter_1k", |b| {
        b.iter(|| {
            frame += 1;
            let (scene, tree_update) = w2_frame(&mut tree, frame);
            std::hint::black_box(scene);
            std::hint::black_box(tree_update)
        })
    });
    group.finish();
}

criterion_group!(benches, text_jitter_1k);
criterion_main!(benches);
