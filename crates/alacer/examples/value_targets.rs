//! The value record names its control (§13.3, ADR-0049, ADR-0046's
//! first re-open rule): the record grows beside the action's routing —
//! the dispatch funnel knows the id the action arrived under, the
//! widget does not, so one additive field (`ValueRecord::target`, the
//! AccessKit-stable id the funnel routed) rides every fact the AT's
//! writes stream, and the funnel's stamp side lives where the knowledge
//! lives. A multi-control form's trail is attributable at last: the log
//! answers "which control answered" beside "did the write land — where,
//! or why not", priced by the reader the rule pre-registered (a trail
//! reader with two sliders on one page — never a node id guessed at
//! the note site).
//!
//! The demo drives the real action tier (`PassPipeline::dispatch_a11y_action`,
//! the exact route the platform's AccessKit adapter rides — never a
//! second channel) against two `Slider` widgets and one read-only
//! `Progress` with the causal observer riding: the beats interleave
//! writes to both sliders, and the *trail* label mirrors what the log
//! streamed — each record carrying its own control's id, the two
//! sliders' facts tellable apart, the progress bar's refusal naming the
//! control that has no write door. The keyboard's arrows ride
//! a ride that streams no value fact (the input tier's records
//! carry those rides until a consumer prices them, ADR-0046's
//! second kill rule), so the attribution stays exclusive to
//! the AT tier.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example value_targets
//! ```
//!
//! The beats: the volume's write, the brightness's write beside it (the
//! same funnel, two controls, two ids), the volume's step, the
//! brightness's refusal (the payload that parses to nothing — the
//! species beside its own control), and the progress bar's read-only
//! refusal. Then the window opens for live play: an AT driving the
//! sliders streams the same attributed records the beats did.
//! `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, Progress, Slider};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Size;
use alacer_core::inspect::{InspectEvent, InspectObserver, ValueOutcome, ValueRecord};
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{A11yAction, Role};
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
/// outcome, the landed value beside a delivery, the refusal species
/// beside a refusal — and the control's id beside it all, the log
/// answering "which control answered" (the M43 attribution, ADR-0049).
/// The record is `Copy`, so it passes by value (the pedantic group's
/// rule).
fn describe(record: ValueRecord) -> String {
    let action = record.action.name();
    let outcome = match record.outcome {
        ValueOutcome::Delivered => format!("the value landed ({})", record.value),
        ValueOutcome::Refused(refusal) => format!("refused: {}", refusal.name()),
    };
    let who = record.target.map_or_else(
        || "control (unrouted)".to_string(),
        |id| format!("control {id}"),
    );
    format!("{action}: {outcome} — {who}")
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
/// funnel's own route, no second channel. The routed id rides the
/// record (the funnel's stamp side, ADR-0049); the beat logs it beside
/// the label so the trail's attribution is visible in the transcript.
fn beat(
    pipeline: &mut PassPipeline,
    label: &'static str,
    target: (Role, &str),
    action: A11yAction,
) {
    let _route =
        tracing::trace_span!(target: "alacer::value_targets", "beat", label = label).entered();
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
        target: "alacer::value_targets",
        consumed = consumed,
        routed_id = id,
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

/// The beats the demo walks before the window opens: the two sliders'
/// writes interleaved — the same funnel answering for two controls, the
/// records naming which — the brightness's refused parse, and the
/// progress bar's read-only refusal. Each a real AT action through the
/// real dispatch against the widget the tree names.
fn run_beats(pipeline: &mut PassPipeline, value_signal: alacer_core::FluxRwSignal<String>) {
    // Beat one: the volume's SetValue — the record names its control.
    beat(
        pipeline,
        "beat 1: SetValue 40 on the volume — the record names its control",
        (Role::Slider, "Volume"),
        A11yAction::SetValue("40".to_string()),
    );
    value_signal.set("the value: 40 — the volume's own answer".to_string());

    // Beat two: the brightness's SetValue — the same funnel, a
    // different control, a different id: the facts a multi-slider form
    // streams are tellable apart.
    beat(
        pipeline,
        "beat 2: SetValue 70 on the brightness — the second control names itself",
        (Role::Slider, "Brightness"),
        A11yAction::SetValue("70".to_string()),
    );
    value_signal.set("the value: 70 — the brightness's own answer".to_string());

    // Beat three: the volume's step — one Increment, the funnel's own
    // arithmetic, the id riding beside it again.
    beat(
        pipeline,
        "beat 3: Increment on the volume — the step lands, the id beside it",
        (Role::Slider, "Volume"),
        A11yAction::Increment,
    );
    value_signal.set("the value: 45 — the volume's step landed".to_string());

    // Beat four: the brightness's refused parse — the species beside
    // its own control: the log answering "which control refused it"
    // without a second observation channel.
    beat(
        pipeline,
        "beat 4: SetValue \"loud\" on the brightness — the refusal names its control",
        (Role::Slider, "Brightness"),
        A11yAction::SetValue("loud".to_string()),
    );
    value_signal.set("the value: 70 — the brightness's parse refusal moved nothing".to_string());

    // Beat five: the reporting widget — a read-only tier has no write
    // door, every AT write refused, never munged into a step; the
    // refusal names the control that has no door (ADR-0027's rule, the
    // M43 attribution beside it).
    beat(
        pipeline,
        "beat 5: SetValue 50 on the progress bar — the read-only refusal names its door-less control",
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
        Label::new("The value facts' attribution, streamed to the causal log (ADR-0049)").boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .value(0.0)
            .step(5.0)
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Slider::new("Brightness", 0.0, 100.0)
            .value(50.0)
            .step(5.0)
            .boxed(),
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

    // The window: live play — an AT driving the sliders streams the
    // same attributed records the beats did; the keyboard's arrows ride
    // the widget's own door, the record's target honest about its
    // absence (the second kill rule's open arm, ADR-0046's rhythm).
    let _ = pipeline.add_child(
        root,
        Label::new("Live: drive the sliders with an AT — the trail mirrors the log").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer value targets",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the value tier's history as data. The
    // refusals streamed beside the deliveries; the controls named
    // themselves.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::value_targets",
        records = read_back.len(),
        attributed = read_back.iter().filter(|record| record.target.is_some()).count(),
        delivered = read_back
            .iter()
            .filter(|record| record.outcome == ValueOutcome::Delivered)
            .count(),
        refusals = read_back
            .iter()
            .filter(|record| matches!(record.outcome, ValueOutcome::Refused(_)))
            .count(),
        "the writes streamed as they resolved — the causal log carries the controls' facts"
    );
}
