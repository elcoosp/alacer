//! Input events routed by pass 1 (§10): pointer events hit-test through the
//! spatial index; keyboard and IME events route to the *focused* widget
//! (§10.3 — keyboard/IME landed with the focus passes in M3).
//!
//! The IME contract is UTF-16-native (GPUI's `EntityInputHandler` semantics):
//! every caret offset and selection range in this module and in
//! [`crate::utf16`] counts **UTF-16 code units**, so surrogate pairs and
//! platform IME protocols agree on indices without conversion bugs.

use crate::geom::{Point, Vector};
use crate::mask::Change;

/// Which phase of the capture→bubble route the event is in (§10.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventPhase {
    /// Root → target descent: ancestors observe the event first.
    Capture,
    /// Target → root ascent: the default; the target sees the event before
    /// its ancestors.
    #[default]
    Bubble,
}

/// Context handed to [`crate::widget::Widget::on_pointer_event`].
///
/// Widgets mutate their own state and *request* masks; the pipeline merges
/// them upward on the event's behalf (the A1 currency, applied at the seam).
#[derive(Debug, Clone, Default)]
pub struct EventCtx {
    requested: Change,
    phase: EventPhase,
}

impl EventCtx {
    /// An empty context in the default ([`EventPhase::Bubble`]) phase.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            requested: Change::empty(),
            phase: EventPhase::Bubble,
        }
    }

    /// An empty context in `phase` (the pass-1 router's constructor).
    #[must_use]
    pub const fn with_phase(phase: EventPhase) -> Self {
        Self {
            requested: Change::empty(),
            phase,
        }
    }

    /// The current routing phase.
    #[must_use]
    pub const fn phase(&self) -> EventPhase {
        self.phase
    }

    /// Requests `change` for the handling node.
    pub fn request(&mut self, change: Change) {
        self.requested |= change;
    }

    /// Drains the requested masks.
    #[must_use]
    pub fn take(&mut self) -> Change {
        std::mem::take(&mut self.requested)
    }
}

/// Which pointer button an event refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// Primary button.
    Left,
    /// Secondary button.
    Right,
    /// Middle button.
    Middle,
}

/// A pointer event, after platform coalescing (§10.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerEvent {
    /// A button was pressed.
    Down {
        /// Event position in window coordinates.
        position: Point,
        /// The button that went down.
        button: MouseButton,
    },
    /// A button was released.
    Up {
        /// Event position in window coordinates.
        position: Point,
        /// The button that came up.
        button: MouseButton,
    },
    /// The pointer moved (compatible consecutive moves coalesce upstream).
    Move {
        /// Event position in window coordinates.
        position: Point,
    },
    /// The pointer left the window.
    Leave,
    /// A wheel scroll; same-direction deltas accumulate upstream (§10.2).
    Wheel {
        /// Scroll delta; positive y scrolls up.
        delta: Vector,
        /// Event position in window coordinates.
        position: Point,
    },
}

impl PointerEvent {
    /// The event position, if the event carries one.
    #[must_use]
    pub fn position(self) -> Option<Point> {
        match self {
            Self::Down { position, .. }
            | Self::Up { position, .. }
            | Self::Wheel { position, .. }
            | Self::Move { position } => Some(position),
            Self::Leave => None,
        }
    }
}

/// A logical key, normalized across platforms (§10.3).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    /// A character key with its text representation already resolved by the
    /// platform (printable input; the IME-free path).
    Char(char),
    /// Enter / Return.
    Enter,
    /// Backspace.
    Backspace,
    /// Forward delete.
    Delete,
    /// Tab (focus cycling; Shift+Tab reverses).
    Tab,
    /// Space (the checkbox's keyboard toggle, M6; also text input via the
    /// character path).
    Space,
    /// Escape.
    Escape,
    /// Left arrow.
    ArrowLeft,
    /// Right arrow.
    ArrowRight,
    /// Up arrow.
    ArrowUp,
    /// Down arrow.
    ArrowDown,
    /// Home.
    Home,
    /// End.
    End,
    /// A key with no framework meaning; widgets may still consume it.
    Unknown,
}

/// Keyboard modifier state, tracked by the platform and attached to every
/// [`KeyEvent`].
///
/// Four independent flags is the honest shape: a bitflags type would add a
/// dependency and zero clarity for exactly four modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "keyboard modifier state is exactly four independent flags by domain definition"
)]
pub struct Modifiers {
    /// Shift.
    pub shift: bool,
    /// Control (Linux/Windows) or Control (macOS).
    pub control: bool,
    /// Alt / Option.
    pub alt: bool,
    /// Super / Command.
    pub meta: bool,
}

impl Modifiers {
    /// The primary modifier for platform accelerators: Command on Apple
    /// platforms, Control elsewhere (egui's quirk-table normalization).
    #[must_use]
    pub const fn primary(self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.meta
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.control
        }
    }
}

