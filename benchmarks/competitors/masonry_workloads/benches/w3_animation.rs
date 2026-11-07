//! W3 — the animation loop: 1k nodes under per-frame translation (§14.2),
//! the Masonry side of the beat (ADR-0013). Case id:
//! `masonry/w3_animation_1k`.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use masonry_workloads::*;
use std::time::Duration;

fn animation_1k(c: &mut Criterion) {
    let mut tree = make_w3_root();
    let mut frame: u64 = 0;
    let mut group = c.benchmark_group("masonry");
    group.sample_size(30);
    group.measurement_time(Duration::from_millis(3000));
    group.warm_up_time(Duration::from_millis(500));
    group.throughput(Throughput::Elements(ANIMATED_NODES as u64));
    group.bench_function("w3_animation_1k", |b| {
        b.iter(|| {
            frame += 1;
            let (scene, tree_update) = w3_frame(&mut tree, frame);
            std::hint::black_box(scene);
            std::hint::black_box(tree_update)
        })
    });
    group.finish();
}

criterion_group!(benches, animation_1k);
criterion_main!(benches);
