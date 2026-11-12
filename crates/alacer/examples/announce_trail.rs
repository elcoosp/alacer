//! The announcement rides the causal log (§13.3, ADR-0039): every speech
//! act the pipeline answers — a delivered announcement, a refused one,
//! an act the last-write-wins rule replaced — streams one
//! `InspectEvent::Announce` the moment the answer resolves, carrying the
//! politeness it asked for and the outcome it got. The hook the M18
//! contract's own refusals pre-registered ("the refusal is the fact a
//! trail reader wants") cashed: a status line, an inspector trail, or a
//! test routes on "why did the live region speak — or refuse" without
//! re-implementing the observation at the call site.
//!
//! The demo drives the M18 announcement seam from the handler side (the
//! ambient `announce` route — handlers cannot reach the pipeline, axiom
//! A1) with the causal observer riding: the app mirrors its own
//! introspection (the spoken line), and the *trail* label mirrors what
//! the log streamed — the observer writes the signal the label binds,
//! the same one-batch currency every handler rides. The refusals stream
//! too: the empty speech act's fact is exactly the one a trail reader
//! wants.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example announce_trail
//! ```
//!
//! Run with `cargo run -p alacer --example announce_trail`. Click *Save*
//! and watch the trail log one delivered record; *Connection lost*
//! speaks assertively; *Speak nothing* streams the empty refusal; *Storm
//! three acts* streams the batch's shape — two superseded facts beside
//! the survivor (last write wins, visible as data). `ALACER_SOFTWARE=1`
//! forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, Row};
use alacer_core::announce::{Politeness, announce};
use alacer_core::flux::RenderBinding;
use alacer_core::inspect::{AnnounceOutcome, AnnounceRecord, InspectEvent, InspectObserver};
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

/// The fact as the trail's reader renders it: the politeness it asked
/// for, the outcome it got, the refusal it named.
fn describe(record: AnnounceRecord) -> String {
    let politeness = record.politeness.name();
    match record.outcome {
        AnnounceOutcome::Delivered => format!("{politeness} — the region carries it now"),
        AnnounceOutcome::Superseded => format!("{politeness} — replaced: last write wins"),
        AnnounceOutcome::Refused(refusal) => {
            format!("{politeness} — refused: {}", refusal.name())
        }
    }
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the answer rode (axiom A1: signals in, never widgets).
struct Trail {
    records: Rc<RefCell<std::collections::VecDeque<AnnounceRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The observer half: boxed into [`alacer::core::passes::PassPipeline::
    /// set_observer`], it receives every `InspectEvent`, keeps the speech
    /// acts' facts, and mirrors the newest into the trail label's signal
    /// — the write queues masks, the frame renders them.
    fn writer(&self) -> TrailWriter {
        TrailWriter {
            records: Rc::clone(&self.records),
            trail_line: self.trail_line,
        }
    }

    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<AnnounceRecord> {
        self.records.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    records: Rc<RefCell<std::collections::VecDeque<AnnounceRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::Announce(record) = event else {
            return;
        };
        {
            let mut ring = self.records.borrow_mut();
            if ring.len() >= 64 {
                ring.pop_front();
            }
            ring.push_back(record);
        }
        self.trail_line.set(describe(record));
    }
}

/// The spoken line the status label mirrors: the app's half of the
/// story — what the handler asked the region to say (the trail label
/// carries the log's half).
fn spoken(text: &str, politeness: Politeness) -> String {
    format!("spoken ({}) : {:?}", politeness.name(), text)
}

/// One speech route: the span names it, the batch is the one write
/// batch, and the mirror is the only bridge back to the window. The
/// act's own fact streams where the pipeline answers it — the drain the
/// frame after the handler ran (the announcement drain's currency).
fn route(
    spoken_line: &alacer_core::FluxRwSignal<String>,
    route_name: &'static str,
    text: &str,
    politeness: Politeness,
) {
    let _route =
        tracing::trace_span!(target: "alacer::announce_trail", "announce", route = route_name)
            .entered();
    alacer_core::flux::Flux::batch_of(|| {
        spoken_line.set(spoken(text, politeness));
        announce(text, politeness);
    });
}

/// The demo's controls: one row, four routes — the same announcement
/// routes the trail streams for, driven live in the window (handlers
/// never touch the pipeline — axiom A1 — and the facts follow where the
/// pipeline answers them).
fn push_buttons(
    spoken_line: &alacer_core::FluxRwSignal<String>,
    actions: alacer_core::NodeId,
    pipeline: &mut alacer_core::passes::PassPipeline,
) {
    let save_click = {
        let spoken_line = *spoken_line;
        move || route(&spoken_line, "save", "Saved", Politeness::Polite)
    };
    let alert_click = {
        let spoken_line = *spoken_line;
        move || {
            route(
                &spoken_line,
                "alert",
                "Connection lost",
                Politeness::Assertive,
            );
        }
    };
    let empty_click = {
        let spoken_line = *spoken_line;
        move || route(&spoken_line, "empty", "   ", Politeness::Polite)
    };
    let storm_click = {
        let spoken_line = *spoken_line;
        move || {
            // Three speech acts in one write batch: the drain answers
            // the whole shape — two superseded facts beside the
            // survivor (last write wins, ADR-0024, visible as data).
            alacer_core::flux::Flux::batch_of(|| {
                spoken_line.set(spoken("3 items left", Politeness::Assertive));
                let _ = announce("Saved", Politeness::Polite);
                let _ = announce("2 items left", Politeness::Polite);
                let _ = announce("3 items left", Politeness::Assertive);
            });
        }
    };
    for (name, handler) in [
        ("Save", Box::new(save_click) as RouteHandler),
        ("Connection lost", Box::new(alert_click) as RouteHandler),
        ("Speak nothing", Box::new(empty_click) as RouteHandler),
        ("Storm three acts", Box::new(storm_click) as RouteHandler),
    ] {
        let _ = pipeline.add_child(actions, Button::new(name).on_click(handler).boxed());
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The speech acts' facts, streamed to the causal log (ADR-0039)").boxed(),
    );

    // The reads the labels mirror: the spoken line is the app's half
    // (handlers write it), the trail line is the log's half (the
    // observer writes it where the pipeline answers the act).
    let spoken_signal = pipeline
        .flux_mut()
        .signal("spoken: (nothing yet)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());

    let spoken_binding = RenderBinding::new("spoken: (nothing yet)".to_string());
    let spoken_label = pipeline.add_child(root, Label::bound(spoken_binding.clone()).boxed());
    let trail_binding = RenderBinding::new("the trail: (nothing streamed yet)".to_string());
    let trail_label = pipeline.add_child(root, Label::bound(trail_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        spoken_signal,
        spoken_label,
        &spoken_binding,
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

    // The observer rides from here on: the speech acts' facts stream
    // into the trail label the frame after each answer resolves.
    let trail = Trail {
        records: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    let actions = pipeline.add_child(root, Row::new().gap(8.0).boxed());
    push_buttons(&spoken_signal, actions, &mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer announce trail",
        (520.0, 240.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the announcement seam's history as data.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::announce_trail",
        records = read_back.len(),
        refusals = read_back
            .iter()
            .filter(|record| matches!(record.outcome, AnnounceOutcome::Refused(_)))
            .count(),
        superseded = read_back
            .iter()
            .filter(|record| record.outcome == AnnounceOutcome::Superseded)
            .count(),
        "the speech acts' facts streamed as they resolved — the causal log carries the announcement trail"
    );
}
