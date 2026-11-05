//! Text layout benchmarks: cold shape vs cached lookup.

use alacer_core::style::TextSpec;
use alacer_text::TextContext;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("text/layout");
    for words in [1_usize, 10, 100] {
        let text = (0..words)
            .map(|i| format!("word{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let spec = TextSpec {
            text: text.clone(),
            font_id: 0,
            font_size: 16.0,
        };
        group.throughput(Throughput::Bytes(text.len() as u64));

        group.bench_with_input(BenchmarkId::new("cold", words), &text, |b, text| {
            b.iter(|| {
                let mut ctx = TextContext::new();
                let layout = ctx.layout_text(text, 16.0, None);
                black_box(layout);
            });
        });

        group.bench_with_input(BenchmarkId::new("cached", words), &spec, |b, spec| {
            let mut ctx = TextContext::new();
            let _ = ctx.layout_text(&spec.text, 16.0, None);
            b.iter(|| {
                let layout = ctx.layout_text(&spec.text, 16.0, None);
                black_box(layout);
            });
        });
    }
    group.finish();
}

fn bench_measure(c: &mut Criterion) {
    let mut group = c.benchmark_group("text/measure");
    group.throughput(Throughput::Elements(1));
    group.bench_function("measure_cached", |b| {
        let mut ctx = TextContext::new();
        let spec = TextSpec {
            text: "the quick brown fox".to_string(),
            font_id: 0,
            font_size: 14.0,
        };
        let _ = ctx.measure(&spec, None);
        b.iter(|| {
            let size = ctx.measure(&spec, None);
            black_box(size);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_layout, bench_measure);
criterion_main!(benches);
