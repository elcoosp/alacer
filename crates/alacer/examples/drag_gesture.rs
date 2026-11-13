//! The drag's story joins the log (§13.3, ADR-0052): the pointer's
//! write — the value tier's drag — reaches the causal log as one
//! per-gesture [`alacer_core::GestureRecord`] per phase, never per move
//! (the flood rule the kill criteria pre-registered): a `Started` the
//! moment the press arms the drag (the value it found), an `Ended` the
//! moment the release or the leave resolves it (the value it landed),
//! the control stamped by the dispatch funnel's pointer route (the M19
//! capture answers the *who* exactly the M43 action route does). The
//! demo drives the real pointer route (`PassPipeline::dispatch_pointer`,
//! the exact path the platform loop rides — never a second channel)
//! against one `Slider` with the causal observer riding: the beats
//! press, move, and release the thumb, and the *trail* label mirrors
//! what the log streamed — two records for a hundred moves' worth of
//! gesture, the story readable from one line.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example drag_gesture
//! ```
//!
//! The beats: the press that arms the drag (the value it found), the
//! stride that moves it (the moves stream nothing — the flood rule),
//! the release that lands it (the value beside the beginning), the
//! leave that ends the second drag honestly, and the wall's silence
//! (the unavailable slider arms no gesture). Then the window opens for
//! live play: drag the thumb and the trail mirrors the log.
//! `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, Slider};
use alacer_core::events::{MouseButton, PointerEvent};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::{Point, Size};
use alacer_core::inspect::{GesturePhase, InspectEvent, InspectObserver};
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{EventResponse, Role};
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the value trail's
/// shape).
const VIEWPORT: Size = Size::new(480.0, 340.0);

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// The fact as the trail's reader renders it: the phase beside the
/// value it found and the value it landed, the control's id beside
/// them — the log answering "where did the drag start, where did it
/// land" from one line. The record is `Copy`, so it passes by value
/// (the pedantic group's rule).
fn describe(record: alacer_core::GestureRecord) -> String {
    let phase = match record.phase {
        GesturePhase::Started => format!("the drag found the value {}", record.from),
        GesturePhase::Ended => format!(
            "the drag landed the value {} (from {})",
            record.to.map_or_else(String::new, |v| v.to_string()),
            record.from
        ),
    };
    let who = record.target.map_or_else(
        || "control (unrouted)".to_string(),
        |id| format!("control {id}"),
    );
    format!("{phase} — {who}")
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the drain rode (axiom A1: signals in, never widgets).
struct Trail {
    gestures: Rc<RefCell<std::collections::VecDeque<alacer_core::GestureRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The observer half: boxed into
    /// [`alacer_core::passes::PassPipeline::set_observer`], it receives
    /// every `InspectEvent`, keeps the gesture facts, and mirrors the
    /// newest into the trail label's signal — the write queues masks,
    /// the frame renders them.
    fn writer(&self) -> TrailWriter {
        TrailWriter {
            gestures: Rc::clone(&self.gestures),
            trail_line: self.trail_line,
        }
    }

    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<alacer_core::GestureRecord> {
        self.gestures.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    gestures: Rc<RefCell<std::collections::VecDeque<alacer_core::GestureRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::Gesture(record) = event else {
            return;
        };
        let line = describe(record);
        {
            let mut ring = self.gestures.borrow_mut();
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

/// The slider's track center, read off the laid-out arena — the
/// position the beats aim the pointer at (the router's own hit test
/// resolves the deepest hit from it).
fn slider_center(pipeline: &PassPipeline) -> Point {
    let arena = pipeline.arena();
    let mut found = None;
    arena.for_each_pre_order(&mut |_id, slot| {
        if found.is_none() && slot.widget().role() == Role::Slider {
            found = slot
                .bounds
                .map(|b| Point::new(b.min_x() + b.width() * 0.5, b.min_y() + b.height() * 0.5));
        }
    });
    found.expect("the slider is mounted and laid out")
}

/// One beat: the span names the route, the real pointer route answers,
/// the gesture's fact streams to the observer, and the response is
/// logged — the funnel's own route, no second channel.
fn beat(pipeline: &mut PassPipeline, label: &'static str, event: &PointerEvent) {
    let _route =
        tracing::trace_span!(target: "alacer::drag_gesture", "beat", label = label).entered();
    let response = pipeline.dispatch_pointer(event);
    settle(pipeline);
    tracing::info!(
        target: "alacer::drag_gesture",
        handled = matches!(response, EventResponse::Handled),
        "{label}"
    );
}

/// The beats the demo walks before the window opens: the press that
/// arms the drag, the moves that stream nothing, the release that
/// lands it, the leave that ends the second drag, and the wall's
/// silence. Each a real pointer event through the real dispatch.
fn run_beats(pipeline: &mut PassPipeline, value_signal: alacer_core::FluxRwSignal<String>) {
    let center = slider_center(pipeline);

    // Beat one: the press arms the drag — one `Started` record, the
    // value it found (the app's own), the funnel's pointer route
    // stamping the control.
    beat(
        pipeline,
        "beat 1: press — the drag arms, the value it found streams",
        &PointerEvent::Down {
            position: center,
            button: MouseButton::Left,
            click_count: 1,
        },
    );

    // Beat two: the moves stride the thumb across the track — thirty
    // dispatched moves stream nothing (the flood rule prices the
    // gesture, never the moves).
    for step in 1..=30_u16 {
        let position = Point::new(center.x + f32::from(step) * 3.0, center.y);
        beat(
            pipeline,
            "beat 2: move — the stride writes the value, the log stays silent",
            &PointerEvent::Move { position },
        );
    }
    value_signal.set("the value: the thumb moved — the moves streamed nothing".to_string());

    // Beat three: the release lands it — one `Ended` record, the value
    // beside the beginning, the story readable from one line.
    let landed = Point::new(center.x + 90.0, center.y);
    beat(
        pipeline,
        "beat 3: release — the drag lands, the ending's record streams",
        &PointerEvent::Up {
            position: landed,
            button: MouseButton::Left,
        },
    );
    value_signal.set("the value: the drag's story — two records, one gesture".to_string());

    // Beat four: a second gesture ended by the pointer leaving — the
    // leave resolves the drag honestly (the capture released), the
    // ending's record streaming like a release's.
    beat(
        pipeline,
        "beat 4: press then leave — the capture released, the drag ended as data",
        &PointerEvent::Down {
            position: center,
            button: MouseButton::Left,
            click_count: 1,
        },
    );
    beat(
        pipeline,
        "beat 4b: leave — the ending's record streams",
        &PointerEvent::Leave,
    );

    // Beat five: the wall's silence — the unavailable slider consumes
    // the press but arms nothing; a gesture that never was streams
    // nothing (the honest absence the trail's own rule names).
    beat(
        pipeline,
        "beat 5: press on the wall — no capture, no write, no gesture fact",
        &PointerEvent::Down {
            position: Point::new(center.x, center.y + 200.0),
            button: MouseButton::Left,
            click_count: 1,
        },
    );
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The drag's story, streamed to the causal log (ADR-0052)").boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .value(10.0)
            .step(1.0)
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Slider::new("Muted", 0.0, 100.0)
            .value(40.0)
            .enabled(false)
            .boxed(),
    );

    let value_signal = pipeline
        .flux_mut()
        .signal("the value: (the tree carries what the app wrote)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let value_binding =
        RenderBinding::new("the value: (the tree carries what the app wrote)".to_string());
    let value_label = pipeline.add_child(root, Label::bound(value_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        value_signal,
        value_label,
        &value_binding,
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

    // The observer rides from here on: every gesture phase's record
    // streams into the trail label the frame after the door resolves.
    let trail = Trail {
        gestures: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    settle(&mut pipeline);
    run_beats(&mut pipeline, value_signal);

    // The window: live play — drag the thumb and the trail mirrors the
    // log; the moves stay silent, the phases speak.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: drag the thumb — the trail mirrors the log, two records per gesture")
            .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer drag gesture",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the drag's history as data. The phases
    // named themselves; the flood rule held.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::drag_gesture",
        records = read_back.len(),
        started = read_back
            .iter()
            .filter(|record| record.phase == GesturePhase::Started)
            .count(),
        ended = read_back
            .iter()
            .filter(|record| record.phase == GesturePhase::Ended)
            .count(),
        attributed = read_back
            .iter()
            .filter(|record| record.target.is_some())
            .count(),
        "the phases streamed as they resolved — the causal log carries the drag's story"
    );
}
