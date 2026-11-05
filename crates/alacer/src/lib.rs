//! # Alacer
//!
//! A retained-mode Rust UI framework in which reactivity, layout, paint, and
//! accessibility consume **one shared invalidation currency** — the `Change`
//! masks of [`alacer_core::mask`] (axiom A1). This facade crate is the public
//! API surface (ADR-9): prelude, re-exports, docs.
//!
//! ```no_run
//! use alacer::prelude::*;
//!
//! let flux = Flux::new();
//! let count = flux.signal(0_i32);
//! let binding = RenderBinding::new("Clicked 0 times".to_string());
//!
//! let mut pipeline = alacer::pipeline_default();
//! let root = pipeline.mount_root(alacer_widgets::Column::new().gap(8.0).boxed());
//! let label = pipeline.add_child(
//!     root,
//!     Box::new(alacer_widgets::Label::bound(binding.clone())) as Box<dyn alacer_core::Widget>,
//! );
//! pipeline.flux_mut().bind_value(
//!     count,
//!     label,
//!     &binding,
//!     Change::LAYOUT.union(Change::PAINT),
//!     |n: &i32| format!("Clicked {n} times"),
//! );
//! ```
//!
//! The M0 dependency spine (architecture §18): winit + wgpu + reactive_graph
//! + taffy + parley + accesskit on one window; the counter and the scrolling
//! list are the canon.

#![doc = include_str!("../../../README.md")]

pub use alacer_a11y as a11y;
pub use alacer_core as core;
pub use alacer_layout as layout;
pub use alacer_platform as platform;
pub use alacer_render as render;
pub use alacer_testing as testing;
pub use alacer_text as text;
pub use alacer_view as view;
pub use alacer_widgets as widgets;

pub use alacer_core::prelude::*;

/// The public prelude: one import for the whole framework vocabulary.
pub mod prelude {
    pub use alacer_core::prelude::*;
    pub use alacer_core::{
        A11yAction, EventResponse, GlyphSource, LabelSource, MeasureFn, PaintSink, Role, Widget,
    };
}

use std::sync::Arc;

/// Assembles the default pipeline: persistent-Taffy layout over the
/// thread-local text measurer, the scene journal, and the AccessKit sink.
#[must_use]
pub fn pipeline_default() -> PassPipeline {
    alacer_core::passes::PassPipeline::new(
        alacer_core::Flux::new(),
        Box::new(alacer_layout::LayoutHost::new(Arc::new(
            alacer_text::ThreadLocalMeasurer,
        ))),
        Box::new(alacer_render::SceneJournalImpl::new()),
        Box::new(alacer_a11y::AccessKitSink::new()),
    )
}

/// Runs a mounted pipeline in a real window until it closes (M0's "on one
/// window").
///
/// # Errors
///
/// Propagates [`alacer_platform::PlatformError`] from the platform layer.
pub fn run_window(
    pipeline: PassPipeline,
    title: &str,
    size: (f32, f32),
    backend: alacer_platform::BackendPreference,
) -> Result<(), alacer_platform::PlatformError> {
    alacer_platform::AlacerApp::new(pipeline, title, size, backend).run()
}
