//! The scroll's story joins the log (§13.3, ADR-0056): the scroll
//! tier's answers — the list's wheel, the AT's unit scrolls and value
//! steps, the app's own programmatic scroll — reach the causal log as
//! one [`alacer_core::ScrollRecord`] per ask, the landed offset beside
//! the ask, the wall the clamp pinned beside both (ADR-0055's second
//! kill rule: the wall named on the tier whose funnel knows no range),
//! the honest refusals named. The demo drives the real routes
//! (`PassPipeline::dispatch_pointer` and `dispatch_a11y_action`, the
//! exact paths the platform loop and the AT ride — never a second
//! channel) against two lists with the causal observer riding: the
//! beats wheel, scroll, step, and refuse, and the *trail* label
//! mirrors what the log streamed — every ask answering as data, the
//! fitted list's refusal included.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example scroll_trail
//! ```
//!
//! The beats: the mid-range wheel that lands what it asked, the past
//! ask the wall turned back (pinned at the max), the ask that wanted
//! the wall exactly (the honest absence), the AT's item scroll and
//! value step through the real action route (attributed), the fitted
//! list's `no_scrollable_content`, the non-finite ask refused before
//! it corrupts, and the app's own scroll-to (no route — the stamp's
//! honest absence). Then the window opens for live play: wheel the
//! list and the trail mirrors the log. `ALACER_SOFTWARE=1` forces the
//! softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, List};
use alacer_core::events::PointerEvent;
use alacer_core::flux::RenderBinding;
use alacer_core::geom::{Point, Size, Vector};
use alacer_core::inspect::{InspectEvent, InspectObserver, ScrollOutcome};
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{A11yAction, Role, ScrollUnit};
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the gesture trail's
/// shape).
const VIEWPORT: Size = Size::new(480.0, 380.0);

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// The fact as the trail's reader renders it: the landed offset beside
/// the ask, the wall the clamp pinned, the refusals named, the
/// control's id beside them — the log answering "did the list scroll —
/// where, or why not" from one line. The record is `Copy`, so it
/// passes by value (the pedantic group's rule).
fn describe(record: alacer_core::ScrollRecord) -> String {
    let outcome = match record.outcome {
        ScrollOutcome::Delivered => {
            let asked = record.requested.map_or_else(String::new, |ask| {
                if ask.total_cmp(&record.offset) == std::cmp::Ordering::Equal {
                    format!(" (the offset {})", record.offset)
                } else {
                    format!(" (asked {ask}, landed {})", record.offset)
                }
            });
            let wall = record.pinned.map_or_else(String::new, |bound| {
                format!(", pinned at the {}", bound.name())
            });
            format!("the scroll landed{asked}{wall}")
        }
        ScrollOutcome::Refused(refusal) => format!("refused: {}", refusal.name()),
    };
    let who = record.target.map_or_else(
        || "control (unrouted)".to_string(),
        |id| format!("control {id}"),
    );
    format!("{outcome} — {who}")
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
    /// newest into the trail label's signal — the write queues masks,
    /// the frame renders them.
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
        let Ok(report) = pipeline.tick(VIEWPORT) else {
            return;
        };
        if report.is_idle() {
            return;
        }
    }
}

/// The list's first row's center, read off the laid-out arena — the
/// position the beats aim the pointer at (the router's own hit test
/// resolves the deepest hit from it).
fn list_center(pipeline: &PassPipeline) -> Point {
    let arena = pipeline.arena();
    let mut found = None;
    arena.for_each_pre_order(&mut |_id, slot| {
        if found.is_none() && slot.widget().role() == Role::ScrollArea && slot.bounds.is_some() {
            found = slot
                .bounds
                .map(|b| Point::new(b.min_x() + b.width() * 0.5, b.min_y() + b.height() * 0.5));
        }
    });
    found.expect("the list is mounted and laid out")
}

