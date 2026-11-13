//! The content clamp's story joins the log (§13.3, ADR-0057): the
//! content's own door — `ListHandle::set_content_height`, the resize
//! every app rides after a structural change — answers on the causal
//! log now, ADR-0056's third kill rule cashed ("the clamp is the
//! content's story, not the ask's; today it rides outside the log").
//! A resize the clamp answers — the content shrank under the offset —
//! streams one [`alacer_core::ScrollRecord`] with the
//! `ContentClamp` action: the landing beside the wall it pinned (the
//! same `RangeBound` vocabulary the ask doors speak, never a second
//! species), no ask beside it (a resize is not an ask), no route
//! behind it (the stamp's honest absence). A resize the offset never
//! felt queues nothing, and a non-finite height refuses before the
//! arithmetic (`NonFiniteContent`) — the corruption class the M50
//! trail exposed at the wheel door, closed at the content's own door.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example content_clamp
//! ```
//!
//! The beats: the clamp's stage set by the app's own scroll, the
//! shrink that pins the landing at the new max, the resize the offset
//! never felt (nothing queues), the growth (nothing queues), the
//! non-finite height refused with the state intact, and the wall
//! spoken identically on both stories. Then the window opens for live
//! play: *Shrink the content* and *Grow the content* ride the same
//! door, the trail mirroring the log. `ALACER_SOFTWARE=1` forces the
//! softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, List};
use alacer_core::flux::RenderBinding;
use alacer_core::inspect::{InspectEvent, InspectObserver, ScrollAction, ScrollOutcome};
use alacer_core::passes::PassPipeline;
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the scroll trail's
/// shape).
const VIEWPORT_WIDTH: f32 = 480.0;
const VIEWPORT_HEIGHT: f32 = 380.0;

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// The fact as the trail's reader renders it: the content clamp's
/// landing beside the wall it pinned, the refusals named, the honest
/// absences visible — the log answering "why did the offset move
/// without any ask" from one line. The record is `Copy`, so it passes
/// by value (the pedantic group's rule).
fn describe(record: alacer_core::ScrollRecord) -> String {
    match record.action {
        ScrollAction::ContentClamp => match record.outcome {
            ScrollOutcome::Delivered => {
                let wall = record.pinned.map_or_else(String::new, |bound| {
                    format!(", pinned at the {}", bound.name())
                });
                format!(
                    "the content clamp landed the offset {}{wall}",
                    record.offset
                )
            }
            ScrollOutcome::Refused(refusal) => format!("refused: {}", refusal.name()),
        },
        _ => match record.outcome {
            ScrollOutcome::Delivered => format!("the scroll landed {}", record.offset),
            ScrollOutcome::Refused(refusal) => format!("refused: {}", refusal.name()),
        },
    }
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the drain rode (axiom A1: signals in, never widgets).
struct Trail {
    scrolls: Rc<RefCell<std::collections::VecDeque<alacer_core::ScrollRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The observer half: boxed into
    /// [`alacer_core::passes::PassPipeline::set_observer`], it receives
    /// every `InspectEvent`, keeps the scroll facts, and mirrors the
    /// newest into the trail label's signal.
    fn writer(&self) -> TrailWriter {
        TrailWriter {
            scrolls: Rc::clone(&self.scrolls),
            trail_line: self.trail_line,
        }
    }

    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<alacer_core::ScrollRecord> {
        self.scrolls.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    scrolls: Rc<RefCell<std::collections::VecDeque<alacer_core::ScrollRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::Scroll(record) = event else {
            return;
        };
        let line = describe(record);
        {
            let mut ring = self.scrolls.borrow_mut();
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
        let Ok(report) = pipeline.tick(alacer_core::geom::Size::new(
            VIEWPORT_WIDTH,
            VIEWPORT_HEIGHT,
        )) else {
            return;
        };
        if report.is_idle() {
            return;
        }
    }
}

/// One beat on the content's own door: the span names the resize, the
/// handle answers, the fact streams to the observer — the app's own
/// route, no second channel.
fn clamp_beat(handle: &alacer::widgets::ListHandle, label: &'static str, height: f32) {
    let _route =
        tracing::trace_span!(target: "alacer::content_clamp", "beat", label = label).entered();
    handle.set_content_height(height);
    tracing::info!(target: "alacer::content_clamp", offset = handle.offset(), "{label}");
}

/// The beats the demo walks before the window opens: the clamp's
/// stage, the pinned landing, the resizes that queue nothing, the
/// non-finite refusal, and the wall spoken identically on both
/// stories. Each a real resize through the app's real door.
fn run_beats(
    handle: &alacer::widgets::ListHandle,
    status_signal: alacer_core::FluxRwSignal<String>,
) {
    // Beat one: the clamp's stage — the app scrolls its own list to
    // the mid-range (the ask-side fact the M50 trail prices; the
    // offset rides to 300).
    let _route =
        tracing::trace_span!(target: "alacer::content_clamp", "beat", label = "beat 1").entered();
    handle.scroll_to(300.0);
    tracing::info!(target: "alacer::content_clamp", offset = handle.offset(), "beat 1: scroll_to(300) — the clamp's stage set");
    status_signal.set("the content clamp: the stage set at 300".to_string());

    // Beat two: the content shrinks under the offset — 300 px of
    // content in a 200 px viewport, the span falls to 100, the clamp
    // lands the offset at the new max. The fact the kill rule priced:
    // the content's story on the log, the wall named, no ask beside
    // it, no route behind it.
    clamp_beat(
        handle,
        "beat 2: content 500 → 300 — the clamp lands the offset at the new max",
        300.0,
    );

    // Beat three: the resize the offset never felt — the span still
    // covers the offset, the clamp moves nothing, nothing queues (the
    // M20 idle-frame property at the content's door).
    clamp_beat(
        handle,
        "beat 3: content 300 → 350 — the span still holds the offset, nothing queues",
        350.0,
    );

    // Beat four: the growth — the span widens, the offset stays,
    // nothing queues (growth never moves an offset).
    clamp_beat(
        handle,
        "beat 4: content 350 → 600 — growth never moves the offset, nothing queues",
        600.0,
    );

    // Beat five: the non-finite height refused before the arithmetic —
    // the corruption class the M50 trail exposed at the wheel door,
    // closed here: the state keeps the height and offset it held, the
    // refusal a named fact on the log.
    clamp_beat(
        handle,
        "beat 5: a NaN height — refused before it corrupts, the state keeps",
        f32::NAN,
    );

    // Beat six: the wall spoken identically on both stories — the next
    // good ask still answers by the old arithmetic (the 600 px
    // content's 400 px wall), then the content shrinks to it and the
    // content clamp pins the same `RangeBound::Max` the ask tier
    // named.
    clamp_beat(
        handle,
        "beat 6: content 600 → 420 — the clamp pins the same wall the asks name",
        420.0,
    );
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The content clamp's story, streamed to the causal log (ADR-0057)").boxed(),
    );
    let list = List::new(200.0);
    let handle = list.handle();
    let _ = pipeline.add_child(root, list.boxed());
    handle.set_content_height(500.0);

    let status_signal = pipeline
        .flux_mut()
        .signal("the content clamp: (the tree carries what the app wrote)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let status_binding =
        RenderBinding::new("the content clamp: (the tree carries what the app wrote)".to_string());
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

    // The observer rides from here on: every answered resize's record
    // streams into the trail label the frame after the door resolves.
    let trail = Trail {
        scrolls: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    settle(&mut pipeline);
    run_beats(&handle, status_signal);

    // The window: live play — the same door the beats rode, driven by
    // buttons: *Shrink the content* falls under the offset (the clamp
    // answers, the wall named), *Grow the content* widens the span
    // (nothing queues). The trail mirrors the log.
    let _ = pipeline.add_child(
        root,
        Button::new("Shrink the content")
            .on_click({
                let handle = handle.clone();
                move || handle.set_content_height(150.0)
            })
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Button::new("Grow the content")
            .on_click({
                let handle = handle.clone();
                move || handle.set_content_height(500.0)
            })
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Label::new(
            "Live: shrink under the offset — the clamp's wall lands on the log; growth queues nothing",
        )
        .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer content clamp",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the content clamp's history as data. The
    // wall named itself; the refusal refused honestly.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::content_clamp",
        records = read_back.len(),
        clamps = read_back
            .iter()
            .filter(|record| record.action == ScrollAction::ContentClamp)
            .count(),
        refused = read_back
            .iter()
            .filter(|record| matches!(record.outcome, ScrollOutcome::Refused(_)))
            .count(),
        pinned = read_back.iter().filter(|record| record.pinned.is_some()).count(),
        "the resizes streamed as they resolved — the causal log carries the content's story"
    );
}
