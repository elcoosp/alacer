//! The keymap rides the causal log (§13.3, ADR-0042): every keymap
//! consult the dispatch funnel runs streams one [`KeyRecord`] — the
//! chord the funnel asked the app's shortcuts about beside the species
//! it got back ([`KeymapConsult`]: a fired binding, a table that named
//! nothing, a window with no keymap at all). The hook ADR-0041's own
//! re-open rule pre-registered ("a consumer needs the shortcut's
//! firings in the causal log") cashed: a status line, an inspector
//! trail, or a test routes on "did a shortcut take this key — or why
//! not" without re-implementing the observation at the chord site.
//!
//! The demo drives the real dispatch (`PassPipeline::dispatch_key`,
//! the dpad demo's shape — never a second channel) with the causal
//! observer riding: the beats run before the window opens, each one a
//! real key event through the funnel, and the *trail* label mirrors
//! what the log streamed. The chord is the fired binding's identity
//! (one binding per chord, ADR-0041), so the record's chord answers
//! "which shortcut" without a second naming channel.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example keymap_trail
//! ```
//!
//! The beats: the uninstalled window's honest answer, the fired chord,
//! the miss that falls through to the framework's fallbacks, the
//! rebind that moves the record's chord with it. Then the window opens
//! for live play — `Ctrl+Up`/`Ctrl+Down` step the count, `Home`
//! resets, and every consult streams beside the focus trail.
//! `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, Row};
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Size;
use alacer_core::inspect::{InspectEvent, InspectObserver, KeyRecord};
use alacer_core::keymap::{Chord, Keymap, KeymapConsult};
use alacer_core::passes::PassPipeline;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the dpad demo's shape).
const VIEWPORT: Size = Size::new(480.0, 300.0);

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// A pressed event for `key` carrying `modifiers` — the demo's only
/// key factory, so every beat is the exact shape the funnel consults.
fn pressed(key: Key, modifiers: Modifiers) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers,
        text: None,
    }
}

/// The plain modifiers: nothing held.
fn none() -> Modifiers {
    Modifiers::default()
}

/// Control held (the command set on Linux/Windows).
fn ctrl() -> Modifiers {
    Modifiers {
        control: true,
        ..Modifiers::default()
    }
}

/// The chord as the trail's reader renders it: the modifier shape and
/// the key — the record's chord *is* the fired binding's identity
/// (one binding per chord, ADR-0041). A real settings UI would grow
/// this into shared vocabulary; the demo keeps its own renderer.
fn describe_chord(chord: Chord) -> String {
    let modifiers = chord.modifiers();
    let mut shape = Vec::new();
    if modifiers.shift {
        shape.push("Shift");
    }
    if modifiers.control {
        shape.push("Ctrl");
    }
    if modifiers.alt {
        shape.push("Alt");
    }
    if modifiers.meta {
        shape.push("Cmd");
    }
    let key = match chord.key() {
        Key::Char(character) => format!("'{character}'"),
        Key::Enter => "Enter".to_string(),
        Key::Backspace => "Backspace".to_string(),
        Key::Delete => "Delete".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Space => "Space".to_string(),
        Key::Escape => "Escape".to_string(),
        Key::ArrowLeft => "ArrowLeft".to_string(),
        Key::ArrowRight => "ArrowRight".to_string(),
        Key::ArrowUp => "ArrowUp".to_string(),
        Key::ArrowDown => "ArrowDown".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::Unknown => "Unknown".to_string(),
    };
    if shape.is_empty() {
        key
    } else {
        format!("{}+{key}", shape.join("+"))
    }
}

/// The fact as the trail's reader renders it: the chord beside the
/// species the consult answered.
fn describe(record: KeyRecord) -> String {
    let chord = describe_chord(record.chord);
    let outcome = match record.outcome {
        KeymapConsult::Fired => "a shortcut took the key".to_string(),
        KeymapConsult::FellThrough => "no binding named it — the fallbacks' key".to_string(),
        KeymapConsult::Uninstalled => {
            "no keymap installed — the window has no shortcuts".to_string()
        }
    };
    format!("{chord}: {outcome}")
}