/// The scrollable list's AccessKit-stable id, read off the real a11y
/// snapshot — the identity the dispatch funnel routes and the stamp
/// carries (the M43 vocabulary).
fn list_id(pipeline: &PassPipeline) -> u64 {
    let tree = pipeline
        .a11y_snapshot()
        .expect("the a11y snapshot after the settle");
    let node = tree
        .find(Role::ScrollArea, "list")
        .unwrap_or_else(|| panic!("the AT can reach the list named `list`"));
    node.id
}

/// One beat: the span names the route, the real pointer route answers,
/// the ask's fact streams to the observer, and the response is logged —
/// the funnel's own route, no second channel.
fn wheel_beat(pipeline: &mut PassPipeline, label: &'static str, delta_y: f32) {
    let _route =
        tracing::trace_span!(target: "alacer::scroll_trail", "beat", label = label).entered();
    let center = list_center(pipeline);
    let response = pipeline.dispatch_pointer(&PointerEvent::Wheel {
        delta: Vector::new(0.0, delta_y),
        position: center,
    });
    settle(pipeline);
    tracing::info!(
        target: "alacer::scroll_trail",
        handled = matches!(response, alacer_core::widget::EventResponse::Handled),
        "{label}"
    );
}

/// One beat on the AT route: the span names the route, the real action
/// tier answers, the scroll streams to the observer, and the response
/// is logged — the funnel's own route, no second channel.
fn act_beat(pipeline: &mut PassPipeline, label: &'static str, action: A11yAction) {
    let _route =
        tracing::trace_span!(target: "alacer::scroll_trail", "beat", label = label).entered();
    let id = list_id(pipeline);
    let consumed = pipeline.dispatch_a11y_action(id, action);
    settle(pipeline);
    tracing::info!(
        target: "alacer::scroll_trail",
        consumed, "{label}"
    );
}

/// One beat on the app's own door: the shared handle's scroll, no
/// route behind it — the record's stamp rides the honest absence (the
/// M43 rule: the stamp fills only the identity the door cannot know,
/// and the app's handle knows no node id at all).
fn handle_beat(handle: &alacer::widgets::ListHandle, label: &'static str, offset: f32) {
    let _route =
        tracing::trace_span!(target: "alacer::scroll_trail", "beat", label = label).entered();
    handle.scroll_to(offset);
}

