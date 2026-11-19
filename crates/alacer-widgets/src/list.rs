//! The scrolling list: a clipped viewport whose children translate without
//! re-measuring (§8.2 item 4 — the translation-only workhorse in action).

use alacer_core::announce::Politeness;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::geom::Vector;
use alacer_core::inspect::{
    HorizontalScrollFact, ScrollAction, ScrollOutcome, ScrollRecord, ScrollRefusal,
};
use alacer_core::style::Style;
use alacer_core::widget::{
    A11yAction, BringIntoViewAsk, EventResponse, LabelSource, Role, ScrollAxisLanding,
    ScrollAxisPosition, ScrollOrientation, ScrollPosition, ScrollUnit, Widget,
};
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

/// Answers one axis's scroll ask (M50, ADR-0056; per axis since M89,
/// ADR-0093): clamps the ask to the range's span, mutates the offset
/// only on a delivery, and returns the record the door streams — the
/// landed offset beside the ask, the wall the clamp pinned, the honest
/// refusals (the non-finite ask the clamp arithmetic would have
/// corrupted, the span-less range that has no scroll to deliver). One
/// helper because the doors — the wheel, the AT actions, the app's own
/// handle, and since M89 the bring-into-view door's two halves —
/// answer by the same arithmetic; each notes its own action species at
/// its own site.
fn answer_axis_ask(
    offset: &mut f32,
    ask: f32,
    span: f32,
) -> (ScrollOutcome, f32, Option<alacer_core::RangeBound>) {
    // The funnel carries only finite asks (the value tier's wall, the
    // value record's own rule): a non-finite delta rode the clamp
    // arithmetic and spread `NaN` into the state — the corruption the
    // trail's design exposed, refused before the arithmetic now.
    if !ask.is_finite() {
        return (
            ScrollOutcome::Refused(ScrollRefusal::NonFiniteAsk),
            *offset,
            None,
        );
    }
    if same_offset(span, 0.0) {
        // The range has no span: no scroll exists to deliver — the
        // offset holds where it is, the ask's wall absent (a write
        // that moved nothing pinned nothing).
        return (
            ScrollOutcome::Refused(ScrollRefusal::NoScrollableContent),
            *offset,
            None,
        );
    }
    let landed = ask.clamp(0.0, span);
    let pinned = pinned_offset_bound(landed, ask, span);
    *offset = landed;
    (ScrollOutcome::Delivered, landed, pinned)
}

/// Answers one vertical scroll ask (the vertical half of every door):
/// the span the height story keeps, the offset the vertical fields
/// carry.
fn answer_scroll_ask(
    state: &mut ScrollState,
    ask: f32,
) -> (ScrollOutcome, f32, Option<alacer_core::RangeBound>) {
    let span = (state.content_height - state.viewport_height).max(0.0);
    answer_axis_ask(&mut state.offset, ask, span)
}

/// Answers one horizontal scroll ask (M89, ADR-0093): the span the
/// width story keeps, the offset the horizontal half carries — the
/// same arithmetic, the same walls, the axis's own declared range.
fn answer_horizontal_ask(
    state: &mut ScrollState,
    ask: f32,
) -> (ScrollOutcome, f32, Option<alacer_core::RangeBound>) {
    let span = (state.content_width - state.viewport_width).max(0.0);
    answer_axis_ask(&mut state.offset_x, ask, span)
}

/// Shared scroll state between the widget and its [`ListHandle`].
///
/// Since M89 (ADR-0093) the state keeps both axes: the vertical story
/// the list always carried (the offset, the content and viewport
/// heights), and the horizontal story beside it — the x offset, the
/// content and viewport widths the two-axis constructor and the
/// handle's width door declare. A vertical-only list keeps the
/// horizontal widths at zero, and every door's x half answers
/// `no_scrollable_content` against that empty span (the honest wall,
/// never a guessed x scroll).
#[derive(Debug, Default)]
pub struct ScrollState {
    offset: f32,
    content_height: f32,
    viewport_height: f32,
    /// The horizontal story beside the vertical (M89, ADR-0093):
    /// positive = scrolled right.
    offset_x: f32,
    content_width: f32,
    viewport_width: f32,
}

