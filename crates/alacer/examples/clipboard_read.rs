//! The clipboard read names its facts (§13.3, ADR-0048, ADR-0047's
//! third re-open rule): the seam's read door grows beside the write
//! door — one *provided* `read_reported` answering [`ClipboardReadFact`],
//! the M37 provided-method discipline (the provided default maps the
//! `Option` door and names its absence honestly, a real engine names
//! its species) — and the round trip's record carries the read's
//! species beside the paste's verdict. The M39 collapse splits at last:
//! the log answers "did the paste refuse for the read, or for the
//! field", not only "it refused".
//!
//! The demo drives the real editing chords (`PassPipeline::dispatch_key`,
//! the clipboard trail's shape — never a second channel) against a real
//! `TextInput` with the causal observer riding, and swaps the engine
//! between beats: the empty store (the read refused with the `empty`
//! species the engine itself named), the screenshot's clipboard (the
//! `non_text` payload named — a paste never invents content from an
//! image), the sandboxed session (the read refused `unsupported`),
//! the bool-only engine (overriding nothing — the provided default
//! answers, the record honest about the `unnamed` absence), and the
//! field's own refusal (a payload that delivered but sanitized to
//! nothing — the read fact riding null, the M39 pair finally named
//! apart). The refusal beats prove the refuse-don't-munge rule once
//! more: a refused read inserts nothing and queues no masks.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example clipboard_read
//! ```
//!
//! The beats: the empty read (the species the engine named), the
//! non-text read, the sandboxed read, the unnamed absence, and the
//! delivered paste that lands what the memory store holds. Then the
//! window opens for live play: a windowed app keeps the arboard engine
//! `pipeline_default` installs — its own `read_reported` maps the OS's
//! read answers onto the same species (`empty` and `non_text` split by
//! the OS's own second answer, `unsupported` and `os_error` honest),
//! so every platform refusal names itself on the log. The demo swaps
//! the assembly's engine for the in-memory store so the beats never
//! touch the operator's OS clipboard. `ALACER_SOFTWARE=1` forces the
//! softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, TextInput};
use alacer_core::clipboard::{
    ClipboardReadFact, ClipboardReadRefusal, ClipboardSeam, ClipboardWriteFact,
    ClipboardWriteRefusal, MemoryClipboard, install,
};
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Size;
use alacer_core::inspect::{ClipboardOutcome, ClipboardRecord, InspectEvent, InspectObserver};
use alacer_core::passes::PassPipeline;
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the clipboard trail's
/// shape).
const VIEWPORT: Size = Size::new(480.0, 300.0);

/// The demo's field contents: the span every beat moves.
const SEEDED: &str = "held text";

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// A pressed event for `key` with Ctrl held — the command set on
/// Linux/Windows (Meta is the same tier on macOS), so every beat is the
/// exact shape the field's own chords match.
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

/// The sandboxed session's clipboard: no subsystem at all — the engine
/// answers its own read species through the reported door (the
/// read-side fact the `Option` door cannot carry, ADR-0048).
struct KioskClipboard;

impl ClipboardSeam for KioskClipboard {
    fn read_text(&self) -> Option<String> {
        None
    }

    fn write_text(&self, _text: &str) -> bool {
        false
    }

    fn read_reported(&self) -> ClipboardReadFact {
        ClipboardReadFact::Refused(ClipboardReadRefusal::Unsupported)
    }

    fn write_reported(&self, _text: &str) -> ClipboardWriteFact {
        ClipboardWriteFact::Refused(ClipboardWriteRefusal::Unsupported)
    }
}

/// The screenshot's clipboard: content, but not text — the engine
/// names the `non_text` payload (a paste never invents content from an
/// image).
struct ImageOnlyClipboard;

impl ClipboardSeam for ImageOnlyClipboard {
    fn read_text(&self) -> Option<String> {
        None
    }

    fn write_text(&self, text: &str) -> bool {
        text.is_empty() // a screenshot clipboard still accepts writes
    }

    fn read_reported(&self) -> ClipboardReadFact {
        ClipboardReadFact::Refused(ClipboardReadRefusal::NonText)
    }
}

/// The bool-only engine: the read answered through the `Option` door
/// alone, the reported door overridden nothing — the provided default
/// answers, the record honest about the absence (`unnamed`).
struct LockedClipboard;

impl ClipboardSeam for LockedClipboard {
    fn read_text(&self) -> Option<String> {
        None
    }

