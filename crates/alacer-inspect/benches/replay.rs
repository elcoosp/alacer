//! The replay burst's cost benchmark: the price of turning the rings'
//! durable history into one wire burst (the `CausalRecorder::replay`
//! door the wire's `Replay` control serves). The ask is answered once
//! per connect — the burst's cost is a connect-time price, never a
//! per-frame one — so the budget it guards is the reconnection
//! experience, not the idle window's A7 contract. Since M142 the file
//! also prices the subscribe gate: the fan-out filter a gated session
//! walks over each drained broadcast batch (the ask's own addition to
//! the drain the sessions have always taken). Since M146 it prices the
//! point ask beside the dump's: the geometry query and the wire
//! conversion the `hit_ask` control's answer takes, the same
//! connect-time rule.

use alacer_core::Role;
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

    // The state asks — the dump, the point and the word (the ask
    // family whose answer is the retained snapshot, not history):
    // priced in one helper beside the fn, the burst's and the gate's
    // groups above this one fn's shape.
    price_state_asks(c);
}

/// The state asks' price: the dump, the point, the word, the region
/// and the path — each answered once per connect (a connect-time price
/// beside the burst's, never a per-frame one: the state rides no
/// broadcast, the observer's door retains it, and the ask is the only
/// door it crosses the wire through), each measured on the same
/// 1 000-node resolved tree (the shape a real panel walks), each
/// landing asserted before the timing (the bench never pricing a
/// refusal it meant to price an answer). One fn per ask — the seven
/// groups the family holds — the shared tree the helper below builds.
fn price_state_asks(c: &mut Criterion) {
    price_dump_ask(c);
    price_point_ask(c);
    price_word_ask(c);
    price_region_ask(c);
    price_path_ask(c);
    price_descendants_ask(c);
    price_siblings_ask(c);
}

