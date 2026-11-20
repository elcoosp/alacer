//! The replay burst's cost benchmark: the price of turning the rings'
//! durable history into one wire burst (the `CausalRecorder::replay`
//! door the wire's `Replay` control serves). The ask is answered once
//! per connect — the burst's cost is a connect-time price, never a
//! per-frame one — so the budget it guards is the reconnection
//! experience, not the idle window's A7 contract.

use alacer_core::inspect::{InputKind, InputRecord, InspectEvent, InspectObserver as _};
use alacer_inspect::CausalRecorder;
use alacer_inspect::recorder::{FRAME_RING_CAP, INPUT_RING_CAP, WRITE_RING_CAP};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

/// One full ring per measured tier (the `*_RING_CAP` order): the burst
/// a panel reconnecting into a busy session reads.
const FILL: usize = 512;

/// Fills the recorder's rings with the steady-state mix: one write, one
/// routed input, and one frame per iteration (the tiers every session
/// streams continuously).
fn filled_recorder() -> CausalRecorder {
    let recorder = CausalRecorder::new("replay-bench");
    let mut handle = recorder.clone_handle();
    for i in 0..FILL {
        handle.observe(InspectEvent::Write(alacer_core::inspect::SignalWrite {
            signal: u64::try_from(i).unwrap_or_default(),
            sequence: u64::try_from(i).unwrap_or_default(),
            origin: std::borrow::Cow::Borrowed("bench"),
        }));
        handle.observe(InspectEvent::Input(InputRecord {
            kind: InputKind::PointerMove,
            target: None,
        }));
        handle.observe(InspectEvent::Frame(alacer_core::inspect::FrameSummary {
            number: u64::try_from(i).unwrap_or_default(),
            layout_ran: true,
            paint_ran: true,
            a11y_ran: false,
            focus_changed: false,
            damage: alacer_core::Damage::NONE,
            viewport: alacer_core::geom::Size::new(800.0, 600.0),
            a11y_updated: 0,
            a11y_removed: 0,
            total_nanos: 0,
            passes: Vec::new(),
            dirty: Vec::new(),
            dirty_truncated: 0,
        }));
    }
    recorder
}

fn replay_burst(c: &mut Criterion) {
    let recorder = filled_recorder();
    let burst = recorder.replay();
    // The rings' whole worth: every filled tier evicts at its own cap
    // (the frame ring is the 256-order one), and the burst names it.
    let expected = WRITE_RING_CAP.min(FILL) + INPUT_RING_CAP.min(FILL) + FRAME_RING_CAP.min(FILL);
    assert_eq!(burst.len(), expected, "the whole rings' worth, one burst");
    c.bench_function("replay/burst_512x3", |b| {
        b.iter(|| {
            let events = recorder.replay();
            black_box(events.len());
        });
    });
}

criterion_group!(benches, replay_burst);
criterion_main!(benches);
