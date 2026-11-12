//! The clipboard rides the causal log (§13.3, ADR-0045): every clipboard
//! round trip the editing chords answer — a copy, a cut, a paste, and
//! every honest refusal — streams one [`ClipboardRecord`], the payload's
//! scalar count beside the outcome and the refusal species naming why
//! nothing moved. The ride ADR-0010's additive rule and the M31/M33/M36
//! rhythm pre-register cashed: a status line, an inspector trail, or a
//! test routes on "did the text move — or why not" without
//! re-implementing the observation at the chord site.
//!
//! The demo drives the real editing chords (`PassPipeline::dispatch_key`,
//! the keymap trail's shape — never a second channel) against a real
//! `TextInput` with the causal observer riding: the beats run before the
//! window opens, each one a real key event through the funnel, and the
//! *trail* label mirrors what the log streamed. The record carries the
//! payload's count, never the payload — a clipboard can carry a secret,
//! and the log is a devtools surface, not a shadow clipboard.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example clipboard_trail
//! ```
//!
//! The beats: the copy without a selection (the honest `no_selection`),
//! the copy's payload counted, the cut that moves the span out before
//! the journal's own step, the paste that lands what the copy moved,
//! and the locked door — a refused write that never deletes. Then the
//! window opens for live play: type, select, and play Ctrl+C/X/V —
//! every round trip streams beside the edit trail. The demo swaps the
//! assembly's arboard engine for the honest in-memory store so the
//! beats never touch the operator's OS clipboard; a windowed app keeps
//! the engine `pipeline_default` installs. `ALACER_SOFTWARE=1` forces
//! the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, TextInput};
use alacer_core::clipboard::{ClipboardSeam, MemoryClipboard, install};
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Size;
use alacer_core::inspect::{ClipboardOutcome, ClipboardRecord, InspectEvent, InspectObserver};
use alacer_core::passes::PassPipeline;
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the keymap trail's shape).
const VIEWPORT: Size = Size::new(480.0, 300.0);

/// The demo's field contents: the span every beat moves.
const SEEDED: &str = "hello log";

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

