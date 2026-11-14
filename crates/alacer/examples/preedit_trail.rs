//! The field's composing state joins the causal log (§13.3, ADR-0067):
//! ADR-0064's first kill rule cashes — a consumer prices the preedit's
//! own trail, the composition's inline shape — the composing state
//! streams its own [`alacer_core::PreeditRecord`] now: the shape named
//! by lengths (never the payload — a field's value can carry a secret,
//! ADR-0011's rule at the composition tier), the platform's
//! preedit-local cursor beside it, one live session riding as one
//! record (the flood rule at the composing tier — never an echo per
//! composition frame). The preedit is not an edit: it lands nowhere,
//! and a commit's landing keeps its own key-edit fact beside it (M58,
//! ADR-0064) — one fact per trail, the trails tellable apart.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example preedit_trail
//! ```
//!
//! The beats: one session composing in three frames named as one
//! growing record, the cleared composition naming its own end, the
//! commit landing beside the session on its own trail, and the
//! spurious clear streaming nothing (the honest absence). Then the
//! window opens for live play. `ALACER_SOFTWARE=1` forces the
//! softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, TextInput};
use alacer_core::events::ImeEvent;
use alacer_core::flux::RenderBinding;
use alacer_core::inspect::{InspectEvent, InspectObserver, PreeditAction};
use alacer_core::passes::PassPipeline;
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the trail demos' shape).
const VIEWPORT: alacer_core::geom::Size = alacer_core::geom::Size::new(480.0, 300.0);

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// The fact as the trail's reader renders it: the composition's shape
/// named by lengths, the preedit-local cursor beside it — "what is the
/// field showing while the IME composes" answered from one record.
fn describe(record: alacer_core::PreeditRecord) -> String {
    let phase = match record.action {
        PreeditAction::Composing => "the composition shows",
        PreeditAction::Cleared => "the composing state cleared",
    };
    let shape = if record.action == PreeditAction::Cleared {
        String::new()
    } else {
        format!(" (shape {})", record.shape)
    };
    let cursor = record
        .cursor
        .map(|cursor| format!(" — the cursor at {cursor}"))
        .unwrap_or_default();
    let control = record
        .target
        .map(|target| format!(" (at {target})"))
        .unwrap_or_default();
    format!("{phase}{shape}{cursor}{control}")
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the drain rode (axiom A1: signals in, never widgets).
struct Trail {
    preedits: Rc<RefCell<std::collections::VecDeque<alacer_core::PreeditRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The observer half: boxed into
    /// [`alacer_core::passes::PassPipeline::set_observer`], it receives
    /// every `InspectEvent`, keeps the preedit facts, and mirrors the
    /// newest into the trail label's signal.
    fn writer(&self) -> TrailWriter {
        TrailWriter {
            preedits: Rc::clone(&self.preedits),
            trail_line: self.trail_line,
        }
    }

    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<alacer_core::PreeditRecord> {
        self.preedits.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    preedits: Rc<RefCell<std::collections::VecDeque<alacer_core::PreeditRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::Preedit(record) = event else {
            return;
        };
        let line = describe(record);
        {
            let mut ring = self.preedits.borrow_mut();
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

/// One composing beat: the frames routed the real way — the fact
/// streaming to the observer the frame after the composing state
/// resolves.
fn preedit_beat(pipeline: &mut PassPipeline, label: &'static str, text: &str) {
    let _route =
        tracing::trace_span!(target: "alacer::preedit_trail", "beat", label = label).entered();
    let _ = pipeline.dispatch_ime(&ImeEvent::Preedit {
        text: text.to_string(),
        cursor_utf16: Some(alacer_core::utf16::len_utf16(text)),
    });
    settle(pipeline);
    tracing::info!(target: "alacer::preedit_trail", "{label}");
}

/// One commit beat: the composition landing as final text — the
/// commit's own fact riding the key-edit trail beside the session's.
fn commit_beat(pipeline: &mut PassPipeline, label: &'static str, text: &str) {
    let _route =
        tracing::trace_span!(target: "alacer::preedit_trail", "beat", label = label).entered();
    let _ = pipeline.dispatch_ime(&ImeEvent::Commit {
        text: text.to_string(),
    });
    settle(pipeline);
    tracing::info!(target: "alacer::preedit_trail", "{label}");
}

/// One empty-preedit beat: the platform's clear — the composing
/// state's own end.
fn clear_beat(pipeline: &mut PassPipeline, label: &'static str) {
    let _route =
        tracing::trace_span!(target: "alacer::preedit_trail", "beat", label = label).entered();
    let _ = pipeline.dispatch_ime(&ImeEvent::Preedit {
        text: String::new(),
        cursor_utf16: None,
    });
    settle(pipeline);
    tracing::info!(target: "alacer::preedit_trail", "{label}");
}

/// The beats the demo walks before the window opens: one session named
/// as one growing record, the cleared state naming its end, the commit
/// landing beside the session, and the spurious clear riding none.
fn run_beats(pipeline: &mut PassPipeline, status_signal: alacer_core::FluxRwSignal<String>) {
    // Beat one: the session composes in three frames — one record, the
    // shape growing to the composition's (the flood rule at the
    // composing tier).
    let _route =
        tracing::trace_span!(target: "alacer::preedit_trail", "beat", label = "beat 1").entered();
    for frame in ["あ", "あい", "あいう"] {
        let _ = pipeline.dispatch_ime(&ImeEvent::Preedit {
            text: frame.to_string(),
            cursor_utf16: Some(alacer_core::utf16::len_utf16(frame)),
        });
    }
    settle(pipeline);
    tracing::info!(
        target: "alacer::preedit_trail",
        "beat 1: three composition frames — one growing record"
    );
    status_signal.set("the composing door: what does the field show".to_string());

    // Beat two: the session dissolves with no landing — the cleared
    // state names its own end, the honest `0` beside it.
    clear_beat(
        pipeline,
        "beat 2: the cleared composition — the end names itself",
    );

    // Beat three: the commit lands beside the session — the commit's
    // shape riding the key-edit trail (M58's record), the composition's
    // shape riding this one, the trails tellable apart.
    preedit_beat(
        pipeline,
        "beat 3: the session composes for the commit",
        "よ",
    );
    commit_beat(
        pipeline,
        "beat 3: the commit lands on the key-edit trail",
        "よ!",
    );

    // Beat four: a clear with no live session behind it — a no-op door
    // answers no fact (the honest absence).
    clear_beat(pipeline, "beat 4: the spurious clear — no fact rides");
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The field's composing state joins the causal log (ADR-0067)").boxed(),
    );
    let field = TextInput::new("field");
    let field_node = pipeline.add_child(root, field.boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("Live: compose with an IME — the session rides as one record").boxed(),
    );

    let status_signal = pipeline
        .flux_mut()
        .signal("the composing door: (the tree answers what it can)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let status_binding =
        RenderBinding::new("the composing door: (the tree answers what it can)".to_string());
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

    // The observer rides from here on: every composing fact streams
    // into the trail label the frame after the state resolves.
    let trail = Trail {
        preedits: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    settle(&mut pipeline);
    pipeline.request_focus(field_node);
    settle(&mut pipeline);
    run_beats(&mut pipeline, status_signal);
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer preedit trail",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the phase beside every fact.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::preedit_trail",
        records = read_back.len(),
        composing = read_back
            .iter()
            .filter(|record| record.action == PreeditAction::Composing)
            .count(),
        cleared = read_back
            .iter()
            .filter(|record| record.action == PreeditAction::Cleared)
            .count(),
        "the composing facts streamed as they resolved — the causal log carries the composition"
    );
}
