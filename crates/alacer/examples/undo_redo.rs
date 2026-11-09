//! The M15 journal driven directly (§10.3, ADR-0021): `EditHistory` as a
//! consumer's vocabulary, no `TextInput` riding along. The buttons route
//! the same record/undo/redo calls a custom widget's edit funnel would —
//! the pure-append coalescing rule (three typed runs undo as one step),
//! the selection each step returns, the app-write reset, and the honest
//! refusal on an exhausted journal (no masks, an idle frame). The
//! observability rides the workspace's own idiom: `tracing` spans at the
//! `alacer::editing` target, the journal's depth read as a field.
//!
//! Run with `cargo run -p alacer --example undo_redo`. Click *Type hi* a
//! few times and watch the depth stay at one step — the coalescing rule —
//! then *Undo* takes the whole run back and *Redo* replays it.
//! *Replace all* is the app's own write: the journal resets and the next
//! undo refuses. `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, Row};
use alacer_core::flux::RenderBinding;
use alacer_core::history::EditHistory;
use alacer_core::utf16;
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The field the journal belongs to: the text and the bounded journal
/// behind it, exactly the pair a custom widget would own.
struct Journal {
    text: String,
    history: EditHistory,
}

impl Journal {
    /// The one edit funnel: apply the edit and journal its inverse in the
    /// same place — the discipline ADR-0021 requires of every consumer.
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

/// The status line: the journal's own introspection reads, live.
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
/// write signals, nothing mutates widgets (axiom A1), so one batch is one
/// scoped re-render.
fn mirror(
    field: &Journal,
    buffer: &alacer_core::FluxRwSignal<String>,
    status_line: &alacer_core::FluxRwSignal<String>,
) {
    buffer.set(display_text(field));
    status_line.set(status(field));
}

/// One edit route: the span names it, the batch is the one write batch,
/// and the mirror is the only bridge back to the window.
fn route(
    journal: &Rc<RefCell<Journal>>,
    route_name: &'static str,
    edit: impl FnOnce(&mut Journal),
    buffer: &alacer_core::FluxRwSignal<String>,
    status_line: &alacer_core::FluxRwSignal<String>,
) {
    let _route =
        tracing::trace_span!(target: "alacer::editing", "journal", route = route_name).entered();
    alacer_core::flux::Flux::batch_of(|| {
        edit(&mut journal.borrow_mut());
        mirror(&journal.borrow(), buffer, status_line);
    });
}

/// The history chord's route: the journal answers, the refusal is the
/// honest idle frame — consumed, no masks, no mirror.
fn history_route(
    journal: &Rc<RefCell<Journal>>,
    route_name: &'static str,
    chord: impl FnOnce(&mut EditHistory, &mut String) -> Option<(usize, usize)>,
    buffer: &alacer_core::FluxRwSignal<String>,
    status_line: &alacer_core::FluxRwSignal<String>,
) {
    let _route =
        tracing::trace_span!(target: "alacer::editing", "journal", route = route_name).entered();
    let stepped = {
        let mut field = journal.borrow_mut();
        let Journal { text, history } = &mut *field;
        chord(history, text).is_some()
    };
    if stepped {
        tracing::trace!(
            target: "alacer::editing",
            depth = journal.borrow().history.undo_depth(),
            "{route_name} routed"
        );
        alacer_core::flux::Flux::batch_of(|| {
            mirror(&journal.borrow(), buffer, status_line);
        });
    }
}

/// The app-write route: the reset rule — the app wrote the value, not the
/// user, so undo never fights the app's own write.
fn replace_route(
    journal: &Rc<RefCell<Journal>>,
    buffer: &alacer_core::FluxRwSignal<String>,
    status_line: &alacer_core::FluxRwSignal<String>,
) {
    route(
        journal,
        "app-write",
        |field| {
            field.text = "replaced by the app".to_string();
            field.history.reset();
        },
        buffer,
        status_line,
    );
}

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// One control's payload: the boxed route the button drives (every route
/// closure is its own type, so the row carries trait objects).
type RouteHandler = Box<dyn FnMut()>;

/// The demo's controls: one row, five routes. Each handler is the route
/// call, nothing else — the route functions own the vocabulary.
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
            history_route(&journal, "undo", EditHistory::undo, &buffer, &status_line);
        }
    };
    let redo_click = {
        let journal = Rc::clone(journal);
        let buffer = *buffer;
        let status_line = *status_line;
        move || {
            history_route(&journal, "redo", EditHistory::redo, &buffer, &status_line);
        }
    };
    let replace_click = {
        let journal = Rc::clone(journal);
        let buffer = *buffer;
        let status_line = *status_line;
        move || {
            replace_route(&journal, &buffer, &status_line);
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

    let journal = Rc::new(RefCell::new(Journal {
        text: String::new(),
        history: EditHistory::new(),
    }));
    // The reads the labels mirror: handlers write these signals, the
    // bindings map them to the labels — the only bridge (axiom A1).
    let buffer_signal = pipeline.flux_mut().signal(display_text(&journal.borrow()));
    let status_signal = pipeline.flux().signal(status(&journal.borrow()));

    let _ = pipeline.add_child(root, Label::new("The journal (ADR-0021)").boxed());
    let buffer_binding = RenderBinding::new(display_text(&journal.borrow()));
    let buffer_label = pipeline.add_child(root, Label::bound(buffer_binding.clone()).boxed());
    let status_binding = RenderBinding::new(status(&journal.borrow()));
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
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
        "Alacer undo/redo journal",
        (420.0, 220.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
