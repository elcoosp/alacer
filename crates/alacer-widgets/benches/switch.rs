//! The switch tier's toggle-route benchmark (M130, ADR-0134): the
//! widget crate's own pricing door. The published corpus prices the
//! a11y snapshot machinery (the M95 rule) — and the switch tier grows
//! no snapshot route, the binary pair riding the `toggled` mapping the
//! sink has carried since M6 — so the tier's cost is priced here, at
//! the widget doors the toggle route rides: the moving toggle (the
//! pointer's full press-and-release, the state write, the mask request)
//! against the idle write (the programmatic write that lands where the
//! on/off already is — the honest floor of a write at the widget seam,
//! the M21 idle path).
//!
//! This bench rides *outside* the published corpus: the coordinated
//! rule grows the baseline at the next published cut, never at a widget
//! bench's (the m130 suite's pin documents the same shape).

use alacer_core::widget::Widget as _;
use alacer_widgets::Switch;
use criterion::{Criterion, criterion_group, criterion_main};

/// The switch pool: the switches the route cycles through, fresh per
/// case so every ask is a moving toggle.
const POOL: usize = 256;

/// A `Left` press-and-release at the pill — the pointer door's full
/// shape.
fn click(switch: &mut Switch) {
    use alacer_core::events::{EventCtx, MouseButton, PointerEvent};
    use alacer_core::geom::Point;
    let mut ctx = EventCtx::new();
    let down = PointerEvent::Down {
        position: Point::new(8.0, 8.0),
        button: MouseButton::Left,
        click_count: 1,
    };
    let up = PointerEvent::Up {
        position: Point::new(8.0, 8.0),
        button: MouseButton::Left,
    };
    let _ = switch.on_pointer_event(&mut ctx, &down);
    let _ = switch.on_pointer_event(&mut ctx, &up);
    std::hint::black_box(ctx.take());
}

/// The moving toggle: every route lands on an off switch and flips it —
/// the pointer's press-and-release, the state write, the mask request.
fn bench_toggle_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("switch");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("toggle_route_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Switch::new(format!("Control {index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for switch in &mut pool {
                    click(switch);
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// The idle write: every write lands where the on/off already is — the
/// write's honest floor (the not-moved answer, no masks, the M21 idle
/// path at the switch tier).
fn bench_idle_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("switch");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("idle_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Switch::new(format!("Control {index}")).on(true))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for switch in &mut pool {
                    // Every switch mounted on: the write lands where the
                    // on/off already is — the not-moved answer, the
                    // write's honest floor.
                    let _ = switch.set_on(true);
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_toggle_route, bench_idle_write);
criterion_main!(benches);
