//! The funnel's wall is a named fact (§13.3, ADR-0055, ADR-0053's
//! second kill rule): a delivered write the range turned back at its
//! edge — the clamp, the snap, the wall that held a no-op against a
//! past ask — carries the pinned bound
//! ([`alacer_core::inspect::ValueRecord::pinned`],
//! [`alacer_core::inspect::RangeBound`]) beside the action and outcome
//! species the record already streams, never a second action species
//! (the kill rule's own words). The ask that wanted the wall exactly —
//! the M47 jumps — rides the honest absence: the species already names
//! the bound it asked for. The demo drives the real dispatch funnel
//! (`PassPipeline::dispatch_a11y_action` and
//! `PassPipeline::dispatch_key`, the exact paths the platform and the
//! screen reader ride — never a second channel) against one focused
//! `Slider` with the causal observer riding: the beats ask past the
//! walls, hold them, and refuse honestly, and the *trail* label
//! mirrors what the log streamed — the wall named where it pinned, the
//! absence honest where it never was.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example pinned_writes
//! ```
//!
//! The beats: the mid-range write (no wall, the absence riding), the
//! ask past the max (`asked 150, landed 100, pinned at the max`), the
//! wall that held the second ask (the no-op delivery answering like
//! the landed write it is), the ask past the min, the jump whose ask
//! *was* the wall (the species names it, the pin absent), the step the
//! wall held, and the two honest refusals (a write that moved nothing
//! pinned nothing). Then the window opens for live play: a screen
//! reader driving the slider — or the arrows at the wall — plays the
//! same funnel. `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, Slider};
use alacer_core::events::{Key, KeyEvent, KeyState};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Size;
use alacer_core::inspect::{InspectEvent, InspectObserver, ValueOutcome};
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{A11yAction, EventResponse, Role};
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the value trail's
/// shape).
const VIEWPORT: Size = Size::new(480.0, 340.0);

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// The fact as the trail's reader renders it: the action beside the
/// outcome, the landed value beside a delivery, the ask beside the
/// landing, the wall the funnel pinned beside them, the control's id
/// beside everything — the log answering "what pinned the value, and
/// where" from one line. The record is `Copy`, so it passes by value
/// (the pedantic group's rule).
fn describe(record: alacer_core::ValueRecord) -> String {
    let action = record.action.name();
    let outcome = match record.outcome {
        ValueOutcome::Delivered => format!("the value landed ({})", record.value),
        ValueOutcome::Refused(refusal) => format!("refused: {}", refusal.name()),
    };
    let ask = record
        .requested
        .map_or_else(String::new, |ask| format!(" (asked {ask})"));
    let wall = record.pinned.map_or_else(String::new, |bound| {
        format!(", pinned at the {}", bound.name())
    });
    let who = record.target.map_or_else(
        || "control (unrouted)".to_string(),
        |id| format!("control {id}"),
    );
    format!("{action}: {outcome}{ask}{wall} — {who}")
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the drain rode (axiom A1: signals in, never widgets).
struct Trail {
    values: Rc<RefCell<std::collections::VecDeque<alacer_core::ValueRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The observer half: boxed into
    /// [`alacer_core::passes::PassPipeline::set_observer`], it receives
    /// every `InspectEvent`, keeps the value facts, and mirrors the
    /// newest into the trail label's signal — the write queues masks,
    /// the frame renders them.
    fn writer(&self) -> TrailWriter {
        TrailWriter {
            values: Rc::clone(&self.values),
            trail_line: self.trail_line,
        }
    }

    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<alacer_core::ValueRecord> {
        self.values.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    values: Rc<RefCell<std::collections::VecDeque<alacer_core::ValueRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::Value(record) = event else {
            return;
        };
        let line = describe(record);
        {
            let mut ring = self.values.borrow_mut();
            if ring.len() >= 64 {
                ring.pop_front();
            }
            ring.push_back(record);
        }
        self.trail_line.set(line);
    }
}

/// Ticks until no pass has work (the idle contract's demo shape, A7).
fn settle(pipeline: &mut PassPipeline) {
    for _ in 0..16 {
        let Ok(report) = pipeline.tick(VIEWPORT) else {
            return;
        };
        if report.is_idle() {
            return;
        }
    }
}

/// The slider's AccessKit-stable id, read off the real a11y snapshot —
/// the identity the dispatch funnel routes and the stamp carries (the
/// M43 vocabulary, the value trail's own route).
fn slider_id(pipeline: &PassPipeline) -> u64 {
    let tree = pipeline
        .a11y_snapshot()
        .expect("the a11y snapshot after the settle");
    let node = tree
        .find(Role::Slider, "Volume")
        .unwrap_or_else(|| panic!("the AT can reach the slider named Volume"));
    node.id
}

/// One beat on the AT route: the span names the route, the real action
/// tier answers, the write streams to the observer, and the response
/// is logged — the funnel's own route, no second channel.
fn act_beat(pipeline: &mut PassPipeline, label: &'static str, action: A11yAction) {
    let _route =
        tracing::trace_span!(target: "alacer::pinned_writes", "beat", label = label).entered();
    let id = slider_id(pipeline);
    let consumed = pipeline.dispatch_a11y_action(id, action);
    settle(pipeline);
    tracing::info!(
        target: "alacer::pinned_writes",
        consumed = consumed,
        "{label}"
    );
}

/// One beat on the key route: the same funnel, the keyboard's door —
/// the M45 stamp riding the record like the action route's does.
fn key_beat(pipeline: &mut PassPipeline, label: &'static str, key: Key) {
    let _route =
        tracing::trace_span!(target: "alacer::pinned_writes", "beat", label = label).entered();
    let response = pipeline.dispatch_key(&KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: alacer_core::events::Modifiers::default(),
        text: None,
    });
    settle(pipeline);
    tracing::info!(
        target: "alacer::pinned_writes",
        handled = matches!(response, EventResponse::Handled),
        "{label}"
    );
}

