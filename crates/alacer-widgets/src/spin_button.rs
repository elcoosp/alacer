//! The spin-button tier (M128, ADR-0132): the bounded number field the
//! AT walks by arrows.
//!
//! "This writes one bounded number, discretely" was a vocabulary hole:
//! an app wiring a number field reached for a slider wearing text —
//! the wrong shape (a slider's write is the *continuous drag* the
//! pointer speaks, a spin button's write is the *discrete step* the
//! arrows speak) — or for a bare text input carrying no value words at
//! all. [`Role::SpinButton`] carries the fact AccessKit's own
//! `SpinButton` role spells, and the numbers ride the value tier's own
//! vocabulary: the value the M3 `value()` contract carries, the span
//! the M97 range word announces, the number the M99 numeric word
//! beside it — never a second value language.
//!
//! Four ways to write one bounded scalar, one funnel: the arrow keys
//! step it (the slider's own key vocabulary — the arrows the value
//! tier already speaks), Home/End jump to the range's ends (the M47
//! named species, never a munged `SetValue`), the stepper clicks step
//! it from the pointer (the discrete write a spin button is — no drag,
//! no gesture log: a click that never was a gesture streams nothing),
//! and the AccessKit value tier answers `SetValue` (parsed, then
//! snapped like every other write), `Increment`, and `Decrement`.
//! Every write clamps to the range, snaps to the step, and requests
//! the same masks — `PAINT | A11Y` — so the word repaints and the a11y
//! tree re-announces the value. The widget never writes a11y directly
//! (A1: masks are the only currency), and a write that lands where the
//! value already is queues no masks — the idle-frame property the
//! storm budget leans on.
//!
//! The value log's discipline is the slider's own (M40/M44/M45/M47):
//! every *AT* write and every *keyboard* step streams one
//! [`alacer_core::ValueRecord`] at the exact place the answer resolves
//! — the ask beside the landing, the wall a pinned landing names (the
//! M49 rule) — while the stepper *clicks* stay off the value log (the
//! pointer's facts ride the dispatch funnel's own record, the flood
//! rule at the discrete tier: one click, one frame, one funnel line).
//!
//! Since M95 (ADR-0099) the field carries the description word beside
//! its name, the honest absence when there is nothing to explain.

use std::cmp::Ordering;

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, KeyEvent, PointerEvent};
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, Widget};

/// A bounded number field with a mandatory name (ADR-5: an unnamed
/// number field reads as a bare number) and the value words the a11y
/// tree announces — the field the AT walks by arrows.
pub struct SpinButton {
    name: String,
    /// The help text the a11y tree carries beside the name (M95,
    /// ADR-0099); `None` is the honest absence.
    description: Option<String>,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    enabled: bool,
    on_change: Option<Box<dyn FnMut(f32)>>,
    focused: bool,
    /// The app's accent override; `None` borrows the theme's accent
    /// role at paint time (ADR-0079).
    accent: Option<Color>,
    /// The app's text-color override; `None` borrows the theme's text
    /// role at paint time (ADR-0079).
    text_color: Option<Color>,
    font_size: f32,
    style: Style,
}

impl SpinButton {
    /// The field band's height (the value word and its steppers).
    const FIELD_HEIGHT: f32 = 22.0;
    /// A stepper button's width.
    const STEP_WIDTH: f32 = 18.0;
    /// A stepper button's height (the band holds the stacked pair).
    const STEP_HEIGHT: f32 = 10.0;
    /// The gap between the stacked steppers.
    const STEP_GAP: f32 = 2.0;
    /// The gap between the value word and the steppers.
    const STEPPER_GAP: f32 = 6.0;
    /// The gap between the name and the field band.
    const GAP: f32 = 6.0;
    /// The field's default width.
    const WIDTH: f32 = 180.0;

    /// A spin button labeled `name`, spanning `min..=max`, starting at
    /// `min`.
    ///
    /// # Panics
    ///
    /// Panics when `min` or `max` is not finite, when `min >= max` (a
    /// degenerate range has no value to carry) — the invariants every
    /// write relies on.
    #[must_use]
    pub fn new(name: impl Into<String>, min: f32, max: f32) -> Self {
        assert!(
            min.is_finite() && max.is_finite() && min < max,
            "the spin button's range must be finite with min < max, got {min}..={max}"
        );
        Self {
            name: name.into(),
            description: None,
            min,
            max,
            step: 1.0,
            value: min,
            enabled: true,
            on_change: None,
            focused: false,
            accent: None,
            text_color: None,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
        }
    }

