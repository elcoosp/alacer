//! The binding names itself in the record (§13.3, ADR-0043): every
//! fired keymap consult's [`KeyRecord`] carries the binding's prose
//! name beside its chord and outcome — the hook ADR-0042's re-open
//! rule pre-registered ("a consumer needs the fired binding's prose
//! name in the record") cashed. A trail reader that wants the
//! shortcut's *name* — a settings UI reading the log, an inspector row
//! a human labels — routes on the record alone, never a chord → name
//! table kept beside the log.
//!
//! The demo drives the real dispatch (`PassPipeline::dispatch_key`,
//! the `keymap_trail` demo's shape — never a second channel) with the
//! causal observer riding, and names its bindings three ways the seam
//! carries: the provided [`Keymap`] table (the name `bind` was given),
//! a custom [`alacer_core::keymap::KeymapSeam`] whose shortcuts come
//! from "somewhere else" and answers `fired_name` beside `dispatch`,
//! and a seam that overrides nothing — the default `None` the record
//! stays honest about. The rebind moves the name with the chord: the
//! settings story, read back from the log.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example shortcut_names
//! ```
//!
//! The beats: the fired chord naming itself, the miss that names
//! nothing, the custom seam's own name, the default's honest absence,
//! the rebind moving the name with the chord. Then the window opens
//! for live play — `Ctrl+Up`/`Ctrl+Down` step the count, `Home`
//! resets, and every consult streams beside the focus trail, the
//! *trail* label mirroring the named record. `ALACER_SOFTWARE=1`
//! forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label};
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Size;
use alacer_core::inspect::{InspectEvent, InspectObserver, KeyRecord};
use alacer_core::keymap::{Chord, Keymap, KeymapConsult, KeymapSeam};
use alacer_core::passes::PassPipeline;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the `keymap_trail` demo's
/// shape).
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

/// The fact as the trail's reader renders it: the chord beside the
/// species the consult answered, the fired binding's prose name beside
/// it when the consult fired and the seam named one — the record's own
/// fields, no side table.
fn describe(record: &KeyRecord) -> String {
    let chord = match record.chord.key() {
        Key::Char(character) => format!("'{character}'"),
        Key::ArrowUp => "ArrowUp".to_string(),
        Key::ArrowDown => "ArrowDown".to_string(),
        Key::Home => "Home".to_string(),
        _ => "…".to_string(),
    };
    let shape = if record.chord.modifiers().control {
        format!("Ctrl+{chord}")
    } else {
        chord
    };
    let outcome = match record.outcome {
        KeymapConsult::Fired => match record.name.as_deref() {
            Some(name) => format!("a shortcut took the key — {name}"),
            None => "a shortcut took the key (the seam names no binding)".to_string(),
        },
        KeymapConsult::FellThrough => "no binding named it — the fallbacks' key".to_string(),
        KeymapConsult::Uninstalled => {
            "no keymap installed — the window has no shortcuts".to_string()
        }
    };
    format!("{shape}: {outcome}")
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
    /// every `InspectEvent`, keeps the named shortcut facts, and
    /// mirrors the newest into the trail label's signal — the write
    /// queues masks, the frame renders them.
    fn writer(&self) -> TrailWriter {
        TrailWriter {
            keys: Rc::clone(&self.keys),
            trail_line: self.trail_line,
        }
    }

    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<KeyRecord> {
        self.keys.borrow().iter().cloned().collect()
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
        let line = describe(&record);
        {
            let mut ring = self.keys.borrow_mut();
            if ring.len() >= 64 {
                ring.pop_front();
            }
            ring.push_back(record);
        }
        self.trail_line.set(line);
    }
}

/// The custom seam the third beat consults: a menu bar's accelerator
/// table standing in for "the shortcuts come from somewhere else" — it
/// answers `fired_name` beside `dispatch`, the trait's provided
/// method, and the record carries its own answer.
struct MenuBarAccelerators;

impl KeymapSeam for MenuBarAccelerators {
    fn dispatch(&self, event: &KeyEvent) -> bool {
        event.presses(&Key::Escape)
    }

