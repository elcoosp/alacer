//! The pointer capture as a consumer's vocabulary (§10.1, ADR-0025): the
//! drag that holds its target demonstrated from the consumer side, the way
//! `input_storm.rs` demonstrates the coalescer. A `DragPanel` — a custom
//! `Widget`, so `role()` and `label()` are required of it, the same
//! a11y-by-construction contract a shipped widget obeys — records every
//! pointer event the pipeline routes to it, and the status line reads the
//! panel's log on demand.
//!
//! Run with `cargo run -p alacer --example drag_capture`. Press on the
//! panel and drag *past its edge*: before M19 the events left with the
//! hit test — the panel went silent the moment the pointer crossed its
//! frame, and a release anywhere else never came back. Since the capture
//! joined the dispatch contract, the press holds its target: the log
//! keeps growing while the gesture is anywhere in the window, and the
//! release rides back to the panel. Release, then wave the pointer across
//! the panel without pressing — only the moves inside the frame arrive
//! now, the hit test's answer. The observability rides the workspace's
//! idiom: `tracing` spans at the `alacer::capture` target, the routed
//! count as a field.
//!
//! The capture contract itself is pinned by the m19 exit suite and the
//! pipeline's unit tests; this window is the same router fed by a real
//! pointer. `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, Row};
use alacer_core::Color;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::flux::{Flux, FluxRwSignal, RenderBinding};
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{MeasureSpec, Size2, Style};
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The consumer-side log the probe records into and the routes read:
/// shared `Rc<RefCell>` state, exactly what a custom widget and its
/// handlers share outside the pipeline (the m8 probe's shape).
#[derive(Default)]
struct DragLog {
    events: Vec<String>,
}

impl DragLog {
    /// The read the status line mirrors: the routed count and the last
    /// routed event.
    fn read(&self) -> String {
        match self.events.last() {
            Some(last) => format!(
                "panel routed {count} events · last: {last}",
                count = self.events.len(),
            ),
            None => "no events yet — press the panel and drag past its edge".to_string(),
        }
    }
}

/// The probe widget: a fixed panel that records every routed pointer
/// event. Painting one quad keeps the demonstration honest — the panel is
/// where the press lands, not an invisible hit target.
struct DragPanel {
    width: f32,
    height: f32,
    log: Rc<RefCell<DragLog>>,
}

impl Widget for DragPanel {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Container
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("drag panel")
    }

    fn style(&self) -> Style {
        Style {
            size: Size2 {
                width: Some(self.width),
                height: Some(self.height),
            },
            ..Style::default()
        }
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(Size::new(self.width, self.height)))
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), ctx.size()),
            color: Color::rgb(106, 76, 147),
            corner_radius: 4.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        let mut log = self.log.borrow_mut();
        match event {
            PointerEvent::Down { position, .. } => log.events.push(format!(
                "press at ({x:.0}, {y:.0})",
                x = position.x,
                y = position.y,
            )),
            PointerEvent::Move { position } => log.events.push(format!(
                "move at ({x:.0}, {y:.0})",
                x = position.x,
                y = position.y,
            )),
            PointerEvent::Up { position, .. } => log.events.push(format!(
                "release at ({x:.0}, {y:.0})",
                x = position.x,
                y = position.y,
            )),
            PointerEvent::Leave => log.events.push("leave".to_string()),
            // The wheel follows the hover, not the capture (ADR-0025): it
            // scrolls what is under the pointer, so it never lands here
            // while the gesture is elsewhere.
            PointerEvent::Wheel { .. } => {}
        }
        EventResponse::Handled
    }
}

/// One read route: the span names it, the log answers, the batch is the
/// one write batch, and the mirror is the only bridge back to the window.
fn read_route(log: &Rc<RefCell<DragLog>>, status_line: &FluxRwSignal<String>) {
    let _route = tracing::trace_span!(
        target: "alacer::capture",
        "capture",
        route = "read",
        routed = log.borrow().events.len(),
    )
    .entered();
    let line = log.borrow().read();
    Flux::batch_of(|| status_line.set(line));
}

/// The reset route: the consumer's own state, emptied — the pipeline's
/// capture is untouched, only the log is.
fn reset_route(log: &Rc<RefCell<DragLog>>, status_line: &FluxRwSignal<String>) {
    let _route = tracing::trace_span!(target: "alacer::capture", "capture", route = "reset")
        .entered();
    log.borrow_mut().events.clear();
    let line = log.borrow().read();
    Flux::batch_of(|| status_line.set(line));
}

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let log = Rc::new(RefCell::new(DragLog::default()));
    // The read the status line mirrors: handlers write this signal, the
    // binding maps it to the label — the only bridge (axiom A1).
    let status_signal = pipeline.flux_mut().signal(log.borrow().read());

    let _ = pipeline.add_child(root, Label::new("The drag capture (ADR-0025)").boxed());
    let _ = pipeline.add_child(
        root,
        Box::new(DragPanel {
            width: 280.0,
            height: 150.0,
            log: Rc::clone(&log),
        }) as Box<dyn Widget>,
    );

    let actions = pipeline.add_child(root, Row::new().gap(8.0).boxed());
    let read_click = {
        let log = Rc::clone(&log);
        move || read_route(&log, &status_signal)
    };
    let reset_click = {
        let log = Rc::clone(&log);
        move || reset_route(&log, &status_signal)
    };
    let _ = pipeline.add_child(
        actions,
        Button::new("Read the log")
            .on_click(Box::new(read_click))
            .boxed(),
    );
    let _ = pipeline.add_child(
        actions,
        Button::new("Reset the log")
            .on_click(Box::new(reset_click))
            .boxed(),
    );

    let status_binding = RenderBinding::new(log.borrow().read());
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        status_signal,
        status_label,
        &status_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |line: &String| line.clone(),
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer drag capture",
        (340.0, 320.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
