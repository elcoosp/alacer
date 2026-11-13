//! The scrolling list: a clipped viewport whose children translate without
//! re-measuring (§8.2 item 4 — the translation-only workhorse in action).

use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::geom::Vector;
use alacer_core::inspect::{ScrollAction, ScrollOutcome, ScrollRecord, ScrollRefusal};
use alacer_core::style::Style;
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, ScrollUnit, Widget};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

/// One line of a11y scroll, in pixels — the platform's wheel convention
/// (`LineDelta` × 40 px/line, §10.2), so an assistive technology's
/// "scroll one item" lands exactly where one wheel notch does.
const A11Y_LINE: f32 = 40.0;

/// The offsets name themselves equal (M50, ADR-0056): the one float
/// equality the widget owns — [`f32::total_cmp`], the slider's
/// `same_value` discipline at the scroll tier — answering every wall
/// comparison. The wall is a computed fact at the door that knows the
/// range, never guessed from the landing by the trail's readers.
fn same_offset(a: f32, b: f32) -> bool {
    a.total_cmp(&b) == Ordering::Equal
}

/// The wall the clamp pinned a delivered ask at (M50, ADR-0056,
/// ADR-0055's second kill rule): `Some(species)` when the *ask* was
/// turned back at the range's own edge — the landing at the wall, the
/// ask somewhere else (the clamp, the wall that held a no-op against
/// a past ask) — `None` when the ask wanted the wall exactly (the
/// landing *is* the ask) and on every landing mid-range. The scroll
/// tier's counterpart to the slider's `pinned_bound` (M49): the
/// compute lives where the knowledge lives — the door that knows the
/// span — never reconstructed by the trail's readers.
fn pinned_offset_bound(landed: f32, ask: f32, span: f32) -> Option<alacer_core::RangeBound> {
    use alacer_core::RangeBound;
    if same_offset(landed, span) && !same_offset(ask, span) {
        Some(RangeBound::Max)
    } else if same_offset(landed, 0.0) && !same_offset(ask, 0.0) {
        Some(RangeBound::Min)
    } else {
        None
    }
}

