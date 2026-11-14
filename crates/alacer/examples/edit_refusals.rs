//! The typing door's refusals join the causal log (§13.3, ADR-0068):
//! ADR-0064's second kill rule cashes — a consumer prices the *closed
//! field's wall* on the log. The M58 record names what the door
//! answered now ([`alacer_core::KeyEditOutcome`], the value trail's own
//! refusal vocabulary, ADR-0046): every delivered edit rides `Landed`
//! (the M58 shape, additive at version 1), and the routed edit key a
//! disabled field ignored rides `Refused(unavailable)` — the ask named
//! by its action and driver, the shape priced at nothing, never a
//! munged delivered edit. The chords' refusals ride behind their own
//! actions (`paste`/`cut` under the `clipboard` driver — ADR-0066's
//! third kill rule riding beside), while the engine never ran: one
//! fact per trail.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example edit_refusals
//! ```
//!
//! The beats: the open door's run, the wall naming the ask it turned
//! away, the chords refusing behind their own actions (the clipboard
//! engine never reached), the navigation keys riding no wall fact, the
//! IME's commit refusing at the wall with its own driver, and the
//! reopened door landing again. The wall's window is the availability
//! tier's own (M22, ADR-0028): focus is evicted the frame the control
//! becomes unavailable — the beats hold the field down *without*
//! settling, so the routed ask reaches the held door, then settle once
//! to drain. Then the window opens for live play; `ALACER_SOFTWARE=1`
//! forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, TextInput};
use alacer_core::events::{ImeEvent, Key, KeyEvent, KeyState, Modifiers};
use alacer_core::flux::RenderBinding;
use alacer_core::inspect::{
    InspectEvent, InspectObserver, KeyEditDriver, KeyEditOutcome, KeyEditRefusal,
};
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

