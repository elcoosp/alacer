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
pub mod clipboard;
pub mod color;
pub mod damage;
pub mod events;
pub mod flux;
pub mod geom;
pub mod identity;
pub mod input;
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
// The reactive trait seam: the framework's signal handles are
// `reactive_graph`'s, so its read/write traits ride the same re-export
// (call sites never need a direct reactive_graph dependency).
pub use geom::{Point, Rect, Size, Vector};
pub use input::EventCoalescer;
pub use inspect::{
    BindingId, DirtyNode, EdgeActivation, FrameSummary, InputKind, InputRecord, InspectEvent,
    InspectObserver, PassTiming, SignalKey, SignalWrite,
};
pub use mask::Change;
pub use passes::{
    A11yDelta, A11ySink, FrameReport, ImePurpose, ImeRequest, LayoutEngine, LayoutError, PassKind,
    PassPipeline, PassRun, SceneJournal, TreeNode, TreeSnapshot,
};
pub use reactive_graph::traits::{Get, GetUntracked, Set};
pub use scene::{
    Glyph, GlyphBitmap, GlyphSource, PaintCtx, PaintSink, Primitive, Quad, SceneSnapshot,
    primitives_extent,
};
pub use style::{
    CrossAxisAlignment, Display, MainAxisAlignment, MeasureFn, MeasureSpec, Style, TextSpec,
};
pub use text_seam::{GlyphRun, PlacedGlyph, TextSeam};
pub use widget::{
    A11yAction, EventResponse, LabelSource, Role, ScrollUnit, TableSlot, TextSelection,
    ToggleState, Widget,
};

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
    pub use crate::widget::{
        EventResponse, LabelSource, Role, TableSlot, TextSelection, ToggleState, Widget,
    };
}

#[cfg(test)]
mod tests {
    use super::widget::{Role, TableSlot, ToggleState};

    #[test]
    fn checkbox_requires_a_name_like_every_input_control() {
        assert!(Role::CheckBox.requires_name());
        assert!(!Role::Label.requires_name());
    }

    #[test]
    fn toggle_state_maps_from_bool() {
        assert_eq!(ToggleState::from(true), ToggleState::On);
        assert_eq!(ToggleState::from(false), ToggleState::Off);
    }

    #[test]
    fn table_slot_variants_carry_their_geometry() {
        let table = TableSlot::Table {
            rows: 3,
            columns: 4,
        };
        assert_eq!(
            table,
            TableSlot::Table {
                rows: 3,
                columns: 4
            }
        );
        let header = TableSlot::Header { column: 1 };
        assert_ne!(header, TableSlot::Header { column: 2 });
        let cell = TableSlot::Cell { row: 2, column: 3 };
        assert_ne!(cell, TableSlot::Cell { row: 3, column: 2 });
    }

    #[test]
    fn role_names_cover_every_variant() {
        let roles = [
            Role::Decoration,
            Role::Container,
            Role::Label,
            Role::Button,
            Role::TextInput,
            Role::ScrollArea,
            Role::ListItem,
            Role::List,
            Role::CheckBox,
            Role::Table,
            Role::TableRow,
            Role::TableCell,
            Role::ColumnHeader,
        ];
        for role in roles {
            assert!(!role.name().is_empty(), "{role:?} has a name");
        }
    }
}