    /// Initial availability (M22, ADR-0028). An unavailable field paints
    /// muted, holds no focus, and refuses every write door honestly —
    /// the value the tree carries is the app's, and the AT reads the
    /// refusal before it happens.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// The description (M95, ADR-0099): the help text beside the name.
    /// Without it the field announces the honest absence.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The step every write snaps to (arrows, steppers, AT acts alike).
    ///
    /// # Panics
    ///
    /// Panics when `step` is not finite and positive — the invariant the
    /// snap arithmetic relies on.
    #[must_use]
    pub fn step(mut self, step: f32) -> Self {
        assert!(
            step.is_finite() && step > 0.0,
            "the spin button's step must be finite and positive, got {step}"
        );
        self.step = step;
        self
    }

    /// Initial value; clamped into the range and snapped to the step.
    #[must_use]
    pub fn value(mut self, value: f32) -> Self {
        self.value = Self::clamp_snap(value, self.min, self.max, self.step);
        self
    }

    /// Installs the change handler; invoked with the *new* value after
    /// every change (arrows, steppers, a11y act) — never for a write
    /// that landed where the value already was.
    #[must_use]
    pub fn on_change(mut self, handler: impl FnMut(f32) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Accent color of the steppers and the focus ring. Without it the
    /// field borrows the theme's accent role at paint time (ADR-0079).
    #[must_use]
    pub fn accent(mut self, accent: Color) -> Self {
        self.accent = Some(accent);
        self
    }

    /// Text color of the name and the value word. Without it the field
    /// borrows the theme's text role at paint time (ADR-0079).
    #[must_use]
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Layout styles.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Current value (test introspection).
    #[must_use]
    pub const fn current_value(&self) -> f32 {
        self.value
    }

    /// The availability the widget reports — the same flag the M22
    /// `enabled()` contract carries into the a11y tree (test
    /// introspection).
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// The one availability write: report whether the flag moved (the
    /// M21 idle-frame property, at the widget seam).
    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        if self.enabled == enabled {
            return false;
        }
        self.enabled = enabled;
        true
    }

    /// Sets the value in place (view reconciliation): clamped and snapped
    /// like every write, fires no handler — programmatic state, not a
    /// user act. A non-finite request is ignored.
    pub fn set_value(&mut self, value: f32) {
        if value.is_finite() {
            self.value = Self::clamp_snap(value, self.min, self.max, self.step);
        }
    }

    /// Replaces the accessible name in place (view reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// The one description write (M95, ADR-0099, view reconciliation):
    /// `None` clears to the honest absence. Report whether the text
    /// moved — the caller requests `A11Y` only when this answers
    /// `true`, so a write that lands where the text already is queues
    /// nothing (the M21 idle-frame property, at the description tier).
    pub fn set_description(&mut self, description: Option<String>) -> bool {
        if self.description == description {
            return false;
        }
        self.description = description;
        true
    }

    /// Replaces the change handler in place (view reconciliation).
    /// `None` detaches it.
    pub fn set_on_change(&mut self, handler: Option<Box<dyn FnMut(f32)>>) {
        self.on_change = handler;
    }

    /// The one float equality the widget owns: `total_cmp` ordering —
    /// `f32::eq` is a lint trap the whole workspace refuses.
    fn same_value(a: f32, b: f32) -> bool {
        a.total_cmp(&b) == Ordering::Equal
    }

    /// The one write funnel: clamp into the range, snap to the step,
    /// clamp again (the snap can land a hair past `max`). Every door —
    /// arrows, steppers, the AT — reaches the value through here.
    fn clamp_snap(value: f32, min: f32, max: f32, step: f32) -> f32 {
        let clamped = value.clamp(min, max);
        let steps = ((clamped - min) / step).round();
        (min + steps * step).clamp(min, max)
    }

    /// The one commit path: the funnel, then — when the value moved — the
    /// masks and the handler. The answer is `true` whenever the request
    /// was answerable (the write landed where the value already is is an
    /// answered request, not a refusal); `false` refuses a request the
    /// widget cannot carry (a non-finite value) — never munged.
    fn commit(&mut self, requested: f32, ctx: Option<&mut EventCtx>) -> bool {
        // The availability tier guards the funnel's single entry (M22,
        // ADR-0028): an unavailable field answers nothing — every door
        // refuses at its own edge too, so the request never gets here.
        if !self.enabled {
            return false;
        }
        if !requested.is_finite() {
            return false;
        }
        let value = Self::clamp_snap(requested, self.min, self.max, self.step);
        if !Self::same_value(value, self.value) {
            self.value = value;
            if let Some(ctx) = ctx {
                ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
            }
            if let Some(handler) = &mut self.on_change {
                handler(value);
            }
        }
        true
    }

    /// Steps by `direction` (the arrows' and the AT step tier's shared
    /// route).
    fn step_by(&mut self, direction: f32, ctx: Option<&mut EventCtx>) -> bool {
        self.commit(self.value + direction * self.step, ctx)
    }

