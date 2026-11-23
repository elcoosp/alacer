//! The reconnect gate's cost benchmark (M145, ADR-0149): the price of
//! the gate a session set persisting across reconnects. The ask rides
//! two connect-time doors — the greeting's `gate` field (the
//! persisted filter rendered back into the ask's vocabulary) and the
//! session's starting filter (the remembered gate the fan-out filters
//! through before the panel asks for anything) — so the budget it
//! guards is the reconnection experience, never a per-frame tax on the
//! app (the replay bench's own rule, at the gate tier).
//!
//! This bench rides *outside* the published corpus: the coordinated
//! rule grows the baseline at the next published cut, never at a bench
//! file's (the m135/m143 rule's own shape — the replay corpus's pins
//! re-agree unchanged).

use alacer_core::inspect::{InputKind, InputRecord, InspectEvent, InspectObserver as _};
use alacer_inspect::CausalRecorder;
use alacer_inspect::wire::{Tier, TierFilter};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::{Arc, Mutex};

/// One full ring per measured tier (the `*_RING_CAP` order): the batch
/// a reconnecting session's first drain filters under the gate it
/// resumed.
const FILL: usize = 512;

/// The persisted gate, shared the way the server shares it: the last
/// session's subscribe ask left behind.
fn persisted_gate() -> Arc<Mutex<TierFilter>> {
    Arc::new(Mutex::new(TierFilter::only([
        Tier::Frames,
        Tier::Writes,
        Tier::Themes,
    ])))
}

/// Fills the recorder's rings with the steady-state mix: one write, one
/// routed input, and one frame per iteration (the tiers every session
/// streams continuously — the replay bench's own filler).
fn filled_recorder() -> CausalRecorder {
    let recorder = CausalRecorder::new("gate-bench");
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
            degradation_step: alacer_core::passes::DegradationStep::None,
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

fn reconnect_gate(c: &mut Criterion) {
    let recorder = filled_recorder();
    // The queue holds the session's worth (the caps bound the rings,
    // not the broadcast — the replay bench's own rule): one write, one
    // input, one frame per fill iteration.
    let batch = Arc::new(recorder.take_broadcast());
    assert_eq!(batch.len(), FILL * 3, "the queue holds the session's worth");

    let mut group = c.benchmark_group("gate");
    group.throughput(criterion::Throughput::Elements(
        batch.len().try_into().unwrap_or_default(),
    ));

    // The greeting's field (M145): the persisted filter rendered back
    // into the ask's vocabulary — one `ask()` per connect, the three
    // remembered tiers named in the event vocabulary's order.
    group.bench_function("greeting_ask_3", |b| {
        let gate = persisted_gate();
        b.iter(|| {
            let ask = gate
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .ask();
            black_box(ask);
        });
    });

    // The reconnecting session's first drain (M145): the remembered
    // filter resumed and the steady-state batch admitted tier by tier —
    // the reconnection price the greeting's gate buys (the M142 filter
    // price beside the one clone the resume adds).
    group.bench_function("reapply_512x3_filtered", |b| {
        let gate = persisted_gate();
        b.iter(|| {
            let filter = gate
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let admitted = batch.iter().filter(|event| filter.admits(event)).count();
            black_box(admitted);
        });
    });
    group.finish();
}

criterion_group!(benches, reconnect_gate);
criterion_main!(benches);
