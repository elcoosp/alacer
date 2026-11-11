//! The journal rides the causal log (§13.3, ADR-0037): every
//! [`EditHistory`] action — an applied step, a refused chord, the app's
//! reset — streams one `InspectEvent::Edit` the tick after it resolves,
//! carrying the action, the depths it left, and the refusal it named.
//! The hook the journal's own docs pre-registered ("the inspector may
//! someday surface the depth") cashed: a status line, an inspector
//! trail, or a test routes on "what did this field's journal do" without
//! re-implementing the observation at the chord site.
//!
//! The demo drives the M15 journal directly (no `TextInput` riding
//! along — the `undo_redo` demo's field) with the causal observer
//! riding: the app mirrors its own introspection (the field line, the
//! journal's depth), and the *trail* label mirrors what the log
//! streamed — the observer writes the signal the label binds, the same
//! one-batch currency every handler rides. The refused chord streams
//! too: the exhausted journal's fact is exactly the one a trail reader
//! wants.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example edit_trail
//! ```
//!
//! Run with `cargo run -p alacer --example edit_trail`. Click *Type hi*
//! a few times and watch the trail log one coalesced fact per press
//! (the depth never moves — the pure-append rule); *Undo*/*Redo* stream
//! the depths they leave; *Replace all* streams the reset; *Undo* on an
//! exhausted journal streams the refusal. `ALACER_SOFTWARE=1` forces
//! the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, Row};
use alacer_core::flux::RenderBinding;
use alacer_core::history::{EditAction, EditHistory};
use alacer_core::inspect::{EditRecord, InspectEvent, InspectObserver};
use alacer_core::utf16;
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// One control's payload: the boxed route the button drives (every route
/// closure is its own type, so the row carries trait objects).
type RouteHandler = Box<dyn FnMut()>;

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// The fact as the trail's reader renders it: the action it names, the
/// depths it left, the refusal it carried.
fn describe(record: &EditRecord) -> String {
    let depths = format!("(undo {} · redo {})", record.undo_depth, record.redo_depth);
    let action = match record.action {
        EditAction::Record if record.coalesced => "record — coalesced into the tip".to_string(),
        EditAction::Record => "record — opened a step".to_string(),
        EditAction::Undo if record.refused => {
            "undo — refused, the journal is exhausted".to_string()
        }
        EditAction::Undo => "undo — the step replayed".to_string(),
        EditAction::Redo => "redo — the step re-pushed".to_string(),
        EditAction::Reset => "reset — the app's write cleared both branches".to_string(),
    };
    format!("{action} {depths}")
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the drain rode (axiom A1: signals in, never widgets).
struct Trail {
    edits: Rc<RefCell<std::collections::VecDeque<EditRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The observer half: boxed into [`alacer_core::passes::PassPipeline::
    /// set_observer`], it receives every `InspectEvent`, keeps the
    /// journal's facts, and mirrors the newest into the trail label's
    /// signal — the write queues masks, the frame renders them.
    fn writer(&self) -> TrailWriter {
        TrailWriter {
            edits: Rc::clone(&self.edits),
            trail_line: self.trail_line,
        }
    }

    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<EditRecord> {
        self.edits.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    edits: Rc<RefCell<std::collections::VecDeque<EditRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::Edit(record) = event else {
            return;
        };
        {
            let mut ring = self.edits.borrow_mut();
            if ring.len() >= 64 {
                ring.pop_front();
            }
            ring.push_back(record);
        }
        self.trail_line.set(describe(&record));
    }
}

/// The field the journal belongs to: the text and the bounded journal
/// behind it, exactly the pair a custom widget would own (the
/// `undo_redo` demo's `Journal`).
struct Journal {
    text: String,
    history: EditHistory,
}

impl Journal {
    /// The one edit funnel: apply the edit and journal its inverse in
    /// the same place — the discipline ADR-0021 requires of every
    /// consumer.
    fn edit(&mut self, at_utf16: usize, to_utf16: usize, inserted: &str) {
        let from = utf16::byte_offset(&self.text, at_utf16);
        let to = utf16::byte_offset(&self.text, to_utf16);
        let removed = self.text[from..to].to_string();
        self.text.replace_range(from..to, inserted);
        let selection_before = if at_utf16 == to_utf16 {
            (at_utf16, at_utf16)
        } else {
            (at_utf16.min(to_utf16), at_utf16.max(to_utf16))
        };
        let caret = at_utf16 + utf16::len_utf16(inserted);
        self.history.record(
            at_utf16,
            removed,
            inserted.to_string(),
            selection_before,
            (caret, caret),
        );
    }

    /// Appends at the end: consecutive appends coalesce into one step.
    fn type_hi(&mut self) {
        let at = utf16::len_utf16(&self.text);
        self.edit(at, at, "hi ");
    }

    /// Removes the last scalar value: its own step, never coalesced.
    fn backspace(&mut self) {
        let Some(last) = self.text.chars().next_back() else {
            return;
        };
        let end = utf16::len_utf16(&self.text);
        self.edit(end - last.len_utf16(), end, "");
    }
}

/// The field line: the buffer, or the placeholder while it is empty.
fn display_text(journal: &Journal) -> String {
    if journal.text.is_empty() {
        "the empty field".to_string()
    } else {
        format!("field: {:?}", journal.text)
    }
}

/// The status line: the journal's own introspection reads, live — the
/// app's half of the story (the trail label carries the log's half).
fn status(journal: &Journal) -> String {
    format!(
        "undo depth: {} · redo: {}",
        journal.history.undo_depth(),
        if journal.history.can_redo() {
            "yes"
        } else {
            "no"
        }
    )
}

/// Mirrors the field's reads into the signals the labels bind — handlers
/// write signals, nothing mutates widgets (axiom A1), so one batch is
/// one scoped re-render. The journal's facts ride the next frame's start
/// drain; the trail label's signal is the observer's write, not ours.
fn mirror(
    journal: &Journal,
    buffer: &alacer_core::FluxRwSignal<String>,
    status_line: &alacer_core::FluxRwSignal<String>,
) {
    buffer.set(display_text(journal));
    status_line.set(status(journal));
}

/// One edit route: the span names it, the batch is the one write batch,
/// and the mirror is the only bridge back to the window. The journal's
/// own fact streams at the next tick's start drain — one frame, the
/// announcement drain's currency.
fn route(
    journal: &Rc<RefCell<Journal>>,
    route_name: &'static str,
    edit: impl FnOnce(&mut Journal),
    buffer: &alacer_core::FluxRwSignal<String>,
    status_line: &alacer_core::FluxRwSignal<String>,
) {
    let _route =
        tracing::trace_span!(target: "alacer::edit_trail", "journal", route = route_name).entered();
    alacer_core::flux::Flux::batch_of(|| {
        edit(&mut journal.borrow_mut());
        mirror(&journal.borrow(), buffer, status_line);
    });
}

/// The demo's controls: one row, five routes — the same journal routes
/// the trail streams for, driven live in the window (the journal is pure
/// state, so handlers never touch the pipeline — axiom A1 — and the
/// facts follow on the next frame's start drain).
fn push_buttons(
    pipeline: &mut alacer_core::passes::PassPipeline,
    actions: alacer_core::NodeId,
    journal: &Rc<RefCell<Journal>>,
    buffer: &alacer_core::FluxRwSignal<String>,
    status_line: &alacer_core::FluxRwSignal<String>,
) {
    let type_click = {
        let journal = Rc::clone(journal);
        let buffer = *buffer;
        let status_line = *status_line;
        move || {
            route(&journal, "type", Journal::type_hi, &buffer, &status_line);
        }
    };
    let backspace_click = {
        let journal = Rc::clone(journal);
        let buffer = *buffer;
        let status_line = *status_line;
        move || {
            route(
                &journal,
                "backspace",
                Journal::backspace,
                &buffer,
                &status_line,
            );
        }
    };
    let undo_click = {
        let journal = Rc::clone(journal);
        let buffer = *buffer;
        let status_line = *status_line;
        move || {
            route(
                &journal,
                "undo",
                |field| {
                    let Journal { text, history } = &mut *field;
                    let _ = history.undo(text);
                },
                &buffer,
                &status_line,
            );
        }
    };
    let redo_click = {
        let journal = Rc::clone(journal);
        let buffer = *buffer;
        let status_line = *status_line;
        move || {
            route(
                &journal,
                "redo",
                |field| {
                    let Journal { text, history } = &mut *field;
                    let _ = history.redo(text);
                },
                &buffer,
                &status_line,
            );
        }
    };
    let replace_click = {
        let journal = Rc::clone(journal);
        let buffer = *buffer;
        let status_line = *status_line;
        move || {
            route(
                &journal,
                "app-write",
                |field| {
                    field.text = "replaced by the app".to_string();
                    field.history.reset();
                },
                &buffer,
                &status_line,
            );
        }
    };
    for (name, handler) in [
        ("Type hi", Box::new(type_click) as RouteHandler),
        ("Backspace", Box::new(backspace_click) as RouteHandler),
        ("Undo", Box::new(undo_click) as RouteHandler),
        ("Redo", Box::new(redo_click) as RouteHandler),
        ("Replace all", Box::new(replace_click) as RouteHandler),
    ] {
        let _ = pipeline.add_child(actions, Button::new(name).on_click(handler).boxed());
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The journal's facts, streamed to the causal log (ADR-0037)").boxed(),
    );

    let journal = Rc::new(RefCell::new(Journal {
        text: String::new(),
        history: EditHistory::new(),
    }));

    // The reads the labels mirror: the field and the journal's depth are
    // the app's half (handlers write these), the trail line is the log's
    // half (the observer writes it at the drain).
    let buffer_signal = pipeline.flux_mut().signal(display_text(&journal.borrow()));
    let status_signal = pipeline.flux_mut().signal(status(&journal.borrow()));
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());

    let buffer_binding = RenderBinding::new(display_text(&journal.borrow()));
    let buffer_label = pipeline.add_child(root, Label::bound(buffer_binding.clone()).boxed());
    let status_binding = RenderBinding::new(status(&journal.borrow()));
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    let trail_binding = RenderBinding::new("the trail: (nothing streamed yet)".to_string());
    let trail_label = pipeline.add_child(root, Label::bound(trail_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        buffer_signal,
        buffer_label,
        &buffer_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |text: &String| text.clone(),
    );
    pipeline.flux_mut().bind_value(
        status_signal,
        status_label,
        &status_binding,
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

    // The observer rides from here on: the journal's facts stream into
    // the trail label the frame after each action resolves.
    let trail = Trail {
        edits: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    let actions = pipeline.add_child(root, Row::new().gap(8.0).boxed());
    push_buttons(
        &mut pipeline,
        actions,
        &journal,
        &buffer_signal,
        &status_signal,
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer edit trail",
        (460.0, 260.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the journal's history as data.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::edit_trail",
        records = read_back.len(),
        refusals = read_back.iter().filter(|record| record.refused).count(),
        resets = read_back
            .iter()
            .filter(|record| record.action == EditAction::Reset)
            .count(),
        "the journal's facts streamed as they resolved — the causal log carries the edit trail"
    );
}