    /// The arrows' door (M45, ADR-0051): the keyboard's step rides the
    /// same shared route the AT tier rides, and the ride streams the
    /// same [`alacer_core::inspect::ValueRecord`] the AT tier streams —
    /// the step's own action species (`Increment`/`Decrement`), the
    /// landed value, the ask beside it (M44, ADR-0050), the wall a
    /// pinned landing names (M49, ADR-0055). The dispatch funnel's key
    /// route holds the stamp while this door answers, so the record
    /// carries the focused control's id — the ride attributed like
    /// every other fact on the log.
    fn step_note(&mut self, direction: f32, ctx: &mut EventCtx) -> bool {
        let ask = self.value + direction * self.step;
        let answered = self.step_by(direction, Some(ctx));
        if answered {
            alacer_core::value::note_write(alacer_core::ValueRecord {
                action: if direction > 0.0 {
                    alacer_core::ValueAction::Increment
                } else {
                    alacer_core::ValueAction::Decrement
                },
                outcome: alacer_core::ValueOutcome::Delivered,
                value: self.value,
                requested: ask.is_finite().then_some(ask),
                target: None,
                // The step's own wall (M49, ADR-0055): an ask the range
                // turned back at its edge names the bound it pinned —
                // the arrow held at the wall, on the log.
                pinned: self.pinned_bound(ask),
            });
        }
        answered
    }

    /// The bounds' door (M47, ADR-0053): the keyboard's Home/End ride
    /// the same shared route the step tier rides, and the ride streams
    /// the jump's own named species — `JumpToMin`/`JumpToMax`, the
    /// landing beside it, the ask beside the landing (the bound *is*
    /// the ask: the funnel's clamp-and-snap never moves it). Never a
    /// munged `SetValue` — the M11 rule's own words at the value tier.
    fn jump_note(
        &mut self,
        bound: f32,
        action: alacer_core::ValueAction,
        ctx: &mut EventCtx,
    ) -> bool {
        let answered = self.commit(bound, Some(ctx));
        if answered {
            alacer_core::value::note_write(alacer_core::ValueRecord {
                action,
                outcome: alacer_core::ValueOutcome::Delivered,
                value: self.value,
                requested: bound.is_finite().then_some(bound),
                target: None,
                // The jump's ask wanted the wall exactly (M49, ADR-0055):
                // the funnel never moved it, so the pin rides the honest
                // absence — the species already names the bound it asked
                // for.
                pinned: self.pinned_bound(bound),
            });
        }
        answered
    }

    /// The bound the funnel pinned a delivered write at (M49, ADR-0055):
    /// `Some(species)` when the *ask* was turned back at the range's own
    /// edge — the landing at the wall, the ask somewhere else — `None`
    /// when the ask wanted the wall exactly (the M47 jumps) and on every
    /// landing mid-range. The one float equality the widget owns
    /// ([`Self::same_value`]) answers every edge.
    fn pinned_bound(&self, ask: f32) -> Option<alacer_core::RangeBound> {
        if Self::same_value(self.value, self.max) && !Self::same_value(ask, self.max) {
            Some(alacer_core::RangeBound::Max)
        } else if Self::same_value(self.value, self.min) && !Self::same_value(ask, self.min) {
            Some(alacer_core::RangeBound::Min)
        } else {
            None
        }
    }

    /// The steppers' band geometry in widget-local coordinates: the
    /// pair's shared left edge (the value word keeps the rest).
    fn steppers_left(width: f32) -> f32 {
        width - 2.0 * Self::STEP_WIDTH - Self::STEPPER_GAP
    }

    /// Which stepper a local point lands on: `Some(direction)` — the
    /// step's sign — inside the stacked pair (the band's bottom edge,
    /// exactly where paint draws it), `None` anywhere else.
    fn stepper_at(local: Point, width: f32, height: f32) -> Option<f32> {
        let left = Self::steppers_left(width);
        if local.x < left || local.x >= width {
            return None;
        }
        // The band rides the widget's bottom edge (paint's own math):
        // the steppers' pair fills it, the up half above the gap.
        let y = local.y - (height - Self::FIELD_HEIGHT);
        let upper = Self::STEP_HEIGHT;
        let lower = upper + Self::STEP_GAP;
        if y < 0.0 {
            None
        } else if y < upper {
            Some(1.0)
        } else if y < lower + Self::STEP_HEIGHT {
            Some(-1.0)
        } else {
            None
        }
    }

    /// Intrinsic size: the name above, the field band below — the value
    /// word and its steppers.
    fn intrinsic(name: &str, font_size: f32) -> Size {
        let text = alacer_core::text_seam::with_seam(|seam| {
            seam.measure(
                &TextSpec {
                    text: name.to_string(),
                    font_id: 0,
                    font_size,
                },
                None,
            )
        });
        Size::new(Self::WIDTH, text.height + Self::GAP + Self::FIELD_HEIGHT)
    }
}

