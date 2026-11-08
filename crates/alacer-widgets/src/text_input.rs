//! The text input: the IME contract's reference widget (§10.3).
//!
//! One line of editable text. Editing is **UTF-16-native**: the caret is a
//! UTF-16 offset, IME preedit composes inline at the caret (with the
//! platform's cursor offset inside the preedit), and commits insert final
//! text — every conversion between the contract's units and Rust's UTF-8
//! strings funnels through `alacer_core::utf16`, so surrogate pairs and CJK
//! never slice (the fcitx5/ibus matrix exercises exactly this).
//!
//! Focus is a first-class: the widget paints its focus ring when
//! [`Widget::set_focused`] says it holds keyboard focus, and reports
//! [`ImePurpose::Text`] (or [`ImePurpose::Secure`]) so the platform attaches
//! the IME while it is focused.

use alacer_core::events::{EventCtx, ImeEvent, KeyEvent, PointerEvent};
use alacer_core::geom::Point;
use alacer_core::passes::ImePurpose;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style};
use alacer_core::utf16;
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// The value-change callback of [`TextInput`].
type ChangeHandler = Box<dyn FnMut(&str)>;

/// The submit callback of [`TextInput`] (M6: the todo canon's Enter seam).
type SubmitHandler = Box<dyn FnMut(&str)>;

/// A single-line editable text input (role `TextInput`, name mandatory —
/// ADR-5; the accessible name is the constructor's, the *value* is the
/// content).
pub struct TextInput {
    /// The accessible name (a screen reader says "Search, edit text").
    name: String,
    /// The committed text (UTF-8; all offsets below are UTF-16).
    text: String,
    /// The caret as a UTF-16 offset into `text` (the contract's unit).
    caret_utf16: usize,
    /// The composing preedit: `(text, cursor within preedit in UTF-16 units)`.
    /// `Some` while an IME composition is in progress.
    preedit: Option<(String, Option<usize>)>,
    /// Whether this widget holds keyboard focus (paints the ring).
    focused: bool,
    placeholder: String,
    purpose: ImePurpose,
    font_size: f32,
    style: Style,
    on_changed: Option<ChangeHandler>,
    on_submit: Option<SubmitHandler>,
}

