//! The M8 coalescer as a consumer's vocabulary (§10.2, ADR-0014, ADR-0023):
//! the frame-batched input queue demonstrated from the consumer side, the
//! way `undo_redo.rs` demonstrates the journal. A `StormPanel` — a custom
//! `Widget`, so `role()` and `label()` are required of it, the same
//! a11y-by-construction contract a shipped widget obeys — records every
//! pointer event the pipeline routes to it, and the status line reads the
//! panel's log on demand.
//!
//! Run with `cargo run -p alacer --example input_storm`. Wave the pointer
//! across the panel: winit delivers every intermediate position, the
//! pipeline's queue merges the frame's worth (compatible moves collapse,
//! ADR-0014), and the log grows at most one entry per frame — then click
//! *Read the log* and watch the count crawl while your hand traveled
//! hundreds of pixels of raw events. *Reset the log* empties it; the
//! count is the batching contract made visible. The observability rides
//! the workspace's idiom: `tracing` spans at the `alacer::input` target,
//! the routed count as a field.
//!
//! The deterministic storm — fifty queued moves routing as one event —
//! is pinned by the m1 corpus's `input_storm` scenario and the m8 exit
//! suite; this window is the same queue fed by a real pointer.
//! `ALACER_SOFTWARE=1` forces the softbuffer path.

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
struct StormLog {
    routed: Vec<Point>,
}

impl StormLog {
    /// The read the status line mirrors: the routed count and, when the
    /// pointer has crossed the panel, the last routed position.
    fn read(&self) -> String {
        match self.routed.last() {
            Some(last) => format!(
                "panel routed {count} moves · last at ({x:.0}, {y:.0})",
                count = self.routed.len(),
                x = last.x,
                y = last.y,
            ),
            None => "no moves routed yet — wave the pointer across the panel".to_string(),
        }
    }
}

/// The probe widget: a fixed panel that records every routed pointer
/// event. Painting one quad keeps the demonstration honest — the panel is
/// where the pointer goes, not an invisible hit target.
struct StormPanel {
    width: f32,
    height: f32,
    log: Rc<RefCell<StormLog>>,
}

impl Widget for StormPanel {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Container
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("storm panel")
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
            color: Color::rgb(61, 90, 128),
            corner_radius: 4.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        if let PointerEvent::Move { position } = event {
            self.log.borrow_mut().routed.push(*position);
        }
        EventResponse::Handled
    }
}

/// One read route: the span names it, the log answers, the batch is the
/// one write batch, and the mirror is the only bridge back to the window.
fn read_route(log: &Rc<RefCell<StormLog>>, status_line: &FluxRwSignal<String>) {
    let line = log.borrow().read();
    let _route = tracing::trace_span!(
        target: "alacer::input",
        "storm",
        route = "read",
        routed = log.borrow().routed.len(),
    )
    .entered();
    Flux::batch_of(|| status_line.set(line));
}

/// The reset route: the consumer's own state, emptied — the pipeline's
/// queue is untouched, only the log is.
fn reset_route(log: &Rc<RefCell<StormLog>>, status_line: &FluxRwSignal<String>) {
    let _route = tracing::trace_span!(target: "alacer::input", "storm", route = "reset").entered();
    log.borrow_mut().routed.clear();
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

    let log = Rc::new(RefCell::new(StormLog::default()));
    // The read the status line mirrors: handlers write this signal, the
    // binding maps it to the label — the only bridge (axiom A1).
    let status_signal = pipeline.flux_mut().signal(log.borrow().read());

    let _ = pipeline.add_child(root, Label::new("The input storm (ADR-0014)").boxed());
    let _ = pipeline.add_child(
        root,
        Box::new(StormPanel {
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
        "Alacer input storm",
        (340.0, 300.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
