//! The radio tier's select-route benchmark (M124, ADR-0128): the widget
//! crate's own pricing door. The published corpus prices the a11y
//! snapshot machinery (the M95 rule) — and the radio tier grows no
//! snapshot route, its words being the set tier's, published since M119
//! — so the tier's cost is priced here, at the widget doors the select
//! route rides: the moving select (the state write, the mask request,
//! the handler call) against the redundant ask (the M21 idle path —
//! the consumed ask that moves nothing, queues nothing, fires nothing).
//!
//! This bench rides *outside* the published corpus: the coordinated
//! rule grows the baseline at the next published cut, never at a widget
//! bench's (the m124 suite's pin documents the same shape).

use alacer_core::widget::Widget as _;
use alacer_widgets::Radio;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

/// The select pool: the members the route cycles through, fresh per
/// case so every ask is a moving select.
const POOL: usize = 256;

/// A `Left` press-and-release at the member's mark — the pointer door's
/// full shape.
fn click(radio: &mut Radio) {
    use alacer_core::events::{EventCtx, MouseButton, PointerEvent};
    use alacer_core::geom::Point;
    let mut ctx = EventCtx::new();
    let down = PointerEvent::Down {
        position: Point::new(4.0, 4.0),
        button: MouseButton::Left,
        click_count: 1,
    };
    let up = PointerEvent::Up {
        position: Point::new(4.0, 4.0),
        button: MouseButton::Left,
    };
    let _ = radio.on_pointer_event(&mut ctx, &down);
    let _ = radio.on_pointer_event(&mut ctx, &up);
    black_box(ctx.take());
}

/// The moving select: every ask lands on an unchecked member and moves
/// the selection — the write, the masks, the handler call.
fn bench_select_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("radio");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("select_route_256", |b| {
        b.iter_batched(
            || {
                (
                    (0..POOL)
                        .map(|index| Radio::new(format!("Member {index}")).position_in_set(index))
                        .collect::<Vec<_>>(),
                    0_usize,
                )
            },
            |(mut pool, mut cursor)| {
                for _ in 0..POOL {
                    click(&mut pool[cursor]);
                    cursor += 1;
                }
                (pool.len(), cursor)
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// The redundant ask: every ask lands on an already-selected member —
/// the consumed ask that moves nothing (the M21 idle path priced, the
/// select route's honest floor).
fn bench_redundant_ask(c: &mut Criterion) {
    let mut group = c.benchmark_group("radio");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("redundant_ask_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Radio::new(format!("Member {index}")).checked(true))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for radio in &mut pool {
                    click(radio);
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_select_route, bench_redundant_ask);
criterion_main!(benches);
