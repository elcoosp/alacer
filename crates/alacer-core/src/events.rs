//! Pointer events routed by pass 1 (M0: pointer only; keyboard/IME land with the
//! focus passes in M3).

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

#[cfg(test)]
mod tests {
    use super::{MouseButton, PointerEvent};
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
}
