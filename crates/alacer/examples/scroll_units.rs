//! The asker's unit joins the scroll record (§13.3, ADR-0058):
//! ADR-0056's first kill rule cashes — a consumer needs the asker's
//! unit named — one item or one page — on the AT's unit scrolls, the
//! distance the ask is priced against. The record carries the unit
//! beside the action species ([`alacer_core::ScrollRecord`]'s
//! additive `unit` field), and rides the honest absence on the
//! stories with no unit: the wheel's coalesced run, the value step's
//! page (a step, not a unit scroll), the app's own door, the
//! content's own clamp (M51). The demo drives the real AT action
//! route (`PassPipeline::dispatch_a11y_action` — the exact path a
//! screen reader's scroll command rides) with the causal observer
//! riding, and the *trail* label mirrors what the log streamed.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example scroll_units
//! ```
//!
//! The beats: the item scroll that lands one line with its unit
//! beside it, the page scroll that lands the viewport with the other
//! unit, the step that rides no unit, the wheel that rides none, and
//! the app door that rides none. Then the window opens for live play:
//! the buttons ride the same action route's vocabulary, the trail
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

/// The fact as the trail's reader renders it: the asker's unit beside
/// the unit-scroll actions, the honest absences visible — the log
/// answering "by how much did the AT ask" from one line. The record
/// is `Copy`, so it passes by value (the pedantic group's rule).
fn describe(record: alacer_core::ScrollRecord) -> String {
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
            format!("the scroll landed{asked}{wall}{unit}")
        }
        ScrollOutcome::Refused(refusal) => format!("refused: {}{unit}", refusal.name()),
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

/// One beat on the AT route: the span names the ask, the real action
/// tier answers, the fact streams to the observer — the funnel's own
/// route, no second channel.
fn act_beat(pipeline: &mut PassPipeline, label: &'static str, action: A11yAction) {
    let _route =
        tracing::trace_span!(target: "alacer::scroll_units", "beat", label = label).entered();
    let id = list_id(pipeline);
    let consumed = pipeline.dispatch_a11y_action(id, action);
    settle(pipeline);
    tracing::info!(target: "alacer::scroll_units", consumed, "{label}");
}

/// One beat on the pointer route: the wheel's notch — the coalesced
/// run that rides no unit (the honest absence the log names).
fn wheel_beat(pipeline: &mut PassPipeline, label: &'static str, delta_y: f32) {
    let _route =
        tracing::trace_span!(target: "alacer::scroll_units", "beat", label = label).entered();
    let response = pipeline.dispatch_pointer(&PointerEvent::Wheel {
        delta: Vector::new(0.0, delta_y),
        position: list_center(pipeline),
    });
    settle(pipeline);
    tracing::info!(
        target: "alacer::scroll_units",
        handled = matches!(response, alacer_core::widget::EventResponse::Handled),
        "{label}"
    );
}

/// The beats the demo walks before the window opens: the AT's unit
/// scrolls with their units beside them, the step's honest absence,
/// the wheel's, and the app door's. Each a real ask through the real
/// routes.
fn run_beats(
    pipeline: &mut PassPipeline,
    handle: &alacer::widgets::ListHandle,
    status_signal: alacer_core::FluxRwSignal<String>,
) {
    // Beat one: the AT's item scroll through the real action route —
    // one line, the asker's unit beside the landing.
    act_beat(
        pipeline,
        "beat 1: AT ScrollDown(Item) — one line, the unit beside it",
        A11yAction::ScrollDown(ScrollUnit::Item),
    );
    status_signal.set("the asker's unit: item scrolls name the item".to_string());

    // Beat two: the page scroll — the other unit, the same vocabulary:
    // one page, the viewport the ask asked by.
    act_beat(
        pipeline,
        "beat 2: AT ScrollDown(Page) — one page, the unit beside it",
        A11yAction::ScrollDown(ScrollUnit::Page),
    );

    // Beat three: the reverse ask — the unit rides the up direction
    // too.
    act_beat(
        pipeline,
        "beat 3: AT ScrollUp(Item) — the reverse ask, the unit rides",
        A11yAction::ScrollUp(ScrollUnit::Item),
    );

    // Beat four: the value step (M10: the scrollbar's value *is* the
    // offset) — a page-wide step, but a *step*, not a unit scroll: the
    // unit rides its honest absence.
    act_beat(
        pipeline,
        "beat 4: AT Increment — the step is not a unit scroll, no unit",
        A11yAction::Increment,
    );

    // Beat five: the wheel — a coalesced run the pointer asked, never
    // a unit ask: the absence again.
    wheel_beat(
        pipeline,
        "beat 5: the wheel — a coalesced run rides no unit",
        -60.0,
    );

    // Beat six: the app's own door — a programmatic ask with no unit
    // in it (the absence the M50 trail already named, the unit
    // beside it now).
    let _route =
        tracing::trace_span!(target: "alacer::scroll_units", "beat", label = "beat 6").entered();
    handle.scroll_to(200.0);
    settle(pipeline);
    tracing::info!(target: "alacer::scroll_units", "beat 6: scroll_to(200) — the app door rides no unit");
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The asker's unit joins the scroll record (ADR-0058)").boxed(),
    );
    let list = List::new(120.0);
    let handle = list.handle();
    let _ = pipeline.add_child(root, list.boxed());
    handle.set_content_height(500.0);

    let status_signal = pipeline
        .flux_mut()
        .signal("the asker's unit: (the tree carries what the app wrote)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let status_binding =
        RenderBinding::new("the asker's unit: (the tree carries what the app wrote)".to_string());
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

    // The window: live play — the AT's unit scrolls ride the real
    // action route (a screen reader's scroll commands), the wheel rides
    // the pointer route; the trail mirrors the log.
    let _ = pipeline.add_child(
        root,
        Label::new(
            "Live: a screen reader's scroll commands name their unit on the log; the wheel rides none",
        )
        .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer scroll units",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the asker's unit beside every unit ask.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::scroll_units",
        records = read_back.len(),
        with_unit = read_back
            .iter()
            .filter(|record| record.unit.is_some())
            .count(),
        delivered = read_back
            .iter()
            .filter(|record| record.outcome == ScrollOutcome::Delivered)
            .count(),
        "the asks streamed as they resolved — the causal log carries the asker's unit"
    );
}