impl TextInput {
    /// A text input with the accessible name `name`.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            text: String::new(),
            caret_utf16: 0,
            preedit: None,
            focused: false,
            placeholder: String::new(),
            purpose: ImePurpose::Text,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
            on_changed: None,
            on_submit: None,
        }
    }

    /// Secret entry: the platform suppresses preedit echo and suggestions.
    #[must_use]
    pub fn secure(mut self) -> Self {
        self.purpose = ImePurpose::Secure;
        self
    }

    /// Placeholder text shown while the value is empty.
    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Installs the change handler — invoked after every committed change to
    /// the value (key edits and IME commits alike).
    #[must_use]
    pub fn on_changed(mut self, handler: impl FnMut(&str) + 'static) -> Self {
        self.on_changed = Some(Box::new(handler));
        self
    }

    /// Installs the submit handler (M6): invoked on Enter with the current
    /// value, then the field clears — the todo canon's `input { on_submit:
    /// … }` seam. Without a handler Enter stays ignored (Tab cycling).
    #[must_use]
    pub fn on_submit(mut self, handler: impl FnMut(&str) + 'static) -> Self {
        self.on_submit = Some(Box::new(handler));
        self
    }

    /// The committed value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.text
    }

    /// The caret as a UTF-16 offset (the contract's unit; test introspection).
    #[must_use]
    pub const fn caret_utf16(&self) -> usize {
        self.caret_utf16
    }

    /// The composing preedit, if any (`(text, cursor_utf16)`).
    #[must_use]
    pub const fn preedit(&self) -> Option<&(String, Option<usize>)> {
        self.preedit.as_ref()
    }

    /// Replaces the whole value and puts the caret at the end (§6.1
    /// reconciliation; fires `on_changed`).
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.text = value.into();
        self.caret_utf16 = utf16::len_utf16(&self.text);
        self.notify_changed();
    }

    /// Replaces the accessible name in place (view reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Replaces the placeholder in place (view reconciliation).
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Replaces the change handler in place (view reconciliation). `None`
    /// detaches it.
    pub fn set_on_changed(&mut self, handler: Option<ChangeHandler>) {
        self.on_changed = handler;
    }

    /// Replaces the submit handler in place (view reconciliation). `None`
    /// detaches it.
    pub fn set_on_submit(&mut self, handler: Option<SubmitHandler>) {
        self.on_submit = handler;
    }

    fn notify_changed(&mut self) {
        if let Some(handler) = &mut self.on_changed {
            handler(&self.text.clone());
        }
    }

    /// Inserts `insertion` at the caret (UTF-16 units of it — commits may
    /// carry any text, including surrogate pairs).
    fn insert(&mut self, insertion: &str) {
        let at = utf16::byte_offset(&self.text, self.caret_utf16);
        self.text.insert_str(at, insertion);
        self.caret_utf16 += utf16::len_utf16(insertion);
        self.notify_changed();
    }

    /// Deletes `count` scalar values before the caret (backspace).
    fn backspace(&mut self) {
        if self.caret_utf16 == 0 {
            return;
        }
        let at = utf16::byte_offset(&self.text, self.caret_utf16);
        let before = utf16::byte_offset(&self.text, self.caret_utf16 - 1);
        self.text.replace_range(before..at, "");
        self.caret_utf16 = utf16::floor_boundary(&self.text, self.caret_utf16 - 1);
        self.notify_changed();
    }

    /// Deletes the scalar value after the caret (forward delete).
    fn delete(&mut self) {
        let at = utf16::byte_offset(&self.text, self.caret_utf16);
        if at >= self.text.len() {
            return;
        }
        let after = self.text[at..]
            .chars()
            .next()
            .map_or(at, |ch| at + ch.len_utf8());
        self.text.replace_range(at..after, "");
        self.notify_changed();
    }

    /// Moves the caret one scalar value left (never into a surrogate pair).
    fn move_left(&mut self) {
        if self.caret_utf16 == 0 {
            return;
        }
        self.caret_utf16 = utf16::floor_boundary(&self.text, self.caret_utf16 - 1);
    }

    /// Moves the caret one scalar value right (never into a surrogate pair).
    fn move_right(&mut self) {
        let len = utf16::len_utf16(&self.text);
        if self.caret_utf16 >= len {
            return;
        }
        let mut next = self.caret_utf16 + 1;
        while next < len && !utf16::is_boundary(&self.text, next) {
            next += 1;
        }
        self.caret_utf16 = next;
    }

    /// The painted run: committed text with the preedit spliced in at the
    /// caret — what the user sees and what the IME anchors to.
    fn display_parts(&self) -> (String, Option<String>, String) {
        match &self.preedit {
            Some((preedit, _)) => {
                let at = utf16::byte_offset(&self.text, self.caret_utf16);
                (
                    self.text[..at].to_string(),
                    Some(preedit.clone()),
                    self.text[at..].to_string(),
                )
            }
            None => (self.text.clone(), None, String::new()),
        }
    }
}

impl Widget for TextInput {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::TextInput
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    /// The M3 a11y contract: editable widgets expose their content (the
    /// committed value — preedit is composing, not committed).
    fn value(&self) -> Option<String> {
        Some(self.text.clone())
    }

    fn style(&self) -> Style {
        Style {
            padding: 8.0,
            ..self.style.clone()
        }
    }

    fn ime_purpose(&self) -> ImePurpose {
        self.purpose
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if !focused {
            // Losing focus cancels any in-flight composition (the platform
            // detaches the IME; a stale preedit would be a ghost).
            self.preedit = None;
        }
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(alacer_core::geom::Size::new(
            180.0,
            (self.font_size * crate::theme::LINE_HEIGHT).max(24.0) + 16.0,
        )))
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let (width, height) = (ctx.size().width, ctx.size().height);
        let border_color = if self.focused {
            crate::theme::ACCENT
        } else {
            crate::theme::BORDER
        };
        crate::paint_background(
            ctx,
            (width, height),
            crate::theme::PANEL,
            6.0,
            (border_color, if self.focused { 2.0 } else { 1.0 }),
        );

        let (before, preedit, after) = self.display_parts();