/// The causal trail the demo shares between the pipeline's observer
/// and the *trail* label's signal: the observer half writes the fact
/// into a bounded ring (the recorder's own discipline, §14.3) and
/// mirrors it into the signal — one write batch, the label re-renders
/// the same frame the consult rode (axiom A1: signals in, never
/// widgets).
struct Trail {
    keys: Rc<RefCell<std::collections::VecDeque<KeyRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The observer half: boxed into
    /// [`alacer_core::passes::PassPipeline::set_observer`], it receives
    /// every `InspectEvent`, keeps the shortcut facts, and mirrors the
    /// newest into the trail label's signal — the write queues masks,
    /// the frame renders them.
    fn writer(&self) -> TrailWriter {
        TrailWriter {
            keys: Rc::clone(&self.keys),
            trail_line: self.trail_line,
        }
    }

    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<KeyRecord> {
        self.keys.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    keys: Rc<RefCell<std::collections::VecDeque<KeyRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::Key(record) = event else {
            return;
        };
        {
            let mut ring = self.keys.borrow_mut();
            if ring.len() >= 64 {
                ring.pop_front();
            }
            ring.push_back(record);
        }
        self.trail_line.set(describe(record));
    }
}

/// The table the beats walk, live-rebinding behind the seam: the trait
/// is the funnel's only contract, so the table moves in place under it
/// (the m35 suite's own shape, the m35 example's wrapper).
struct Shared(Rc<RefCell<Keymap>>);

