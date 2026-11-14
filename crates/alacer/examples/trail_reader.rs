//! The trail's provided reader as a consumer's vocabulary (§13.3,
//! ADR-0069): the app reads its own causal log through the
//! [`alacer_core::TrailLog`] the seam provides — no hand-rolled
//! observer, no per-file `Rc<RefCell<VecDeque>>` collector. One
//! installation (`pipeline.set_observer`) and the app holds a reading
//! handle: the speech acts it asks for stream into the ring one frame
//! after the doors answer them, and each *Read the log* press drains
//! the window — the facts handed back oldest first, the summary priced
//! by the app's own renderer.
//!
//! The demo walks the beats live: *Speak* queues the ambient
//! announcement route, and the facts land on the reader the frame the
//! pipeline answers them — a *Read* prices the frames *before it*
//! (the drain's currency, never the handler's own batch). The window's
//! bound label mirrors the log; the tip is named by its species, the
//! payload never on the line (ADR-0011's rule at the reader's tier).
//! After the window closes the session's facts are read back once more
//! through the same handle:
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example trail_reader
//! ```
//!
//! The workspace's idiom: `tracing` spans at the `alacer::trail_reader`
//! target, one per route. `ALACER_SOFTWARE=1` forces the softbuffer path.
//!
//! Run with `cargo run -p alacer --example trail_reader`.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, Row};
use alacer_core::announce::{Politeness, announce};
use alacer_core::flux::{Flux, RenderBinding};
use alacer_core::inspect::{AnnounceOutcome, InspectEvent};
use alacer_core::trail::TrailLog;
use reactive_graph::traits::Set as _;

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// The fact as the reader renders it: the species named, the payload
/// never on the line (the lengths, never the text).
fn describe(event: &InspectEvent) -> String {
    match event {
        InspectEvent::Edge(_) => "an edge activated".to_string(),
        InspectEvent::Write(_) => "a signal write".to_string(),
        InspectEvent::Input(_) => "an input routed".to_string(),
        InspectEvent::Focus(_) => "a focus door answered".to_string(),
        InspectEvent::Edit(_) => "a journal action answered".to_string(),
        InspectEvent::Announce(record) => match record.outcome {
            AnnounceOutcome::Delivered => "an announcement delivered".to_string(),
            AnnounceOutcome::Superseded => "an announcement superseded".to_string(),
            AnnounceOutcome::Refused(_) => "an announcement refused".to_string(),
        },
        InspectEvent::Key(_) => "a key routed".to_string(),
        InspectEvent::Clipboard(_) => "a clipboard round trip".to_string(),
        InspectEvent::Value(_) => "a value ask answered".to_string(),
        InspectEvent::Gesture(_) => "a gesture answered".to_string(),
        InspectEvent::Scroll(_) => "a scroll ask answered".to_string(),
        InspectEvent::TextWrite(_) => "an AT text write answered".to_string(),
        InspectEvent::RouteDrop(_) => "an ask dropped at the boundary".to_string(),
        InspectEvent::KeyEdit(_) => "a typing-door edit answered".to_string(),
        InspectEvent::Preedit(_) => "a composing state answered".to_string(),
        InspectEvent::Frame(_) => "a frame completed".to_string(),
    }
}

/// The drained window as the label carries it: the count beside the
/// tip's species, the drain's own window discipline named when the
/// read emptied it.
fn summarize(events: &[InspectEvent]) -> String {
    match events.last() {
        None => "the trail: (drained — the log holds nothing new)".to_string(),
        Some(tip) => format!(
            "the trail: {} facts since the last read — tip: {}",
            events.len(),
            describe(tip)
        ),
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    // The provided reader: one installation, the app holds the reading
    // handle — the seam's own implementation beside the trait, never a
    // per-app collector (ADR-0069's demonstration).
    let trail = TrailLog::new();
    pipeline.set_observer(Box::new(trail.clone()));

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The causal log, read through the provided reader (ADR-0069)").boxed(),
    );

    // The summary the label mirrors: the app's read of its own log,
    // written where the reader hands the window over.
    let summary = pipeline
        .flux_mut()
        .signal("the trail: (nothing read yet)".to_string());
    let summary_binding = RenderBinding::new("the trail: (nothing read yet)".to_string());
    let summary_label = pipeline.add_child(root, Label::bound(summary_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        summary,
        summary_label,
        &summary_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );

    // The routes: two speech acts (the polite save, the assertive
    // alert) and the read — the drain that prices the frames before
    // it, the summary the only bridge back to the window.
    let actions = pipeline.add_child(root, Row::new().gap(8.0).boxed());
    let save_click = move || {
        let _route =
            tracing::trace_span!(target: "alacer::trail_reader", "announce", route = "save")
                .entered();
        Flux::batch_of(|| {
            let _ = announce("Saved", Politeness::Polite);
        });
    };
    let alert_click = move || {
        let _route =
            tracing::trace_span!(target: "alacer::trail_reader", "announce", route = "alert")
                .entered();
        Flux::batch_of(|| {
            let _ = announce("Connection lost", Politeness::Assertive);
        });
    };
    let read_click = {
        let trail = trail.clone();
        move || {
            let events = trail.drain();
            let _route = tracing::trace_span!(
                target: "alacer::trail_reader",
                "read",
                facts = events.len()
            )
            .entered();
            let line = summarize(&events);
            Flux::batch_of(|| summary.set(line));
        }
    };
    for (name, handler) in [
        ("Speak (polite)", Box::new(save_click) as Box<dyn FnMut()>),
        (
            "Speak (assertive)",
            Box::new(alert_click) as Box<dyn FnMut()>,
        ),
        ("Read the log", Box::new(read_click) as Box<dyn FnMut()>),
    ] {
        let _ = pipeline.add_child(actions, Button::new(name).on_click(handler).boxed());
    }

    let _ = pipeline.add_child(
        root,
        Label::new("Each Read prices the frames before it — the facts stream one frame after the doors answer")
            .boxed(),
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer trail reader",
        (520.0, 240.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed since the last read, the handle still holding the ring.
    let read_back = trail.drain();
    tracing::info!(
        target: "alacer::trail_reader",
        facts = read_back.len(),
        "the session's facts streamed as the routes answered them — the provided reader carries the trail"
    );
}
