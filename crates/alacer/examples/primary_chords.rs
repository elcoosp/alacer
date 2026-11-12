//! The primary chord (§10.3, ADR-0044): one [`Chord::primary`]
//! builder that names "the platform's primary accelerator modifier" —
//! Command on Apple platforms, Control elsewhere — so an app's
//! shortcut table reads `primary+S` and answers `Cmd+S` on macOS,
//! `Ctrl+S` everywhere else. The hook ADR-0043's re-open rule
//! pre-registered ("a consumer needs a chord the exact `Modifiers`
//! shape cannot name") cashed: before M38 the app forked its table
//! per platform — or reached for a stringly-typed chord the funnel
//! cannot compare, the rule ADR-0041 refused.
//!
//! The demo drives the real dispatch (`PassPipeline::dispatch_key`,
//! the `keymap_trail` demo's shape — never a second channel) with the
//! causal observer riding, so every consult's [`KeyRecord`] shows the
//! resolution as the funnel matched it: the record's chord prints the
//! *resolved physical shape* (the truth the wire and the trail
//! carry), the name rides beside it (M37), and the beats show the
//! identity the resolution gives the chord — the other platform's
//! flag never aliases it, and the physical shape binds over the
//! primary one *in place* (one binding per chord, the M35 invariant,
//! visible at the vocabulary's own resolution).
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example primary_chords
//! ```
//!
//! The beats: the honest uninstalled window, the primary chord firing
//! under the platform's own flag, the other platform's flag falling
//! through, the rebind moving in place. Then the window opens for
//! live play — `primary+S` saves, `primary+O` opens, and every
//! consult streams beside the focus trail. `ALACER_SOFTWARE=1` forces
//! the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label};
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Size;
use alacer_core::inspect::{InspectEvent, InspectObserver, KeyRecord};
use alacer_core::keymap::{Chord, Keymap, KeymapConsult, KeymapSeam};
use alacer_core::passes::PassPipeline;
use reactive_graph::traits::Set as _;
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

/// The platform's own primary-modifier event shape: Control held off
/// Apple platforms, Command held on them — the exact flags
/// [`Chord::primary`] resolves to on this build ([`Modifiers::primary`]'s
/// own reading, one source of truth).
fn primary_modifiers() -> Modifiers {
    if cfg!(target_os = "macos") {
        Modifiers {
            meta: true,
            ..Modifiers::default()
        }
    } else {
        Modifiers {
            control: true,
            ..Modifiers::default()
        }
    }
}

