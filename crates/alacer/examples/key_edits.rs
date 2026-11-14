//! The typing door's edits join the causal log (§13.3, ADR-0064):
//! ADR-0062's first kill rule cashes — a consumer needs the keyboard's
//! and IME's edits on the log — a typed run, a backspace, a word
//! chord, a committed composition each stream their own fact now
//! ([`alacer_core::KeyEditRecord`]): the change's shape named by
//! lengths (never the payload — a field's value can carry a secret,
//! the clipboard trail's rule), the landing selection beside it, the
//! driver who wrote it, and the journal's own fold. A coalesced typed
//! run lands as one record — the flood rule, never a per-keystroke
//! flood.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example key_edits
//! ```
//!
//! The beats: the keyboard's typed run named as one fact, the
//! backspace's span, the word chord's species, the IME's committed
//! composition named by its driver, and the app door's write beside
//! them streaming nothing — the honest absence (the app's write is
//! the text-write trail's fact, the doors tellable apart). The
//! clipboard chords ride the clipboard trail — the round trip is that
//! trail's fact — and since M60 their field changes name their own
//! facts beside it (the paste's shape, the cut's span, the chord's
//! driver; ADR-0066). Then the window opens for live play.
//! `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, TextInput};
use alacer_core::events::{ImeEvent, Key, KeyEvent, KeyState, Modifiers};
use alacer_core::flux::RenderBinding;
use alacer_core::inspect::{InspectEvent, InspectObserver, KeyEditDriver};
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

