//! The pointer cursor seam as a consumer's vocabulary (§10.1, ADR-0156):
//! what the widget tier declares, the pipeline resolves, and the platform
//! applies. One column carries the whole story — a link that hovers as
//! the pointer, a text field as the text caret, a plain label that
//! declares nothing (the honest default), and a custom grab handle that
//! names its own shape through the [`Widget::cursor_style`] hook.
//!
//! The stock widgets' words come with the crates — `Link` overrides the
//! hook once, in `alacer-widgets`, and every consumer inherits it. The
//! platform half rides the frame: the hover resolves through
//! [`PassPipeline::cursor_at`] and the window hears only the deltas —
//! the applied shape traces at the `alacer::platform` target, so a
//! subscriber names every move:
//!
//! ```sh
//! RUST_LOG=alacer=debug cargo run -p alacer --example cursor_styles
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU
//! machines).
//!
//! Run with `cargo run -p alacer --example cursor_styles`.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer_core::cursor::CursorStyle;
use alacer_core::geom::Rect;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{Size2, Style};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_widgets::{BoxedWidget as _, Column, Label, Link, TextInput};

/// The grab handle: a canvas-tier control that names its own hover shape
/// — the open hand a draggable face shows while a drag can arm (the
/// M152 hook is the widget's door, not a registry of blessed controls).
struct GrabHandle {
    name: String,
}

impl GrabHandle {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Widget for GrabHandle {
    fn role(&self) -> Role {
        Role::Canvas
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    fn style(&self) -> Style {
        Style {
            size: Size2 {
                width: Some(180.0),
                height: Some(48.0),
            },
            ..Style::default()
        }
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        ctx.push_quad(Quad {
            rect: Rect::new(alacer_core::geom::Point::zero(), ctx.size()),
            color: ctx.palette().accent,
            corner_radius: 8.0,
            border_color: alacer_core::Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }

    fn cursor_style(&self) -> CursorStyle {
        CursorStyle::Grab
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("Move the pointer — the hover names the face it rests on.").boxed(),
    );
    let _ = pipeline.add_child(root, Link::new("Documentation").boxed());
    let _ = pipeline.add_child(root, TextInput::new("Search").boxed());
    let _ = pipeline.add_child(root, GrabHandle::new("Handle").boxed());

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer cursor styles",
        (420.0, 260.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}