/// The beats the demo walks before the window opens: the mid-range
/// wheel, the walls, the AT's rides, the refusals, and the app's own
/// door. Each a real ask through the real routes.
fn run_beats(
    pipeline: &mut PassPipeline,
    handle: &alacer::widgets::ListHandle,
    fitted_pipeline: &mut PassPipeline,
    status_signal: alacer_core::FluxRwSignal<String>,
) {
    // Beat one: the mid-range wheel — the ask lands what it asked, the
    // wall absent (a landing the clamp never touched pins nothing).
    wheel_beat(
        pipeline,
        "beat 1: wheel — the ask lands, the landing beside it streams",
        -60.0,
    );

    // Beat two: the ask past the high wall — the clamp turns it back,
    // the record naming the bound the landing pinned beside the ask.
    wheel_beat(
        pipeline,
        "beat 2: wheel past the max — the wall pins the landing",
        -10_000.0,
    );

    // Beat three: the ask past the low wall — the other edge, the other
    // species (the min).
    wheel_beat(
        pipeline,
        "beat 3: wheel past the min — the low wall pins the landing",
        10_000.0,
    );
    status_signal.set("the scroll: the walls name themselves — min and max".to_string());

    // Beat four: the AT's item scroll through the real action route —
    // one line, the ride attributed (the stamp beside it).
    act_beat(
        pipeline,
        "beat 4: AT ScrollDown(Item) — one line, the ride attributed",
        A11yAction::ScrollDown(ScrollUnit::Item),
    );

    // Beat five: the AT's value step (M10: the scrollbar's value *is*
    // the offset) — one page, a scroll fact on the scroll log, never a
    // value fact on the value log.
    act_beat(
        pipeline,
        "beat 5: AT Increment — one page, the step is a scroll fact",
        A11yAction::Increment,
    );

    // Beat six: the app's own door — the ask that wanted the wall
    // exactly pins nothing (the landing *is* the ask), the stamp's
    // honest absence beside it.
    handle_beat(
        handle,
        "beat 6: scroll_to(max) — the ask wanted the wall exactly, no pin",
        400.0,
    );

    // Beat seven: the fitted list's refusal — the content fits the
    // viewport, no scroll exists to deliver (the honest refusal the
    // trail prices).
    let _route =
        tracing::trace_span!(target: "alacer::scroll_trail", "beat", label = "beat 7").entered();
    // The wheel rides the hit test: the fitted list sits high in its
    // window's column, the same spot the scrollable list's own wheel
    // beats aim at.
    let response = fitted_pipeline.dispatch_pointer(&PointerEvent::Wheel {
        delta: Vector::new(0.0, -60.0),
        position: list_center(fitted_pipeline),
    });
    settle(fitted_pipeline);
    tracing::info!(
        target: "alacer::scroll_trail",
        handled = matches!(response, alacer_core::widget::EventResponse::Handled),
        "beat 7: wheel over the fitted list — no scrollable content, the refusal streams"
    );

    // Beat eight: the non-finite ask refused before the arithmetic —
    // the debt the trail's design exposed (a NaN delta rode the clamp
    // and corrupted the state), the door refusing before it now.
    wheel_beat(
        pipeline,
        "beat 8: a NaN wheel — refused before it corrupts, the offset keeps",
        f32::NAN,
    );
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The scroll's story, streamed to the causal log (ADR-0056)").boxed(),
    );
    let list = List::new(120.0);
    let handle = list.handle();
    let _ = pipeline.add_child(root, list.boxed());
    handle.set_content_height(500.0);

    let status_signal = pipeline
        .flux_mut()
        .signal("the scroll: (the tree carries what the app wrote)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let status_binding =
        RenderBinding::new("the scroll: (the tree carries what the app wrote)".to_string());
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

    // The observer rides from here on: every scroll ask's record
    // streams into the trail label the frame after the door resolves.
    let trail = Trail {
        scrolls: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    // The fitted list's pipeline: the refusal's stage (the content fits
    // the viewport — a range with no span).
    let mut fitted_pipeline = pipeline_default();
    let fitted_root = fitted_pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let fitted_list = List::new(120.0);
    let fitted_handle = fitted_list.handle();
    let _ = fitted_pipeline.add_child(fitted_root, fitted_list.boxed());
    fitted_handle.set_content_height(80.0);
    let fitted_observer = Trail {
        scrolls: Rc::clone(&trail.scrolls),
        trail_line: trail_signal,
    };
    fitted_pipeline.set_observer(Box::new(fitted_observer.writer()));
    settle(&mut fitted_pipeline);

    settle(&mut pipeline);
    run_beats(&mut pipeline, &handle, &mut fitted_pipeline, status_signal);

    // The window: live play — wheel the list and the trail mirrors the
    // log; the AT's rides stay attributed.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: wheel the list — the trail mirrors the log, one record per ask").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer scroll trail",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the scroll's history as data. The walls
    // named themselves; the refusals refused honestly.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::scroll_trail",
        records = read_back.len(),
        delivered = read_back
            .iter()
            .filter(|record| record.outcome == ScrollOutcome::Delivered)
            .count(),
        pinned = read_back.iter().filter(|record| record.pinned.is_some()).count(),
        refused = read_back
            .iter()
            .filter(|record| matches!(record.outcome, ScrollOutcome::Refused(_)))
            .count(),
        attributed = read_back
            .iter()
            .filter(|record| record.target.is_some())
            .count(),
        "the asks streamed as they resolved — the causal log carries the scroll's story"
    );
}