/// Answers one scroll ask (M50, ADR-0056): clamps the ask to the
/// range's span, mutates the state only on a delivery, and returns the
/// record the door streams — the landed offset beside the ask, the
/// wall the clamp pinned, the honest refusals (the non-finite ask the
/// clamp arithmetic would have corrupted, the span-less range that has
/// no scroll to deliver). One helper because the three doors — the
/// wheel, the AT actions, the app's own handle — answer by the same
/// arithmetic; each notes its own action species at its own site.
fn answer_scroll_ask(
    state: &mut ScrollState,
    ask: f32,
) -> (ScrollOutcome, f32, Option<alacer_core::RangeBound>) {
    // The funnel carries only finite asks (the value tier's wall, the
    // value record's own rule): a non-finite delta rode the clamp
    // arithmetic and spread `NaN` into the state — the corruption the
    // trail's design exposed, refused before the arithmetic now.
    if !ask.is_finite() {
        return (
            ScrollOutcome::Refused(ScrollRefusal::NonFiniteAsk),
            state.offset,
            None,
        );
    }
    let span = (state.content_height - state.viewport_height).max(0.0);
    if same_offset(span, 0.0) {
        // The range has no span: no scroll exists to deliver — the
        // offset holds where it is, the ask's wall absent (a write
        // that moved nothing pinned nothing).
        return (
            ScrollOutcome::Refused(ScrollRefusal::NoScrollableContent),
            state.offset,
            None,
        );
    }
    let landed = ask.clamp(0.0, span);
    let pinned = pinned_offset_bound(landed, ask, span);
    state.offset = landed;
    (ScrollOutcome::Delivered, landed, pinned)
}

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

    fn is_scroll_container(&self) -> bool {
        true
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        if let PointerEvent::Wheel { delta, .. } = event {
            let mut state = self.state.lock().expect("scroll state");
            // Wheel deltas: positive y scrolls up; scrolling down accumulates
            // (§10.2: same-direction deltas coalesce upstream).
            let ask = state.offset - delta.y;
            let (outcome, landed, pinned) = answer_scroll_ask(&mut state, ask);
            drop(state);
            // The door's fact (M50, ADR-0056): the landed offset beside
            // the ask, the wall the clamp pinned, the refusals named —
            // the wheel route's stamp rides from the funnel.
            alacer_core::scroll::note_scroll(ScrollRecord {
                action: ScrollAction::Wheel,
                outcome,
                offset: landed,
                requested: ask.is_finite().then_some(ask),
                target: None,
                pinned,
            });
            // Translation-only: compose moves children, never layout.
            ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
            return EventResponse::Handled;
        }
        EventResponse::Ignored
    }

    fn on_a11y_action(&mut self, action: A11yAction) -> bool {
        // M10 (ADR-0016): a scroll area answers the AT vocabulary —
        // `ScrollUp`/`ScrollDown` travel by the requested unit (an item is
        // one line, a page is the viewport), and `Increment`/`Decrement`
        // step by a page, the honest value-step for a scrollbar whose value
        // *is* the offset. The pipeline merges `PAINT | A11Y` on consumption
        // (§7.2 pass 10), so bounds re-announce without a measure call.
        //
        // Since M50 (ADR-0056) every answered ask streams its fact on the
        // scroll log too — the landed offset beside the ask, the wall the
        // clamp pinned, the refusals named — while the actions outside the
        // scroll tier (the press, the value tier's writes) stay refused and
        // silent: a question outside the tier is not a tier fact (the
        // ADR-0046 rule the scroll tier rides).
        let mut state = self.state.lock().expect("scroll state");
        let (down, action_species) = match action {
            A11yAction::ScrollDown(_) => (true, ScrollAction::ScrollDown),
            A11yAction::ScrollUp(_) => (false, ScrollAction::ScrollUp),
            // The step asks (the scrollbar's value *is* the offset, M10)
            // name their own species on the record — the AT's ask, the
            // scroll tier's answer (M50, ADR-0056).
            A11yAction::Increment => (true, ScrollAction::Increment),
            A11yAction::Decrement => (false, ScrollAction::Decrement),
            // The value tier (M11, ADR-0017) is not a scroll area's
            // contract — refused, never munged into a scroll, and never
            // a scroll fact (the tier boundary the log prices).
            A11yAction::DefaultPress
            | A11yAction::SetValue(_)
            | A11yAction::ReplaceSelectedText(_)
            | A11yAction::SetTextSelection(_) => return false,
        };
        let unit = match action {
            A11yAction::ScrollDown(unit) | A11yAction::ScrollUp(unit) => unit,
            _ => ScrollUnit::Page,
        };
        let distance = match unit {
            ScrollUnit::Item => A11Y_LINE,
            ScrollUnit::Page => state.viewport_height,
        };
        let ask = if down {
            state.offset + distance
        } else {
            state.offset - distance
        };
        let (outcome, landed, pinned) = answer_scroll_ask(&mut state, ask);
        drop(state);
        alacer_core::scroll::note_scroll(ScrollRecord {
            action: action_species,
            outcome,
            offset: landed,
            requested: ask.is_finite().then_some(ask),
            target: None,
            pinned,
        });
        true
    }
}

/// Shared control over a [`List`]'s scroll offset.
#[derive(Clone)]
pub struct ListHandle {
    state: Arc<Mutex<ScrollState>>,
}

impl ListHandle {
    /// Updates the content height (call after structural changes).
    ///
    /// # Panics
    ///
    /// If the shared scroll state's lock is poisoned — a panicked writer
    /// means the frame is corrupt either way.
    pub fn set_content_height(&self, height: f32) {
        let mut state = self.state.lock().expect("scroll state");
        state.content_height = height;
        state.offset = state
            .offset
            .clamp(0.0, (height - state.viewport_height).max(0.0));
    }

