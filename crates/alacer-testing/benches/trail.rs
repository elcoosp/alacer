//! The trail benchmark (§13.3, ADR-0069): the provided reader priced on
//! the route it serves. The announce route is the reader's delivery
//! vehicle — the same shape the `announce` bench runs (100 speech acts
//! over a 1,000-node arena), with the [`TrailLog`] installed as the
//! pipeline's observer and the drain closing each beat. The published
//! budget (`trail_reader_overhead`, ≤ 2× the unobserved
//! `announce/route_100_announcements_1k`) keeps the reader honest: the
//! trail rides the observe branch an installed observer already pays,
//! the ring's push is a deque append, and the drain hands the window
//! over — never a re-price of the route itself. The ring case prices
//! the reader's own lifecycle: fill to cap (the oldest evicted, the
//! eviction counted) and drain, per 1,024 facts.
//!
//! The bench lives in the harness crate because that is where the whole
//! pipeline is assembled with the real widget set (the DAG's harness
//! layer drives every surface).

use alacer_core::announce::Politeness;
use alacer_core::inspect::{InputKind, InputRecord, InspectEvent, InspectObserver, Tee};
use alacer_core::trail::TrailLog;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label, Panel};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const ROUTES: usize = 100;
const FILLER: usize = 1000;
const PANEL_HEIGHT: f32 = 0.4;
const RING: usize = 1024;

fn bench_trail_reader(c: &mut Criterion) {
    let mut group = c.benchmark_group("trail");
    group.throughput(Throughput::Elements(
        u64::try_from(ROUTES).expect("non-negative"),
    ));

    // The reader installed on the real route: the announce bench's
    // shape with the harness's trail door, the drain as the beat's
    // read — the facts the routes streamed this beat, handed over
    // oldest first.
    group.bench_function("reader_announce_100_1k", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        let trail = harness.trail();
        let root = harness.mount(Column::new().boxed());
        let _ = harness.add_child(root, Label::new("The announcements").boxed());
        for i in 0..FILLER {
            let _ = harness.add_child(
                root,
                Panel::new(PANEL_HEIGHT, alacer_widgets::theme::PANEL).boxed(),
            );
            let _ = i;
        }
        harness.settle();

        let texts: Vec<String> = (0..ROUTES).map(|i| format!("Saved {i} items")).collect();
        b.iter(|| {
            for (i, text) in texts.iter().enumerate() {
                let politeness = if i % 2 == 0 {
                    Politeness::Polite
                } else {
                    Politeness::Assertive
                };
                let accepted = harness.pipeline_mut().announce(text.clone(), politeness);
                black_box(accepted);
                let report = harness.tick();
                black_box(report);
            }
            let events = trail.drain();
            black_box(events.len());
        });
    });

    group.finish();

    // The reader's own lifecycle: fill to cap — the oldest evicted, the
    // eviction counted — and the drain, per 1,024 facts.
    let mut ring = c.benchmark_group("trail_ring");
    ring.throughput(Throughput::Elements(
        u64::try_from(RING).expect("non-negative"),
    ));
    ring.bench_function("fill_drain_1024", |b| {
        let mut trail = TrailLog::with_cap(RING);
        b.iter(|| {
            for _ in 0..RING {
                trail.observe(InspectEvent::Input(InputRecord {
                    kind: InputKind::PointerMove,
                    target: None,
                }));
            }
            let events = trail.drain();
            black_box(events.len());
        });
    });
    ring.finish();

    // The tee's fan-out (ADR-0070): two readers behind the slot the
    // announce route already prices — the same shape the reader case
    // runs, the second door's observation priced beside the first's.
    let mut tee_group = c.benchmark_group("trail");
    tee_group.throughput(Throughput::Elements(
        u64::try_from(ROUTES).expect("non-negative"),
    ));
    tee_group.bench_function("tee_announce_100_1k", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        let first = TrailLog::new();
        let second = TrailLog::new();
        harness.pipeline_mut().set_observer(Box::new(Tee::new(
            Box::new(first.clone()),
            Box::new(second.clone()),
        )));
        let root = harness.mount(Column::new().boxed());
        let _ = harness.add_child(root, Label::new("The announcements").boxed());
        for i in 0..FILLER {
            let _ = harness.add_child(
                root,
                Panel::new(PANEL_HEIGHT, alacer_widgets::theme::PANEL).boxed(),
            );
            let _ = i;
        }
        harness.settle();

        let texts: Vec<String> = (0..ROUTES).map(|i| format!("Saved {i} items")).collect();
        b.iter(|| {
            for (i, text) in texts.iter().enumerate() {
                let politeness = if i % 2 == 0 {
                    Politeness::Polite
                } else {
                    Politeness::Assertive
                };
                let accepted = harness.pipeline_mut().announce(text.clone(), politeness);
                black_box(accepted);
                let report = harness.tick();
                black_box(report);
            }
            let first_events = first.drain();
            let second_events = second.drain();
            black_box(first_events.len() + second_events.len());
        });
    });
    tee_group.finish();
}

criterion_group!(benches, bench_trail_reader);
criterion_main!(benches);
