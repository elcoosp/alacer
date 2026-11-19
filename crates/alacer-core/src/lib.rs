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
//! - [`preedit`] — the preedit trail: the field's composing state
//!   reaches the causal log as one growing record per live session —
//!   the composition's inline shape named by lengths, never the
//!   payload — beside the commit's key-edit fact (ADR-0067, the IME
//!   tier's own record).
//! - [`keymap`] — the keymap seam: the app's window-level shortcuts, named
//!   by one installed [`KeymapSeam`] the dispatch funnel consults when the
//!   focused widget did not answer, the consult as data
//!   ([`keymap::KeymapConsult`]) and one [`inspect::KeyRecord`] streamed
//!   per funnel consult — the fired binding's prose name riding the
//!   record since M37 (§10.3, ADR-0041, ADR-0042, ADR-0043).
//! - [`theme`] — the runtime theme seam: the palette the widgets borrow
//!   for their un-overridden paint roles, installed as one
//!   [`ThemeSeam`] (`PassPipeline::set_theme`), resolved once per paint
//!   pass into [`scene::PaintCtx`], the swap riding `Change::PAINT` on
//!   the tree and one [`inspect::ThemeRecord`] on the causal log
//!   (§7.2 pass 8, ADR-0079).
//! - [`settings`] — the OS setting seam: the dark-mode fact the
//!   platform adapter installs as one [`SettingSeam`]
//!   (`PassPipeline::set_settings`), resolved once per paint pass into
//!   [`scene::PaintCtx`] beside the palette, the flip riding
//!   `Change::PAINT` on the tree and one [`inspect::SettingRecord`] on
//!   the causal log (§7.2 pass 8, ADR-0080); since M77 the reduced-motion
//!   and since M79 the high-contrast facts ride the same seam (one seam,
//!   one door, never a second seam per fact); since M75 the composition
//!   door rides beside it — the palette the OS's fact resolves into,
//!   installed as one [`SettingComposer`] (`PassPipeline::set_composer`,
//!   ADR-0081) and re-swapped on every flip that lands.
//! - [`flux`] — the mask bridge over `reactive_graph`: effects never mutate widgets;
//!   every effect drain ends by queuing `Change` masks (architecture §5.2).
//! - [`passes`] — the pass contract: `event` → `update_focusable` →
//!   `update_focus` → `mutate` → `style` → `layout` → `compose` → `paint` →
//!   `accessibility` → `action`, each explicit, ordered, and independently
//!   traceable (§7.2).
//!
//! The layering is a DAG: this crate depends on no other Alacer crate.

pub mod a11y_echo;
pub mod action;
pub mod announce;
pub mod arena;
pub mod clipboard;
pub mod color;
pub mod damage;
pub mod events;
pub mod flux;
pub mod frame_refusal;
pub mod geom;
pub mod gesture;
pub mod history;
pub mod identity;
pub mod input;
pub mod inspect;
pub mod key_edit;
pub mod keymap;
pub mod mask;
pub mod num;
pub mod passes;
pub mod preedit;
pub mod route_drop;
pub mod scene;
pub mod scroll;
pub mod settings;
pub mod style;
pub mod text_seam;
pub mod text_write;
pub mod theme;
pub mod time;
pub mod trail;
pub mod utf16;
pub mod value;
pub mod view_patch;
pub mod widget;