    /// Programmatic scroll (tests, a11y actions).
    ///
    /// Since M50 (ADR-0056) the door streams its fact on the scroll log
    /// too — the app's own ask, the landed offset beside it, the wall
    /// the clamp pinned, the honest refusals. The door rides no route,
    /// so the record's stamp carries the honest absence (the M43 rule:
    /// the stamp fills only the identity the door cannot know — and
    /// the app's handle knows no node id at all).
    ///
    /// # Panics
    ///
    /// If the shared scroll state's lock is poisoned — a panicked writer
    /// means the frame is corrupt either way.
    pub fn scroll_to(&self, offset: f32) {
        let mut state = self.state.lock().expect("scroll state");
        let (outcome, landed, pinned) = answer_scroll_ask(&mut state, offset);
        drop(state);
        alacer_core::scroll::note_scroll(ScrollRecord {
            action: ScrollAction::ScrollTo,
            outcome,
            offset: landed,
            requested: offset.is_finite().then_some(offset),
            target: None,
            pinned,
        });
    }

    /// Current offset.
    ///
    /// # Panics
    ///
    /// If the shared scroll state's lock is poisoned — a panicked writer
    /// means the frame is corrupt either way.
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
    use alacer_core::widget::{A11yAction, EventResponse, Role, ScrollUnit, Widget};

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

    #[test]
    fn a11y_scroll_actions_travel_by_their_unit_and_clamp() {
        let mut list = List::new(100.0);
        let handle = list.handle();
        handle.set_content_height(500.0);
        let ctx = EventCtx::new();

        assert!(list.on_a11y_action(A11yAction::ScrollDown(ScrollUnit::Item)));
        assert_eq!(handle.offset(), 40.0, "one item = one line = 40 px");

        assert!(list.on_a11y_action(A11yAction::ScrollDown(ScrollUnit::Page)));
        assert_eq!(handle.offset(), 140.0, "one page = the viewport (100 px)");

        assert!(list.on_a11y_action(A11yAction::ScrollUp(ScrollUnit::Item)));
        assert_eq!(handle.offset(), 100.0);

        // Overscroll clamps at both ends, like the wheel path.
        assert!(list.on_a11y_action(A11yAction::ScrollUp(ScrollUnit::Page)));
        assert!(list.on_a11y_action(A11yAction::ScrollUp(ScrollUnit::Page)));
        assert_eq!(handle.offset(), 0.0, "the top clamps at zero");
        assert!(list.on_a11y_action(A11yAction::ScrollDown(ScrollUnit::Page)));
        assert!(list.on_a11y_action(A11yAction::ScrollDown(ScrollUnit::Page)));
        assert!(list.on_a11y_action(A11yAction::ScrollDown(ScrollUnit::Page)));
        assert!(list.on_a11y_action(A11yAction::ScrollDown(ScrollUnit::Page)));
        assert_eq!(
            handle.offset(),
            400.0,
            "the bottom clamps at content minus viewport"
        );
        let _ = ctx;
    }

    #[test]
    fn a11y_increment_and_decrement_step_by_a_page() {
        let mut list = List::new(100.0);
        let handle = list.handle();
        handle.set_content_height(500.0);

        // A scrollbar's value *is* the offset: the numeric step is a page.
        assert!(list.on_a11y_action(A11yAction::Increment));
        assert_eq!(handle.offset(), 100.0);
        assert!(list.on_a11y_action(A11yAction::Increment));
        assert_eq!(handle.offset(), 200.0);
        assert!(list.on_a11y_action(A11yAction::Decrement));
        assert_eq!(handle.offset(), 100.0);
    }

    #[test]
    fn a11y_press_is_refused_and_the_offset_never_moves() {
        let mut list = List::new(100.0);
        let handle = list.handle();
        handle.set_content_height(500.0);

        assert!(!list.on_a11y_action(A11yAction::DefaultPress));
        assert_eq!(handle.offset(), 0.0, "a press is not a scroll");
        assert_eq!(list.role(), Role::ScrollArea);
    }
}
