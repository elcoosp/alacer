//! Alacer's core: the single invalidation currency and everything that consumes it.
//!
//! Every cross-cutting consumer — layout, paint, accessibility, devtools — consumes
//! the same per-node [`Change`] masks (axiom A1 of the architecture). No subsystem
//! invents private invalidation. This crate owns:
//!
//! - [`mask::Change`] — per-node bitflags `STRUCTURE | LAYOUT | PAINT | A11Y | FOCUS`,
//!   the one currency of change.
//! - [`identity`] — id-path prefix hashing, the single identity shared by element
//!   state, hit-testing, and AccessKit node ids (architecture §6.3).
//! - [`arena`] — the retained widget tree: safe slot arena, `WidgetMut` guards that
//!   merge invalidation flags upward on drop, and lease-and-panic re-entrancy
//!   protection (architecture §7.1).
//! - [`widget`] — the `Widget` trait where `role` and `label` are *required*
//!   methods, making accessibility a type-level contract (ADR-5).
//! - [`flux`] — the mask bridge over `reactive_graph`: effects never mutate widgets;
//!   every effect drain ends by queuing `Change` masks (architecture §5.2).
//! - [`passes`] — the pass contract: mutate → style → layout → compose → paint →
//!   accessibility, each explicit, ordered, and independently traceable (§7.2).
//!
//! The layering is a DAG: this crate depends on no other Alacer crate.

pub mod arena;
pub mod color;
pub mod damage;
pub mod events;
pub mod flux;
pub mod geom;
pub mod identity;
pub mod inspect;
pub mod mask;
pub mod num;
pub mod passes;
pub mod scene;
pub mod style;
pub mod text_seam;
pub mod utf16;
pub mod widget;

pub use arena::{Arena, NodeId, WidgetMut};
pub use color::Color;
pub use damage::{DAMAGE_REGION_BUDGET, Damage, DamageBuilder};
pub use events::{
    EventPhase, ImeEvent, Key, KeyEvent, KeyState, Modifiers, MouseButton, PointerEvent,
};
pub use flux::{Flux, FluxCounters, FluxRwSignal, FluxWriter, RenderBinding};
pub use geom::{Point, Rect, Size, Vector};
pub use inspect::{
    BindingId, DirtyNode, EdgeActivation, FrameSummary, InputKind, InputRecord, InspectEvent,
    InspectObserver, PassTiming, SignalKey, SignalWrite,
};
pub use mask::Change;
pub use passes::{
    A11yDelta, A11ySink, FrameReport, ImePurpose, ImeRequest, LayoutEngine, LayoutError, PassKind,
    PassPipeline, PassRun, SceneJournal, TreeNode, TreeSnapshot,
};
pub use scene::{
    Glyph, GlyphBitmap, GlyphSource, PaintCtx, PaintSink, Primitive, Quad, SceneSnapshot,
    primitives_extent,
};
pub use style::{
    CrossAxisAlignment, Display, MainAxisAlignment, MeasureFn, MeasureSpec, Style, TextSpec,
};
pub use text_seam::{GlyphRun, PlacedGlyph, TextSeam};
pub use widget::{A11yAction, EventResponse, LabelSource, Role, Widget};

/// The Alacer prelude: import this once and the framework vocabulary is in scope.
pub mod prelude {
    pub use crate::arena::{Arena, NodeId, WidgetMut};
    pub use crate::color::Color;
    pub use crate::damage::Damage;
    pub use crate::events::{
        EventPhase, ImeEvent, Key, KeyEvent, KeyState, Modifiers, MouseButton, PointerEvent,
    };
    pub use crate::flux::{Flux, RenderBinding};
    pub use crate::geom::{Point, Rect, Size, Vector};
    pub use crate::inspect::{InspectEvent, InspectObserver};
    pub use crate::mask::Change;
    pub use crate::passes::{PassKind, PassPipeline};
    pub use crate::style::{CrossAxisAlignment, Display, MainAxisAlignment, Style};
    pub use crate::widget::{EventResponse, LabelSource, Role, Widget};
}
