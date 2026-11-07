//! W3 — the animation loop: 1k nodes, per-frame translation and 10% fade
//! (§14.2), the egui side of the beat (ADR-0013). Case id:
//! `egui/w3_animation_1k`.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use egui_workloads::*;
use std::time::Duration;

fn animation_1k(c: &mut Criterion) {
    let ctx = egui::Context::default();
    let mut frame: u64 = 0;
    let mut group = c.benchmark_group("egui");
    group.sample_size(30);
    group.measurement_time(Duration::from_millis(3000));
    group.warm_up_time(Duration::from_millis(500));
    group.throughput(Throughput::Elements(ANIMATED_NODES as u64));
    group.bench_function("w3_animation_1k", |b| {
        b.iter(|| {
            frame += 1;
            let (primitives, shapes) = run_frame(&ctx, |ui| build_w3(ui, frame as f32 / 60.0));
            std::hint::black_box(primitives);
            std::hint::black_box(shapes)
        })
    });
    group.finish();
}

criterion_group!(benches, animation_1k);
criterion_main!(benches);