/// Whether a key went down or up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyState {
    /// The key went down. Editing acts on this state only.
    Pressed,
    /// The key came up.
    Released,
}

/// A keyboard event routed to the focused widget (§10.3).
///
/// `text` carries the platform's text synthesis for the press (the
/// `Char` path); editing widgets insert it at the caret.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyEvent {
    /// The logical key.
    pub key: Key,
    /// Down or up.
    pub state: KeyState,
    /// Modifier state at event time.
    pub modifiers: Modifiers,
    /// The platform's text for this press (printable input), if any.
    pub text: Option<String>,
}

impl KeyEvent {
    /// A character press (`state` = [`KeyState::Pressed`], no modifiers).
    #[must_use]
    pub fn char(ch: char) -> Self {
        Self {
            key: Key::Char(ch),
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: Some(ch.to_string()),
        }
    }

    /// A special-key press (no text, no modifiers).
    #[must_use]
    pub fn pressed(key: Key) -> Self {
        Self {
            key,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        }
    }

    /// Whether this is a press of `key`.
    #[must_use]
    pub fn presses(&self, key: &Key) -> bool {
        self.state == KeyState::Pressed && &self.key == key
    }
}

/// An Input Method Editor event (§10.3, the IME contract).
///
/// Semantics follow GPUI's `EntityInputHandler` / egui's platform quirk
/// table: preedit ("marked") text is *composing* input — it is displayed
/// inline (with the platform cursor offset in UTF-16 code units from the
/// preedit start) until the IME commits final text or the composition is
/// cancelled by an empty preedit.
#[derive(Debug, Clone, PartialEq)]
pub enum ImeEvent {
    /// Composition in progress: `text` is the current marked text, shown
    /// inline at the widget's caret; `cursor_utf16` is the IME's cursor
    /// offset *within the preedit*, in UTF-16 code units (`None` = the IME
    /// sent no cursor). An empty `text` cancels the composition.
    Preedit {
        /// The marked (composing) text.
        text: String,
        /// The IME cursor offset inside the preedit, in UTF-16 code units.
        cursor_utf16: Option<usize>,
    },
    /// The composition (or direct input) committed final text; the widget
    /// inserts it at the caret and clears any preedit.
    Commit {
        /// The committed text.
        text: String,
    },
}

#[cfg(test)]
mod tests {
    use super::{ImeEvent, Key, KeyEvent, KeyState, Modifiers, MouseButton, PointerEvent};
    use crate::geom::{Point, Vector};
    #[test]
    fn positions_surface_from_every_carrying_variant() {
        let down = PointerEvent::Down {
            position: Point::new(1.0, 2.0),
            button: MouseButton::Left,
        };
        assert_eq!(down.position(), Some(Point::new(1.0, 2.0)));

        let wheel = PointerEvent::Wheel {
            delta: Vector::new(0.0, 12.0),
            position: Point::new(3.0, 4.0),
        };
        assert_eq!(wheel.position(), Some(Point::new(3.0, 4.0)));

        assert_eq!(PointerEvent::Leave.position(), None);
    }

    #[test]
    fn key_event_constructors_press_without_modifiers() {
        let ch = KeyEvent::char('a');
        assert!(ch.presses(&Key::Char('a')));
        assert_eq!(ch.state, KeyState::Pressed);
        assert_eq!(ch.text.as_deref(), Some("a"));

        let enter = KeyEvent::pressed(Key::Enter);
        assert!(enter.presses(&Key::Enter));
        assert_eq!(enter.text, None);
        assert!(!enter.modifiers.shift, "no modifiers by default");
    }

    #[test]
    fn ime_events_carry_utf16_cursor_semantics() {
        let preedit = ImeEvent::Preedit {
            text: "にほんご".to_string(),
            cursor_utf16: Some(4),
        };
        assert!(matches!(
            preedit,
            ImeEvent::Preedit {
                cursor_utf16: Some(4),
                ..
            }
        ));
        let cancel = ImeEvent::Preedit {
            text: String::new(),
            cursor_utf16: None,
        };
        assert_eq!(
            cancel,
            ImeEvent::Preedit {
                text: String::new(),
                cursor_utf16: None,
            },
            "empty preedit is a cancellation"
        );
    }

    #[test]
    fn primary_modifier_normalizes_per_platform() {
        let mut mods = Modifiers {
            meta: true,
            ..Modifiers::default()
        };
        #[cfg(target_os = "macos")]
        assert!(mods.primary());
        #[cfg(not(target_os = "macos"))]
        assert!(!mods.primary(), "meta alone is not primary off macOS");
        mods.control = true;
        #[cfg(not(target_os = "macos"))]
        assert!(mods.primary());
    }
}
