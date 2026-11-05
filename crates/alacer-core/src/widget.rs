//! The `Widget` trait: accessibility is a type-level contract (ADR-5).
//!
//! `role` and `label` are **required** trait methods — a widget that cannot be
//! named cannot be compiled (Masonry's required-method design). The debug-mode
//! CI walker in `alacer-testing` fails builds on unnamed input controls; the
//! contract holds by construction, not by discipline.

use crate::events::{EventCtx, ImeEvent, KeyEvent, PointerEvent};
use crate::scene::PaintCtx;
use crate::style::Style;
use std::borrow::Cow;
use std::fmt;
use std::sync::{Arc, Mutex};

/// Semantic role of a widget, mapped onto AccessKit roles by `alacer-a11y`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// No meaningful semantic; used for pure decorations.
    Decoration,
    /// A generic grouping container.
    Container,
    /// Static text.
    Label,
    /// An actionable button.
    Button,
    /// A single-line text input.
    TextInput,
    /// A scrollable region.
    ScrollArea,
    /// One row of a list.
    ListItem,
    /// A collection of list items.
    List,
}

impl Role {
    /// Roles whose accessible name is mandatory; the CI walker fails on
    /// unnamed widgets of these roles (§11.1).
    #[must_use]
    pub const fn requires_name(self) -> bool {
        matches!(self, Self::Button | Self::TextInput)
    }

    /// Human-readable role name, used by the CI walker's diagnostics.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Decoration => "decoration",
            Self::Container => "container",
            Self::Label => "label",
            Self::Button => "button",
            Self::TextInput => "text_input",
            Self::ScrollArea => "scroll_area",
            Self::ListItem => "list_item",
            Self::List => "list",
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Where a widget's accessible name comes from.
#[derive(Clone)]
pub enum LabelSource {
    /// A fixed string owned by the widget.
    Text(Cow<'static, str>),
    /// A live string shared with reactive state (e.g. a bound label).
    Computed(Arc<Mutex<String>>),
    /// A render binding — the same slot the widget paints from.
    Binding(crate::flux::RenderBinding<String>),
    /// The name is the concatenation of descendant names (containers).
    FromChildren,
}

impl fmt::Debug for LabelSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(t) => f.debug_tuple("Text").field(&t).finish(),
            Self::Computed(s) => {
                if let Ok(guard) = s.lock() {
                    f.debug_tuple("Computed").field(&guard.as_str()).finish()
                } else {
                    f.debug_tuple("Computed").field(&"<poisoned>").finish()
                }
            }
            Self::Binding(b) => f
                .debug_tuple("Binding")
                .field(&b.with(|v| v.len()))
                .finish(),
            Self::FromChildren => f.write_str("FromChildren"),
        }
    }
}

impl LabelSource {
    /// Resolves to the current accessible name.
    ///
    /// # Panics
    ///
    /// Never in release; in debug builds a poisoned lock panics (same as
    /// `.unwrap()` semantics chosen deliberately: a poisoned name cell is a
    /// widget-implementation bug).
    #[must_use]
    pub fn resolve(&self) -> String {
        match self {
            Self::Text(t) => t.to_string(),
            Self::Computed(cell) => cell.lock().map(|g| g.clone()).unwrap_or_default(),
            Self::Binding(binding) => binding.with(|v| v.clone()),
            Self::FromChildren => String::new(),
        }
    }

    /// A fixed-text source from any string-like value.
    #[must_use]
    pub fn text(t: impl Into<Cow<'static, str>>) -> Self {
        Self::Text(t.into())
    }
}

/// An accessibility action requested by an assistive technology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A11yAction {
    /// The default action — "press" for buttons.
    DefaultPress,
}

/// What happened to a routed event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventResponse {
    /// The widget did not consume the event; it bubbles on.
    #[default]
    Ignored,
    /// The widget consumed the event; bubbling stops.
    Handled,
}

