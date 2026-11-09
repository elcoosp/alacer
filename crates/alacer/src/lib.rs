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
//! The M0 dependency spine (architecture §18): `winit` + `wgpu` + `reactive_graph`
//! + taffy + parley + accesskit on one window; the counter and the scrolling
//! list are the canon.

#![doc = include_str!("../../../README.md")]

pub use alacer_a11y as a11y;
pub use alacer_core as core;
pub use alacer_inspect as inspect;
pub use alacer_layout as layout;
pub use alacer_macro::rsx;
pub use alacer_platform as platform;
pub use alacer_render as render;
pub use alacer_testing as testing;
pub use alacer_text as text;
pub use alacer_view as view;
pub use alacer_widgets as widgets;

pub use alacer_core::prelude::*;

/// The hot-reload runtime (§13.2, ADR-7): the view generation contract
/// wrapped in `subsecond::call` (feature `hot-reload`) plus the patch
/// channel the loop answers with one scoped re-render.
pub mod hot;

/// The public prelude: one import for the whole framework vocabulary.
pub mod prelude {
    pub use crate::rsx;
    pub use alacer_core::prelude::*;
    pub use alacer_core::{
        A11yAction, EventResponse, GlyphSource, LabelSource, MeasureFn, PaintSink, Role,
        ScrollUnit, Widget,
    };
    pub use alacer_view::{HotView, PatchReport, ViewNode, ViewRoot};
    pub use alacer_widgets::default_registry;
}

use std::rc::Rc;
use std::sync::Arc;

/// Assembles the default pipeline: persistent-Taffy layout over the
/// thread-local text measurer, the scene journal, and the AccessKit sink.
///
/// The facade is an assembly site: it installs this thread's text seam so
/// widgets (core-only dependencies) can shape and measure text (ADR-9).
#[must_use]
pub fn pipeline_default() -> PassPipeline {
    alacer_core::text_seam::install(Rc::new(alacer_text::ThreadLocalTextSeam));
    // The clipboard seam's engine rides the adopted arboard core (ADR-0020):
    // the platform owns the adapter dependency; the assembly installs it so
    // headless pipelines answer the clipboard chords with the same escape
    // semantics the window does (a refused read is an honest None).
    let clipboard = Rc::new(alacer_platform::clipboard::ArboardClipboard::new());
    alacer_core::clipboard::install(clipboard);
    alacer_core::passes::PassPipeline::new(
        alacer_core::Flux::new(),
        Box::new(alacer_layout::LayoutHost::new(Arc::new(
            alacer_text::ThreadLocalMeasurer,
        ))),
        Box::new(alacer_render::SceneJournalImpl::new()),
        Box::new(alacer_a11y::AccessKitSink::new()),
    )
}

/// Mounts a [`alacer_view::ViewNode`] tree as the pipeline's root element,
/// using the default M0 element registry (ADR-9: the concrete set registers
/// itself where the widget types live).
///
/// # Panics
///
/// Panics when `view` names an unregistered element.
#[must_use]
pub fn mount_view(
    view: &alacer_view::ViewNode,
    pipeline: &mut alacer_core::passes::PassPipeline,
) -> alacer_view::ViewRoot {
    alacer_view::ViewRoot::mount(view, pipeline, alacer_widgets::default_registry())
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