        alacer_core::text_seam::with_seam(|seam| {
            let text_color = if self.text.is_empty() && preedit.is_none() {
                crate::theme::TEXT_MUTED
            } else {
                crate::theme::TEXT
            };
            // The visible text: committed prefix + composing preedit +
            // committed suffix (the preedit composes *inline* at the caret).
            let visible = format!("{}{}{}", before, preedit.clone().unwrap_or_default(), after);
            let empty = visible.is_empty();
            let source_text = if empty {
                self.placeholder.clone()
            } else {
                visible
            };
            if !source_text.is_empty() {
                let layout = seam.layout_glyphs(&source_text, self.font_size, Some(width - 4.0));
                for glyph in &layout.glyphs {
                    ctx.push_glyph(alacer_core::scene::Glyph {
                        position: Point::new(2.0 + glyph.x, glyph.y),
                        size: glyph.size,
                        color: text_color,
                        font_id: glyph.font_id,
                        glyph_id: glyph.glyph_id,
                        clip: None,
                    });
                }
            }

            // The caret: a 2px accent bar at the insertion point, drawn only
            // while focused (§10.3: the caret is focus's visible half).
            if self.focused {
                let caret_x = if empty {
                    2.0
                } else {
                    let prefix = format!(
                        "{before}{}",
                        preedit.as_ref().map_or(String::new(), |p| {
                            // During composition the caret sits at the IME's own
                            // cursor within the preedit (UTF-16 units of it).
                            let cursor = self.preedit.as_ref().and_then(|(_, c)| *c).unwrap_or(0);
                            let at = utf16::byte_offset(p, cursor);
                            p[..at].to_string()
                        })
                    );
                    seam.layout_glyphs(&prefix, self.font_size, None).width
                };
                let caret_top = (height - self.font_size) * 0.5;
                ctx.push_quad(alacer_core::scene::Quad::filled(
                    alacer_core::geom::Rect::new(
                        Point::new(2.0 + caret_x, caret_top),
                        alacer_core::geom::Size::new(2.0, self.font_size),
                    ),
                    crate::theme::ACCENT,
                ));
                // The preedit underline: composing text is visually distinct.
                if preedit.is_some() {
                    let preedit_width = seam
                        .layout_glyphs(preedit.as_deref().unwrap_or(""), self.font_size, None)
                        .width;
                    ctx.push_quad(alacer_core::scene::Quad::filled(
                        alacer_core::geom::Rect::new(
                            Point::new(
                                2.0 + seam.layout_glyphs(&before, self.font_size, None).width,
                                caret_top + self.font_size + 1.0,
                            ),
                            alacer_core::geom::Size::new(preedit_width, 1.5),
                        ),
                        crate::theme::ACCENT,
                    ));
                }
            }
        });
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        // Clicking the field requests focus (the pipeline resolves it in
        // pass 3; the ring repaints via the FOCUS/PAINT masks it merges).
        if matches!(event, PointerEvent::Down { .. }) {
            ctx.request(alacer_core::Change::FOCUS);
            return EventResponse::Handled;
        }
        EventResponse::Ignored
    }

    fn on_key_event(&mut self, ctx: &mut EventCtx, event: &KeyEvent) -> EventResponse {
        // Editing acts on presses only (§10.3); preedit state makes keys
        // inert — the IME owns the composition until commit/cancel.
        if event.state != alacer_core::events::KeyState::Pressed || self.preedit.is_some() {
            return EventResponse::Ignored;
        }
        let mut changed = true;
        match &event.key {
            alacer_core::events::Key::Char(ch) => {
                self.insert(&ch.to_string());
            }
            alacer_core::events::Key::Backspace => self.backspace(),
            alacer_core::events::Key::Delete => self.delete(),
            alacer_core::events::Key::ArrowLeft => self.move_left(),
            alacer_core::events::Key::ArrowRight => self.move_right(),
            alacer_core::events::Key::Home => self.caret_utf16 = 0,
            alacer_core::events::Key::End => self.caret_utf16 = utf16::len_utf16(&self.text),
            alacer_core::events::Key::Enter => {
                // The submit seam (M6): fire with the current value, then
                // clear. The widget mutates only itself (rule 1); the
                // handler's writes ride the batch the caller chose.
                if let Some(handler) = &mut self.on_submit {
                    let value = self.text.clone();
                    handler(&value);
                    self.text.clear();
                    self.caret_utf16 = 0;
                    ctx.request(
                        alacer_core::Change::PAINT
                            .union(alacer_core::Change::LAYOUT)
                            .union(alacer_core::Change::A11Y),
                    );
                    return EventResponse::Handled;
                }
                changed = false;
            }
            _ => changed = false,
        }
        if changed {
            // Value and/or caret moved: repaint the field; value changes are
            // a11y-visible too.
            ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
            EventResponse::Handled
        } else {
            EventResponse::Ignored
        }
    }

    fn on_ime_event(&mut self, ctx: &mut EventCtx, event: &ImeEvent) -> EventResponse {
        match event {
            // Composition in progress: show the marked text inline. An empty
            // preedit cancels the composition (winit's synthetic clear).
            ImeEvent::Preedit { text, cursor_utf16 } => {
                if text.is_empty() {
                    self.preedit = None;
                } else {
                    self.preedit = Some((text.clone(), *cursor_utf16));
                }
                ctx.request(alacer_core::Change::PAINT);
                EventResponse::Handled
            }
            // Final text: insert at the caret, clear the preedit. Commits may
            // carry surrogate pairs — `insert` counts UTF-16 units.
            ImeEvent::Commit { text } => {
                self.preedit = None;
                if !text.is_empty() {
                    self.insert(text);
                }
                ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
                EventResponse::Handled
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TextInput;
    use alacer_core::events::{EventCtx, ImeEvent, Key, KeyEvent, KeyState, Modifiers};
    use alacer_core::passes::ImePurpose;
    use alacer_core::utf16;
    use alacer_core::widget::{EventResponse, Role, Widget};

    fn key(key: Key) -> KeyEvent {
        KeyEvent {
            key,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        }
    }

    fn type_char(input: &mut TextInput, ch: char) {
        let mut ctx = EventCtx::new();
        let event = KeyEvent::char(ch);
        let _ = input.on_key_event(&mut ctx, &event);
    }

    #[test]
    fn role_name_and_value_satisfy_the_contract() {
        let input = TextInput::new("Search");
        assert_eq!(input.role(), Role::TextInput);
        assert_eq!(input.label().resolve(), "Search");
        assert_eq!(input.value(), "");
        assert!(input.focusable());
        assert_eq!(input.ime_purpose(), ImePurpose::Text);
    }

    #[test]
    fn enter_submits_then_clears_the_field() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));
        let sink = std::rc::Rc::clone(&seen);
        let mut input = TextInput::new("Add a todo")
            .on_submit(move |value| sink.borrow_mut().push(value.to_string()));
        for ch in "write docs".chars() {
            let mut ctx = EventCtx::new();
            let _ = input.on_key_event(&mut ctx, &KeyEvent::char(ch));
        }
        let mut ctx = EventCtx::new();
        let enter = KeyEvent {
            key: Key::Enter,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert_eq!(input.on_key_event(&mut ctx, &enter), EventResponse::Handled);
        assert_eq!(seen.borrow().as_slice(), ["write docs"]);
        assert_eq!(input.value(), "", "the field clears after submit");
        assert_eq!(input.caret_utf16(), 0);
        let masks = ctx.take();
        assert!(
            masks.contains(alacer_core::Change::A11Y) && masks.contains(alacer_core::Change::PAINT),
            "submit repaints the cleared field and re-announces, got {masks:?}"
        );
    }

    #[test]
    fn enter_without_a_handler_stays_ignored() {
        let mut input = TextInput::new("Search");
        let mut ctx = EventCtx::new();
        let enter = KeyEvent {
            key: Key::Enter,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert_eq!(input.on_key_event(&mut ctx, &enter), EventResponse::Ignored);
    }

    #[test]
    fn typing_builds_ascii_and_cjk_values() {
        let mut input = TextInput::new("Search");
        for ch in "héllo".chars() {
            type_char(&mut input, ch);
        }
        assert_eq!(input.value(), "héllo");
        assert_eq!(input.caret_utf16(), 5);

        let mut cjk = TextInput::new("Input");
        for ch in "にほんご".chars() {
            type_char(&mut cjk, ch);
        }
        assert_eq!(cjk.value(), "にほんご");
        // CJK BMP runes: one UTF-16 unit each.
        assert_eq!(cjk.caret_utf16(), 4);
    }

    #[test]
    fn backspace_never_slices_a_surrogate_pair() {
        let mut input = TextInput::new("Emoji");
        type_char(&mut input, '\u{1F600}'); // 😀 = 2 UTF-16 units
        type_char(&mut input, 'x');
        assert_eq!(input.caret_utf16(), 3);

        // Backspace from the end removes 'x' (floor 3-1=2 is the pair end).
        let mut ctx = EventCtx::new();
        let _ = input.on_key_event(&mut ctx, &key(Key::Backspace));
        assert_eq!(input.value(), "\u{1F600}");

        // Backspace again removes the whole pair — never half of it.
        let mut ctx = EventCtx::new();
        let _ = input.on_key_event(&mut ctx, &key(Key::Backspace));
        assert_eq!(input.value(), "");
        assert_eq!(input.caret_utf16(), 0);
    }

    #[test]
    fn arrows_walk_scalar_boundaries() {
        let mut input = TextInput::new("Input");
        for ch in "a\u{1F600}b".chars() {
            type_char(&mut input, ch);
        }
        assert_eq!(input.caret_utf16(), 4);

        let mut ctx = EventCtx::new();
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowLeft));
        assert_eq!(input.caret_utf16(), 3, "'b' start");
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowLeft));
        assert_eq!(input.caret_utf16(), 1, "floors to the pair's start");
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowLeft));
        assert_eq!(input.caret_utf16(), 0);
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowLeft));
        assert_eq!(input.caret_utf16(), 0, "clamped at home");

        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowRight));
        assert_eq!(input.caret_utf16(), 1);
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowRight));
        assert_eq!(input.caret_utf16(), 3, "skips the mid-pair position");
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowRight));
        assert_eq!(input.caret_utf16(), 4);
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowRight));
        assert_eq!(input.caret_utf16(), 4, "clamped at end");

        let _ = input.on_key_event(&mut ctx, &key(Key::Home));
        assert_eq!(input.caret_utf16(), 0);
        let _ = input.on_key_event(&mut ctx, &key(Key::End));
        assert_eq!(input.caret_utf16(), 4);
    }

    #[test]
    fn ime_preedit_composes_and_cancels() {
        let mut input = TextInput::new("Input");
        let mut ctx = EventCtx::new();
        let preedit = ImeEvent::Preedit {
            text: "にほ".to_string(),
            cursor_utf16: Some(2),
        };
        assert_eq!(
            input.on_ime_event(&mut ctx, &preedit),
            EventResponse::Handled
        );
        assert_eq!(
            input.preedit().map(|(t, c)| (t.as_str(), *c)),
            Some(("にほ", Some(2)))
        );
        assert_eq!(input.value(), "", "preedit is composing, not committed");

        // Empty preedit cancels.
        let cancel = ImeEvent::Preedit {
            text: String::new(),
            cursor_utf16: None,
        };
        let _ = input.on_ime_event(&mut ctx, &cancel);
        assert_eq!(input.preedit(), None);
        assert_eq!(input.value(), "");
    }

    #[test]
    fn ime_commit_inserts_at_the_caret_and_clears_preedit() {
        let mut input = TextInput::new("Input");
        type_char(&mut input, 'a');
        let mut ctx = EventCtx::new();
        let _ = input.on_ime_event(
            &mut ctx,
            &ImeEvent::Preedit {
                text: "ん".to_string(),
                cursor_utf16: Some(1),
            },
        );
        let commit = ImeEvent::Commit {
            text: "日本".to_string(),
        };
        let _ = input.on_ime_event(&mut ctx, &commit);
        assert_eq!(input.value(), "a日本");
        assert_eq!(input.preedit(), None);
        // a=1 unit + 日本=2 units.
        assert_eq!(input.caret_utf16(), 3);
    }

    #[test]
    fn commit_with_surrogate_pairs_counts_utf16() {
        let mut input = TextInput::new("Input");
        let mut ctx = EventCtx::new();
        let _ = input.on_ime_event(
            &mut ctx,
            &ImeEvent::Commit {
                text: "\u{1F600}".to_string(), // 😀 = 4 UTF-8 bytes, 2 UTF-16 units
            },
        );
        assert_eq!(input.value(), "\u{1F600}");
        assert_eq!(input.caret_utf16(), 2);
        assert_eq!(utf16::len_utf16(input.value()), 2);
    }

    #[test]
    fn keys_are_inert_while_composing() {
        let mut input = TextInput::new("Input");
        let mut ctx = EventCtx::new();
        let _ = input.on_ime_event(
            &mut ctx,
            &ImeEvent::Preedit {
                text: "ん".to_string(),
                cursor_utf16: Some(1),
            },
        );
        // A character press during composition is ignored — the IME owns it.
        assert_eq!(
            input.on_key_event(&mut ctx, &KeyEvent::char('x')),
            EventResponse::Ignored
        );
        assert_eq!(input.value(), "");
    }

    #[test]
    fn losing_focus_cancels_the_composition() {
        let mut input = TextInput::new("Input");
        let mut ctx = EventCtx::new();
        let _ = input.on_ime_event(
            &mut ctx,
            &ImeEvent::Preedit {
                text: "ん".to_string(),
                cursor_utf16: Some(1),
            },
        );
        input.set_focused(true);
        input.set_focused(false);
        assert_eq!(input.preedit(), None, "no ghost preedit after blur");
    }

    #[test]
    fn secure_inputs_request_the_secure_ime() {
        let input = TextInput::new("Password").secure();
        assert_eq!(input.ime_purpose(), ImePurpose::Secure);
    }

    #[test]
    fn set_value_moves_the_caret_to_the_end() {
        let mut input = TextInput::new("Search");
        input.set_value("にほんご");
        assert_eq!(input.value(), "にほんご");
        assert_eq!(input.caret_utf16(), 4);
    }
}
