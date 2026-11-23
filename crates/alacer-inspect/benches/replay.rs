//! The replay burst's cost benchmark: the price of turning the rings'
//! durable history into one wire burst (the `CausalRecorder::replay`
//! door the wire's `Replay` control serves). The ask is answered once
//! per connect — the burst's cost is a connect-time price, never a
//! per-frame one — so the budget it guards is the reconnection
//! experience, not the idle window's A7 contract. Since M142 the file
//! also prices the subscribe gate: the fan-out filter a gated session
//! walks over each drained broadcast batch (the ask's own addition to
//! the drain the sessions have always taken).

use alacer_core::inspect::{InputKind, InputRecord, InspectEvent, InspectObserver as _};
use alacer_inspect::CausalRecorder;
use alacer_inspect::WireEvent;
use alacer_inspect::recorder::{FRAME_RING_CAP, INPUT_RING_CAP, WRITE_RING_CAP};
use alacer_inspect::wire::{Tier, TierFilter};
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

fn replay_burst(c: &mut Criterion) {
    let recorder = filled_recorder();
    let burst = recorder.replay();
    // The rings' whole worth: every filled tier evicts at its own cap
    // (the frame ring is the 256-order one), and the burst names it.
    let expected = WRITE_RING_CAP.min(FILL) + INPUT_RING_CAP.min(FILL) + FRAME_RING_CAP.min(FILL);
    assert_eq!(burst.len(), expected, "the whole rings' worth, one burst");
    // The replay tier's own group (the corpus's group + parameter
    // naming — the ids the published baseline and the bounds sentence
    // name).
    let mut group = c.benchmark_group("replay");
    group.bench_function("burst_512x3", |b| {
        b.iter(|| {
            let events = recorder.replay();
            black_box(events.len());
        });
    });

    // The cursor ask (M140): the same filled recorder, the replay
    // filtered to the facts above the cursor — the connect-time price
    // a returning panel pays when it already carried the first half of
    // the history. The filter walks every stamped entry either way
    // (the stamps are the cursor's own currency), so the price sits
    // beside the whole burst's, never below it.
    let cursor = u64::try_from(FILL * 2).unwrap_or_default();
    let filtered = recorder.replay_after(cursor);
    assert!(
        !filtered.is_empty() && filtered.len() < burst.len(),
        "the cursor answers the delta ({} of {expected})",
        filtered.len()
    );
    group.bench_function("burst_512x3_after_1024", |b| {
        b.iter(|| {
            let events = recorder.replay_after(cursor);
            black_box(events.len());
        });
    });

    // The stats ask (M141): the occupancy answer's price — one row per
    // ring beside the fact count, read on the same filled recorder.
    // Like the burst, the ask is answered once per connect — a
    // connect-time price, never a per-frame one.
    group.bench_function("stats_512x3", |b| {
        b.iter(|| {
            let WireEvent::Stats { facts, rings } = recorder.stats() else {
                panic!("the stats answer");
            };
            black_box((facts, rings.len()));
        });
    });
    group.finish();

    // The subscribe gate (M142): the fan-out filter's price — a gated
    // session's drained broadcast batch admitted tier by tier, the
    // frames through, the writes and inputs silent. The gate rides the
    // drain the sessions have always taken (the batch is the queue's
    // own contents, the drain never priced twice); what the ask adds
    // is one tier word per event, priced here on the same filled
    // session's steady-state batch — a per-drain price bounded by the
    // queue's own 4 096 cap, never a per-frame tax on the app.
    let batch = recorder.take_broadcast();
    // The queue holds every observed fact (the caps bound the rings,
    // not the broadcast — the 4 096-order queue never spilled here):
    // one write, one input, one frame per fill iteration.
    assert_eq!(batch.len(), FILL * 3, "the queue holds the session's worth");
    let mut group = c.benchmark_group("subscribe");
    group.bench_function("broadcast_512x3_filtered", |b| {
        let gate = TierFilter::only([Tier::Frames]);
        b.iter_batched(
            || batch.clone(),
            |events| {
                let admitted = events.iter().filter(|event| gate.admits(event)).count();
                black_box(admitted);
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();

    // The tree dump (the inspector's ask-and-answer control): the
    // answer's price — the wire conversion over the retained snapshot
    // plus the serialization the send path takes, measured on a
    // 1 000-node resolved tree (the shape a real panel walks). The ask
    // is answered once per connect — a connect-time price beside the
    // burst's, never a per-frame one: the tree rides no broadcast, the
    // observer's door retains it as state, and the ask is the only
    // door it crosses the wire through.
    let recorder = CausalRecorder::new("tree-dump-bench");
    recorder.clone_handle().tree(dump_snapshot(1_000));
    let mut group = c.benchmark_group("tree_dump");
    group.throughput(criterion::Throughput::Elements(1_000));
    group.bench_function("answer_1k_nodes", |b| {
        b.iter(|| {
            let answer = alacer_inspect::wire::wire_tree(recorder.latest_tree().as_deref());
            let payload = serde_json::to_string(&answer).expect("the dump's payload");
            black_box(payload.len());
        });
    });
    group.finish();
}

/// Builds a resolved snapshot of `count` nodes: one root over the
/// members, every field named — the honest explicitness the tree
/// node's own review demands (no `Default` to mask a missed tier).
fn dump_snapshot(count: usize) -> std::sync::Arc<alacer_core::TreeSnapshot> {
    use alacer_core::{Point, Rect, Role, Size, TreeNode, TreeSnapshot};
    let root_id = 0_u64;
    let mut nodes = Vec::with_capacity(count);
    nodes.push(TreeNode {
        id: root_id,
        role: Role::Container,
        label: "window".to_string(),
        bounds: Some(Rect::new(Point::new(0.0, 0.0), Size::new(800.0, 600.0))),
        children: (1..u64::try_from(count).unwrap_or_default()).collect(),
        value: None,
        toggled: None,
        table_slot: None,
        selection: None,
        live: None,
        enabled: true,
        description: None,
        placeholder: None,
        value_range: None,
        numeric_value: None,
        secure_entry: false,
        scroll_orientation: None,
        scroll_position: None,
        secure_revealed: false,
        read_only: false,
        required: false,
        invalid: None,
        selected: None,
        multiselectable: false,
        busy: false,
        expanded: None,
        auto_complete: None,
        sort_direction: None,
        level: None,
        current: None,
        has_popup: None,
        role_description: None,
        position_in_set: None,
        size_of_set: None,
        keyboard_shortcut: None,
        multiline: false,
        language: None,
        search_field: false,
    });
    for id in 1..u64::try_from(count).unwrap_or_default() {
        nodes.push(TreeNode {
            id,
            role: Role::Button,
            label: format!("tool {id}"),
            bounds: Some(Rect::new(
                Point::new(
                    f32::from(u16::try_from(id % 40).unwrap_or_default()) * 20.0,
                    f32::from(u16::try_from(id / 40).unwrap_or_default()) * 28.0,
                ),
                Size::new(18.0, 24.0),
            )),
            children: Vec::new(),
            value: None,
            toggled: None,
            table_slot: None,
            selection: None,
            live: None,
            enabled: true,
            description: None,
            placeholder: None,
            value_range: None,
            numeric_value: None,
            secure_entry: false,
            scroll_orientation: None,
            scroll_position: None,
            secure_revealed: false,
            read_only: false,
            required: false,
            invalid: None,
            selected: None,
            multiselectable: false,
            busy: false,
            expanded: None,
            auto_complete: None,
            sort_direction: None,
            level: None,
            current: None,
            has_popup: None,
            role_description: None,
            position_in_set: None,
            size_of_set: None,
            keyboard_shortcut: None,
            multiline: false,
            language: None,
            search_field: false,
        });
    }
    std::sync::Arc::new(TreeSnapshot {
        nodes,
        root: Some(root_id),
    })
}

criterion_group!(benches, replay_burst);
criterion_main!(benches);