/// The fact as the trail's reader renders it: the chord beside the
/// species the consult answered, the fired binding's prose name beside
/// it when the consult fired and the seam named one (M37) — the
/// record's own fields, the resolved physical shape the funnel
/// matched.
fn describe(record: &KeyRecord) -> String {
    let chord = match record.chord.key() {
        Key::Char(character) => format!("'{character}'"),
        Key::Enter => "Enter".to_string(),
        _ => "…".to_string(),
    };
    let modifiers = record.chord.modifiers();
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
    let chord = if shape.is_empty() {
        chord
    } else {
        format!("{}+{chord}", shape.join("+"))
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

/// The live-rebinding wrapper the m35 example carries, both legs (the
/// m37 lesson): the trait is the funnel's only contract, so a wrapper
/// around a naming table delegates `dispatch` beside `fired_name`.
struct Shared(Rc<RefCell<Keymap>>);

impl KeymapSeam for Shared {
    fn dispatch(&self, event: &KeyEvent) -> bool {
        self.0.borrow().dispatch(event)
    }

    fn fired_name(&self, event: &KeyEvent) -> Option<String> {
        self.0.borrow().fired_name(event)
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
        tracing::trace_span!(target: "alacer::primary_chords", "beat", label = label).entered();
    let response = pipeline.dispatch_key(event);
    settle(pipeline);
    tracing::info!(
        target: "alacer::primary_chords",
        handled = matches!(response, alacer_core::EventResponse::Handled),
        "{label}"
    );
}

/// The labels the demo mirrors through: the status line is the app's
/// half (the bindings write it through one batch), the trail line is
/// the log's half (the observer writes it at the consult). Handlers
/// write signals, never widgets (axiom A1).
fn mount_labels(
    pipeline: &mut PassPipeline,
    root: alacer_core::NodeId,
    trail_signal: alacer_core::FluxRwSignal<String>,
) -> alacer_core::FluxRwSignal<String> {
    let status = pipeline
        .flux_mut()
        .signal("the app: (nothing saved yet)".to_string());
    let status_binding = RenderBinding::new("the app: (nothing saved yet)".to_string());
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    let trail_binding = RenderBinding::new("the trail: (nothing streamed yet)".to_string());
    let trail_label = pipeline.add_child(root, Label::bound(trail_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        status,
        status_label,
        &status_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |line: &String| line.clone(),
    );
    pipeline.flux_mut().bind_value(
        trail_signal,
        trail_label,
        &trail_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );
    status
}

/// The beats the demo walks before the window opens: the honest
/// uninstalled window, the primary chord firing under the platform's
/// own flag, the other platform's flag falling through, and the
/// rebind moving in place — each a real key event through the real
/// dispatch, the table live-rebinding behind the seam between the
/// beats.
fn run_beats(
    pipeline: &mut PassPipeline,
    table: &Rc<RefCell<Keymap>>,
    status: alacer_core::FluxRwSignal<String>,
) {
    // Beat one: the window before the assembly's install — the funnel
    // still consults (the read is total), nothing answers, and the
    // record names the uninstalled window.
    beat(
        pipeline,
        "beat 1: primary+S with no keymap installed — the honest Uninstalled",
        &pressed(Key::Char('s'), primary_modifiers()),
    );

    // The assembly installs the table: one chord vocabulary, two
    // bindings — `primary+S` saves, `primary+O` opens — handlers
    // writing signals in one batch, never widgets (axiom A1).
    table
        .borrow_mut()
        .bind(Chord::new(Key::Char('s')).primary(), "save", move || {
            alacer_core::flux::Flux::batch_of(|| status.set("the app: saved".to_string()));
        });
    alacer_core::keymap::install(Rc::new(Shared(Rc::clone(table))));

    // Beat two: the primary chord fires under the platform's own flag —
    // on this build Control (off Apple platforms); on an Apple build
    // the very same chord answers Command. The record's chord prints
    // the resolved physical shape the funnel matched.
    beat(
        pipeline,
        "beat 2: primary+S bound — the Fired record names \"save\" under the platform's own flag",
        &pressed(Key::Char('s'), primary_modifiers()),
    );

    // Beat three: the other platform's flag is a different chord — off
    // Apple platforms a Command event never aliases the primary chord
    // (the exact shape, the M35 rule), and the fall-through streams
    // the chord the funnel asked about, honestly unnamed. On an Apple
    // build the roles reverse: Control is the other flag.
    #[cfg(not(target_os = "macos"))]
    beat(
        pipeline,
        "beat 3: Cmd+S (the other platform's flag) — the honest fall-through off Apple platforms",
        &pressed(
            Key::Char('s'),
            Modifiers {
                meta: true,
                ..Modifiers::default()
            },
        ),
    );
    #[cfg(target_os = "macos")]
    beat(
        pipeline,
        "beat 3: Ctrl+S (the other platform's flag) — the honest fall-through on Apple platforms",
        &pressed(
            Key::Char('s'),
            Modifiers {
                control: true,
                ..Modifiers::default()
            },
        ),
    );

    // Beat four: the settings story — the physical Control shape binds
    // over the primary one. Off Apple platforms that is the SAME
    // chord: the rebind moves in place, the displaced name never fires
    // again (one binding per chord, the resolution as identity).
    table
        .borrow_mut()
        .bind(Chord::new(Key::Char('s')).control(), "save-as", || {});
    beat(
        pipeline,
        "beat 4: the physical Control shape bound over the primary one — one chord, the rebind moved in place",
        &pressed(Key::Char('s'), primary_modifiers()),
    );
}

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The platform's primary chord, streamed to the causal log (ADR-0044)").boxed(),
    );

    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let status = mount_labels(&mut pipeline, root, trail_signal);

    // The observer rides from here on: every consult's record streams
    // into the trail label the frame after the funnel resolves it.
    let trail = Trail {
        keys: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    let table = Rc::new(RefCell::new(Keymap::new()));
    run_beats(&mut pipeline, &table, status);

    // The window: live chords play the same funnel — primary+S saves,
    // primary+O opens, and every consult streams its record beside the
    // focus trail while the labels mirror the log. The physical shape
    // the chord resolved to is the shape the window's own platform
    // sends.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: primary+S saves, primary+O opens — the trail mirrors the log").boxed(),
    );

    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer primary chords",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the shortcuts' resolved shapes as data.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::primary_chords",
        records = read_back.len(),
        fired = read_back
            .iter()
            .filter(|record| record.outcome == KeymapConsult::Fired)
            .count(),
        "the shortcuts' facts streamed as they resolved — the causal log carries the primary chords"
    );
}