impl Widget for SpinButton {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::SpinButton
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    /// The description tier (M95, ADR-0099): the help text beside the
    /// name, the honest absence when the field has nothing to explain.
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn style(&self) -> Style {
        Style {
            padding: 2.0,
            ..self.style.clone()
        }
    }

    /// The M3 value contract: the AT reads the bounded scalar, not just
    /// the name.
    fn value(&self) -> Option<String> {
        Some(self.value.to_string())
    }

    /// The range tier (M97, ADR-0101): the span every door — arrows,
    /// steppers, the AT's own acts — funnels through is the span the
    /// tree announces. The step rides beside the walls: the snap
    /// arithmetic is a declared fact (the `SpinButton::new`/`step()`
    /// invariant), never a guessed default. The widening `f32` → `f64`
    /// is exact.
    fn value_range(&self) -> Option<alacer_core::ValueRange> {
        Some(
            alacer_core::ValueRange::new(f64::from(self.min), f64::from(self.max))
                .with_step(f64::from(self.step)),
        )
    }

    /// The numeric tier (M99, ADR-0103): the number behind the value's
    /// spoken word — the scalar the funnel holds, `aria-valuenow` beside
    /// the `aria-valuetext` the M3 contract already carried. The same
    /// widening `f32` → `f64` is exact.
    fn numeric_value(&self) -> Option<f64> {
        Some(f64::from(self.value))
    }

    /// The availability tier (M22, ADR-0028): the flag the a11y tree
    /// carries — an unavailable field reports itself before the AT
    /// tries the write it will refuse.
    fn enabled(&self) -> bool {
        self.enabled
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let size = ctx.size();
        // The borrowed roles resolve per paint pass (ADR-0079); the
        // look still tells the truth the tree carries (M22, ADR-0028):
        // an unavailable field paints the muted roles, and the focused
        // field's steppers carry the accent ring.
        let palette = ctx.palette();
        let stepper = if self.enabled {
            self.accent.unwrap_or(palette.accent)
        } else {
            palette.border
        };
        let label_color = if self.enabled {
            self.text_color.unwrap_or(palette.text)
        } else {
            palette.text_muted
        };
        let band = size.height - Self::FIELD_HEIGHT;
        let steppers_left = Self::steppers_left(size.width);

        // The name, above the band.
        alacer_core::text_seam::with_seam(|seam| {
            let name_layout = seam.layout_glyphs(&self.name, self.font_size, None);
            for glyph in &name_layout.glyphs {
                ctx.push_glyph(alacer_core::scene::Glyph {
                    position: Point::new(glyph.x, glyph.y),
                    size: glyph.size,
                    color: label_color,
                    font_id: glyph.font_id,
                    glyph_id: glyph.glyph_id,
                    clip: None,
                });
            }

            // The value word, the field's own number — the word the
            // tree carries, painted where the user reads it.
            let word = self.value.to_string();
            let word_layout = seam.layout_glyphs(&word, self.font_size, None);
            let word_origin = Point::new(
                0.0,
                band + (Self::FIELD_HEIGHT - word_layout.height).max(0.0) * 0.5,
            );
            for glyph in &word_layout.glyphs {
                ctx.push_glyph(alacer_core::scene::Glyph {
                    position: Point::new(word_origin.x + glyph.x, word_origin.y + glyph.y),
                    size: glyph.size,
                    color: label_color,
                    font_id: glyph.font_id,
                    glyph_id: glyph.glyph_id,
                    clip: None,
                });
            }
        });

        // The stacked steppers: the discrete step's own affordance —
        // up above, down below, deterministic pixels with no glyph
        // dependency (the snapshot corpus stays font-free).
        let border = if self.focused && self.enabled {
            palette.accent_pressed
        } else {
            palette.border
        };
        let up_height = Self::STEP_HEIGHT;
        ctx.push_quad(alacer_core::scene::Quad {
            rect: Rect::new(
                Point::new(steppers_left, band),
                Size::new(Self::STEP_WIDTH, up_height),
            ),
            color: stepper,
            corner_radius: 2.0,
            border_color: border,
            border_width: if self.focused { 2.0 } else { 1.0 },
            clip: None,
        });
        ctx.push_quad(alacer_core::scene::Quad {
            rect: Rect::new(
                Point::new(steppers_left, band + up_height + Self::STEP_GAP),
                Size::new(Self::STEP_WIDTH, Self::STEP_HEIGHT),
            ),
            color: stepper,
            corner_radius: 2.0,
            border_color: border,
            border_width: if self.focused { 2.0 } else { 1.0 },
            clip: None,
        });
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(Self::intrinsic(
            &self.name,
            self.font_size,
        )))
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// The action tier (M156, ADR-0161): the value tier's three asks —
    /// the write and the two steps — said while the field is available.
    /// The word is the door's own answer, so a closed door (the M22
    /// availability tier) advertises none; the payload's merits stay the
    /// door's (a malformed value is still refused, still named).
    fn a11y_actions(&self) -> alacer_core::widget::A11yActions {
        if self.enabled() {
            alacer_core::widget::A11yActions::SET_VALUE
                | alacer_core::widget::A11yActions::INCREMENT
                | alacer_core::widget::A11yActions::DECREMENT
        } else {
            alacer_core::widget::A11yActions::empty()
        }
    }