/// The clipboard as a locked door: every read honest (`None`), every
/// write refused — the permission/escape story the refuse-don't-munge
/// rule is written for (the m39 suite's own seam).
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
/// beside a refusal — the length names the fact, never the payload.
fn describe(record: &ClipboardRecord) -> String {
    let action = record.action.name();
    let outcome = match record.outcome {
        ClipboardOutcome::Delivered => format!("the payload moved ({})", record.chars),
        ClipboardOutcome::Refused(refusal) => format!("refused: {}", refusal.name()),
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
        tracing::trace_span!(target: "alacer::clipboard_trail", "beat", label = label).entered();
    let response = pipeline.dispatch_key(event);
    settle(pipeline);
    tracing::info!(
        target: "alacer::clipboard_trail",
        handled = matches!(response, alacer_core::EventResponse::Handled),
        "{label}"
    );
}

/// The labels the demo mirrors through: the clipboard line is the
/// app's half (the beats write what the seam holds), the trail line is
/// the log's half (the observer writes it at the drain). Handlers
/// write signals, never widgets (axiom A1).
fn mount_labels(
    pipeline: &mut PassPipeline,
    root: alacer_core::NodeId,
    clipboard_signal: alacer_core::FluxRwSignal<String>,
    trail_signal: alacer_core::FluxRwSignal<String>,
) {
    let clipboard_binding =
        RenderBinding::new("the clipboard: (empty — nothing moved yet)".to_string());
    let clipboard_label = pipeline.add_child(root, Label::bound(clipboard_binding.clone()).boxed());
    let trail_binding = RenderBinding::new("the trail: (nothing streamed yet)".to_string());
    let trail_label = pipeline.add_child(root, Label::bound(trail_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        clipboard_signal,
        clipboard_label,
        &clipboard_binding,
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

/// The beats the demo walks before the window opens: the refusal, the
/// counted copy, the cut that precedes the journal's step, the paste
/// that lands what the cut moved, and the locked door whose refused
/// write never deletes — each a real chord through the real dispatch
/// against the focused field.
fn run_beats(
    pipeline: &mut PassPipeline,
    clipboard: &Rc<MemoryClipboard>,
    clipboard_signal: alacer_core::FluxRwSignal<String>,
) {
    // Beat one: the copy without a selection — a copy never invents
    // content, and the refusal streams too (the M28 honesty, the
    // clipboard tier's own).
    beat(
        pipeline,
        "beat 1: Ctrl+C with no selection — the honest no_selection refusal",
        &ctrl(Key::Char('c')),
    );
    clipboard_signal.set("the clipboard: (empty — a refused copy wrote nothing)".to_string());

    // Beat two: the counted copy — select all, then copy; the record
    // carries the payload's scalar count beside the delivery.
    let _ = pipeline.dispatch_key(&ctrl(Key::Char('a')));
    settle(pipeline);
    beat(
        pipeline,
        "beat 2: Ctrl+A, Ctrl+C — the copy's payload counted",
        &ctrl(Key::Char('c')),
    );
    clipboard_signal.set(format!(
        "the clipboard: {} chars — what the record counted",
        clipboard.text().map_or(0, |text| text.chars().count())
    ));

    // Beat three: the cut — the write lands, the fact streams, and the
    // journal's deletion follows it (the trail reads clipboard → edit,
    // the causal order the round trip ran).
    let _ = pipeline.dispatch_key(&ctrl(Key::Char('a')));
    settle(pipeline);
    beat(
        pipeline,
        "beat 3: Ctrl+A, Ctrl+X — the cut moves the span out, the journal follows",
        &ctrl(Key::Char('x')),
    );
    clipboard_signal.set(format!(
        "the clipboard: {} chars — the field emptied behind it",
        clipboard.text().map_or(0, |text| text.chars().count())
    ));

    // Beat four: the paste — the sanitized read lands what the copy
    // moved, the count naming what landed.
    beat(
        pipeline,
        "beat 4: Ctrl+V — the paste lands what the cut moved out",
        &ctrl(Key::Char('v')),
    );
    clipboard_signal.set(format!(
        "the clipboard: {} chars — the field restored",
        clipboard.text().map_or(0, |text| text.chars().count())
    ));

    // Beat five: the locked door — the platform refuses the write, the
    // record names `write_refused`, and the selection survives the
    // clipboard that failed it (the refuse-don't-munge rule, now a
    // fact on the log).
    install(Rc::new(LockedClipboard) as Rc<dyn ClipboardSeam>);
    let _ = pipeline.dispatch_key(&ctrl(Key::Char('a')));
    settle(pipeline);
    beat(
        pipeline,
        "beat 5: Ctrl+X through a locked clipboard — the refused write never deletes",
        &ctrl(Key::Char('x')),
    );
    clipboard_signal.set("the clipboard: (locked — the write refused)".to_string());

    // The demo unlocks the door for the window's live play: a fresh
    // in-memory store carrying what the copy moved, so the field's
    // next paste is the operator's own.
    clipboard.set(SEEDED);
    install(Rc::clone(clipboard) as Rc<dyn ClipboardSeam>);
    clipboard_signal.set(format!(
        "the clipboard: {} chars — the door unlocked for live play",
        SEEDED.chars().count()
    ));
}

fn main() {
    let mut pipeline = pipeline_default();

    // The demo's own assembly: the honest in-memory store replaces the
    // arboard engine `pipeline_default` installs, so the beats never
    // touch the operator's OS clipboard (the harness's discipline,
    // ADR-0020 — a windowed app keeps the engine).
    let clipboard = Rc::new(MemoryClipboard::default());
    install(Rc::clone(&clipboard) as Rc<dyn ClipboardSeam>);

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The clipboard's facts, streamed to the causal log (ADR-0045)").boxed(),
    );
    let field = pipeline.add_child(
        root,
        TextInput::new(SEEDED)
            .placeholder("type, select, then play Ctrl+C / X / V")
            .boxed(),
    );

    let clipboard_signal = pipeline
        .flux_mut()
        .signal("the clipboard: (empty — nothing moved yet)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    mount_labels(&mut pipeline, root, clipboard_signal, trail_signal);

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
    run_beats(&mut pipeline, &clipboard, clipboard_signal);

    // The window: live play — type, select, Ctrl+C/X/V. Every round
    // trip streams beside the edit trail while the labels mirror the
    // log; the focus rides the field the beats left it on.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: type, select, Ctrl+C/X/V — the trail mirrors the log").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer clipboard trail",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the round trips' history as data. The
    // payload never rode the log; the counts did.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::clipboard_trail",
        records = read_back.len(),
        delivered = read_back
            .iter()
            .filter(|record| record.outcome == ClipboardOutcome::Delivered)
            .count(),
        refusals = read_back
            .iter()
            .filter(|record| {
                matches!(record.outcome, ClipboardOutcome::Refused(_))
            })
            .count(),
        "the round trips streamed as they resolved — the causal log carries the clipboard trail"
    );
}