/// The retained widget contract.
///
/// Widgets are never re-created on state change (§6.1): they are created once,
/// mounted into the [`crate::arena::Arena`], and mutated in place. Reactive
/// updates land through `alacer-core::flux::RenderBinding`s, not through this
/// trait.
pub trait Widget: 'static {
    /// The widget's semantic role. Required: the a11y contract starts here.
    fn role(&self) -> Role;

    /// The widget's accessible name source. Required: see ADR-5.
    fn label(&self) -> LabelSource;

    /// Layout styles. Paint properties stay on the widget (§7.2 pass 5).
    fn style(&self) -> Style {
        Style::default()
    }

    /// Emits this widget's own primitives (background, text). Children are
    /// painted by the pass system walking the tree; containers override only
    /// for decorations.
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let _ = ctx;
    }

    /// Scroll offset this widget applies to its children during compose
    /// (translation-only updates; §8.2). Non-scrolling widgets return zero.
    fn scroll_offset(&self) -> crate::geom::Vector {
        crate::geom::Vector::zero()
    }

    /// Whether this widget's children translate with [`Self::scroll_offset`].
    ///
    /// The arena indexes scroll containers so the compose pass visits only
    /// real candidates instead of walking every node each frame.
    fn is_scroll_container(&self) -> bool {
        false
    }

    /// The widget's intrinsic measure request, consumed by the layout engine's
    /// measure functions.
    fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
        None
    }

    /// Handles a routed pointer event targeting this widget.
    ///
    /// Mutate your own state here and request masks through `ctx` — never
    /// touch other widgets (the pass system owns the tree).
    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        let _ = (ctx, event);
        EventResponse::Ignored
    }

    /// Handles a key event routed to this widget because it holds focus
    /// (§10.3 — keyboard input is focus-routed, never hit-tested). Editing
    /// widgets act on [`KeyState::Pressed`] only; returning [`EventResponse::
    /// Ignored`] lets the framework take the key (Tab's focus cycling).
    fn on_key_event(&mut self, ctx: &mut EventCtx, event: &KeyEvent) -> EventResponse {
        let _ = (ctx, event);
        EventResponse::Ignored
    }

    /// Handles an IME event routed to this widget because it holds focus
    /// (§10.3, the IME contract's event half). Preedit is *composing* input:
    /// display it inline (the platform cursor is a UTF-16 offset within the
    /// preedit); an empty preedit cancels; [`ImeEvent::Commit`] inserts final
    /// text at the caret and clears any preedit.
    fn on_ime_event(&mut self, ctx: &mut EventCtx, event: &ImeEvent) -> EventResponse {
        let _ = (ctx, event);
        EventResponse::Ignored
    }

    /// Notifies the widget that keyboard focus moved to it (`true`) or away
    /// (`false`) — the focus-ring hook (§7.2 pass 3). Widgets store the flag
    /// and repaint on their next look; the framework already merges the
    /// PAINT mask on both endpoints of the move.
    fn set_focused(&mut self, focused: bool) {
        let _ = focused;
    }

    /// The IME flavor this widget wants while focused (§10.3). [`ImePurpose::
    /// Off`] (the default) tells the platform no IME is needed; the pipeline
    /// derives the frame's [`crate::passes::ImeRequest`] from the focused
    /// widget's answer.
    fn ime_purpose(&self) -> crate::passes::ImePurpose {
        crate::passes::ImePurpose::Off
    }

    /// The widget's exposed value for the a11y tree (M3 contract): editable
    /// widgets report their content so screen readers read the text, not just
    /// the name. Valueless roles return `None`.
    fn value(&self) -> Option<String> {
        None
    }

    /// Handles an accessibility action targeting this widget. Returns `true`
    /// when consumed. Making widgets *actionable* completes the contract in
    /// §11.5: reachable, named, and actionable.
    fn on_a11y_action(&mut self, action: A11yAction) -> bool {
        let _ = action;
        false
    }

    /// Whether this widget accepts direct pointer hits (scroll containers do;
    /// decorations do not).
    fn accepts_pointer(&self) -> bool {
        true
    }

    /// Whether this widget can hold keyboard focus (§7.2 pass 2 recomputes
    /// the candidates from this predicate after structure changes).
    ///
    /// The default derives from the semantic role: actionable inputs are
    /// focusable. Override for custom controls.
    fn focusable(&self) -> bool {
        matches!(self.role(), Role::Button | Role::TextInput)
    }

    /// The `Any` escape hatch for in-place patching (§6.1: view diffs mutate
    /// retained widgets in place; concrete patchers downcast through this,
    /// the same discipline Masonry's `WidgetMut` machinery uses). Implement
    /// as `{ self }`.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