impl alacer_core::keymap::KeymapSeam for Shared {
    fn dispatch(&self, event: &KeyEvent) -> bool {
        self.0.borrow().dispatch(event)
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
/// consult streams to the observer, and the response is logged — the
/// funnel's own order, no second channel.
fn beat(pipeline: &mut PassPipeline, label: &'static str, event: &KeyEvent) {
    let _route =
        tracing::trace_span!(target: "alacer::keymap_trail", "beat", label = label).entered();
    let response = pipeline.dispatch_key(event);
    settle(pipeline);
    tracing::info!(
        target: "alacer::keymap_trail",
        handled = matches!(response, alacer_core::EventResponse::Handled),
        "{label}"
    );
}

/// The labels the demo mirrors through: the count is the app's half
/// (the bindings write it through one batch), the trail line is the
/// log's half (the observer writes it at the consult). Handlers write
/// signals, never widgets (axiom A1).
fn mount_labels(
    pipeline: &mut PassPipeline,
    root: alacer_core::NodeId,
    trail_signal: alacer_core::FluxRwSignal<String>,
) -> alacer_core::FluxRwSignal<i32> {
    let count = pipeline.flux_mut().signal(0_i32);
    let count_binding = RenderBinding::new("Shortcuts fired: 0".to_string());
    let count_label = pipeline.add_child(root, Label::bound(count_binding.clone()).boxed());
    let trail_binding = RenderBinding::new("the trail: (nothing streamed yet)".to_string());
    let trail_label = pipeline.add_child(root, Label::bound(trail_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        count,
        count_label,
        &count_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Shortcuts fired: {n}"),
    );
    pipeline.flux_mut().bind_value(
        trail_signal,
        trail_label,
        &trail_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );
    count
}

/// The beats the demo walks before the window opens: the uninstalled
/// window's honest answer, the fired chord naming itself, the miss
/// that falls through, and the rebind moving the record's chord with
/// it — each a real key event through the real dispatch, the table
/// live-rebinding behind the seam between the beats.
fn run_beats(
    pipeline: &mut PassPipeline,
    table: &Rc<RefCell<Keymap>>,
    count: alacer_core::FluxRwSignal<i32>,
) {
    // Beat one: the window before the assembly's install — the funnel
    // still consults (the read is total), nothing answers, and the
    // record names the uninstalled window instead of collapsing it
    // into a table's silence.
    beat(
        pipeline,
        "beat 1: Ctrl+Up with no keymap installed — the honest Uninstalled",
        &pressed(Key::ArrowUp, ctrl()),
    );

    // The assembly installs the table: the chords step the count
    // wherever the focus sits, Home resets it — handlers write signals
    // in one batch, never widgets (axiom A1).
    table.borrow_mut().bind(
        Chord::new(Key::ArrowLeft).control(),
        "increment",
        move || {
            alacer_core::flux::Flux::batch_of(|| {
                let current = count.try_get_untracked().unwrap_or_default();
                count.set(current + 1);
            });
        },
    );
    table
        .borrow_mut()
        .bind(Chord::new(Key::Home), "reset", move || {
            alacer_core::flux::Flux::batch_of(|| count.set(0));
        });
    alacer_core::keymap::install(Rc::new(Shared(Rc::clone(table))));

    // Beat two: the fired chord — the binding runs, the funnel answers
    // Handled before the fallbacks, and the record's chord names the
    // shortcut that took the key.
    beat(
        pipeline,
        "beat 2: Ctrl+Left bound — the Fired record names the chord",
        &pressed(Key::ArrowLeft, ctrl()),
    );

    // Beat three: the miss — a seam answered, no binding named the
    // chord, and the fall-through streams too (that is exactly the
    // fact a trail reader wants: why didn't my shortcut fire).
    beat(
        pipeline,
        "beat 3: plain 'x' unbound — the FellThrough record",
        &pressed(Key::Char('x'), none()),
    );

    // Beat four: the settings story — the same chord rebinds to the
    // vertical pair; the displaced action never fires again and the
    // record's chord moves with the rebind.
    {
        let mut table = table.borrow_mut();
        table.unbind(&Chord::new(Key::ArrowLeft).control());
        table.bind(
            Chord::new(Key::ArrowUp).control(),
            "increment (up)",
            move || {
                alacer_core::flux::Flux::batch_of(|| {
                    let current = count.try_get_untracked().unwrap_or_default();
                    count.set(current + 1);
                });
            },
        );
        table.bind(
            Chord::new(Key::ArrowDown).control(),
            "decrement (down)",
            move || {
                alacer_core::flux::Flux::batch_of(|| {
                    let current = count.try_get_untracked().unwrap_or_default();
                    count.set(current - 1);
                });
            },
        );
    }
    beat(
        pipeline,
        "beat 4: Ctrl+Up after the rebind — the chord moved with it",
        &pressed(Key::ArrowUp, ctrl()),
    );
}

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The shortcuts' facts, streamed to the causal log (ADR-0042)").boxed(),
    );

    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let count = mount_labels(&mut pipeline, root, trail_signal);

    // The observer rides from here on: every consult's record streams
    // into the trail label the frame after the funnel resolves it.
    let trail = Trail {
        keys: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    let table = Rc::new(RefCell::new(Keymap::new()));
    run_beats(&mut pipeline, &table, count);

    // The window: live chords play the same funnel — Ctrl+Up /
    // Ctrl+Down step, Home resets, and every consult streams beside
    // the focus trail while the labels mirror the log. *Unbind chords*
    // is the settings story's way back: the unbound keys fall through
    // honestly, each fall-through its own record.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: Ctrl+Up/Down step, Home resets — the trail mirrors the log").boxed(),
    );
    let controls = pipeline.add_child(root, Row::new().gap(8.0).boxed());
    let live_table = Rc::clone(&table);
    let _ = pipeline.add_child(
        controls,
        Button::new("Unbind chords")
            .on_click(move || {
                let mut table = live_table.borrow_mut();
                table.unbind(&Chord::new(Key::ArrowUp).control());
                table.unbind(&Chord::new(Key::ArrowDown).control());
            })
            .boxed(),
    );

    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer keymap trail",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the shortcuts' history as data.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::keymap_trail",
        records = read_back.len(),
        fired = read_back
            .iter()
            .filter(|record| record.outcome == KeymapConsult::Fired)
            .count(),
        fell_through = read_back
            .iter()
            .filter(|record| record.outcome == KeymapConsult::FellThrough)
            .count(),
        "the shortcuts' facts streamed as they resolved — the causal log carries the key trail"
    );
}
