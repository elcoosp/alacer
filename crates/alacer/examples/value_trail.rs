//! The value tier rides the causal log (§13.3, ADR-0046): every
//! value-tier write the AT asks — a `SetValue`, an `Increment`, a
//! `Decrement` — streams one [`ValueRecord`], the landed value beside
//! the outcome and the refusal species naming why nothing moved. The
//! ride ADR-0010's additive rule and the M31/M33/M36/M39 rhythm
//! pre-register cashed: a status line, an inspector trail, or a test
//! routes on "did the write land — where, or why not" without
//! re-implementing the observation at the action site.
//!
//! The demo drives the real action tier (`PassPipeline::dispatch_a11y_action`,
//! the exact route the platform's AccessKit adapter rides — never a
//! second channel) against real `Slider` and `Progress` widgets with
//! the causal observer riding: the beats run before the window opens,
//! each one a real AT action through the funnel, and the *trail* label
//! mirrors what the log streamed. The refusals are the demo's point:
//! the locked slider (`unavailable`), the payload that parses to
//! nothing (`invalid_payload`), the payload the funnel cannot carry
//! (`non_finite_value`), and the reporting widget that never had a
//! write door (`no_write_door`) each stream their own species — the
//! four facts the widget's `bool` collapsed into one silence.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example value_trail
//! ```
//!
//! The beats: the write that lands, the step that lands beside it, the
//! no-op that answers without moving (the M20 idle-frame property, the
//! log naming it), the four refusals, and the live window — where a
//! screen reader driving the sliders streams the same records the
//! beats did (the keyboard's arrows ride the same funnel too, but the
//! input tier's records carry their story; the value record documents
//! the AT's tier). `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, Progress, Slider};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Size;
use alacer_core::inspect::{InspectEvent, InspectObserver, ValueAction, ValueOutcome, ValueRecord};
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{A11yAction, Role};
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the clipboard trail's
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
/// outcome, the landed value beside a delivery, the refusal species
/// beside a refusal — the tree's own value, never a secret. The record
/// is `Copy`, so it passes by value (the pedantic group's rule).
fn describe(record: ValueRecord) -> String {
    let action = record.action.name();
    let outcome = match record.outcome {
        ValueOutcome::Delivered => format!("the value landed ({})", record.value),
        ValueOutcome::Refused(refusal) => format!("refused: {}", refusal.name()),
    };
    format!("{action}: {outcome}")
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the drain rode (axiom A1: signals in, never widgets).
struct Trail {
    values: Rc<RefCell<std::collections::VecDeque<ValueRecord>>>,
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
    fn all(&self) -> Vec<ValueRecord> {
        self.values.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    values: Rc<RefCell<std::collections::VecDeque<ValueRecord>>>,
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

/// One beat: the span names the route, the real action tier answers,
/// the write streams to the observer, and the response is logged — the
/// funnel's own route, no second channel.
fn beat(
    pipeline: &mut PassPipeline,
    label: &'static str,
    target: (Role, &str),
    action: A11yAction,
) {
    let _route =
        tracing::trace_span!(target: "alacer::value_trail", "beat", label = label).entered();
    let tree = pipeline
        .a11y_snapshot()
        .expect("the a11y snapshot after the settle");
    let node = tree.find(target.0, target.1).unwrap_or_else(|| {
        panic!(
            "the AT can reach the {} named {}",
            target.0.name(),
            target.1
        )
    });
    let id = node.id;
    let consumed = pipeline.dispatch_a11y_action(id, action);
    settle(pipeline);
    tracing::info!(
        target: "alacer::value_trail",
        consumed = consumed,
        "{label}"
    );
}

/// The labels the demo mirrors through: the value line is the app's
/// half (the beats write what the tree carries), the trail line is the
/// log's half (the observer writes it at the drain). Handlers write
/// signals, never widgets (axiom A1).
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

/// The beats the demo walks before the window opens: the write that
/// lands, the step beside it, the no-op that answers without moving,
/// and the four refusals — each a real AT action through the real
/// dispatch against the widget the tree names.
fn run_beats(pipeline: &mut PassPipeline, value_signal: alacer_core::FluxRwSignal<String>) {
    // Beat one: the AT's SetValue — the funnel parses, clamps, and
    // snaps, and the landed value rides the record.
    beat(
        pipeline,
        "beat 1: SetValue 80 — the write lands, the value counted",
        (Role::Slider, "Volume"),
        A11yAction::SetValue("80".to_string()),
    );
    value_signal.set("the value: 80 — what the record says the tree carries".to_string());

    // Beat two: the step tier — one Increment, the funnel's own
    // arithmetic (the step is 5), the landed value beside it.
    beat(
        pipeline,
        "beat 2: Increment — one step, the funnel's own arithmetic",
        (Role::Slider, "Volume"),
        A11yAction::Increment,
    );
    value_signal.set("the value: 85 — the step landed".to_string());

    // Beat three: the no-op — a write that lands where the value
    // already is is an answered request, not a refusal (the M20
    // idle-frame property, now named on the log).
    beat(
        pipeline,
        "beat 3: SetValue 85 again — the no-op answers, no masks queued",
        (Role::Slider, "Volume"),
        A11yAction::SetValue("85".to_string()),
    );
    value_signal.set("the value: 85 — the no-op answered, the frame idle".to_string());

    // Beat four: the payload that parses to nothing — a SetValue never
    // invents a value from prose (`invalid_payload`).
    beat(
        pipeline,
        "beat 4: SetValue \"loud\" — the parse refusal names itself",
        (Role::Slider, "Volume"),
        A11yAction::SetValue("loud".to_string()),
    );

    // Beat five: the payload that parses but is not a finite scalar —
    // "inf" parses fine, the funnel's wall is its own fact
    // (`non_finite_value`, the m20 suite's two refusals named apart).
    beat(
        pipeline,
        "beat 5: SetValue \"inf\" — the funnel's wall names itself",
        (Role::Slider, "Volume"),
        A11yAction::SetValue("inf".to_string()),
    );
    value_signal.set("the value: 85 — both refusals moved nothing".to_string());

    // Beat six: the availability tier — the locked slider refuses
    // before the request is even parsed, the tree keeping the value the
    // app wrote (`unavailable`, M22's door).
    beat(
        pipeline,
        "beat 6: SetValue 10 on the locked slider — the availability refusal",
        (Role::Slider, "Locked"),
        A11yAction::SetValue("10".to_string()),
    );

    // Beat seven: the reporting widget — a read-only tier has no write
    // door, every AT write refused, never munged into a step (`no_write_door`,
    // ADR-0027's own rule, now a fact on the log).
    beat(
        pipeline,
        "beat 7: SetValue 50 on the progress bar — the read-only tier's refusal",
        (Role::ProgressIndicator, "Batch"),
        A11yAction::SetValue("50".to_string()),
    );
    value_signal.set("the value: the app's — every refusal moved nothing".to_string());
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The value tier's facts, streamed to the causal log (ADR-0046)").boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Slider::new("Volume", 0.0, 200.0)
            .value(0.0)
            .step(5.0)
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Slider::new("Locked", 0.0, 100.0).enabled(false).boxed(),
    );
    let _ = pipeline.add_child(root, Progress::new("Batch", 0.0, 100.0).boxed());

    let value_signal = pipeline
        .flux_mut()
        .signal("the value: (the tree carries what the app wrote)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    mount_labels(&mut pipeline, root, value_signal, trail_signal);

    // The observer rides from here on: every value write's record
    // streams into the trail label the frame after the act resolves.
    let trail = Trail {
        values: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    settle(&mut pipeline);
    run_beats(&mut pipeline, value_signal);

    // The window: live play — a screen reader driving the sliders
    // streams the same records the beats did; the keyboard's arrows
    // ride the same funnel, the input tier's records carrying their
    // story (the value record documents the AT's tier, ADR-0046).
    let _ = pipeline.add_child(
        root,
        Label::new("Live: drive the sliders with an AT — the trail mirrors the log").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer value trail",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the value tier's history as data. The
    // refusals streamed beside the deliveries; the species named why.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::value_trail",
        records = read_back.len(),
        delivered = read_back
            .iter()
            .filter(|record| record.outcome == ValueOutcome::Delivered)
            .count(),
        refusals = read_back
            .iter()
            .filter(|record| matches!(record.outcome, ValueOutcome::Refused(_)))
            .count(),
        set_values = read_back
            .iter()
            .filter(|record| record.action == ValueAction::SetValue)
            .count(),
        "the value writes streamed as they resolved — the causal log carries the value trail"
    );
}
