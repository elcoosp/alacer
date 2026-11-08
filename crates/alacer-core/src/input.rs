//! Frame-batched input (§10.2, ADR-0014): pointer storms cost one frame of
//! work, not N events.
//!
//! The coalescer is the scheduling contract's input half (ADR-2): raw
//! platform events queue here, compatible consecutive events merge, and the
//! batch routes at the start of the next [`crate::passes::PassPipeline::tick`]
//! — the same one-frame currency every signal write already obeys (axiom A1).
//! The platform layer pushes and never drains; the pipeline owns the drain.
//!
//! - Compatible consecutive `Move` events merge (latest position wins).
//! - Same-direction wheel deltas accumulate on both axes into one event.
//! - The drain produces the pointer events the event pass routes.

use crate::events::PointerEvent;
use crate::geom::{Point, Vector};

/// Coalesces raw platform pointer events into frame-routed ones.
///
/// Lives on the [`crate::passes::PassPipeline`]: the platform queues, the
/// tick drains (§10.2, ADR-0014).
#[derive(Debug, Default)]
pub struct EventCoalescer {
    pending: Vec<PointerEvent>,
    wheel_accumulated: Vector,
    wheel_position: Option<Point>,
}

impl EventCoalescer {
    /// An empty coalescer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a raw event.
    pub fn push(&mut self, event: PointerEvent) {
        match event {
            PointerEvent::Move { position } => {
                if let Some(PointerEvent::Move { position: prev }) = self.pending.last_mut() {
                    *prev = position;
                    return;
                }
                self.pending.push(PointerEvent::Move { position });
            }
            PointerEvent::Wheel { delta, position } => {
                // Same-direction deltas accumulate on both axes (§10.2); a
                // direction flip flushes the accumulated run first.
                let same_direction = self.wheel_accumulated.dot(delta) > 0.0;
                if same_direction {
                    self.wheel_accumulated += delta;
                } else {
                    self.flush_wheel();
                    self.wheel_accumulated = delta;
                }
                self.wheel_position = Some(position);
            }
            other => {
                self.flush_wheel();
                self.pending.push(other);
            }
        }
    }

    /// Drains the coalesced events (one batch per frame).
    #[must_use]
    pub fn drain(&mut self) -> Vec<PointerEvent> {
        self.flush_wheel();
        std::mem::take(&mut self.pending)
    }

    fn flush_wheel(&mut self) {
        if self.wheel_accumulated != Vector::zero() {
            let position = self.wheel_position.unwrap_or_default();
            self.pending.push(PointerEvent::Wheel {
                delta: self.wheel_accumulated,
                position,
            });
            self.wheel_accumulated = Vector::zero();
        }
    }

    /// Number of pending (un-drained) events.
    #[must_use]
    pub fn pending(&self) -> usize {
        self.pending.len() + usize::from(self.wheel_accumulated != Vector::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::EventCoalescer;
    use crate::events::{MouseButton, PointerEvent};
    use crate::geom::{Point, Vector};

    #[test]
    fn consecutive_moves_coalesce_to_the_latest() {
        let mut c = EventCoalescer::new();
        for i in 0..50 {
            c.push(PointerEvent::Move {
                position: Point::new(crate::num::i32_to_f32(i), 0.0),
            });
        }
        let drained = c.drain();
        assert_eq!(drained.len(), 1, "50 moves -> one event");
        let PointerEvent::Move { position } = drained[0] else {
            panic!("move expected");
        };
        assert_eq!(position, Point::new(49.0, 0.0), "latest position wins");
    }

    #[test]
    fn same_direction_wheel_deltas_accumulate() {
        let mut c = EventCoalescer::new();
        for _ in 0..10 {
            c.push(PointerEvent::Wheel {
                delta: Vector::new(0.0, -12.0),
                position: Point::new(5.0, 5.0),
            });
        }
        let drained = c.drain();
        assert_eq!(drained.len(), 1);
        let PointerEvent::Wheel { delta, .. } = drained[0] else {
            panic!("wheel expected");
        };
        assert_eq!(delta, Vector::new(0.0, -120.0));
    }

    #[test]
    fn horizontal_wheel_deltas_accumulate_too() {
        let mut c = EventCoalescer::new();
        for _ in 0..4 {
            c.push(PointerEvent::Wheel {
                delta: Vector::new(6.0, 0.0),
                position: Point::zero(),
            });
        }
        let drained = c.drain();
        assert_eq!(drained.len(), 1, "the x axis survives the coalescer");
        let PointerEvent::Wheel { delta, .. } = drained[0] else {
            panic!("wheel expected");
        };
        assert_eq!(delta, Vector::new(24.0, 0.0));
    }

    #[test]
    fn direction_flip_flushes_the_accumulated_run() {
        let mut c = EventCoalescer::new();
        c.push(PointerEvent::Wheel {
            delta: Vector::new(0.0, -30.0),
            position: Point::zero(),
        });
        c.push(PointerEvent::Wheel {
            delta: Vector::new(0.0, 15.0),
            position: Point::zero(),
        });
        let drained = c.drain();
        assert_eq!(drained.len(), 2, "down-run then up-run");
        let PointerEvent::Wheel { delta, .. } = drained[0] else {
            panic!("wheel expected");
        };
        assert_eq!(delta, Vector::new(0.0, -30.0));
    }

    #[test]
    fn clicks_interrupt_move_coalescing() {
        let mut c = EventCoalescer::new();
        c.push(PointerEvent::Move {
            position: Point::new(1.0, 1.0),
        });
        c.push(PointerEvent::Down {
            position: Point::new(2.0, 2.0),
            button: MouseButton::Left,
        });
        c.push(PointerEvent::Move {
            position: Point::new(3.0, 3.0),
        });
        let drained = c.drain();
        assert_eq!(
            drained.len(),
            3,
            "move, click, move — nothing merged across the click"
        );
    }
}