/// The tree dump's price (the inspector's ask-and-answer control): the
/// answer's price — the wire conversion over the retained snapshot plus
/// the serialization the send path takes.
fn price_dump_ask(c: &mut Criterion) {
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

/// The point ask's price (M146, ADR-0150): the geometry query over the
/// retained snapshot (the pre-order walk, the containment tests, the
/// ancestry rebuild) plus the wire conversion and serialization the
/// send path takes. The landing the bench prices: a point inside the
/// grid's own geometry (the cell at column 1, row 1 — (20, 30) rides
/// its (20..38, 28..52) span), the walk descending the root beside that
/// one cell.
fn price_point_ask(c: &mut Criterion) {
    let recorder = CausalRecorder::new("hit-ask-bench");
    recorder.clone_handle().tree(dump_snapshot(1_000));
    let answer = alacer_inspect::wire::wire_hit(recorder.latest_tree().as_deref(), [20.0, 30.0]);
    let alacer_inspect::WireEvent::Hit {
        landed: true,
        ancestry,
        ..
    } = &answer
    else {
        panic!("the grid's own point lands");
    };
    assert_eq!(
        ancestry.len(),
        2,
        "the ancestry is the root beside the one cell the point rides"
    );
    let mut group = c.benchmark_group("hit_ask");
    group.throughput(criterion::Throughput::Elements(1_000));
    group.bench_function("answer_1k_nodes", |b| {
        b.iter(|| {
            let answer =
                alacer_inspect::wire::wire_hit(recorder.latest_tree().as_deref(), [20.0, 30.0]);
            let payload = serde_json::to_string(&answer).expect("the hit's payload");
            black_box(payload.len());
        });
    });
    group.finish();
}

/// The word ask's price (M147, ADR-0151): the resolved snapshot's query
/// (the substring funnel, the role narrowing) plus the wire conversion
/// and serialization the send path takes — the needle every tool cell's
/// name carries, so the answer is the matches' whole worth.
fn price_word_ask(c: &mut Criterion) {
    let recorder = CausalRecorder::new("find-ask-bench");
    recorder.clone_handle().tree(dump_snapshot(1_000));
    let answer = alacer_inspect::wire::wire_find(
        recorder.latest_tree().as_deref(),
        "tool",
        Some(Role::Button),
    );
    let alacer_inspect::WireEvent::Find { matches, .. } = &answer else {
        panic!("the word ask answers");
    };
    assert_eq!(
        matches.len(),
        999,
        "every tool cell's name carries the needle — the matches' whole worth"
    );
    let mut group = c.benchmark_group("find_ask");
    group.throughput(criterion::Throughput::Elements(1_000));
    group.bench_function("answer_1k_nodes", |b| {
        b.iter(|| {
            let answer = alacer_inspect::wire::wire_find(
                recorder.latest_tree().as_deref(),
                "tool",
                Some(Role::Button),
            );
            let payload = serde_json::to_string(&answer).expect("the find's payload");
            black_box(payload.len());
        });
    });
    group.finish();
}

/// The region ask's price (M148, ADR-0152): the geometry query over the
/// retained snapshot (the bounds intersections, tree order) plus the
/// wire conversion and serialization the send path takes — the rect
/// covering the grid's whole frame, so the answer is the matches' whole
/// worth.
fn price_region_ask(c: &mut Criterion) {
    let recorder = CausalRecorder::new("region-ask-bench");
    recorder.clone_handle().tree(dump_snapshot(1_000));
    let answer = alacer_inspect::wire::wire_region(
        recorder.latest_tree().as_deref(),
        [0.0, 0.0, 800.0, 700.0],
    );
    let alacer_inspect::WireEvent::Region { matches, .. } = &answer else {
        panic!("the region ask answers");
    };
    assert_eq!(
        matches.len(),
        1_000,
        "the grid's whole frame touches every node — the matches' whole worth"
    );
    let mut group = c.benchmark_group("region_ask");
    group.throughput(criterion::Throughput::Elements(1_000));
    group.bench_function("answer_1k_nodes", |b| {
        b.iter(|| {
            let answer = alacer_inspect::wire::wire_region(
                recorder.latest_tree().as_deref(),
                [0.0, 0.0, 800.0, 700.0],
            );
            let payload = serde_json::to_string(&answer).expect("the region's payload");
            black_box(payload.len());
        });
    });
    group.finish();
}

/// The path ask's price (M149, ADR-0153): the ancestry query over the
/// retained snapshot (the parentage pass over the resolved nodes, the
/// chain rebuild) plus the wire conversion and serialization the send
/// path takes — the last cell of the grid, the chain the walk's own
/// two-node shape answers.
fn price_path_ask(c: &mut Criterion) {
    let recorder = CausalRecorder::new("path-ask-bench");
    recorder.clone_handle().tree(dump_snapshot(1_000));
    let answer = alacer_inspect::wire::wire_path(recorder.latest_tree().as_deref(), 999);
    let alacer_inspect::WireEvent::Path {
        landed: true,
        ancestry,
        ..
    } = &answer
    else {
        panic!("the grid cell's chain answers");
    };
    assert_eq!(
        ancestry.len(),
        2,
        "the chain is the root beside the one cell the id names"
    );
    let mut group = c.benchmark_group("path_ask");
    group.throughput(criterion::Throughput::Elements(1_000));
    group.bench_function("answer_1k_nodes", |b| {
        b.iter(|| {
            let answer = alacer_inspect::wire::wire_path(recorder.latest_tree().as_deref(), 999);
            let payload = serde_json::to_string(&answer).expect("the path's payload");
            black_box(payload.len());
        });
    });
    group.finish();
}

/// The descendant ask's price (M150, ADR-0154): the beneath query's
/// whole worth — the subtree walk over the retained snapshot, the wire
/// conversion, the serialization — the matches' whole answer, priced
/// beside the family's.
fn price_descendants_ask(c: &mut Criterion) {
    let recorder = CausalRecorder::new("descendants-ask-bench");
    recorder.clone_handle().tree(dump_snapshot(1_000));
    let answer = alacer_inspect::wire::wire_descendants(recorder.latest_tree().as_deref(), 0);
    let alacer_inspect::WireEvent::Descendants {
        landed: true,
        beneath,
        ..
    } = &answer
    else {
        panic!("the root's subtree answers");
    };
    assert_eq!(
        beneath.len(),
        999,
        "the beneath is the whole grid the root stands over — the anchor left out"
    );
    let mut group = c.benchmark_group("descendants_ask");
    group.throughput(criterion::Throughput::Elements(1_000));
    group.bench_function("answer_1k_nodes", |b| {
        b.iter(|| {
            let answer =
                alacer_inspect::wire::wire_descendants(recorder.latest_tree().as_deref(), 0);
            let payload = serde_json::to_string(&answer).expect("the descendants' payload");
            black_box(payload.len());
        });
    });
    group.finish();
}

/// The sibling ask's price (M151, ADR-0155): the beside query's whole
/// worth — the parentage pass, the row rebuild, the wire conversion,
/// the serialization — priced beside the family's.
fn price_siblings_ask(c: &mut Criterion) {
    let recorder = CausalRecorder::new("siblings-ask-bench");
    recorder.clone_handle().tree(dump_snapshot(1_000));
    let answer = alacer_inspect::wire::wire_siblings(recorder.latest_tree().as_deref(), 500);
    let alacer_inspect::WireEvent::Siblings {
        landed: true,
        beside,
        ..
    } = &answer
    else {
        panic!("the grid cell's row answers");
    };
    assert_eq!(
        beside.len(),
        998,
        "the row is the grid the cell stands in — the asked cell left out"
    );
    let mut group = c.benchmark_group("siblings_ask");
    group.throughput(criterion::Throughput::Elements(1_000));
    group.bench_function("answer_1k_nodes", |b| {
        b.iter(|| {
            let answer =
                alacer_inspect::wire::wire_siblings(recorder.latest_tree().as_deref(), 500);
            let payload = serde_json::to_string(&answer).expect("the siblings' payload");
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
        actions: alacer_core::widget::A11yActions::empty(),
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
            actions: alacer_core::widget::A11yActions::empty(),
        });
    }
    std::sync::Arc::new(TreeSnapshot {
        nodes,
        root: Some(root_id),
    })
}

criterion_group!(benches, replay_burst);
criterion_main!(benches);
