//! The drag's wall joins the log (§13.3, ADR-0054): a drag that slams
//! an edge is the pointer's own bound jump, and since M48 the gesture
//! record's `Ended` phase carries the bound the landing pinned
//! ([`alacer_core::inspect::RangeBound::Min`]/[`alacer_core::inspect::RangeBound::Max`])
//! beside the landing it produced — ADR-0053's first kill rule cashed
//! on the gesture vocabulary's own growth, never the AT tier's species
//! munged onto a pointer act. The demo drives the real pointer route
//! (`PassPipeline::dispatch_pointer`, the exact path the platform loop
//! rides — never a second channel) against one `Slider` with the
//! causal observer riding: the beats press, stride, slam the walls,
//! and land mid-track, and the *trail* label mirrors what the log
//! streamed — the wall named where it was pinned, the absence honest
//! where it never was.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example drag_bounds
//! ```
//!
//! The beats: the press that arms the drag (the beginning carries no
//! bound — the landing is the answer's fact), the stride that moves it
//! silently, the release at the max (the high wall, named), the leave
//! at the min (the low wall, named by the leave's own honest ending),
//! the mid-track landing (no wall, the absence riding), and the wall's
//! silence (the unavailable slider arms no gesture at all). Then the
//! window opens for live play: slam an edge and the trail names it.
//! `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, Slider};
use alacer_core::events::{MouseButton, PointerEvent};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::inspect::{GesturePhase, InspectEvent, InspectObserver, RangeBound};
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{EventResponse, Role};
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the gesture trail's
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
/// value it found and the value it landed, the wall a landing pinned
/// beside them, the control's id beside everything — the log answering
/// "where did the drag land, and did it pin a wall" from one line. The
/// record is `Copy`, so it passes by value (the pedantic group's rule).
fn describe(record: alacer_core::GestureRecord) -> String {
    let phase = match record.phase {
        GesturePhase::Started => format!("the drag found the value {}", record.from),
        GesturePhase::Ended => format!(
            "the drag landed the value {} (from {}){}",
            record.to.map_or_else(String::new, |v| v.to_string()),
            record.from,
            record.bound.map_or_else(String::new, |bound| format!(
                ", pinned at the {}",
                bound.name()
            )),
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

/// The slider's track center and frame, read off the laid-out arena —
/// the geometry the beats aim the pointer with (the router's own hit
/// test resolves the deepest hit from it; the walls live just past the
/// frame's edges).
fn slider_geometry(pipeline: &PassPipeline) -> (Point, Rect) {
    let arena = pipeline.arena();
    let mut found = None;
    arena.for_each_pre_order(&mut |_id, slot| {
        if found.is_none() && slot.widget().role() == Role::Slider {
            let bounds = slot.bounds.expect("the slider is laid out");
            let center = Point::new(
                bounds.min_x() + bounds.width() * 0.5,
                bounds.min_y() + bounds.height() * 0.5,
            );
            found = Some((center, bounds));
        }
    });
    found.expect("the slider is mounted and laid out")
}

/// One beat: the span names the route, the real pointer route answers,
/// the gesture's fact streams to the observer, and the response is
/// logged — the funnel's own route, no second channel.
fn beat(pipeline: &mut PassPipeline, label: &'static str, event: &PointerEvent) {
    let _route =
        tracing::trace_span!(target: "alacer::drag_bounds", "beat", label = label).entered();
    let response = pipeline.dispatch_pointer(event);
    settle(pipeline);
    tracing::info!(
        target: "alacer::drag_bounds",
        handled = matches!(response, EventResponse::Handled),
        "{label}"
    );
}

/// The beats the demo walks before the window opens: the press that
/// arms the drag, the silent stride, the slam and release at the max,
/// the leave at the min, the mid-track landing's honest absence, and
/// the wall's silence. Each a real pointer event through the real
/// dispatch.
fn run_beats(pipeline: &mut PassPipeline, value_signal: alacer_core::FluxRwSignal<String>) {
    let (center, bounds) = slider_geometry(pipeline);

    // Beat one: the press arms the drag — one `Started` record, the
    // value it found, and no bound: the landing, and so the wall, is
    // the gesture's answer, never its beginning's fact.
    beat(
        pipeline,
        "beat 1: press — the drag arms, the beginning carries no bound",
        &PointerEvent::Down {
            position: center,
            button: MouseButton::Left,
            click_count: 1,
        },
    );

    // Beat two: the stride moves the thumb — the moves stream nothing
    // (the flood rule prices the gesture, never the moves).
    for step in 1..=10_u16 {
        let position = Point::new(center.x + f32::from(step) * 3.0, center.y);
        beat(
            pipeline,
            "beat 2: move — the stride writes the value, the log stays silent",
            &PointerEvent::Move { position },
        );
    }

    // Beat three: the release at the max — the pointer slams the high
    // wall and the `Ended` record names it: the bound jump the
    // *pointer* asked for, on the drag's own record.
    let past_max = Point::new(bounds.max_x() + 60.0, center.y);
    beat(
        pipeline,
        "beat 3: stride past the edge, release — the max is named",
        &PointerEvent::Move { position: past_max },
    );
    beat(
        pipeline,
        "beat 3b: release — the landing pins the max, the wall streams",
        &PointerEvent::Up {
            position: past_max,
            button: MouseButton::Left,
        },
    );
    value_signal.set("the value: the high wall, named — pinned at the max".to_string());

    // Beat four: a second gesture, ended by the pointer leaving at the
    // min — the leave resolves the drag honestly (the capture
    // released) and its landing names the low wall exactly the
    // release's answer does: one vocabulary, both doors.
    beat(
        pipeline,
        "beat 4: press — a new gesture arms",
        &PointerEvent::Down {
            position: center,
            button: MouseButton::Left,
            click_count: 1,
        },
    );
    let past_min = Point::new(bounds.min_x() - 60.0, center.y);
    beat(
        pipeline,
        "beat 4b: stride past the low edge",
        &PointerEvent::Move { position: past_min },
    );
    beat(
        pipeline,
        "beat 4c: leave — the capture released, the landing pins the min",
        &PointerEvent::Leave,
    );
    value_signal.set("the value: the low wall, named — pinned at the min".to_string());

    // Beat five: a drag that lands mid-track pinned no wall — the
    // `Ended` record carries the honest absence, the bound naming only
    // a landing at the range's own edge.
    beat(
        pipeline,
        "beat 5: press — a new gesture arms",
        &PointerEvent::Down {
            position: center,
            button: MouseButton::Left,
            click_count: 1,
        },
    );
    let mid = Point::new(center.x + 8.0, center.y);
    beat(
        pipeline,
        "beat 5b: a short stride, release — no wall, the absence honest",
        &PointerEvent::Up {
            position: mid,
            button: MouseButton::Left,
        },
    );
    value_signal.set("the value: a mid-track landing — no wall, the absence honest".to_string());

    // Beat six: the wall's silence — the unavailable slider consumes
    // the press but arms nothing; a gesture that never was streams
    // nothing, bound or otherwise.
    beat(
        pipeline,
        "beat 6: press on the muted slider — no capture, no write, no fact",
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
        Label::new("The drag's wall, streamed to the causal log (ADR-0054)").boxed(),
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

    // The window: live play — slam an edge and the trail names the
    // wall; the moves stay silent, the phases speak.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: slam an edge — the trail names the wall the landing pinned").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer drag bounds",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the drag's history as data. The walls
    // named themselves; the flood rule held.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::drag_bounds",
        records = read_back.len(),
        started = read_back
            .iter()
            .filter(|record| record.phase == GesturePhase::Started)
            .count(),
        ended = read_back
            .iter()
            .filter(|record| record.phase == GesturePhase::Ended)
            .count(),
        walls = read_back
            .iter()
            .filter(|record| record.bound.is_some())
            .count(),
        named_max = read_back
            .iter()
            .filter(|record| record.bound == Some(RangeBound::Max))
            .count(),
        named_min = read_back
            .iter()
            .filter(|record| record.bound == Some(RangeBound::Min))
            .count(),
        "the walls streamed as they were pinned — the causal log carries the drag's wall"
    );
}
