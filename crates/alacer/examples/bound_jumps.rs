//! The keyboard's jumps are named facts (§13.3, ADR-0053, ADR-0051's
//! kill arm): the slider's `Home`/`End` stream the bound's own named
//! species — [`alacer_core::inspect::ValueAction::JumpToMin`] and
//! [`alacer_core::inspect::ValueAction::JumpToMax`] — through the same
//! value record the AT tier streams, never a munged `SetValue` (the
//! M11 rule's own words at the value tier). The bound is the ask the
//! record carries (M44's discipline: the ask beside the landing), the
//! landing the funnel's own arithmetic, and the M45 key route stamps
//! the focused control — the jump attributed like every other fact on
//! the log. The demo drives the real dispatch funnel
//! (`PassPipeline::dispatch_key`, the exact path the platform loop
//! rides) against one focused `Slider` with the causal observer
//! riding: the beats walk the range's ends and step between them, and
//! the *trail* label mirrors what the log streamed — the steps'
//! `increment` beside the jumps' named species, one funnel, four named
//! facts.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example bound_jumps
//! ```
//!
//! The beats: the `End` jump (the maximum, the ask beside the landing),
//! the `Home` jump beside it (the other edge, the other species), the
//! arrow's step between them, and the no-op jump (`Home` at the minimum
//! already lands where the value is — the funnel answered, the log
//! naming it, the M20 idle-frame property). Then the window opens for
//! live play: `Home`/`End`/arrows on the focused slider play the same
//! funnel. `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, Slider};
use alacer_core::events::{Key, KeyEvent, KeyState};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Size;
use alacer_core::inspect::{InspectEvent, InspectObserver, ValueAction, ValueOutcome};
use alacer_core::passes::PassPipeline;
use alacer_core::widget::EventResponse;
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
/// landing, the control's id beside them all — the log answering
/// "which bound did the jump land" from one line. The record is `Copy`,
/// so it passes by value (the pedantic group's rule).
fn describe(record: alacer_core::ValueRecord) -> String {
    let action = record.action.name();
    let outcome = match record.outcome {
        ValueOutcome::Delivered => format!("the value landed ({})", record.value),
        ValueOutcome::Refused(refusal) => format!("refused: {}", refusal.name()),
    };
    let ask = record
        .requested
        .map_or_else(String::new, |ask| format!(" (asked {ask})"));
    let who = record.target.map_or_else(
        || "control (unrouted)".to_string(),
        |id| format!("control {id}"),
    );
    format!("{action}: {outcome}{ask} — {who}")
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

/// One beat: the span names the route, the real dispatch funnel
/// answers, the write streams to the observer, and the response is
/// logged — the funnel's own route, no second channel. The key route's
/// stamp rides the record (M45's discipline); the beat logs it beside
/// the label so the trail's attribution is visible in the transcript.
fn beat(pipeline: &mut PassPipeline, label: &'static str, key: Key) {
    let _route =
        tracing::trace_span!(target: "alacer::bound_jumps", "beat", label = label).entered();
    let response = pipeline.dispatch_key(&KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: alacer_core::events::Modifiers::default(),
        text: None,
    });
    settle(pipeline);
    tracing::info!(
        target: "alacer::bound_jumps",
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

/// The beats the demo walks before the window opens: the End jump, the
/// Home jump beside it, the arrow's step between them, and the no-op
/// jump. Each a real key chord through the real dispatch against the
/// focused control.
fn run_beats(pipeline: &mut PassPipeline, value_signal: alacer_core::FluxRwSignal<String>) {
    // Beat one: the End jump — the maximum, the bound the ask, the
    // landing beside it, the key route's stamp on the record.
    beat(
        pipeline,
        "beat 1: End — the jump to the maximum streams its own species",
        Key::End,
    );
    value_signal.set("the value: 100 — the maximum's own answer".to_string());

    // Beat two: the Home jump beside it — the other edge, the other
    // species, the two jumps tellable apart on one log.
    beat(
        pipeline,
        "beat 2: Home — the jump to the minimum streams its own species",
        Key::Home,
    );
    value_signal.set("the value: 0 — the minimum's own answer".to_string());

    // Beat three: the arrow's step between them — the same key route,
    // the species telling the rides apart (the step's `increment`
    // beside the jumps' named species).
    beat(
        pipeline,
        "beat 3: ArrowRight — the step lands beside the jumps, named apart",
        Key::ArrowRight,
    );
    value_signal.set("the value: 5 — the step's own answer".to_string());

    // Beat four: the no-op jump — Home at the minimum already lands
    // where the value is; the funnel answered, the log naming it (the
    // M20 idle-frame property, one fact for the silence it was).
    beat(
        pipeline,
        "beat 4: Home at the minimum — the no-op delivery answers like the landed write it is",
        Key::Home,
    );
    value_signal.set("the value: 5 — the no-op jump named what it was".to_string());
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The keyboard's jumps, named on the causal log (ADR-0053)").boxed(),
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
    // streams into the trail label the frame after the chord resolves.
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

    // The window: live play — Home/End/arrows on the focused slider
    // play the same funnel the beats did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: focus the slider, press Home/End/arrows — the trail mirrors the log")
            .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer bound jumps",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the jumps' history as data. The species
    // named themselves; no jump was munged.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::bound_jumps",
        records = read_back.len(),
        jumps = read_back
            .iter()
            .filter(|record| matches!(
                record.action,
                ValueAction::JumpToMin | ValueAction::JumpToMax
            ))
            .count(),
        steps = read_back
            .iter()
            .filter(|record| matches!(
                record.action,
                ValueAction::Increment | ValueAction::Decrement
            ))
            .count(),
        "the chords streamed as they resolved — the causal log carries the bounds' facts"
    );
}