pub use a11y_echo::A11Y_ECHO_TRAIL_CAP;
pub use action::ACTION_TRAIL_CAP;
pub use announce::{ANNOUNCEMENT_NODE_HASH, Announcement, Politeness};
pub use arena::{Arena, NodeId, WidgetMut};
pub use clipboard::CLIPBOARD_TRAIL_CAP;
pub use color::Color;
pub use damage::{DAMAGE_REGION_BUDGET, Damage, DamageBuilder};
pub use events::{
    EventPhase, ImeEvent, Key, KeyEvent, KeyState, Modifiers, MouseButton, PointerEvent,
};
pub use flux::{Flux, FluxCounters, FluxRwSignal, FluxWriter, RenderBinding};
pub use frame_refusal::FRAME_REFUSAL_TRAIL_CAP;
pub use geom::{Point, Rect, Size, Vector};
pub use gesture::GESTURE_TRAIL_CAP;
pub use history::EDIT_TRAIL_CAP;
pub use input::EventCoalescer;
pub use inspect::{
    A11yEchoRecord, ActionAsk, ActionOutcome, ActionRecord, ActionRefusal, ActionRider, BindingId,
    ClipboardAction, ClipboardOutcome, ClipboardRecord, ClipboardRefusal, DirtyNode,
    EdgeActivation, FocusRecord, FocusRider, FrameRefusalRecord, FrameRefusalSpecies, FrameSummary,
    GesturePhase, GestureRecord, InputKind, InputRecord, InspectEvent, InspectObserver,
    KeyEditAction, KeyEditDriver, KeyEditOutcome, KeyEditRecord, KeyEditRefusal, KeyRecord,
    PassTiming, PreeditAction, PreeditRecord, RangeBound, RouteDropRecord, RouteDropRider,
    RouteDropSpecies, ScrollAction, ScrollOutcome, ScrollRecord, ScrollRefusal, SettingRecord,
    SignalKey, SignalWrite, Tee, TextWriteAction, TextWriteOutcome, TextWriteRecord,
    TextWriteRefusal, TextWriteSource, ThemeRecord, ValueAction, ValueOutcome, ValueRecord,
    ValueRefusal, ViewPatchRecord,
};
pub use key_edit::KEY_EDIT_TRAIL_CAP;
pub use keymap::{Chord, Keymap, KeymapConsult, KeymapSeam};
pub use mask::Change;
pub use passes::{
    A11yDelta, A11ySink, FocusLanding, FocusMoveOutcome, FocusRefusal, FrameReport, ImePurpose,
    ImeRequest, LayoutEngine, LayoutError, PassKind, PassPipeline, PassRun, SceneJournal,
    ScrollIntoViewLanding, ScrollIntoViewOutcome, ScrollIntoViewRefusal, TreeNode, TreeSnapshot,
};
pub use preedit::PREEDIT_TRAIL_CAP;
pub use route_drop::ROUTE_DROP_TRAIL_CAP;
pub use scroll::SCROLL_TRAIL_CAP;
pub use text_write::TEXT_WRITE_TRAIL_CAP;
pub use theme::THEME_TRAIL_CAP;
pub use value::VALUE_TRAIL_CAP;
pub use view_patch::VIEW_PATCH_TRAIL_CAP;
// The reactive trait seam: the framework's signal handles are
// `reactive_graph`'s, so its read/write traits ride the same re-export
// (call sites never need a direct reactive_graph dependency).
pub use reactive_graph::traits::{Get, GetUntracked, Set};
pub use scene::{
    Glyph, GlyphBitmap, GlyphSource, PaintCtx, PaintSink, Primitive, Quad, SceneSnapshot,
    primitives_extent,
};
pub use settings::SETTING_TRAIL_CAP;
pub use settings::{
    CanonComposer, CanonSettings, FactShape, KioskComposer, SettingComposer, SettingSeam,
    ShapedComposer,
};
pub use style::{
    CrossAxisAlignment, Display, MainAxisAlignment, MeasureFn, MeasureSpec, Style, TextSpec,
};
pub use text_seam::{GlyphRun, PlacedGlyph, TextSeam};
pub use theme::{CanonPalette, Palette, ThemeSeam};
pub use trail::{TRAIL_RING_CAP, TrailLog};
pub use widget::{
    A11yAction, AutoCompleteKind, BringIntoViewAsk, CurrentKind, EventResponse, HasPopupKind,
    InvalidKind, LabelSource, Role, ScrollAxisLanding, ScrollLanding, ScrollUnit, SortDirection,
    TableSlot, TextSelection, ToggleState, ValueRange, Widget,
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
        ActionRecord, ClipboardRecord, GestureRecord, HorizontalScrollFact, InspectEvent,
        InspectObserver, KeyEditOutcome, KeyEditRecord, KeyEditRefusal, KeyRecord, PreeditRecord,
        ScrollRecord, SettingRecord, Tee, TextWriteRecord, ThemeRecord, ValueRecord,
    };
    pub use crate::keymap::{Chord, Keymap, KeymapConsult, KeymapSeam};
    pub use crate::mask::Change;
    pub use crate::passes::{PassKind, PassPipeline};
    pub use crate::settings::{
        CanonComposer, CanonSettings, FactShape, KioskComposer, SettingComposer, SettingSeam,
        ShapedComposer,
    };
    pub use crate::style::{CrossAxisAlignment, Display, MainAxisAlignment, Style};
    pub use crate::theme::{CanonPalette, Palette, ThemeSeam};
    pub use crate::time::{ManualClock, MonotonicClock, TimeSeam};
    pub use crate::trail::{TRAIL_RING_CAP, TrailLog};
    pub use crate::widget::{
        BringIntoViewAsk, EventResponse, InvalidKind, LabelSource, Role, ScrollAxisLanding,
        ScrollOrientation, ScrollUnit, TableSlot, TextSelection, ToggleState, Widget,
    };
}

#[cfg(test)]
mod tests {
    use super::widget::{Role, TableSlot, ToggleState};

    #[test]
    fn checkbox_requires_a_name_like_every_input_control() {
        assert!(Role::CheckBox.requires_name());
        assert!(!Role::Label.requires_name());
        // The radio tier (M124): the member and the group it reads as
        // both carry the ADR-5 rule — an unnamed member is a bare mark,
        // an unnamed group a bare box.
        assert!(Role::Radio.requires_name());
        assert!(Role::RadioGroup.requires_name());
        // The link tier (M125): an unnamed link reads as bare styled
        // text — the "goes somewhere" the AT cannot announce.
        assert!(Role::Link.requires_name());
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
            Role::Canvas,
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
            Role::Radio,
            Role::RadioGroup,
            Role::Link,
        ];
        for role in roles {
            assert!(!role.name().is_empty(), "{role:?} has a name");
        }
    }
}
