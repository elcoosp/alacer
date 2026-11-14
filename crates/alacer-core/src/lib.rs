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
//! - [`announce`] — the announcement seam: the app's reserved live-region
//!   node, spoken by assistive technologies through the same snapshot and
//!   delta as every other a11y fact (§11.6, ADR-0024).
//! - [`gesture`] — the gesture trail: the value tier's drag reaches the
//!   causal log as one per-gesture record per phase — the beginning, the
//!   end — never per move (ADR-0052, the flood rule the kill criteria
//!   pre-registered).
//! - [`keymap`] — the keymap seam: the app's window-level shortcuts, named
//!   by one installed [`KeymapSeam`] the dispatch funnel consults when the
//!   focused widget did not answer, the consult as data
//!   ([`keymap::KeymapConsult`]) and one [`inspect::KeyRecord`] streamed
//!   per funnel consult — the fired binding's prose name riding the
//!   record since M37 (§10.3, ADR-0041, ADR-0042, ADR-0043).
//! - [`flux`] — the mask bridge over `reactive_graph`: effects never mutate widgets;
//!   every effect drain ends by queuing `Change` masks (architecture §5.2).
//! - [`passes`] — the pass contract: `event` → `update_focusable` →
//!   `update_focus` → `mutate` → `style` → `layout` → `compose` → `paint` →
//!   `accessibility` → `action`, each explicit, ordered, and independently
//!   traceable (§7.2).
//!
//! The layering is a DAG: this crate depends on no other Alacer crate.

pub mod announce;
pub mod arena;
pub mod clipboard;
pub mod color;
pub mod damage;
pub mod events;
pub mod flux;
pub mod geom;
pub mod gesture;
pub mod history;
pub mod identity;
pub mod input;
pub mod inspect;
pub mod keymap;
pub mod mask;
pub mod num;
pub mod passes;
pub mod route_drop;
pub mod scene;
pub mod scroll;
pub mod style;
pub mod text_seam;
pub mod text_write;
pub mod utf16;
pub mod value;
pub mod widget;

pub use announce::{ANNOUNCEMENT_NODE_HASH, Announcement, Politeness};
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
    BindingId, ClipboardAction, ClipboardOutcome, ClipboardRecord, ClipboardRefusal, DirtyNode,
    EdgeActivation, FocusRecord, FocusRider, FrameSummary, GesturePhase, GestureRecord, InputKind,
    InputRecord, InspectEvent, InspectObserver, KeyRecord, PassTiming, RangeBound, RouteDropRecord,
    RouteDropSpecies, ScrollAction, ScrollOutcome, ScrollRecord, ScrollRefusal, SignalKey,
    SignalWrite, TextWriteAction, TextWriteOutcome, TextWriteRecord, TextWriteRefusal,
    TextWriteSource, ValueAction, ValueOutcome, ValueRecord, ValueRefusal,
};
pub use keymap::{Chord, Keymap, KeymapConsult, KeymapSeam};
pub use mask::Change;
pub use passes::{
    A11yDelta, A11ySink, FocusLanding, FocusMoveOutcome, FocusRefusal, FrameReport, ImePurpose,
    ImeRequest, LayoutEngine, LayoutError, PassKind, PassPipeline, PassRun, SceneJournal, TreeNode,
    TreeSnapshot,
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
    pub use crate::announce::{Politeness, announce};
    pub use crate::arena::{Arena, NodeId, WidgetMut};
    pub use crate::color::Color;
    pub use crate::damage::Damage;
    pub use crate::events::{
        EventPhase, ImeEvent, Key, KeyEvent, KeyState, Modifiers, MouseButton, PointerEvent,
    };
    pub use crate::flux::{Flux, RenderBinding};
    pub use crate::geom::{Point, Rect, Size, Vector};
    pub use crate::inspect::{
        ClipboardRecord, GestureRecord, InspectEvent, InspectObserver, KeyRecord, ScrollRecord,
        TextWriteRecord, ValueRecord,
    };
    pub use crate::keymap::{Chord, Keymap, KeymapConsult, KeymapSeam};
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
            Role::Slider,
            Role::ProgressIndicator,
            Role::Table,
            Role::TableRow,
            Role::TableCell,
            Role::ColumnHeader,
            Role::LiveRegion,
        ];
        for role in roles {
            assert!(!role.name().is_empty(), "{role:?} has a name");
        }
    }
}
