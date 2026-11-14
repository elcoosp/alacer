//! The AT's text write joins the causal log (§13.3, ADR-0061):
//! ADR-0046's second kill rule cashes — a consumer needs the input
//! tier's rides priced — the editing tier's AT door streams its own
//! record now. The write's shape is named by lengths
//! ([`alacer_core::TextWriteRecord`]'s `replaced`/`inserted`, UTF-16
//! units — the whole value for a `set_value`, the selection's span
//! for a `replace_selection`), the payload itself never riding the
//! log (a field's value can carry a secret — the clipboard trail's
//! rule), the selection ask riding beside the landing (the M44 rule
//! at the text tier — the forgiving clamp visible as data), and the
//! closed door's refusal streaming too (`unavailable` — the value
//! tier's own species, ADR-0028's tier).
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example at_text_writes
//! ```
//!
//! The beats: the selection ask naming what it asked beside where it
//! landed, the paste-shaped replacement pricing the span it consumed,
//! the whole-value rewrite pricing the value it replaced, and the
//! locked field refusing honestly. Then the window opens for live
//! play: a screen reader's writes ride the same action route's
//! vocabulary, the trail mirroring the log.
//! `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, TextInput};
use alacer_core::flux::RenderBinding;
use alacer_core::inspect::{InspectEvent, InspectObserver, TextWriteOutcome};
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{A11yAction, Role, TextSelection};
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

/// The fact as the trail's reader renders it: the write's shape named
/// by lengths, the ask beside the landing, the honest absences
/// visible — the log answering "did the screen reader rewrite this
/// field, what shape, where did the caret land, or why not" from one
/// record. The payload itself never rides the log (the clipboard
/// trail's rule); the record is `Copy`, so it passes by value (the
/// pedantic group's rule).
fn describe(record: alacer_core::TextWriteRecord) -> String {
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
            format!("the text write landed{shape}{ask} — {landed}")
        }
        TextWriteOutcome::Refused(refusal) => {
            format!("refused: {} — {landed}", refusal.name())
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

/// One beat on the AT route: the span names the ask, the real editing
/// tier answers, the fact streams to the observer — the funnel's own
/// route, no second channel.
fn at_beat(pipeline: &mut PassPipeline, label: &'static str, name: &str, action: A11yAction) {
    let _route =
        tracing::trace_span!(target: "alacer::at_text_writes", "beat", label = label).entered();
    let id = input_id(pipeline, name);
    let consumed = pipeline.dispatch_a11y_action(id, action);
    settle(pipeline);
    tracing::info!(target: "alacer::at_text_writes", consumed, "{label}");
}

/// The beats the demo walks before the window opens: the selection ask
/// beside its landing, the paste-shaped replacement, the whole-value
/// rewrite, and the locked field's honest refusal. Each a real ask
/// through the real routes.
fn run_beats(pipeline: &mut PassPipeline, status_signal: alacer_core::FluxRwSignal<String>) {
    // Beat one: the selection ask — the ask rides beside the landing
    // (the M44 rule at the text tier), the forgiving clamp visible as
    // data.
    at_beat(
        pipeline,
        "beat 1: AT SetTextSelection 1..100 — the ask beside the landing",
        "field",
        A11yAction::SetTextSelection(TextSelection::new(1, 100)),
    );
    status_signal.set("the AT's text writes: shape, landing, refusal".to_string());

    // Beat two: the paste-shaped replacement — the span the write
    // consumed rides beside the payload's own length (the payload
    // itself never riding the log).
    at_beat(
        pipeline,
        "beat 2: AT ReplaceSelectedText — the span consumed, priced by lengths",
        "field",
        A11yAction::ReplaceSelectedText("HOWDY".into()),
    );

    // Beat three: the whole-value rewrite — the value the field
    // carried is the span the write consumed, the journal resetting
    // beneath it (the AT's write is an app write, ADR-0021's rule).
    at_beat(
        pipeline,
        "beat 3: AT SetValue — the whole value priced, the journal reset beneath",
        "field",
        A11yAction::SetValue("written by the reader".into()),
    );

    // Beat four: the locked field — the availability tier's closed
    // door refuses honestly, and the refusal streams (the value
    // tier's own species, `unavailable`).
    at_beat(
        pipeline,
        "beat 4: AT SetValue at the locked field — the closed door's refusal",
        "locked",
        A11yAction::SetValue("typed".into()),
    );
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The AT's text write joins the causal log (ADR-0061)").boxed(),
    );
    let mut field = TextInput::new("field");
    field.set_value("hello trail");
    let _ = pipeline.add_child(root, field.boxed());
    let mut locked = TextInput::new("locked");
    locked.set_value("read only");
    locked.set_enabled(false);
    let _ = pipeline.add_child(root, locked.boxed());

    let status_signal = pipeline
        .flux_mut()
        .signal("the AT's text writes: (the tree carries what the reader wrote)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let status_binding = RenderBinding::new(
        "the AT's text writes: (the tree carries what the reader wrote)".to_string(),
    );
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
    run_beats(&mut pipeline, status_signal);

    // The window: live play — a screen reader's writes ride the same
    // action route's vocabulary (set, replace, move the selection),
    // the trail mirroring the log.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: a screen reader's text writes name their shape and landing on the log")
            .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer AT text writes",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the shape beside every write.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::at_text_writes",
        records = read_back.len(),
        delivered = read_back
            .iter()
            .filter(|record| record.outcome == TextWriteOutcome::Delivered)
            .count(),
        refused = read_back
            .iter()
            .filter(|record| matches!(record.outcome, TextWriteOutcome::Refused(_)))
            .count(),
        "the writes streamed as they resolved — the causal log carries the AT's text writes"
    );
}
