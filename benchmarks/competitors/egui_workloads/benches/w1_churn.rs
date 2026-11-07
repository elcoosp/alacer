//! W1 — 10k-label tree under 10%/frame style churn (§14.2), the egui side
//! of the beat (ADR-0013). Case id: `egui/w1_churn_full_10k`.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use egui_workloads::*;
use std::time::Duration;

fn churn_full_10k(c: &mut Criterion) {
    let ctx = egui::Context::default();
    let mut frame: u64 = 0;
    let mut group = c.benchmark_group("egui");
    group.sample_size(30);
    group.measurement_time(Duration::from_millis(3000));
    group.warm_up_time(Duration::from_millis(500));
    group.throughput(Throughput::Elements((SECTIONS * LEAVES_PER_SECTION) as u64));
    group.bench_function("w1_churn_full_10k", |b| {
        b.iter(|| {
            frame += 1;
            let (primitives, shapes) = run_frame(&ctx, |ui| build_w1(ui, frame));
            std::hint::black_box(primitives);
            std::hint::black_box(shapes)
        })
    });
    group.finish();
}

criterion_group!(benches, churn_full_10k);
criterion_main!(benches);