    fn write_text(&self, _text: &str) -> bool {
        false
    }
}

/// The fact as the trail's reader renders it: the action beside the
/// outcome, the payload's count beside a delivery, the refusal species
/// beside a refusal — and the read's own species beside a refused
/// paste, the log answering "did the paste refuse for the read, or for
/// the field".
fn describe(record: &ClipboardRecord) -> String {
    let action = record.action.name();
    let outcome = match record.outcome {
        ClipboardOutcome::Delivered => format!("the payload moved ({})", record.chars),
        ClipboardOutcome::Refused(refusal) => {
            let engine = record.engine.map_or(String::new(), |species| {
                format!(" (write: {})", species.name())
            });
            let read = record.read.map_or(String::new(), |species| {
                format!(" (read: {})", species.name())
            });
            format!("refused: {}{}{}", refusal.name(), engine, read)
        }
    };
    format!("{action}: {outcome}")
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the drain rode (axiom A1: signals in, never widgets).
struct Trail {
    clipboards: Rc<RefCell<std::collections::VecDeque<ClipboardRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The observer half: boxed into
    /// [`alacer_core::passes::PassPipeline::set_observer`], it receives
    /// every `InspectEvent`, keeps the round-trip facts, and mirrors
    /// the newest into the trail label's signal — the write queues
    /// masks, the frame renders them.
    fn writer(&self) -> TrailWriter {
        TrailWriter {
            clipboards: Rc::clone(&self.clipboards),
            trail_line: self.trail_line,
        }
    }

    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<ClipboardRecord> {
        self.clipboards.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    clipboards: Rc<RefCell<std::collections::VecDeque<ClipboardRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::Clipboard(record) = event else {
            return;
        };
        let line = describe(&record);
        {
            let mut ring = self.clipboards.borrow_mut();
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

/// One beat: the span names the route, the real dispatch answers, the
/// round trip streams to the observer, and the response is logged —
/// the funnel's own order, no second channel.
fn beat(pipeline: &mut PassPipeline, label: &'static str, event: &KeyEvent) {
    let _route =
        tracing::trace_span!(target: "alacer::clipboard_read", "beat", label = label).entered();
    let response = pipeline.dispatch_key(event);
    settle(pipeline);
    tracing::info!(
        target: "alacer::clipboard_read",
        handled = matches!(response, alacer_core::EventResponse::Handled),
        "{label}"
    );
}

/// The labels the demo mirrors through: the engine line is the app's
/// half (the beats write which engine the field rode), the trail line
/// is the log's half (the observer writes it at the drain). Handlers
/// write signals, never widgets (axiom A1).
fn mount_labels(
    pipeline: &mut PassPipeline,
    root: alacer_core::NodeId,
    engine_signal: alacer_core::FluxRwSignal<String>,
    trail_signal: alacer_core::FluxRwSignal<String>,
) {
    let engine_binding = RenderBinding::new("the engine: (the seam the field rides)".to_string());
    let engine_label = pipeline.add_child(root, Label::bound(engine_binding.clone()).boxed());
    let trail_binding = RenderBinding::new("the trail: (nothing streamed yet)".to_string());
    let trail_label = pipeline.add_child(root, Label::bound(trail_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        engine_signal,
        engine_label,
        &engine_binding,
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

/// The beats the demo walks before the window opens: the empty read
/// (the species the engine named), the non-text read, the sandboxed
/// read, the bool-only engine's honest absence, and the delivered
/// paste that lands what the store holds — each a real chord through
/// the real dispatch against the focused field.
fn run_beats(
    pipeline: &mut PassPipeline,
    clipboard: &Rc<MemoryClipboard>,
    engine_signal: alacer_core::FluxRwSignal<String>,
) {
    // Beat one: the empty store — the engine naming its own species
    // beside the round trip's `empty_clipboard` verdict.
    install(Rc::clone(clipboard) as Rc<dyn ClipboardSeam>);
    beat(
        pipeline,
        "beat 1: Ctrl+V over an empty store — the read names `empty`",
        &ctrl(Key::Char('v')),
    );
    engine_signal.set("the engine: the memory store, empty — the read refused".to_string());

    // Beat two: the screenshot's clipboard — content, but not text; the
    // `non_text` payload named, the paste never inventing content from
    // an image.
    install(Rc::new(ImageOnlyClipboard) as Rc<dyn ClipboardSeam>);
    beat(
        pipeline,
        "beat 2: Ctrl+V over a screenshot's clipboard — the read names `non_text`",
        &ctrl(Key::Char('v')),
    );
    engine_signal.set("the engine: the screenshot — content, but not text".to_string());

    // Beat three: the sandboxed session — no subsystem at all, the
    // read refused with the species the engine answered.
    install(Rc::new(KioskClipboard) as Rc<dyn ClipboardSeam>);
    beat(
        pipeline,
        "beat 3: Ctrl+V in the sandboxed session — the read names `unsupported`",
        &ctrl(Key::Char('v')),
    );
    engine_signal.set("the engine: the sandbox — no clipboard subsystem at all".to_string());

    // Beat four: the bool-only engine — the reported door overridden
    // nothing, the provided default answering, the record honest about
    // the absence. The refused read inserts nothing (ADR-0020).
    install(Rc::new(LockedClipboard) as Rc<dyn ClipboardSeam>);
    beat(
        pipeline,
        "beat 4: Ctrl+V through a bool-only engine — the absence named `unnamed`",
        &ctrl(Key::Char('v')),
    );
    engine_signal
        .set("the engine: the locked door — the read refused, nothing inserted".to_string());

    // Beat five: the field's own refusal — the store holds only a
    // payload that sanitizes to nothing; the read delivered, the
    // verdict `empty_clipboard` all the same, the read fact riding
    // null. The M39 pair, named apart at last.
    install(Rc::clone(clipboard) as Rc<dyn ClipboardSeam>);
    clipboard.set("");
    beat(
        pipeline,
        "beat 5: Ctrl+V over a payload that sanitizes to nothing — the field refused, the read null",
        &ctrl(Key::Char('v')),
    );
    engine_signal
        .set("the engine: the memory store — the read delivered, the field refused".to_string());

    // Beat six: the delivered paste — the payload lands what the store
    // holds, no read refusal riding.
    clipboard.set(SEEDED);
    beat(
        pipeline,
        "beat 6: Ctrl+V — the paste lands what the store holds",
        &ctrl(Key::Char('v')),
    );
    engine_signal.set(format!(
        "the engine: the memory store — {} chars landed, the read null",
        SEEDED.chars().count()
    ));
}

fn main() {
    let mut pipeline = pipeline_default();

    // The demo's own assembly: the honest in-memory store replaces the
    // arboard engine `pipeline_default` installs, so the beats never
    // touch the operator's OS clipboard (the harness's discipline,
    // ADR-0020 — a windowed app keeps the engine, whose own
    // `read_reported` maps the OS's read answers onto the species).
    let clipboard = Rc::new(MemoryClipboard::default());
    install(Rc::clone(&clipboard) as Rc<dyn ClipboardSeam>);

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The clipboard read's facts, streamed to the causal log (ADR-0048)").boxed(),
    );
    let field = pipeline.add_child(
        root,
        TextInput::new(SEEDED)
            .placeholder("paste here, or play Ctrl+C / V")
            .boxed(),
    );

    let engine_signal = pipeline
        .flux_mut()
        .signal("the engine: (the seam the field rides)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    mount_labels(&mut pipeline, root, engine_signal, trail_signal);

    // The observer rides from here on: every round trip's record
    // streams into the trail label the frame after the chord resolves.
    let trail = Trail {
        clipboards: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    // Focus the field, then walk the beats: every chord routes to the
    // focused input, the exact route the window's real keys ride.
    pipeline.request_focus(field);
    settle(&mut pipeline);
    run_beats(&mut pipeline, &clipboard, engine_signal);

    // The window: live play — type, select, Ctrl+C/V. A windowed app
    // keeps the arboard engine; its read refusals name their species on
    // the log while the labels mirror the trail.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: Ctrl+C in another app, then Ctrl+V — the trail mirrors the log").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer clipboard read",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the round trips' history as data. The
    // payload never rode the log; the counts and the read species did.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::clipboard_read",
        records = read_back.len(),
        delivered = read_back
            .iter()
            .filter(|record| record.outcome == ClipboardOutcome::Delivered)
            .count(),
        refusals = read_back
            .iter()
            .filter(|record| matches!(record.outcome, ClipboardOutcome::Refused(_)))
            .count(),
        reads_named = read_back.iter().filter(|record| record.read.is_some()).count(),
        "the round trips streamed as they resolved — the causal log carries the read's facts"
    );
}
