//! The spin-button tier's step-route benchmark (M128, ADR-0132): the
//! widget crate's own pricing door. The published corpus prices the
//! a11y snapshot machinery (the M95 rule) — and the spin-button tier
//! grows no snapshot route, its words being the value tier's,
//! published since M3/M97/M99 — so the tier's cost is priced here, at
//! the widget doors the step route rides: the moving step (the state
//! write, the mask request, the handler call, the value log's record)
//! against the redundant ask (the M21 idle path — the arrow the wall
//! held, the write that landed where the value already was and moved
//! nothing).
//!
//! This bench rides *outside* the published corpus: the coordinated
//! rule grows the baseline at the next published cut, never at a
//! widget bench's (the m128 suite's pin documents the same shape).

use alacer_core::events::{EventCtx, Key, KeyEvent, KeyState, Modifiers};
use alacer_core::widget::Widget as _;
use alacer_widgets::SpinButton;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

/// The step pool: the fields the route cycles through, fresh per case
/// so every ask is a moving step.
const POOL: usize = 256;

/// An `ArrowUp` press on the field's key door — the keyboard route's
/// full shape, the value log's record riding the answer.
fn arrow_up(field: &mut SpinButton) {
    let mut ctx = EventCtx::new();
    let event = KeyEvent {
        key: Key::ArrowUp,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    };
    let _ = field.on_key_event(&mut ctx, &event);
    black_box(ctx.take());
}

/// The moving step: every ask lands mid-range and moves the value —
/// the write, the masks, the handler call, the record the M45 rule
/// streams through the keyboard's own door.
fn bench_step_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("spin_button");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("step_route_256", |b| {
        b.iter_batched(
            || {
                (
                    (0..POOL)
                        .map(|index| SpinButton::new(format!("Field {index}"), 0.0, 4.0))
                        .collect::<Vec<_>>(),
                    0_usize,
                )
            },
            |(mut pool, mut cursor)| {
                for _ in 0..POOL {
                    arrow_up(&mut pool[cursor]);
                    cursor += 1;
                }
                (pool.len(), cursor)
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// The redundant ask: every ask lands at the max wall — the arrow the
/// range turned back, the consumed ask that moves nothing (the M21
/// idle path priced, the step route's honest floor).
fn bench_redundant_ask(c: &mut Criterion) {
    let mut group = c.benchmark_group("spin_button");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("redundant_ask_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| SpinButton::new(format!("Field {index}"), 0.0, 4.0).value(4.0))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for field in &mut pool {
                    arrow_up(field);
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_step_route, bench_redundant_ask);
criterion_main!(benches);