    fn fired_name(&self, event: &KeyEvent) -> Option<String> {
        event
            .presses(&Key::Escape)
            .then(|| "dismiss-menu".to_string())
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
        tracing::trace_span!(target: "alacer::shortcut_names", "beat", label = label).entered();
    let response = pipeline.dispatch_key(event);
    settle(pipeline);
    tracing::info!(
        target: "alacer::shortcut_names",
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
    let count_binding = RenderBinding::new("Named shortcuts fired: 0".to_string());
    let count_label = pipeline.add_child(root, Label::bound(count_binding.clone()).boxed());
    let trail_binding = RenderBinding::new("the trail: (nothing streamed yet)".to_string());
    let trail_label = pipeline.add_child(root, Label::bound(trail_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        count,
        count_label,
        &count_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Named shortcuts fired: {n}"),
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

/// The beats the demo walks before the window opens: the fired chord
/// naming itself, the miss that names nothing, the custom seam's own
/// name, the default's honest absence, and the rebind moving the name
/// with the chord — each a real key event through the real dispatch.
fn run_beats(pipeline: &mut PassPipeline, count: alacer_core::FluxRwSignal<i32>) {
    // Beat one: the provided table's binding names itself — the name
    // `bind` was given rides the record beside the chord.
    let mut keymap = Keymap::new();
    keymap.bind(
        Chord::new(Key::ArrowLeft).control(),
        "increment",
        move || {
            alacer_core::flux::Flux::batch_of(|| {
                let current = count.try_get_untracked().unwrap_or_default();
                count.set(current + 1);
            });
        },
    );
    alacer_core::keymap::install(Rc::new(keymap));
    beat(
        pipeline,
        "beat 1: Ctrl+Left bound as \"increment\" — the record names it",
        &pressed(Key::ArrowLeft, ctrl()),
    );

    // Beat two: the miss — the consult streams its fall-through and no
    // name, there is no binding to name.
    beat(
        pipeline,
        "beat 2: plain 'x' unbound — the FellThrough record names nothing",
        &pressed(Key::Char('x'), none()),
    );

    // Beat three: the custom seam — a menu bar's accelerators answer
    // `fired_name` beside `dispatch`, and the record carries their own
    // name (the trait grew beside `dispatch`, ADR-0043's rule).
    alacer_core::keymap::install(Rc::new(MenuBarAccelerators));
    beat(
        pipeline,
        "beat 3: Escape through a custom seam — \"dismiss-menu\" rides the record",
        &pressed(Key::Escape, none()),
    );

    // Beat four: the default's honest absence — a seam that overrides
    // nothing fires, and the record says so instead of guessing a name.
    alacer_core::keymap::install(Rc::new(Anonymous));
    beat(
        pipeline,
        "beat 4: Escape through a nameless seam — the record stays honest",
        &pressed(Key::Escape, none()),
    );

    // Beat five: the settings story — the installed table rebinds the
    // chord, and the name moves with it; the displaced name never fires
    // again.
    let mut keymap = Keymap::new();
    keymap.bind(
        Chord::new(Key::ArrowUp).control(),
        "increment (up)",
        move || {
            alacer_core::flux::Flux::batch_of(|| {
                let current = count.try_get_untracked().unwrap_or_default();
                count.set(current + 1);
            });
        },
    );
    keymap.bind(
        Chord::new(Key::ArrowDown).control(),
        "decrement (down)",
        move || {
            alacer_core::flux::Flux::batch_of(|| {
                let current = count.try_get_untracked().unwrap_or_default();
                count.set(current - 1);
            });
        },
    );
    alacer_core::keymap::install(Rc::new(keymap));
    beat(
        pipeline,
        "beat 5: Ctrl+Up bound as \"increment (up)\" — the name moved with the chord",
        &pressed(Key::ArrowUp, ctrl()),
    );
}

/// A seam that names nothing: `fired_name` keeps the trait's default
/// (`None`) and the record stays honest about the absence.
struct Anonymous;

impl KeymapSeam for Anonymous {
    fn dispatch(&self, event: &KeyEvent) -> bool {
        event.presses(&Key::Escape)
    }
}

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The shortcuts' names, streamed in the causal log (ADR-0043)").boxed(),
    );

    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let count = mount_labels(&mut pipeline, root, trail_signal);

    // The observer rides from here on: every consult's named record
    // streams into the trail label the frame after the funnel resolves
    // it.
    let trail = Trail {
        keys: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    run_beats(&mut pipeline, count);

    // The window: live chords play the same funnel — Ctrl+Up /
    // Ctrl+Down step the count, and every consult streams its
    // binding's name beside the focus trail while the labels mirror
    // the log.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: Ctrl+Up/Down step the count — the trail mirrors the log").boxed(),
    );

    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer shortcut names",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the shortcuts' names as data.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::shortcut_names",
        records = read_back.len(),
        named = read_back
            .iter()
            .filter(|record| record.name.is_some())
            .count(),
        "the shortcuts' facts streamed as they resolved — the causal log carries the named trail"
    );
}
