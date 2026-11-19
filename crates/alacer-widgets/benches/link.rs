//! The link tier's open-route benchmark (M125, ADR-0129): the widget
//! crate's own pricing door, the M124 radio bench's own shape. The
//! published corpus prices the a11y snapshot machinery (the M95 rule)
//! — and the link tier grows no snapshot route — so the tier's cost is
//! priced here, at the widget doors the open route rides: the
//! activation (the handler call, the mask request) against the wall
//! (the M22 refusal — the unavailable link's door answering before the
//! request is even parsed).
//!
//! This bench rides *outside* the published corpus: the coordinated
//! rule grows the baseline at the next published cut, never at a
//! widget bench's (the m125 suite's pin documents the same shape).

use alacer_core::widget::Widget as _;
use alacer_widgets::Link;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

/// The activation pool: the links the route cycles through, fresh per
/// case so every ask is an open.
const POOL: usize = 256;

/// A `Left` press-and-release on the link's label — the pointer door's
/// full shape.
fn click(link: &mut Link) {
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
    let _ = link.on_pointer_event(&mut ctx, &down);
    let _ = link.on_pointer_event(&mut ctx, &up);
    black_box(ctx.take());
}

/// The open route: every activation lands on an available link and
/// fires the handler — the write, the masks, the call.
fn bench_open_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("link");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("open_route_256", |b| {
        b.iter_batched(
            || {
                (
                    (0..POOL)
                        .map(|index| Link::new(format!("Link {index}")).on_open(|| {}))
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

/// The wall: every activation lands on an unavailable link — the M22
/// refusal priced (the door that answers before the request is even
/// parsed, the open route's honest floor).
fn bench_refusal_wall(c: &mut Criterion) {
    let mut group = c.benchmark_group("link");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("refusal_wall_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Link::new(format!("Link {index}")).enabled(false))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for link in &mut pool {
                    click(link);
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_open_route, bench_refusal_wall);
criterion_main!(benches);
