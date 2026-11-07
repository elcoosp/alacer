//! W2 — 1k paragraphs re-wrapped under a ±5% width jitter (§14.2), the
//! egui side of the beat (ADR-0013). Case id: `egui/w2_text_jitter_1k`.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use egui_workloads::*;
use std::time::Duration;

fn text_jitter_1k(c: &mut Criterion) {
    let ctx = egui::Context::default();
    let mut frame: u64 = 0;
    let mut group = c.benchmark_group("egui");
    group.sample_size(30);
    group.measurement_time(Duration::from_millis(3000));
    group.warm_up_time(Duration::from_millis(500));
    group.throughput(Throughput::Elements(PARAGRAPHS as u64));
    group.bench_function("w2_text_jitter_1k", |b| {
        b.iter(|| {
            frame += 1;
            let (primitives, shapes) = run_frame(&ctx, |ui| build_w2(ui, frame));
            std::hint::black_box(primitives);
            std::hint::black_box(shapes)
        })
    });
    group.finish();
}

criterion_group!(benches, text_jitter_1k);
criterion_main!(benches);