    /// The value tier (M20, ADR-0026's contract at the field tier):
    /// `SetValue` parses then rides the one funnel, `Increment` /
    /// `Decrement` step by the step — the same masks as an arrow.
    /// `DefaultPress` is not a value control's contract — refused,
    /// never munged into a write (the M11 rule). Every value-tier
    /// action the AT asks streams one
    /// [`alacer_core::inspect::ValueRecord`] — the answer's own
    /// species, the refusal streaming too (the clipboard trail's
    /// shape, ADR-0045), the ask beside the landing (M44, ADR-0050).
    fn on_a11y_action(&mut self, action: A11yAction) -> bool {
        // The record's action vocabulary is the AT's value tier: a
        // question outside it (a press, a scroll, a text edit) is not a
        // value fact — refused silently, never munged (the M11 rule).
        let tier = match action {
            A11yAction::SetValue(_) => alacer_core::ValueAction::SetValue,
            A11yAction::Increment => alacer_core::ValueAction::Increment,
            A11yAction::Decrement => alacer_core::ValueAction::Decrement,
            A11yAction::DefaultPress
            | A11yAction::ScrollUp(_)
            | A11yAction::ScrollDown(_)
            | A11yAction::ScrollLeft(_)
            | A11yAction::ScrollRight(_)
            | A11yAction::ReplaceSelectedText(_)
            | A11yAction::SetTextSelection(_) => return false,
        };
        // The availability tier (M22, ADR-0028): an unavailable field
        // has no write door — every action is refused honestly, never
        // munged, and the refusal streams (the tree keeps the value the
        // app wrote). The ask never entered arithmetic — the honest
        // absence rides it (M44, ADR-0050).
        if !self.enabled {
            alacer_core::value::note_write(alacer_core::ValueRecord {
                action: tier,
                outcome: alacer_core::ValueOutcome::Refused(alacer_core::ValueRefusal::Unavailable),
                value: self.value,
                requested: None,
                target: None,
                pinned: None,
            });
            return false;
        }
        // The ask (M44, ADR-0050): the finite scalar the door computed
        // before the clamp-and-snap — the parse for `SetValue`, the
        // step arithmetic for the step tier.
        let mut asked: Option<f32> = None;
        let answered = match action {
            A11yAction::SetValue(value) => {
                // The parse is an if/else, not a match: the refusal is a
                // fact the tier records, not a silent arm.
                if let Ok(parsed) = value.trim().parse::<f32>() {
                    asked = Some(parsed);
                    self.commit(parsed, None)
                } else {
                    alacer_core::value::note_write(alacer_core::ValueRecord {
                        action: tier,
                        outcome: alacer_core::ValueOutcome::Refused(
                            alacer_core::ValueRefusal::InvalidPayload,
                        ),
                        value: self.value,
                        requested: None,
                        target: None,
                        pinned: None,
                    });
                    return false;
                }
            }
            A11yAction::Increment | A11yAction::Decrement => {
                let direction = if action == A11yAction::Increment {
                    1.0
                } else {
                    -1.0
                };
                let ask = self.value + direction * self.step;
                asked = ask.is_finite().then_some(ask);
                self.commit(ask, None)
            }
            // The tier match above routed the rest; unreachable here.
            _ => false,
        };
        // The funnel's own wall: the payload parsed but the funnel
        // carries only finite scalars — "inf" parses fine, and the wall
        // is its own fact (a different species than the parse's).
        if answered {
            alacer_core::value::note_write(alacer_core::ValueRecord {
                action: tier,
                outcome: alacer_core::ValueOutcome::Delivered,
                value: self.value,
                requested: asked,
                target: None,
                // The ask's own wall (M49, ADR-0055): a delivered write
                // the range turned back at its edge names the bound it
                // pinned; an ask the landing answers exactly rides the
                // honest absence.
                pinned: asked.and_then(|ask| self.pinned_bound(ask)),
            });
        } else {
            alacer_core::value::note_write(alacer_core::ValueRecord {
                action: tier,
                outcome: alacer_core::ValueOutcome::Refused(
                    alacer_core::ValueRefusal::NonFiniteValue,
                ),
                value: self.value,
                requested: None,
                target: None,
                pinned: None,
            });
        }
        answered
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        match event {
            PointerEvent::Down { .. } => {
                // The wall (M22, ADR-0028): a disabled field consumes the
                // press but arms nothing — no focus, no repaint, no
                // write.
                if !self.enabled {
                    return EventResponse::Handled;
                }
                ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::FOCUS));
                EventResponse::Handled
            }
            PointerEvent::Up { position, .. } => {
                // The stepper's discrete write: the release over the
                // stacked pair steps by the pair's own sign — one click,
                // one step, the funnel's clamp-and-snap deciding where it
                // lands. The click stays off the *value* log (the flood
                // rule at the discrete tier): the pointer's facts ride
                // the dispatch funnel's own record, and a release with
                // no stepper beneath it is the focus's answer, never a
                // write.
                if self.enabled
                    && let Some(bounds) = ctx.bounds()
                {
                    let local =
                        Point::new(position.x - bounds.min_x(), position.y - bounds.min_y());
                    if let Some(direction) =
                        Self::stepper_at(local, bounds.width(), bounds.height())
                    {
                        let _ = self.step_by(direction, Some(ctx));
                    }
                }
                EventResponse::Handled
            }
            PointerEvent::Move { .. } | PointerEvent::Wheel { .. } | PointerEvent::Leave => {
                EventResponse::Ignored
            }
        }
    }

    fn on_key_event(&mut self, ctx: &mut EventCtx, event: &KeyEvent) -> EventResponse {
        // The availability tier (M22, ADR-0028): the keyboard never
        // reaches an unavailable control — it holds no focus, and a
        // routed step key is ignored so the framework keeps it.
        if !self.enabled {
            return EventResponse::Ignored;
        }
        if event.state != alacer_core::events::KeyState::Pressed {
            return EventResponse::Ignored;
        }
        let response = match event.key {
            alacer_core::events::Key::ArrowLeft | alacer_core::events::Key::ArrowDown => {
                self.step_note(-1.0, ctx)
            }
            alacer_core::events::Key::ArrowRight | alacer_core::events::Key::ArrowUp => {
                self.step_note(1.0, ctx)
            }
            alacer_core::events::Key::Home => {
                self.jump_note(self.min, alacer_core::ValueAction::JumpToMin, ctx)
            }
            alacer_core::events::Key::End => {
                self.jump_note(self.max, alacer_core::ValueAction::JumpToMax, ctx)
            }
            _ => return EventResponse::Ignored,
        };
        if response {
            EventResponse::Handled
        } else {
            EventResponse::Ignored
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SpinButton;
    use alacer_core::events::{
        EventCtx, Key, KeyEvent, KeyState, Modifiers, MouseButton, PointerEvent,
    };
    use alacer_core::geom::{Point, Rect, Size};
    use alacer_core::mask::Change;
    use alacer_core::widget::{A11yAction, EventResponse, Role, Widget};

    /// The float equality the tests own (the widget's own rule).
    fn same(a: f32, b: f32) -> bool {
        a.total_cmp(&b) == std::cmp::Ordering::Equal
    }

    fn arrow(key: Key) -> KeyEvent {
        KeyEvent {
            key,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        }
    }

    fn click_at(x: f32, y: f32) -> (PointerEvent, PointerEvent) {
        (
            PointerEvent::Down {
                position: Point::new(x, y),
                button: MouseButton::Left,
                click_count: 1,
            },
            PointerEvent::Up {
                position: Point::new(x, y),
                button: MouseButton::Left,
            },
        )
    }

    #[test]
    fn role_name_and_value_satisfy_the_contract() {
        let field = SpinButton::new("Quantity", 1.0, 12.0);
        assert_eq!(field.role(), Role::SpinButton);
        assert_eq!(field.label().resolve(), "Quantity");
        assert!(field.focusable(), "spin buttons hold focus");
        assert!(field.role().requires_name());
        assert_eq!(Widget::value(&field), Some("1".to_string()));
        assert_eq!(field.table_slot(), None);
        assert_eq!(
            Widget::description(&field),
            None,
            "an undescribed field announces the absence"
        );
    }

    #[test]
    fn the_value_words_ride_the_tree() {
        // M3/M97/M99 at the field tier: the spoken word, the number
        // beside it, the span and the step the AT reads before it asks.
        let field = SpinButton::new("Quantity", 1.0, 12.0).step(2.0).value(5.0);
        assert_eq!(Widget::value(&field), Some("5".to_string()));
        assert_eq!(Widget::numeric_value(&field), Some(5.0));
        let range = Widget::value_range(&field).expect("the span rides");
        assert_eq!(range.min, 1.0);
        assert_eq!(range.max, 12.0);
        assert_eq!(range.step, Some(2.0));
    }

    #[test]
    fn the_description_word_rides_the_builder() {
        let mut field = SpinButton::new("Quantity", 1.0, 12.0);
        field = field.description("How many, one to twelve");
        assert_eq!(
            Widget::description(&field),
            Some("How many, one to twelve"),
            "the help text the tree carries beside the name"
        );
        assert!(
            !field.set_description(Some("How many, one to twelve".to_string())),
            "the idle write answers false — no masks"
        );
        assert!(field.set_description(None), "clearing to the absence moved");
    }

    #[test]
    fn arrows_step_and_home_end_jump() {
        let mut field = SpinButton::new("Quantity", 1.0, 12.0).step(2.0).value(5.0);
        let mut ctx = EventCtx::new();
        assert_eq!(
            field.on_key_event(&mut ctx, &arrow(Key::ArrowUp)),
            EventResponse::Handled
        );
        assert!(same(field.current_value(), 7.0));
        assert!(ctx.take().contains(Change::A11Y));

        let mut ctx = EventCtx::new();
        let _ = field.on_key_event(&mut ctx, &arrow(Key::ArrowDown));
        assert!(same(field.current_value(), 5.0), "down mirrors up");

        let mut ctx = EventCtx::new();
        let _ = field.on_key_event(&mut ctx, &arrow(Key::End));
        assert!(same(field.current_value(), 12.0), "End jumps to the max");
        let mut ctx = EventCtx::new();
        let _ = field.on_key_event(&mut ctx, &arrow(Key::Home));
        assert!(same(field.current_value(), 1.0), "Home jumps to the min");

        let mut ctx = EventCtx::new();
        assert_eq!(
            field.on_key_event(&mut ctx, &arrow(Key::Enter)),
            EventResponse::Ignored,
            "a key with no value meaning is handed on"
        );
    }

    #[test]
    fn a_step_at_the_wall_is_the_idle_frame() {
        // The arrow held at the wall: the ask is answered, the funnel
        // lands where the value already is — no masks, no handler, the
        // M21 idle frame at the field's own door.
        let mut field = SpinButton::new("Quantity", 1.0, 12.0)
            .value(12.0)
            .on_change(|_| panic!("a write at the wall fires nothing"));
        let mut ctx = EventCtx::new();
        assert_eq!(
            field.on_key_event(&mut ctx, &arrow(Key::ArrowUp)),
            EventResponse::Handled,
            "the ask is answered"
        );
        assert!(same(field.current_value(), 12.0), "the wall held");
        assert_eq!(ctx.take(), Change::empty(), "the idle frame queues nothing");
    }

    #[test]
    fn steppers_step_from_the_pointer() {
        let mut field = SpinButton::new("Quantity", 1.0, 12.0).value(5.0);
        // The widget's frame at (100, 100), 180 wide, 44 tall: the
        // band rides the bottom edge (22 tall), the steppers' left edge
        // sits at 180 - 2*18 - 6 = 138 local — up spans y 22..32, down
        // y 34..44.
        let bounds = Rect::new(Point::new(100.0, 100.0), Size::new(180.0, 44.0));
        let up_x = 100.0 + 147.0;
        let (down, up) = click_at(up_x, 100.0 + 27.0);
        let mut ctx =
            EventCtx::with_phase_and_bounds(alacer_core::events::EventPhase::Bubble, Some(bounds));
        assert_eq!(
            field.on_pointer_event(&mut ctx, &down),
            EventResponse::Handled
        );
        assert_eq!(
            field.on_pointer_event(&mut ctx, &up),
            EventResponse::Handled
        );
        assert!(same(field.current_value(), 6.0), "the up stepper stepped");
        let masks = ctx.take();
        assert!(
            masks.contains(Change::PAINT) && masks.contains(Change::A11Y),
            "the step repaints and re-announces, got {masks:?}"
        );

        // The down stepper mirrors.
        let mut ctx =
            EventCtx::with_phase_and_bounds(alacer_core::events::EventPhase::Bubble, Some(bounds));
        let (down, up) = click_at(up_x, 100.0 + 39.0);
        let _ = field.on_pointer_event(&mut ctx, &down);
        let _ = field.on_pointer_event(&mut ctx, &up);
        assert!(
            same(field.current_value(), 5.0),
            "the down stepper mirrored"
        );

        // A release off the steppers is the focus's answer, never a
        // write.
        let mut ctx =
            EventCtx::with_phase_and_bounds(alacer_core::events::EventPhase::Bubble, Some(bounds));
        let (down, up) = click_at(100.0 + 40.0, 100.0 + 27.0);
        let _ = field.on_pointer_event(&mut ctx, &down);
        let _ = field.on_pointer_event(&mut ctx, &up);
        assert!(same(field.current_value(), 5.0), "no stepper, no write");
    }

    #[test]
    fn the_at_value_tier_writes_and_refuses_honestly() {
        let seen = std::rc::Rc::new(std::cell::Cell::new(0.0_f32));
        let sink = std::rc::Rc::clone(&seen);
        let mut field = SpinButton::new("Quantity", 1.0, 12.0).on_change(move |v| sink.set(v));

        assert!(
            field.on_a11y_action(A11yAction::SetValue("10".to_string())),
            "an in-range numeric SetValue is answered"
        );
        assert!(same(field.current_value(), 10.0));
        assert!(same(seen.get(), 10.0), "the handler observed the write");
        assert_eq!(Widget::value(&field), Some("10".to_string()));

        assert!(
            !field.on_a11y_action(A11yAction::SetValue("loud".to_string())),
            "a non-numeric SetValue is refused, never munged into a step"
        );
        assert!(same(field.current_value(), 10.0));
        assert!(
            !field.on_a11y_action(A11yAction::SetValue("inf".to_string())),
            "a non-finite value is not a value the widget can carry"
        );

        assert!(field.on_a11y_action(A11yAction::Increment));
        assert!(same(field.current_value(), 11.0));
        assert!(field.on_a11y_action(A11yAction::Decrement));
        assert!(same(field.current_value(), 10.0));

        assert!(
            !field.on_a11y_action(A11yAction::DefaultPress),
            "a value control has no press action — the M11 refusal rule"
        );
    }

    #[test]
    fn every_write_clamps_and_snaps_to_the_step() {
        let mut field = SpinButton::new("Quantity", 1.0, 12.0).step(2.0);
        assert!(field.on_a11y_action(A11yAction::SetValue("3.1".to_string())));
        assert!(
            same(field.current_value(), 3.0),
            "the AT's write snaps like an arrow, got {}",
            field.current_value()
        );
        assert!(field.on_a11y_action(A11yAction::SetValue("99".to_string())));
        assert!(same(field.current_value(), 12.0), "the range is a wall");
    }

    #[test]
    fn programmatic_set_value_fires_no_handler() {
        let mut field = SpinButton::new("Quantity", 1.0, 12.0).on_change(|_| panic!("no user act"));
        field.set_value(7.0);
        assert!(same(field.current_value(), 7.0));
        field.set_value(f32::NAN);
        assert!(same(field.current_value(), 7.0), "NaN is ignored");
    }

    #[test]
    fn a_degenerate_range_or_step_is_a_panic() {
        let result = std::panic::catch_unwind(|| SpinButton::new("Broken", 10.0, 1.0));
        assert!(result.is_err(), "min >= max has no value to carry");
        let result = std::panic::catch_unwind(|| SpinButton::new("Broken", 1.0, 12.0).step(0.0));
        assert!(result.is_err(), "a zero step divides by zero");
    }

    #[test]
    fn a_disabled_field_refuses_every_write_door() {
        // M22, ADR-0028: the AT tier, the pointer, and the keys all refuse
        // honestly — the value the tree carries is the app's, and the
        // refusal queues nothing.
        let mut field = SpinButton::new("Quantity", 1.0, 12.0)
            .enabled(false)
            .on_change(|_| panic!("an unavailable field must not write"));

        for action in [
            A11yAction::SetValue("7".to_string()),
            A11yAction::Increment,
            A11yAction::Decrement,
            A11yAction::DefaultPress,
        ] {
            assert!(
                !field.on_a11y_action(action),
                "an unavailable value control has no write door"
            );
        }

        let bounds = Rect::new(Point::new(100.0, 100.0), Size::new(180.0, 40.0));
        let mut ctx =
            EventCtx::with_phase_and_bounds(alacer_core::events::EventPhase::Bubble, Some(bounds));
        let (down, up) = click_at(100.0 + 147.0, 100.0 + 4.0);
        assert_eq!(
            field.on_pointer_event(&mut ctx, &down),
            EventResponse::Handled,
            "the wall consumes the press"
        );
        let _ = field.on_pointer_event(&mut ctx, &up);
        assert!(
            ctx.take().is_empty(),
            "a refused press queues no masks — an idle frame"
        );
        assert_eq!(
            field.on_key_event(&mut ctx, &arrow(Key::ArrowUp)),
            EventResponse::Ignored,
            "the keyboard never reaches an unavailable control"
        );
        assert!(!field.focusable(), "an unavailable control holds no focus");
        assert!(
            same(field.current_value(), 1.0),
            "the refusals wrote nothing"
        );

        // The write answers the move (the M21 idle-frame property).
        assert!(!field.set_enabled(false), "already unavailable");
        assert!(field.set_enabled(true), "the write moved");
        assert!(field.is_enabled());
    }

    #[test]
    fn the_programmatic_writes_answer_the_contract() {
        let mut field = SpinButton::new("Quantity", 1.0, 12.0);
        field.set_name("Copies");
        assert_eq!(field.label().resolve(), "Copies");
        field.set_value(4.0);
        assert!(same(field.current_value(), 4.0));
        field.set_on_change(None);
        assert!(
            field.on_a11y_action(A11yAction::Increment),
            "a detached handler still writes — the value is the widget's"
        );
        assert!(same(field.current_value(), 5.0));
    }
}