/// The labels the demo mirrors through: the value line is the app's
/// half (the beats write what the tree carries), the trail line
/// is the log's half (the observer writes it at the drain). Handlers
/// write signals, never widgets (axiom A1).
fn mount_labels(
    pipeline: &mut PassPipeline,
    root: alacer_core::NodeId,
    value_signal: alacer_core::FluxRwSignal<String>,
    trail_signal: alacer_core::FluxRwSignal<String>,
) {
    let value_binding =
        RenderBinding::new("the value: (the tree carries what the app wrote)".to_string());
    let value_label = pipeline.add_child(root, Label::bound(value_binding.clone()).boxed());
    let trail_binding = RenderBinding::new("the trail: (nothing streamed yet)".to_string());
    let trail_label = pipeline.add_child(root, Label::bound(trail_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        value_signal,
        value_label,
        &value_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );
    pipeline.flux_mut().bind_value(
        trail_signal,
        trail_label,
        &trail_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );
}

/// The beats the demo walks before the window opens: the mid-range
/// write's honest absence, the asks past both walls, the wall that
/// held, the jump whose ask was the wall, the step the wall held, and
/// the two honest refusals. Each a real action or chord through the
/// real dispatch against the focused control.
fn run_beats(pipeline: &mut PassPipeline, value_signal: alacer_core::FluxRwSignal<String>) {
    // Beat one: the mid-range write — the landing answers the ask
    // exactly, no wall pinned, the absence honest.
    act_beat(
        pipeline,
        "beat 1: SetValue 42 — the ask lands, no wall pinned",
        A11yAction::SetValue("42".to_string()),
    );
    value_signal.set("the value: 42 — the ask answered, the absence honest".to_string());

    // Beat two: the ask past the high wall — the clamp lands the
    // maximum and the record names it: asked 150, landed 100, pinned
    // at the max.
    act_beat(
        pipeline,
        "beat 2: SetValue 150 — the range turns the ask back, the max is named",
        A11yAction::SetValue("150".to_string()),
    );
    value_signal.set("the value: 100 — asked 150, pinned at the max".to_string());

    // Beat three: the wall that held — the value already at the
    // maximum, the second past ask turned back again: the no-op
    // delivery answers like the landed write it is (the M20
    // idle-frame property), the log naming the bound that held.
    act_beat(
        pipeline,
        "beat 3: SetValue 150 again — the wall held, the no-op names it",
        A11yAction::SetValue("150".to_string()),
    );
    value_signal.set("the value: 100 — the wall held the second ask".to_string());

    // Beat four: the ask past the low wall — the other edge, the other
    // species.
    act_beat(
        pipeline,
        "beat 4: SetValue -5 — the min is named",
        A11yAction::SetValue("-5".to_string()),
    );
    value_signal.set("the value: 0 — asked -5, pinned at the min".to_string());

    // Beat five: the jump whose ask *was* the wall — the species
    // already names the bound it asked for, the pin riding the honest
    // absence (the funnel never moved an ask that wanted exactly
    // this).
    key_beat(
        pipeline,
        "beat 5: End — the jump's ask is the wall, the pin rides absent",
        Key::End,
    );
    value_signal.set("the value: 100 — the jump asked for exactly this".to_string());

    // Beat six: the step the wall held — the arrow's ask past the
    // maximum turns back, the step tier's own wall named on the log
    // (the M45 key route's stamp riding beside).
    key_beat(
        pipeline,
        "beat 6: ArrowRight at the max — the step's ask turned back, the wall named",
        Key::ArrowRight,
    );
    value_signal.set("the value: 100 — the step held at the wall".to_string());

    // Beats seven and eight: the honest refusals — a write that moved
    // nothing pinned nothing: the unparsable payload and the
    // non-finite ask each name their own species, the pin absent.
    act_beat(
        pipeline,
        "beat 7: SetValue loud — the parse refuses, the pin absent",
        A11yAction::SetValue("loud".to_string()),
    );
    act_beat(
        pipeline,
        "beat 8: SetValue inf — the funnel's own wall refuses, the pin absent",
        A11yAction::SetValue("inf".to_string()),
    );
    value_signal.set("the value: 100 — the refusals moved nothing".to_string());
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The funnel's wall, named on the causal log (ADR-0055)").boxed(),
    );
    let slider = pipeline.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .value(50.0)
            .step(5.0)
            .boxed(),
    );

    let value_signal = pipeline
        .flux_mut()
        .signal("the value: (the tree carries what the app wrote)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    mount_labels(&mut pipeline, root, value_signal, trail_signal);

    // The observer rides from here on: every value write's record
    // streams into the trail label the frame after the door resolves.
    let trail = Trail {
        values: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    settle(&mut pipeline);

    // Focus the slider: the keyboard's route is focus-routed, the way
    // the platform delivers keys (the mounted node's own handle — the
    // arena's vocabulary, the same `NodeId` the AT's snapshot carries
    // as its id hash).
    pipeline.request_focus(slider);
    settle(&mut pipeline);
    run_beats(&mut pipeline, value_signal);

    // The window: live play — a screen reader driving the slider, or
    // the arrows at the wall, plays the same funnel the beats did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: ask past the walls (AT or arrows) — the trail names the pinned bound")
            .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer pinned writes",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the funnel's walls as data. The walls
    // named themselves; the absences held.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::pinned_writes",
        records = read_back.len(),
        delivered = read_back
            .iter()
            .filter(|record| record.outcome == ValueOutcome::Delivered)
            .count(),
        pinned = read_back
            .iter()
            .filter(|record| record.pinned.is_some())
            .count(),
        asked_walls = read_back
            .iter()
            .filter(|record| {
                record.pinned.is_none()
                    && record.requested.is_some_and(|ask| {
                        ask.total_cmp(&record.value) == std::cmp::Ordering::Equal
                    })
            })
            .count(),
        "the walls streamed as they pinned — the causal log carries the funnel's walls"
    );
}
