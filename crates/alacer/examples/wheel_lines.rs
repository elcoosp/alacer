//! The wheel's line count joins the scroll record (§13.3, ADR-0059):
//! ADR-0058's second kill rule cashes — a consumer needs the wheel's
//! line count named — the coalesced run's notional lines — so the
//! record carries the run priced by the platform's own convention
//! (one line = 40 px, §10.2): a notch answers 1.0, the trackpad's
//! pixel runs their honest fraction, and a slam past the wall still
//! names the run it was (the count is computed before the clamp
//! arithmetic — the count is the ask's story, never the landing's).
//! The count rides the honest absence on the stories with no wheel
//! run behind them: the AT's unit scrolls (their unit is the
//! distance, M52), the value steps, the app's own door, the content's
//! own clamp (M51). The demo drives the real pointer route
//! (`PassPipeline::dispatch_pointer` — the exact path a mouse wheel
//! rides) with the causal observer riding, and the *trail* label
//! mirrors what the log streamed.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example wheel_lines
//! ```
//!
//! The beats: the one-notch run, the two-notch run, the trackpad's
//! fractional run, the slam past the wall beside its pin, the NaN run
//! refusing with no count at all, and the unit scroll, the step, the
//! app door, and the clamp riding the absence. Then the window opens
//! for live play: the wheel rides the same pointer route, the trail
//! mirroring the log. `ALACER_SOFTWARE=1` forces the softbuffer path.

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

/// The viewport the demo's beats settle into (the scroll trail's
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

/// The fact as the trail's reader renders it: the run's notional lines
/// beside the wheel action, the honest absences visible — the log
/// answering "how big was the run" from one line. The record is
/// `Copy`, so it passes by value (the pedantic group's rule).
fn describe(record: alacer_core::ScrollRecord) -> String {
    let run = record
        .lines
        .map_or_else(String::new, |lines| format!(" (the run priced {lines})"));
    let unit = record
        .unit
        .map_or_else(String::new, |unit| format!(" ({})", unit.name()));
    match record.outcome {
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
            format!("the scroll landed{asked}{wall}{unit}{run}")
        }
        ScrollOutcome::Refused(refusal) => format!("refused: {}{unit}{run}", refusal.name()),
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
        let Ok(report) = pipeline.tick(VIEWPORT) else {
            return;
        };
        if report.is_idle() {
            return;
        }
    }
}

/// The list's current bounds center (the wheel beat's aim — the
/// container's own bounds translate with the offset).
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

/// One beat on the pointer route: the wheel's run, the delta the
/// platform delivered — the run the record prices.
fn wheel_beat(pipeline: &mut PassPipeline, label: &str, delta_y: f32) {
    let _route =
        tracing::trace_span!(target: "alacer::wheel_lines", "beat", label = label).entered();
    let response = pipeline.dispatch_pointer(&PointerEvent::Wheel {
        delta: Vector::new(0.0, delta_y),
        position: list_center(pipeline),
    });
    settle(pipeline);
    tracing::info!(
        target: "alacer::wheel_lines",
        handled = matches!(response, alacer_core::widget::EventResponse::Handled),
        "{label}"
    );
}

/// One beat on the AT route: the unit scroll whose unit is the
/// distance — the count riding its honest absence (a second magnitude
/// would be the same story told twice).
fn act_beat(pipeline: &mut PassPipeline, label: &'static str, action: A11yAction) {
    let _route =
        tracing::trace_span!(target: "alacer::wheel_lines", "beat", label = label).entered();
    let tree = pipeline
        .a11y_snapshot()
        .expect("the a11y snapshot after the settle");
    let node = tree
        .find(Role::ScrollArea, "list")
        .unwrap_or_else(|| panic!("the AT can reach the list named `list`"));
    let consumed = pipeline.dispatch_a11y_action(node.id, action);
    settle(pipeline);
    tracing::info!(target: "alacer::wheel_lines", consumed, "{label}");
}

/// The beats the demo walks before the window opens: the wheel's runs
/// priced by the platform's own convention, the slam naming the run it
/// was, the NaN run refusing with no count, and the stories with no
/// run riding the absence. Each a real ask through the real routes.
fn run_beats(
    pipeline: &mut PassPipeline,
    handle: &alacer::widgets::ListHandle,
    status_signal: alacer_core::FluxRwSignal<String>,
) {
    // Beat one: the one-notch run — -40 px is one line by the
    // platform's own convention.
    wheel_beat(
        pipeline,
        "beat 1: the wheel's notch — the run priced 1.0",
        -40.0,
    );
    status_signal.set("the wheel's run: the count prices the pointer's ask".to_string());

    // Beat two: the coalesced run — two notches in one batch, priced
    // 2.0, the run's own magnitude.
    wheel_beat(pipeline, "beat 2: the coalesced run — priced 2.0", -80.0);

    // Beat three: the trackpad's pixel run — -100 px prices its
    // honest fraction, 2.5 lines.
    wheel_beat(pipeline, "beat 3: the trackpad's run — priced 2.5", -100.0);

    // Beat four: the slam past the wall — the landing pins at the max,
    // the count prices what the pointer asked (the run it was, not
    // the landing the clamp produced).
    wheel_beat(
        pipeline,
        "beat 4: the slam — 250 lines beside the pin",
        -10_000.0,
    );

    // Beat five: the NaN run — the refusal names the wall, the count
    // riding the absence (a count of a run that never was would be
    // the lie the M44 rule forbids).
    wheel_beat(
        pipeline,
        "beat 5: the NaN run — refused, no count",
        f32::NAN,
    );

    // Beat six: the AT's unit scroll — the unit is the distance (M52);
    // the count rides its honest absence.
    act_beat(
        pipeline,
        "beat 6: AT ScrollDown(Item) — the unit is the distance, no count",
        A11yAction::ScrollDown(ScrollUnit::Item),
    );

    // Beat seven: the app's own door — a programmatic ask with no run
    // in it.
    let _route =
        tracing::trace_span!(target: "alacer::wheel_lines", "beat", label = "beat 7").entered();
    handle.scroll_to(200.0);
    settle(pipeline);
    tracing::info!(target: "alacer::wheel_lines", "beat 7: scroll_to(200) — the app door rides no run");
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The wheel's line count joins the scroll record (ADR-0059)").boxed(),
    );
    let list = List::new(120.0);
    let handle = list.handle();
    let _ = pipeline.add_child(root, list.boxed());
    handle.set_content_height(500.0);

    let status_signal = pipeline
        .flux_mut()
        .signal("the wheel's run: (the tree carries what the app wrote)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let status_binding =
        RenderBinding::new("the wheel's run: (the tree carries what the app wrote)".to_string());
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

    settle(&mut pipeline);
    run_beats(&mut pipeline, &handle, status_signal);

    // The window: live play — the wheel rides the real pointer route,
    // the run pricing itself on the log; the trail mirrors it.
    let _ = pipeline.add_child(
        root,
        Label::new(
            "Live: turn the wheel over the list — every run prices its notional lines on the log",
        )
        .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer wheel lines",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the run's count beside every wheel ask.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::wheel_lines",
        records = read_back.len(),
        with_lines = read_back
            .iter()
            .filter(|record| record.lines.is_some())
            .count(),
        delivered = read_back
            .iter()
            .filter(|record| record.outcome == ScrollOutcome::Delivered)
            .count(),
        "the runs streamed as they resolved — the causal log carries the wheel's line count"
    );
}