impl ScrollState {
    /// Current scroll offset (positive = scrolled down).
    #[must_use]
    pub fn offset(&self) -> f32 {
        self.offset
    }

    /// Current horizontal offset (positive = scrolled right) — the
    /// axis the two-axis doors move (M89, ADR-0093).
    #[must_use]
    pub fn offset_x(&self) -> f32 {
        self.offset_x
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
    /// The busy tier's word (M117, ADR-0121): whether the list's
    /// subtree is being updated — `false` the honest absence.
    busy: bool,
    /// The live tier's word (M120, ADR-0124): the list's declared live
    /// region — `None` the honest absence, the container that declared
    /// no region.
    live: Option<Politeness>,
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
                offset_x: 0.0,
                content_width: 0.0,
                viewport_width: 0.0,
            })),
            style: Style {
                clip_overflow: true,
                size: alacer_core::style::Size2 {
                    width: None,
                    height: Some(height),
                },
                ..Style::column()
            },
            busy: false,
            live: None,
        }
    }

    /// A list that also scrolls horizontally (M89, ADR-0093): the
    /// viewport's two axes are both declared — `width` fixed in the
    /// style and kept as the horizontal clamp's viewport half, `height`
    /// exactly [`List::new`]'s. The content's width is the app's own
    /// story, declared through the handle's `set_content_width` like
    /// the height it mirrors; until it is, every door's x half answers
    /// `no_scrollable_content` against the empty span.
    #[must_use]
    pub fn two_axis(width: f32, height: f32) -> Self {
        Self {
            state: Arc::new(Mutex::new(ScrollState {
                offset: 0.0,
                content_height: 0.0,
                viewport_height: height,
                offset_x: 0.0,
                content_width: 0.0,
                viewport_width: width,
            })),
            style: Style {
                clip_overflow: true,
                size: alacer_core::style::Size2 {
                    width: Some(width),
                    height: Some(height),
                },
                ..Style::column()
            },
            busy: false,
            live: None,
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

    /// Mounts the list's updating word (M117, ADR-0121): the subtree's
    /// fact said by the tree from the first frame — AccessKit's `Busy`
    /// property (ARIA's `aria-busy`) names the region the AT holds its
    /// speech on while the stream is still writing it, the declared
    /// counterpart of the announcement machinery the framework already
    /// owns (§11.6, ADR-0024). The word prices nothing but the word:
    /// the list's doors answer exactly as they always did — the word
    /// names the updating fact, it walls nothing, the scroll and
    /// selection routes the widget's own. The app's live door rides
    /// [`List::set_busy`].
    #[must_use]
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    /// The busy door (M117, ADR-0121): the app's updating fact moved —
    /// the word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). Report whether the
    /// word moved — the caller requests `A11Y` only when this answers
    /// `true`, so a write that lands where the word already is queues
    /// nothing (the M21 idle-frame property, at the busy tier).
    pub fn set_busy(&mut self, busy: bool) -> bool {
        if self.busy == busy {
            return false;
        }
        self.busy = busy;
        true
    }

    /// Mounts the list's declared live region (M120, ADR-0124): the
    /// region whose changes the AT speaks, said by the tree from the
    /// first frame — AccessKit's `Live` property (ARIA's `aria-live`)
    /// names the region to the AT, the politeness ([`Politeness`]) the
    /// announcement seam's own vocabulary carries, the declared
    /// counterpart of the pipeline's reserved region (§11.6, ADR-0024
    /// — the imperative half the framework owns, the declarative half
    /// the app says). The word prices nothing but the word: the list's
    /// doors answer exactly as they always did — the word names the
    /// region the AT watches, it walls nothing. The app's live door
    /// rides [`List::set_live`].
    #[must_use]
    pub fn live(mut self, live: Politeness) -> Self {
        self.live = Some(live);
        self
    }

    /// The live door (M120, ADR-0124): the app's declared region
    /// moved — the word the tree says follows it on the re-announcement
    /// the caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). `None` clears the
    /// word — the honest absence restored, the container that declared
    /// no region. Report whether the word moved — the caller requests
    /// `A11Y` only when this answers `true`, so a write that lands
    /// where the word already is queues nothing (the M21 idle-frame
    /// property, at the live tier).
    pub fn set_live(&mut self, live: Option<Politeness>) -> bool {
        if self.live == live {
            return false;
        }
        self.live = live;
        true
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
        Vector::new(state.offset_x, state.offset)
    }

    fn busy(&self) -> bool {
        self.busy
    }

    fn live(&self) -> Option<Politeness> {
        self.live
    }

    /// The declared axis (M98, ADR-0102): the constructor's own word —
    /// [`List::new`] names the y axis its only story, so the tree says
    /// `vertical`; [`List::two_axis`] declared both axes, and a
    /// two-axis scroller has no single-axis word to say — ARIA's
    /// `undefined` is the spec's own word for "both", the honest
    /// absence. The viewport half is the constructor's declaration (the
    /// handle's doors move the offsets and the content spans, never the
    /// viewport it was built with), so the answer is constructor-fixed.
    fn scroll_orientation(&self) -> Option<ScrollOrientation> {
        let state = self.state.lock().expect("scroll state");
        if state.viewport_width > 0.0 {
            None
        } else {
            Some(ScrollOrientation::Vertical)
        }
    }

    /// The resolved position and extent (M101, ADR-0105): the offsets
    /// the doors land and the walls the clamps hold, per axis — the
    /// same spans every scroll door here answers by, read at the a11y
    /// pass the currency paid for. Each axis rides its own honesty:
    /// an empty span (the content fits the viewport, or the width
    /// story was never declared) carries no position — the same wall
    /// every door answers `no_scrollable_content` with — and a
    /// container with no live axis at all answers `None`.
    fn scroll_position(&self) -> Option<ScrollPosition> {
        let state = self.state.lock().expect("scroll state");
        let vertical_span = (state.content_height - state.viewport_height).max(0.0);
        let horizontal_span = (state.content_width - state.viewport_width).max(0.0);
        let vertical = (!same_offset(vertical_span, 0.0)).then(|| ScrollAxisPosition {
            offset: state.offset,
            max: vertical_span,
        });
        let horizontal = (!same_offset(horizontal_span, 0.0)).then(|| ScrollAxisPosition {
            offset: state.offset_x,
            max: horizontal_span,
        });
        (vertical.is_some() || horizontal.is_some()).then_some(ScrollPosition {
            vertical,
            horizontal,
        })
    }

    fn is_scroll_container(&self) -> bool {
        true
    }

    /// The container's half of the bring-into-view door (M88,
    /// ADR-0092; two-axis since M89, ADR-0093): the ask arrives
    /// precomputed — the pipeline read the bounds ledger and priced
    /// the minimal offsets that land the aimed node fully inside this
    /// viewport on both axes — and the list answers each axis by the
    /// same clamp arithmetic every door here answers by
    /// (`answer_axis_ask`): the span walls, the named refusals, the
    /// state moved only on a delivery. A vertical-only list answers
    /// the x half `no_scrollable_content` against its undeclared span
    /// (the honest wall, the ask the geometry priced riding beside
    /// it). The record streams under
    /// [`ScrollAction::BringIntoView`], the stamps riding from the
    /// route — the container (`target`) and the aimed node (`aimed`)
    /// are the pipeline's facts to fill, never the widget's to guess.
    fn bring_into_view(
        &mut self,
        ask: BringIntoViewAsk,
    ) -> Option<alacer_core::widget::ScrollLanding> {
        let mut state = self.state.lock().expect("scroll state");
        let (outcome, landed, pinned) = answer_scroll_ask(&mut state, ask.vertical);
        let (x_outcome, x_landed, x_pinned) = answer_horizontal_ask(&mut state, ask.horizontal);
        drop(state);
        // The door's fact: the ask beside the landing on both axes,
        // the walls the clamps pinned, the refusals named — the
        // route's stamps fill the container and the aimed node (the
        // M43 rule at the scroll tier; the widget knows neither id).
        alacer_core::scroll::note_scroll(ScrollRecord {
            action: ScrollAction::BringIntoView,
            outcome,
            offset: landed,
            requested: ask.vertical.is_finite().then_some(ask.vertical),
            target: None,
            pinned,
            unit: None,
            lines: None,
            aimed: None,
            horizontal: Some(HorizontalScrollFact {
                outcome: x_outcome,
                offset: x_landed,
                requested: ask.horizontal.is_finite().then_some(ask.horizontal),
                pinned: x_pinned,
            }),
        });
        Some(alacer_core::widget::ScrollLanding {
            outcome,
            offset: landed,
            pinned,
            horizontal: ScrollAxisLanding {
                outcome: x_outcome,
                offset: x_landed,
                pinned: x_pinned,
            },
        })
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        if let PointerEvent::Wheel { delta, .. } = event {
            let mut state = self.state.lock().expect("scroll state");
            // Wheel deltas: positive y scrolls up; scrolling down accumulates
            // (§10.2: same-direction deltas coalesce upstream). The x half
            // beside it (M89, ADR-0093): positive x scrolls right, the
            // platform's own mapping — the same clamp arithmetic on the
            // declared horizontal span, the walls named per axis.
            let ask = state.offset - delta.y;
            let (outcome, landed, pinned) = answer_scroll_ask(&mut state, ask);
            let ask_x = state.offset_x + delta.x;
            let (x_outcome, x_landed, x_pinned) = answer_horizontal_ask(&mut state, ask_x);
            drop(state);
            // The door's fact (M50, ADR-0056): the landed offset beside
            // the ask, the wall the clamp pinned, the refusals named —
            // the wheel route's stamp rides from the funnel. The wheel
            // is a coalesced run, not a unit ask — the unit rides its
            // honest absence (M52, ADR-0058). The run's own story rides
            // beside it (M53, ADR-0059 — ADR-0058's second kill rule
            // cashed): the coalesced delta priced by the platform's
            // line convention — one line = 40 px, §10.2, the rule the
            // platform's wheel conversion prices `LineDelta` with — so
            // a notch answers 1.0 and the trackpad's pixel runs answer
            // their honest fraction, computed before the clamp
            // arithmetic (a pinned run still names the run it was: the
            // count is the ask's story, never the landing's). A
            // non-finite run prices no count: the refusal names the
            // wall, the count of a run that never was riding the
            // absence the M44 rule forbids guessing. The x run rides
            // the horizontal half beside the vertical story (M89) —
            // the line count stays the y run's own story, the vertical
            // convention the field's contract names.
            let lines = delta.y.is_finite().then(|| delta.y.abs() / A11Y_LINE);
            alacer_core::scroll::note_scroll(ScrollRecord {
                action: ScrollAction::Wheel,
                outcome,
                offset: landed,
                requested: ask.is_finite().then_some(ask),
                target: None,
                pinned,
                unit: None,
                lines,
                aimed: None,
                horizontal: Some(HorizontalScrollFact {
                    outcome: x_outcome,
                    offset: x_landed,
                    requested: ask_x.is_finite().then_some(ask_x),
                    pinned: x_pinned,
                }),
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
        // *is* the offset. Since M91 (ADR-0095) the vocabulary grew the
        // horizontal pair beside them — `ScrollLeft`/`ScrollRight` travel
        // by the same unit contract on the x axis (an item is one line, a
        // page is the viewport's *width*), consumed by the same clamp
        // arithmetic every door answers by, the record's species naming
        // the ask and the horizontal fact carrying its story beside the
        // vertical identity. The pipeline merges `PAINT | A11Y` on
        // consumption (§7.2 pass 10), so bounds re-announce without a
        // measure call.
        //
        // Since M50 (ADR-0056) every answered ask streams its fact on the
        // scroll log too — the landed offset beside the ask, the wall the
        // clamp pinned, the refusals named — while the actions outside the
        // scroll tier (the press, the value tier's writes) stay refused and
        // silent: a question outside the tier is not a tier fact (the
        // ADR-0046 rule the scroll tier rides).
        let mut state = self.state.lock().expect("scroll state");
        // The axis the ask travels: the vertical pair and the value
        // steps ride the y axis (M10), the horizontal pair the x axis
        // beside them (M91, ADR-0095) — the enum's own rule, the pair
        // gaining its consuming widget in the same cut.
        let (down, action_species, horizontal) = match action {
            A11yAction::ScrollDown(_) => (true, ScrollAction::ScrollDown, false),
            A11yAction::ScrollUp(_) => (false, ScrollAction::ScrollUp, false),
            A11yAction::ScrollRight(_) => (true, ScrollAction::ScrollRight, true),
            A11yAction::ScrollLeft(_) => (false, ScrollAction::ScrollLeft, true),
            // The step asks (the scrollbar's value *is* the offset, M10)
            // name their own species on the record — the AT's ask, the
            // scroll tier's answer (M50, ADR-0056).
            A11yAction::Increment => (true, ScrollAction::Increment, false),
            A11yAction::Decrement => (false, ScrollAction::Decrement, false),
            // The value tier (M11, ADR-0017) is not a scroll area's
            // contract — refused, never munged into a scroll, and never
            // a scroll fact (the tier boundary the log prices).
            A11yAction::DefaultPress
            | A11yAction::SetValue(_)
            | A11yAction::ReplaceSelectedText(_)
            | A11yAction::SetTextSelection(_) => return false,
        };
        // The value steps ride their page too (M54, ADR-0060 —
        // ADR-0058's first kill rule cashed): the step's page is a
        // unit-like distance a reader may want attributed the same
        // way, and the vocabulary's own contract names it — the step
        // steps a page, always (M10, ADR-0016).
        let unit = match action {
            A11yAction::ScrollDown(unit)
            | A11yAction::ScrollUp(unit)
            | A11yAction::ScrollLeft(unit)
            | A11yAction::ScrollRight(unit) => unit,
            _ => ScrollUnit::Page,
        };
        // The unit's distance: an item is one line, a page is the
        // viewport — the viewport's *width* on the x axis (M91,
        // ADR-0095), the height the y axis has always kept.
        let distance = match unit {
            ScrollUnit::Item => A11Y_LINE,
            ScrollUnit::Page => {
                if horizontal {
                    state.viewport_width
                } else {
                    state.viewport_height
                }
            }
        };
        // The horizontal asks ride their own half: the clamp answers
        // on the declared width span, the record's vertical fields
        // carry the identity answer (the axis unasked holds the offset
        // it had — the same convention the content door's width half
        // answers by), the x story riding the record's horizontal half
        // beside the species naming the ask.
        if horizontal {
            let ask = if down {
                state.offset_x + distance
            } else {
                state.offset_x - distance
            };
            let held = state.offset();
            let (x_outcome, x_landed, x_pinned) = answer_horizontal_ask(&mut state, ask);
            drop(state);
            alacer_core::scroll::note_scroll(ScrollRecord {
                action: action_species,
                outcome: ScrollOutcome::Delivered,
                offset: held,
                requested: None,
                target: None,
                pinned: None,
                unit: Some(unit),
                lines: None,
                aimed: None,
                horizontal: Some(HorizontalScrollFact {
                    outcome: x_outcome,
                    offset: x_landed,
                    requested: ask.is_finite().then_some(ask),
                    pinned: x_pinned,
                }),
            });
            return true;
        }
        let ask = if down {
            state.offset + distance
        } else {
            state.offset - distance
        };
        let (outcome, landed, pinned) = answer_scroll_ask(&mut state, ask);
        drop(state);
        // The door's fact (M50, ADR-0056), the asker's unit beside the
        // unit-scroll species (M52, ADR-0058 — the ask the distance is
        // priced against) and beside the value steps too (M54,
        // ADR-0060 — ADR-0058's first kill rule cashed: the step's
        // page is a unit-like distance, attributed the same way; the
        // count the wheel prices rides no unit, the step carries the
        // unit and no count).
        alacer_core::scroll::note_scroll(ScrollRecord {
            action: action_species,
            outcome,
            offset: landed,
            requested: ask.is_finite().then_some(ask),
            target: None,
            pinned,
            unit: if matches!(
                action_species,
                ScrollAction::ScrollUp
                    | ScrollAction::ScrollDown
                    | ScrollAction::Increment
                    | ScrollAction::Decrement
            ) {
                Some(unit)
            } else {
                None
            },
            // The wheel run's story is the wheel door's alone (M53,
            // ADR-0059): the AT's asks ride their unit's magnitude, the
            // absence honest here.
            lines: None,
            aimed: None,
            horizontal: None,
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
    /// Since M51 (ADR-0057) the door names its facts on the scroll log
    /// too — ADR-0056's third kill rule cashed ("the clamp is the
    /// content's story, not the ask's; today it rides outside the
    /// log"): a resize the clamp answers — the content shrank under the
    /// offset — streams one `ScrollAction::ContentClamp` record, the
    /// landing beside the wall it pinned (the same
    /// `pinned_offset_bound` arithmetic the ask doors answer by,
    /// spoken identically — never a second action species), no ask
    /// beside it (a resize is not an ask) and no route behind it (the
    /// stamp's honest absence, the app door's rule). A resize the
    /// offset never felt — growth, a shrink the span still covers —
    /// queues nothing (the M20 idle-frame property at the content's
    /// own door). A non-finite height refuses before the arithmetic
    /// (`ScrollRefusal::NonFiniteContent`): the debt the M50 trail's
    /// design exposed at the wheel door, still open here — a non-finite
    /// height rode the clamp and spread `NaN` into the offset *and*
    /// the stored span alike. The state keeps the height and offset it
    /// held; the refusal is a named fact on the log.
    ///
    /// # Panics
    ///
    /// If the shared scroll state's lock is poisoned — a panicked writer
    /// means the frame is corrupt either way.
    pub fn set_content_height(&self, height: f32) {
        let mut state = self.state.lock().expect("scroll state");
        // The non-finite wall (M51, ADR-0057): the same refusal the ask
        // tier names, at the content's own door — refused before the
        // arithmetic, the state keeping what it held.
        if !height.is_finite() {
            let held = state.offset;
            drop(state);
            alacer_core::scroll::note_scroll(ScrollRecord {
                action: ScrollAction::ContentClamp,
                outcome: ScrollOutcome::Refused(ScrollRefusal::NonFiniteContent),
                offset: held,
                requested: None,
                target: None,
                pinned: None,
                unit: None,
                lines: None,
                aimed: None,
                horizontal: None,
            });
            return;
        }
        state.content_height = height;
        let span = (height - state.viewport_height).max(0.0);
        let clamped = state.offset.clamp(0.0, span);
        let pinned = pinned_offset_bound(clamped, state.offset, span);
        let moved = !same_offset(clamped, state.offset);
        state.offset = clamped;
        drop(state);
        if moved {
            // The content clamp's fact (M51, ADR-0057): the landing
            // beside the wall it pinned, no ask beside it, no route
            // behind it. A resize the offset never felt queues
            // nothing.
            alacer_core::scroll::note_scroll(ScrollRecord {
                action: ScrollAction::ContentClamp,
                outcome: ScrollOutcome::Delivered,
                offset: clamped,
                requested: None,
                target: None,
                pinned,
                unit: None,
                lines: None,
                aimed: None,
                horizontal: None,
            });
        }
    }

    /// Updates the content width (call after structural changes) — the
    /// horizontal story's own content door (M89, ADR-0093), the
    /// height door's rules spoken on the x axis: a shrink the `offset_x`
    /// felt clamps it to the declared span and streams one
    /// `ScrollAction::ContentClamp` record whose horizontal half
    /// carries the landing beside the wall it pinned, the vertical
    /// fields riding the identity answer (the axis unasked holds the
    /// offset it had — the same convention the bring-into-view door's
    /// visible half answers by); a resize the offset never felt queues
    /// nothing; a non-finite width refuses on the horizontal half
    /// (`non_finite_content`), the state keeping the width and offset
    /// it held.
    ///
    /// # Panics
    ///
    /// If the shared scroll state's lock is poisoned — a panicked writer
    /// means the frame is corrupt either way.
    pub fn set_content_width(&self, width: f32) {
        let mut state = self.state.lock().expect("scroll state");
        // The non-finite wall (M51's rule on the x axis): refused
        // before the arithmetic, the state keeping what it held.
        if !width.is_finite() {
            let held_y = state.offset;
            let held_x = state.offset_x;
            drop(state);
            alacer_core::scroll::note_scroll(ScrollRecord {
                action: ScrollAction::ContentClamp,
                // The vertical half rides the identity answer: the
                // height story was not this record's ask.
                outcome: ScrollOutcome::Delivered,
                offset: held_y,
                requested: None,
                target: None,
                pinned: None,
                unit: None,
                lines: None,
                aimed: None,
                horizontal: Some(HorizontalScrollFact {
                    outcome: ScrollOutcome::Refused(ScrollRefusal::NonFiniteContent),
                    offset: held_x,
                    requested: None,
                    pinned: None,
                }),
            });
            return;
        }
        state.content_width = width;
        let span = (width - state.viewport_width).max(0.0);
        let clamped = state.offset_x.clamp(0.0, span);
        let pinned = pinned_offset_bound(clamped, state.offset_x, span);
        let moved = !same_offset(clamped, state.offset_x);
        let held_y = state.offset;
        state.offset_x = clamped;
        drop(state);
        if moved {
            // The horizontal content clamp's fact (M89, ADR-0093): the
            // landing beside the wall it pinned on the x half, the
            // vertical fields riding the identity answer, no ask
            // beside it, no route behind it. A resize the offset never
            // felt queues nothing.
            alacer_core::scroll::note_scroll(ScrollRecord {
                action: ScrollAction::ContentClamp,
                outcome: ScrollOutcome::Delivered,
                offset: held_y,
                requested: None,
                target: None,
                pinned: None,
                unit: None,
                lines: None,
                aimed: None,
                horizontal: Some(HorizontalScrollFact {
                    outcome: ScrollOutcome::Delivered,
                    offset: clamped,
                    requested: None,
                    pinned,
                }),
            });
        }
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
            unit: None,
            lines: None,
            aimed: None,
            horizontal: None,
        });
    }

    /// Programmatic horizontal scroll — the app's own two-axis handle
    /// door (M90, ADR-0094), ADR-0093's third re-open rule cashed: the
    /// x half beside [`ListHandle::scroll_to`], priced by the same
    /// clamp arithmetic every door here answers by (the declared width
    /// span, the named walls, the refusals). The record is the same
    /// named-wall record the vertical door streams — the x story
    /// riding its horizontal half (ADR-0056's axis rule) — while the
    /// vertical fields ride the identity answer: the axis unasked
    /// holds the offset it had, delivered, no ask beside it (the same
    /// convention the M89 content door's width half answers by). A
    /// vertical-only list answers `no_scrollable_content` against its
    /// undeclared span (the honest wall, never a guessed x scroll).
    /// The door rides no route, so the record's stamp carries the
    /// honest absence (the M43 rule: the app's handle knows no node id
    /// at all).
    ///
    /// # Panics
    ///
    /// If the shared scroll state's lock is poisoned — a panicked writer
    /// means the frame is corrupt either way.
    pub fn scroll_to_x(&self, offset: f32) {
        let mut state = self.state.lock().expect("scroll state");
        let held = state.offset();
        let (x_outcome, x_landed, x_pinned) = answer_horizontal_ask(&mut state, offset);
        drop(state);
        alacer_core::scroll::note_scroll(ScrollRecord {
            action: ScrollAction::ScrollTo,
            // The vertical half rides the identity answer: the height
            // story was not this record's ask.
            outcome: ScrollOutcome::Delivered,
            offset: held,
            requested: None,
            target: None,
            pinned: None,
            unit: None,
            lines: None,
            aimed: None,
            horizontal: Some(HorizontalScrollFact {
                outcome: x_outcome,
                offset: x_landed,
                requested: offset.is_finite().then_some(offset),
                pinned: x_pinned,
            }),
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

    /// Current horizontal offset (positive = scrolled right) — the
    /// axis the two-axis doors move (M89, ADR-0093).
    ///
    /// # Panics
    ///
    /// If the shared scroll state's lock is poisoned — a panicked writer
    /// means the frame is corrupt either way.
    #[must_use]
    pub fn offset_x(&self) -> f32 {
        self.state.lock().expect("scroll state").offset_x()
    }
}

#[cfg(test)]
mod tests {
    use super::List;
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
    fn lists_carry_their_live_word() {
        // M120, ADR-0124: the live tier's declared word — the region
        // whose changes the AT speaks, the politeness the announcement
        // seam's own vocabulary carries.
        use alacer_core::announce::Politeness;
        let mut list = List::new(80.0);
        assert!(Widget::live(&list).is_none(), "the absence first");
        list = list.live(Politeness::Polite);
        assert_eq!(Widget::live(&list), Some(Politeness::Polite));
        assert!(
            list.set_live(Some(Politeness::Assertive)),
            "the door reports the word moved"
        );
        assert_eq!(Widget::live(&list), Some(Politeness::Assertive));
        assert!(
            !list.set_live(Some(Politeness::Assertive)),
            "the write that lands where the word already is queues nothing"
        );
        assert!(list.set_live(None), "the door carries the word away");
        assert!(Widget::live(&list).is_none(), "the honest absence restored");
        assert!(
            !list.set_live(None),
            "clearing the absence twice queues nothing"
        );
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
        let list = List::new(100.0);
        let handle = list.handle();
        handle.set_content_height(300.0);
        handle.scroll_to(250.0);
        assert_eq!(handle.offset(), 200.0);
        handle.scroll_to(-50.0);
        assert_eq!(handle.offset(), 0.0);
    }

    #[test]
    fn scroll_to_x_respects_bounds_and_leaves_y_alone() {
        let list = List::two_axis(100.0, 100.0);
        let handle = list.handle();
        handle.set_content_height(300.0);
        handle.set_content_width(250.0);
        handle.scroll_to_x(200.0);
        assert_eq!(handle.offset_x(), 150.0, "250 content - 100 viewport");
        assert_eq!(handle.offset(), 0.0, "the y axis rides the identity");
        handle.scroll_to_x(-50.0);
        assert_eq!(handle.offset_x(), 0.0);
        // A vertical-only list answers the x ask honestly.
        let vertical = List::new(100.0);
        let vertical_handle = vertical.handle();
        vertical_handle.set_content_height(300.0);
        vertical_handle.scroll_to_x(50.0);
        assert_eq!(
            vertical_handle.offset_x(),
            0.0,
            "no declared span, no scroll"
        );
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
    fn state_is_shareable_through_the_handle() {
        // The sharing contract, read through the doors instead of the
        // raw state (the M91 sweep removed the dead `state()` accessor
        // — the doc called it harness introspection and no harness ever
        // called it): the handle's write shows through the widget's
        // compose read, one `Arc` between them.
        let list = List::new(80.0);
        let handle = list.handle();
        handle.set_content_height(200.0);
        handle.scroll_to(12.0);
        assert_eq!(
            list.scroll_offset(),
            Vector::new(0.0, 12.0),
            "handle and widget share state"
        );
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
