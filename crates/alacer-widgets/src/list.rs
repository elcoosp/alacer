//! The scrolling list: a clipped viewport whose children translate without
//! re-measuring (§8.2 item 4 — the translation-only workhorse in action).

use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::geom::Vector;
use alacer_core::style::Style;
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};
use std::sync::{Arc, Mutex};

/// Shared scroll state between the widget and its [`ListHandle`].
#[derive(Debug, Default)]
pub struct ScrollState {
    offset: f32,
    content_height: f32,
    viewport_height: f32,
}

impl ScrollState {
    /// Current scroll offset (positive = scrolled down).
    #[must_use]
    pub fn offset(&self) -> f32 {
        self.offset
    }
}

/// A vertical scrolling list.
///
/// Children are mounted into the arena by the app; the list clips its
/// viewport and translates children by the scroll offset during compose —
/// zero measure calls per scroll tick (§8.2).
pub struct List {
    state: Arc<Mutex<ScrollState>>,
    style: Style,
}

impl List {
    /// A list occupying `height` pixels.
    #[must_use]
    pub fn new(height: f32) -> Self {
        Self {
            state: Arc::new(Mutex::new(ScrollState {
                offset: 0.0,
                content_height: 0.0,
                viewport_height: height,
            })),
            style: Style {
                clip_overflow: true,
                size: alacer_core::style::Size2 {
                    width: None,
                    height: Some(height),
                },
                ..Style::column()
            },
        }
    }

    /// The shared handle for scroll manipulation (wheels, flings, tests).
    #[must_use]
    pub fn handle(&self) -> ListHandle {
        ListHandle {
            state: Arc::clone(&self.state),
        }
    }

    /// Grow factor.
    #[must_use]
    pub fn grow(mut self, grow: f32) -> Self {
        self.style.grow = grow;
        self
    }

    /// Shared-borrow layout style access for view reconciliation (§6.1).
    #[must_use]
    pub const fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    /// Builds the boxed widget.
    #[must_use]
    pub fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Widget for List {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::ScrollArea
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("list")
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn scroll_offset(&self) -> Vector {
        let state = self.state.lock().expect("scroll state");
        Vector::new(0.0, state.offset)
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        if let PointerEvent::Wheel { delta, .. } = event {
            let mut state = self.state.lock().expect("scroll state");
            // Wheel deltas: positive y scrolls up; scrolling down accumulates
            // (§10.2: same-direction deltas coalesce upstream).
            state.offset = (state.offset - delta.y)
                .clamp(0.0, (state.content_height - state.viewport_height).max(0.0));
            drop(state);
            // Translation-only: compose moves children, never layout.
            ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
            return EventResponse::Handled;
        }
        EventResponse::Ignored
    }
}

/// Shared control over a [`List`]'s scroll offset.
#[derive(Clone)]
pub struct ListHandle {
    state: Arc<Mutex<ScrollState>>,
}

impl ListHandle {
    /// Updates the content height (call after structural changes).
    pub fn set_content_height(&self, height: f32) {
        let mut state = self.state.lock().expect("scroll state");
        state.content_height = height;
        state.offset = state
            .offset
            .clamp(0.0, (height - state.viewport_height).max(0.0));
    }

    /// Programmatic scroll (tests, a11y actions).
    pub fn scroll_to(&self, offset: f32) {
        let mut state = self.state.lock().expect("scroll state");
        state.offset = offset.clamp(0.0, (state.content_height - state.viewport_height).max(0.0));
    }

    /// Current offset.
    #[must_use]
    pub fn offset(&self) -> f32 {
        self.state.lock().expect("scroll state").offset()
    }

    /// The raw state (harness introspection).
    #[must_use]
    pub fn state(&self) -> Arc<Mutex<ScrollState>> {
        Arc::clone(&self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::{List, ScrollState};
    use alacer_core::events::{EventCtx, PointerEvent};
    use alacer_core::geom::{Point, Vector};
    use alacer_core::widget::{EventResponse, Role, Widget};

    #[test]
    fn list_clips_and_reports_scroll_area_role() {
        let list = List::new(200.0);
        assert_eq!(list.role(), Role::ScrollArea);
        let style = list.style();
        assert!(style.clip_overflow);
        assert_eq!(style.size.height, Some(200.0));
    }

    #[test]
    fn wheel_scrolls_and_clamps_to_content() {
        let mut list = List::new(100.0);
        let handle = list.handle();
        handle.set_content_height(500.0);

        let mut ctx = EventCtx::new();
        let wheel = PointerEvent::Wheel {
            delta: Vector::new(0.0, -60.0),
            position: Point::new(0.0, 0.0),
        };
        assert_eq!(
            list.on_pointer_event(&mut ctx, &wheel),
            EventResponse::Handled
        );
        assert_eq!(handle.offset(), 60.0);
        assert!(
            ctx.take().contains(alacer_core::Change::PAINT),
            "translation-only repaint requested"
        );

        // Overscroll clamps at the bottom of the content.
        let big = PointerEvent::Wheel {
            delta: Vector::new(0.0, -10_000.0),
            position: Point::new(0.0, 0.0),
        };
        let _ = list.on_pointer_event(&mut ctx, &big);
        assert_eq!(handle.offset(), 400.0, "500 content - 100 viewport");

        // Scrolling up past the top clamps at zero.
        let up = PointerEvent::Wheel {
            delta: Vector::new(0.0, 10_000.0),
            position: Point::new(0.0, 0.0),
        };
        let _ = list.on_pointer_event(&mut ctx, &up);
        assert_eq!(handle.offset(), 0.0);
    }

    #[test]
    fn scroll_to_respects_bounds() {
        let mut list = List::new(100.0);
        let handle = list.handle();
        handle.set_content_height(300.0);
        handle.scroll_to(250.0);
        assert_eq!(handle.offset(), 200.0);
        handle.scroll_to(-50.0);
        assert_eq!(handle.offset(), 0.0);
        let _ = &mut list; // list stays mutable-usable
    }

    #[test]
    fn scroll_offset_surfaces_for_compose() {
        let list = List::new(50.0);
        let handle = list.handle();
        handle.set_content_height(200.0);
        handle.scroll_to(30.0);
        assert_eq!(list.scroll_offset(), Vector::new(0.0, 30.0));
    }

    #[test]
    fn state_is_shareable() {
        let list = List::new(80.0);
        let handle = list.handle();
        let a = handle.state();
        let b = list.handle().state();
        a.lock().expect("state").offset = 12.0;
        assert_eq!(
            b.lock().expect("state").offset(),
            12.0,
            "handle and widget share state"
        );
        let _ = std::any::TypeId::of::<ScrollState>();
    }
}