/// The fact as the trail's reader renders it: the edit's shape named
/// by lengths, the landing selection and the driver beside it — "what
/// did the typist change in this field" answered from one record.
fn describe(record: alacer_core::KeyEditRecord) -> String {
    let who = match record.driver {
        KeyEditDriver::Keyboard => "the keyboard",
        KeyEditDriver::Ime => "the IME's commit",
        KeyEditDriver::Clipboard => "the clipboard's chord",
    };
    let fold = if record.coalesced {
        " — the journal folded the run"
    } else {
        ""
    };
    let control = record
        .target
        .map(|target| format!(" (at {target})"))
        .unwrap_or_default();
    format!(
        "{who} {} ({} replaced by {}) — the selection at {}..{}{fold}{control}",
        record.action.name(),
        record.replaced,
        record.inserted,
        record.selection.anchor,
        record.selection.focus,
    )
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the drain rode (axiom A1: signals in, never widgets).
struct Trail {
    edits: Rc<RefCell<std::collections::VecDeque<alacer_core::KeyEditRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The observer half: boxed into
    /// [`alacer_core::passes::PassPipeline::set_observer`], it receives
    /// every `InspectEvent`, keeps the key-edit facts, and mirrors the
    /// newest into the trail label's signal.
    fn writer(&self) -> TrailWriter {
        TrailWriter {
            edits: Rc::clone(&self.edits),
            trail_line: self.trail_line,
        }
    }

    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<alacer_core::KeyEditRecord> {
        self.edits.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    edits: Rc<RefCell<std::collections::VecDeque<alacer_core::KeyEditRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::KeyEdit(record) = event else {
            return;
        };
        let line = describe(record);
        {
            let mut ring = self.edits.borrow_mut();
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

/// One keyboard beat: the event routed the real way — the fact
/// streaming to the observer the frame after the edit resolves.
fn key_beat(pipeline: &mut PassPipeline, label: &'static str, event: &KeyEvent) {
    let _route = tracing::trace_span!(target: "alacer::key_edits", "beat", label = label).entered();
    let _consumed = pipeline.dispatch_key(event);
    settle(pipeline);
    tracing::info!(target: "alacer::key_edits", "{label}");
}

/// One IME beat: the composition the real way — the preedit the field
/// shows, the commit landing as final text, the record naming the
/// driver.
fn ime_beat(pipeline: &mut PassPipeline, label: &'static str, text: &str) {
    let _route = tracing::trace_span!(target: "alacer::key_edits", "beat", label = label).entered();
    let _ = pipeline.dispatch_ime(&ImeEvent::Preedit {
        text: text.to_string(),
        cursor_utf16: Some(alacer_core::utf16::len_utf16(text)),
    });
    settle(pipeline);
    let _ = pipeline.dispatch_ime(&ImeEvent::Commit {
        text: text.to_string(),
    });
    settle(pipeline);
    tracing::info!(target: "alacer::key_edits", "{label}");
}

/// One beat on the app door: the imperative write through the mutate
/// seam — the exact path the consumer's code rides. No key-edit fact
/// streams: the app's write is the text-write trail's, the doors
/// tellable apart on the log.
fn app_beat(pipeline: &mut PassPipeline, label: &'static str, node: alacer_core::NodeId) {
    let _route = tracing::trace_span!(target: "alacer::key_edits", "beat", label = label).entered();
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_value("written by the app");
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    settle(pipeline);
    tracing::info!(target: "alacer::key_edits", "{label}");
}

/// The character press the platform delivers when the user types.
fn char_key(ch: char) -> KeyEvent {
    KeyEvent::char(ch)
}

/// The Alt+Backspace press (the word chord; ADR-0019) — the exact
/// event the platform delivers when the user deletes word-wise.
fn alt_backspace() -> KeyEvent {
    KeyEvent {
        key: Key::Backspace,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            alt: true,
            ..Modifiers::default()
        },
        text: None,
    }
}

/// The beats the demo walks before the window opens: a typed run named
/// as one fact, the backspace's span, the word chord's species, the
/// IME's commit named by its driver, and the app door's write beside
/// them riding none — the honest absence.
fn run_beats(
    pipeline: &mut PassPipeline,
    status_signal: alacer_core::FluxRwSignal<String>,
    field_node: alacer_core::NodeId,
) {
    // Beat one: the keyboard's typed run — three keystrokes, one
    // record: the journal's own fold bounds the trail's (the flood
    // rule), the shape riding as the run's.
    for ch in "hey".chars() {
        let _ = pipeline.dispatch_key(&char_key(ch));
    }
    settle(pipeline);
    tracing::info!(
        target: "alacer::key_edits",
        "beat 1: the typed run — three keystrokes, one fact (the flood rule)"
    );
    status_signal.set("the typing door: what did the typist change".to_string());

    // Beat two: the backspace — the span it consumed named, the caret
    // it left beside it.
    key_beat(
        pipeline,
        "beat 2: the backspace — the span it consumed named",
        &KeyEvent::pressed(Key::Backspace),
    );

    // Beat three: the word chord — its own species, never a munged
    // Backspace (the M13 word tier's vocabulary on the log).
    for ch in "hello brave".chars() {
        let _ = pipeline.dispatch_key(&char_key(ch));
    }
    key_beat(
        pipeline,
        "beat 3: the word chord — the run it consumed named",
        &alt_backspace(),
    );

    // Beat four: the IME's committed composition — the driver named,
    // the surrogate pair riding whole (the commit's UTF-16 shape).
    ime_beat(
        pipeline,
        "beat 4: the IME's commit — the driver named",
        "よ!",
    );

    // Beat five: the app door's write beside them — no key-edit fact
    // streams (the text-write trail names it), the doors tellable
    // apart on the log.
    app_beat(
        pipeline,
        "beat 5: the app's set_value — no typing-door fact rides",
        field_node,
    );
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The typing door's edits join the causal log (ADR-0064)").boxed(),
    );
    let field = TextInput::new("field");
    let field_node = pipeline.add_child(root, field.boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("Live: a typed run, a chord, a commit — each names its fact").boxed(),
    );

    let status_signal = pipeline
        .flux_mut()
        .signal("the typing door: (the tree answers what it can)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let status_binding =
        RenderBinding::new("the typing door: (the tree answers what it can)".to_string());
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

    // The observer rides from here on: every key edit's record
    // streams into the trail label the frame after the edit resolves.
    let trail = Trail {
        edits: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    settle(&mut pipeline);
    pipeline.request_focus(field_node);
    settle(&mut pipeline);
    run_beats(&mut pipeline, status_signal, field_node);
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer key edits",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the driver beside every edit.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::key_edits",
        records = read_back.len(),
        keyboard = read_back
            .iter()
            .filter(|record| record.driver == KeyEditDriver::Keyboard)
            .count(),
        ime = read_back
            .iter()
            .filter(|record| record.driver == KeyEditDriver::Ime)
            .count(),
        "the edits streamed as they resolved — the causal log carries the typing door"
    );
}