/// The fact as the trail's reader renders it: the door's answer named
/// beside the ask — a landing's shape by lengths, a refusal's species
/// beside the value it left untouched — "what happened to my edit"
/// answered from one record.
fn describe(record: alacer_core::KeyEditRecord) -> String {
    let who = match record.driver {
        KeyEditDriver::Keyboard => "the keyboard",
        KeyEditDriver::Ime => "the IME's commit",
        KeyEditDriver::Clipboard => "the clipboard's chord",
    };
    let control = record
        .target
        .map(|target| format!(" (at {target})"))
        .unwrap_or_default();
    match record.outcome {
        KeyEditOutcome::Landed => {
            let fold = if record.coalesced {
                " — the journal folded the run"
            } else {
                ""
            };
            format!(
                "{who} landed ({} replaced by {}) — the selection at {}..{}{fold}{control}",
                record.replaced, record.inserted, record.selection.anchor, record.selection.focus,
            )
        }
        KeyEditOutcome::Refused(refusal) => {
            let species = match refusal {
                KeyEditRefusal::Unavailable => "unavailable",
            };
            format!("{who} refused — the wall held ({species}), the value untouched{control}")
        }
    }
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

/// Re-arms the wall's window: the door open just long enough to take
/// the focus back, then held down with no settle in between — the frame
/// the control becomes unavailable, the route still holding the field
/// (the M22 eviction is the frame's pass, not the door).
fn arm_wall(pipeline: &mut PassPipeline, field_node: alacer_core::NodeId) {
    let mut guard = pipeline.arena_mut().mutate(field_node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_enabled(true);
    }
    drop(guard);
    pipeline.request_focus(field_node);
    settle(pipeline);
    let mut guard = pipeline.arena_mut().mutate(field_node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_enabled(false);
    }
    drop(guard);
}

/// One beat: the label logged, the ask routed, the frame settled — the
/// fact streaming to the observer the frame after the door answered.
fn beat(pipeline: &mut PassPipeline, label: &str, route: impl FnOnce(&mut PassPipeline)) {
    let _span =
        tracing::trace_span!(target: "alacer::edit_refusals", "beat", label = label).entered();
    route(pipeline);
    settle(pipeline);
    tracing::info!(target: "alacer::edit_refusals", "{label}");
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

/// The beats the demo walks before the window opens: the open door's
/// run, the wall naming its ask, the chords refusing behind their own
/// actions, the navigation keys riding none, the IME's commit refused
/// with its own driver, and the reopened door landing again.
fn run_beats(
    pipeline: &mut PassPipeline,
    status_signal: alacer_core::FluxRwSignal<String>,
    field_node: alacer_core::NodeId,
) {
    // Beat one: the open door's run — the M58 shape, named additively
    // (`Landed`, the default the wall refuses).
    beat(pipeline, "beat 1: the open door — the run lands", |p| {
        for ch in "hi".chars() {
            let _ = p.dispatch_key(&KeyEvent::char(ch));
        }
    });

    // Beat two: the wall names the ask it turned away — the routed edit
    // key the held field ignored, the species beside the driver, the
    // shape priced at nothing (the value untouched).
    arm_wall(pipeline, field_node);
    beat(
        pipeline,
        "beat 2: the wall — the ask it turned away",
        |p| {
            let _ = p.dispatch_key(&KeyEvent::char('x'));
        },
    );

    // Beat three: the chords refuse behind their own actions — the
    // paste and the cut name themselves (the clipboard's driver, the
    // M60 rule), and the engine never ran: one fact per trail.
    arm_wall(pipeline, field_node);
    beat(
        pipeline,
        "beat 3: the chords' wall — paste and cut name themselves",
        |p| {
            let _ = p.dispatch_key(&ctrl(Key::Char('v')));
            let _ = p.dispatch_key(&ctrl(Key::Char('x')));
        },
    );

    // Beat four: the keys an open door answers without editing — the
    // caret's walk — ride no wall fact: the wall prices edit asks,
    // never every routed keystroke.
    arm_wall(pipeline, field_node);
    beat(
        pipeline,
        "beat 4: navigation at the wall — no facts ride",
        |p| {
            let _ = p.dispatch_key(&KeyEvent::pressed(Key::ArrowLeft));
            let _ = p.dispatch_key(&KeyEvent::pressed(Key::ArrowRight));
        },
    );

    // Beat five: the IME's commit into the closed field — the typing
    // door's own ask, refused with the wall's species, the IME driver
    // named (the attribution rule at the typing tier).
    arm_wall(pipeline, field_node);
    beat(
        pipeline,
        "beat 5: the IME's commit refuses at the wall",
        |p| {
            let _ = p.dispatch_ime(&ImeEvent::Commit {
                text: "composed".to_string(),
            });
        },
    );

    // Beat six: the reopened door lands again — the wall is the tier's,
    // not the field's identity.
    let mut guard = pipeline.arena_mut().mutate(field_node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_enabled(true);
    }
    drop(guard);
    pipeline.request_focus(field_node);
    settle(pipeline);
    beat(
        pipeline,
        "beat 6: the reopened door — the edit lands",
        |p| {
            let _ = p.dispatch_key(&KeyEvent::char('!'));
        },
    );

    status_signal.set("the wall: hold the field down and type".to_string());
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The typing door's refusals join the causal log (ADR-0068)").boxed(),
    );
    let field = TextInput::new("field");
    let field_node = pipeline.add_child(root, field.boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("Live: typing lands; hold the field down and it names the wall").boxed(),
    );

    let status_signal = pipeline
        .flux_mut()
        .signal("the wall: (the door answers what it hears)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let status_binding =
        RenderBinding::new("the wall: (the door answers what it hears)".to_string());
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

    // The observer rides from here on: every fact the door answers —
    // landed or refused — streams into the trail label the frame after
    // it resolves.
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
        "Alacer edit refusals",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the door's answer beside every ask.
    let read_back = trail.all();
    let (landed, refused) = read_back.iter().fold(
        (0_usize, 0_usize),
        |(landed, refused), record| match record.outcome {
            KeyEditOutcome::Landed => (landed + 1, refused),
            KeyEditOutcome::Refused(_) => (landed, refused + 1),
        },
    );
    tracing::info!(
        target: "alacer::edit_refusals",
        records = read_back.len(),
        landed,
        refused,
        "the door's answers streamed as they resolved — the wall priced beside the landings"
    );
}
