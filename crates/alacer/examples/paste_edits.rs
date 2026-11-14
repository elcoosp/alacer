//! The clipboard chords' field changes join the key-edit vocabulary
//! (§13.3, ADR-0066): ADR-0064's third kill rule cashes — a consumer
//! prices the chords' edits *at the field* on the log — the paste's
//! shape and the cut's span stream their own
//! [`alacer_core::KeyEditRecord`] now: the chord's own action
//! (`paste`/`cut`, never a munged insert or delete) and driver
//! (`clipboard`) beside the shape priced by lengths (never the
//! payload — a field's value can carry a secret, ADR-0045), while the
//! round trip keeps the clipboard trail's fact (one fact per trail,
//! never a double-priced edit).
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example paste_edits
//! ```
//!
//! The beats: the paste named by its driver and shape, the paste over
//! a selection pricing both spans, the cut's span, the refused paste
//! streaming nothing on the typing door (the honest absence — the
//! clipboard trail names the refusal), and the copy that edits nothing
//! riding none. The demo swaps the clipboard engine between beats (the
//! `clipboard_engine` pattern): an in-memory store seeded per beat, so
//! the log is deterministic. Then the window opens for live play.
//! `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, TextInput};
use alacer_core::clipboard::{MemoryClipboard, install};
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
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
/// did the paste change in this field" answered from one record.
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

/// One chord beat: the event routed the real way — the fact streaming
/// to the observer the frame after the edit resolves.
fn chord_beat(pipeline: &mut PassPipeline, label: &'static str, event: &KeyEvent) {
    let _route =
        tracing::trace_span!(target: "alacer::paste_edits", "beat", label = label).entered();
    let _consumed = pipeline.dispatch_key(event);
    settle(pipeline);
    tracing::info!(target: "alacer::paste_edits", "{label}");
}

/// A Ctrl press (the clipboard chords' command set; ADR-0020).
fn ctrl(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            ..Modifiers::default()
        },
        text: None,
    }
}

/// The beats the demo walks before the window opens: the paste named
/// by its driver and shape, the paste over a selection pricing both
/// spans, the cut's span, the refused paste riding the honest absence,
/// and the copy that edits nothing streaming none.
fn run_beats(
    pipeline: &mut PassPipeline,
    status_signal: alacer_core::FluxRwSignal<String>,
    store: &Rc<MemoryClipboard>,
    field_node: alacer_core::NodeId,
) {
    // Beat one: the paste at the caret — the chord's own action and
    // driver, the payload's shape priced by lengths.
    store.set("seed text");
    for ch in "start: ".chars() {
        let _ = pipeline.dispatch_key(&KeyEvent::char(ch));
    }
    chord_beat(
        pipeline,
        "beat 1: the paste — the chord's own action and driver",
        &ctrl(Key::Char('v')),
    );
    status_signal.set("the paste door: what did the chord change".to_string());

    // Beat two: the paste over a selection — both spans priced, the
    // selection the paste consumed beside the payload it carried.
    store.set("the new value");
    chord_beat(
        pipeline,
        "beat 2: select-all then paste — both spans priced",
        &ctrl(Key::Char('a')),
    );
    chord_beat(pipeline, "beat 2: the paste lands", &ctrl(Key::Char('v')));

    // Beat three: the cut — the span the deletion took when the value
    // moved out, the round trip's fact streaming first (clipboard →
    // edit, the causal order the cut ran).
    store.set("");
    chord_beat(
        pipeline,
        "beat 3: select-all then cut — the span it moved out",
        &ctrl(Key::Char('a')),
    );
    chord_beat(pipeline, "beat 3: the cut lands", &ctrl(Key::Char('x')));

    // Beat four: the refused paste — the store empty, the read refusing,
    // the field inserting nothing: the journal recorded nothing, the
    // typing door answers no fact (the clipboard trail names the
    // refusal).
    store.set("");
    chord_beat(
        pipeline,
        "beat 4: the refused paste — the honest absence",
        &ctrl(Key::Char('v')),
    );

    // Beat five: the copy — a chord that edits nothing: no journal
    // step, no typing-door fact, the round trip the clipboard trail's.
    chord_beat(
        pipeline,
        "beat 5: select-all then copy — edits nothing, streams none",
        &ctrl(Key::Char('a')),
    );
    chord_beat(pipeline, "beat 5: the copy lands", &ctrl(Key::Char('c')));

    // The app door's write beside them — no chord fact rides (the
    // text-write trail names it), the doors tellable apart on the log.
    let _route =
        tracing::trace_span!(target: "alacer::paste_edits", "beat", label = "beat 6").entered();
    let mut guard = pipeline.arena_mut().mutate(field_node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_value("written by the app");
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    settle(pipeline);
    tracing::info!(
        target: "alacer::paste_edits",
        "beat 6: the app's set_value — no chord fact rides"
    );
}

fn main() {
    let mut pipeline = pipeline_default();

    // The demo's own clipboard store: an in-memory engine the beats
    // seed per beat, so the log is deterministic (headless CI never
    // races the operator's OS clipboard — the `clipboard_engine`
    // pattern).
    let store = Rc::new(MemoryClipboard::default());
    install(Rc::clone(&store) as Rc<dyn alacer_core::clipboard::ClipboardSeam>);

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The clipboard chords' field changes join the causal log (ADR-0066)").boxed(),
    );
    let field = TextInput::new("field");
    let field_node = pipeline.add_child(root, field.boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("Live: a paste, a cut — each names its shape and driver").boxed(),
    );

    let status_signal = pipeline
        .flux_mut()
        .signal("the paste door: (the tree answers what it can)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let status_binding =
        RenderBinding::new("the paste door: (the tree answers what it can)".to_string());
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

    // The observer rides from here on: every chord's field change
    // streams into the trail label the frame after the edit resolves.
    let trail = Trail {
        edits: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    settle(&mut pipeline);
    pipeline.request_focus(field_node);
    settle(&mut pipeline);
    run_beats(&mut pipeline, status_signal, &store, field_node);
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer paste edits",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the chord's driver beside every edit.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::paste_edits",
        records = read_back.len(),
        chords = read_back
            .iter()
            .filter(|record| record.driver == KeyEditDriver::Clipboard)
            .count(),
        keyboard = read_back
            .iter()
            .filter(|record| record.driver == KeyEditDriver::Keyboard)
            .count(),
        "the edits streamed as they resolved — the causal log carries the chords' field changes"
    );
}
