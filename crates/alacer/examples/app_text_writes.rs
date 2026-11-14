//! The app's text write joins the causal log (§13.3, ADR-0062):
//! ADR-0061's second kill rule cashes — a consumer needs the app's own
//! writes attributed — the imperative `set_value` door streams its own
//! fact now, `source: AppDoor` beside the AT door's asks
//! (`source: AtDoor`). The write's shape is named by lengths
//! ([`alacer_core::TextWriteRecord`]'s `replaced`/`inserted`, UTF-16
//! units — the value the field carried, the value the app handed over),
//! the payload itself never riding the log (a field's value can carry a
//! secret — the clipboard trail's rule), and the journal's reset still
//! streaming beside the door's fact (the depths are the journal's story,
//! the write is the door's).
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example app_text_writes
//! ```
//!
//! The beats: the app's write through the mutate seam (the door the
//! view reconciliation's patches ride), the AT's ask through the action
//! route — one funnel, two doors, every fact naming which door it rode.
//! Then the window opens for live play: edit the field by keyboard and
//! watch the trail mirror the log. `ALACER_SOFTWARE=1` forces the
//! softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, TextInput};
use alacer_core::flux::RenderBinding;
use alacer_core::inspect::{InspectEvent, InspectObserver, TextWriteOutcome};
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{A11yAction, Role};
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the trail demos' shape).
const VIEWPORT: alacer_core::geom::Size = alacer_core::geom::Size::new(480.0, 380.0);

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// The fact as the trail's reader renders it: the door named beside the
/// write's shape, the ask beside the landing, the honest absences
/// visible — the log answering "did the screen reader rewrite this
/// field, or did the app — what shape, where did the caret land" from
/// one record. The payload itself never rides the log (the clipboard
/// trail's rule); the record is `Copy`, so it passes by value (the
/// pedantic group's rule).
fn describe(record: alacer_core::TextWriteRecord) -> String {
    let who = match record.source {
        alacer_core::TextWriteSource::AppDoor => "the app's write",
        alacer_core::TextWriteSource::AtDoor => "the AT's write",
    };
    let shape = match (record.replaced, record.inserted) {
        (Some(replaced), Some(inserted)) => {
            format!(" ({replaced} replaced by {inserted})")
        }
        _ => String::new(),
    };
    let ask = record
        .requested
        .map(|ask| format!(" (asked {}..{})", ask.anchor, ask.focus))
        .unwrap_or_default();
    let landed = format!(
        "the selection at {}..{}",
        record.selection.anchor, record.selection.focus
    );
    match record.outcome {
        TextWriteOutcome::Delivered => {
            format!("{who} landed{shape}{ask} — {landed}")
        }
        TextWriteOutcome::Refused(refusal) => {
            format!("{who} refused: {} — {landed}", refusal.name())
        }
    }
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the drain rode (axiom A1: signals in, never widgets).
struct Trail {
    writes: Rc<RefCell<std::collections::VecDeque<alacer_core::TextWriteRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The observer half: boxed into
    /// [`alacer_core::passes::PassPipeline::set_observer`], it receives
    /// every `InspectEvent`, keeps the text-write facts, and mirrors
    /// the newest into the trail label's signal.
    fn writer(&self) -> TrailWriter {
        TrailWriter {
            writes: Rc::clone(&self.writes),
            trail_line: self.trail_line,
        }
    }

    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<alacer_core::TextWriteRecord> {
        self.writes.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    writes: Rc<RefCell<std::collections::VecDeque<alacer_core::TextWriteRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::TextWrite(record) = event else {
            return;
        };
        let line = describe(record);
        {
            let mut ring = self.writes.borrow_mut();
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

/// The input's AccessKit-stable id, read off the real a11y snapshot —
/// the identity the dispatch funnel routes and the stamp carries (the
/// M43 vocabulary).
fn input_id(pipeline: &PassPipeline, name: &str) -> u64 {
    let tree = pipeline
        .a11y_snapshot()
        .expect("the a11y snapshot after the settle");
    let node = tree
        .find(Role::TextInput, name)
        .unwrap_or_else(|| panic!("the AT can reach the input named `{name}`"));
    node.id
}

/// One beat on the app door: the imperative write through the mutate
/// seam — the exact path the consumer's code and the view
/// reconciliation's patches ride — the fact streaming to the observer.
fn app_beat(
    pipeline: &mut PassPipeline,
    label: &'static str,
    node: alacer_core::NodeId,
    value: &str,
) {
    let _route =
        tracing::trace_span!(target: "alacer::app_text_writes", "beat", label = label).entered();
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_value(value);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    settle(pipeline);
    tracing::info!(target: "alacer::app_text_writes", "{label}");
}

/// One beat on the AT door: the action routed through the real funnel —
/// the other door on the same write vocabulary, its fact naming the AT.
fn at_beat(pipeline: &mut PassPipeline, label: &'static str, name: &str, action: A11yAction) {
    let _route =
        tracing::trace_span!(target: "alacer::app_text_writes", "beat", label = label).entered();
    let id = input_id(pipeline, name);
    let consumed = pipeline.dispatch_a11y_action(id, action);
    settle(pipeline);
    tracing::info!(target: "alacer::app_text_writes", consumed, "{label}");
}

/// The beats the demo walks before the window opens: the app's whole-
/// value write (the mutate seam), the AT's whole-value write (the action
/// route) — the two doors tellable apart on the log.
fn run_beats(
    pipeline: &mut PassPipeline,
    status_signal: alacer_core::FluxRwSignal<String>,
    field_node: alacer_core::NodeId,
) {
    // Beat one: the app's write through the mutate seam — the door the
    // view reconciliation's patches ride; the journal resets beneath it
    // (ADR-0021's reset rule: undo never fights the app's own write).
    app_beat(
        pipeline,
        "beat 1: the app's set_value — the mutate seam's door names itself",
        field_node,
        "written by the app",
    );
    status_signal.set("the app's writes and the AT's asks: one log, two doors".to_string());

    // Beat two: the AT's whole-value write through the action route —
    // the same action vocabulary, the other door, the funnel's stamp
    // beside it (the M43 rule).
    at_beat(
        pipeline,
        "beat 2: the AT's SetValue — the action route's door names itself",
        "field",
        A11yAction::SetValue("written by the reader".into()),
    );

    // Beat three: the app writes again — every fact names its door, the
    // trail reading app → AT → app without a second channel.
    app_beat(
        pipeline,
        "beat 3: the app writes again — the doors stay tellable apart",
        field_node,
        "the app wrote once more",
    );
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The app's text write joins the causal log (ADR-0062)").boxed(),
    );
    let mut field = TextInput::new("field");
    field.set_value("hello doors");
    let field_node = pipeline.add_child(root, field.boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the app's writes and the AT's asks both name their door").boxed(),
    );

    let status_signal = pipeline
        .flux_mut()
        .signal("the doors: (the tree carries what the writers wrote)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let status_binding =
        RenderBinding::new("the doors: (the tree carries what the writers wrote)".to_string());
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        status_signal,
        status_label,
        &status_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );
    let trail_binding = RenderBinding::new("the trail: (nothing streamed yet)".to_string());
    let trail_label = pipeline.add_child(root, Label::bound(trail_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        trail_signal,
        trail_label,
        &trail_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );

    // The observer rides from here on: every text write's record
    // streams into the trail label the frame after the door resolves.
    let trail = Trail {
        writes: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    settle(&mut pipeline);
    run_beats(&mut pipeline, status_signal, field_node);

    // The window: live play — type to write through the keyboard door
    // (the field's own editing tier), and the trail mirrors the log.
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer app text writes",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the door named beside every write.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::app_text_writes",
        records = read_back.len(),
        app_door = read_back
            .iter()
            .filter(|record| record.source == alacer_core::TextWriteSource::AppDoor)
            .count(),
        at_door = read_back
            .iter()
            .filter(|record| record.source == alacer_core::TextWriteSource::AtDoor)
            .count(),
        "the writes streamed as they resolved — the causal log names every door"
    );
}
