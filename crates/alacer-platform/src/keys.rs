//! The platform input translation seam (§10.3): winit's keyboard and IME
//! events mapped onto Alacer's UTF-16-native contract.
//!
//! The one subtle conversion — and the reason this module exists — is that
//! winit reports IME preedit cursor offsets **byte-wise** (UTF-8), while the
//! IME contract (GPUI's `EntityInputHandler` semantics) counts **UTF-16 code
//! units**. Every preedit enters the pipeline through
//! [`ime_cursor_utf16`], which funnels the conversion through
//! `alacer_core::utf16` so surrogate pairs never slice.

use alacer_core::events::{ImeEvent, Key, KeyEvent, KeyState, Modifiers};
use alacer_core::utf16;
use winit::event::ElementState;
use winit::keyboard::{Key as WinitKey, NamedKey};

/// Maps one winit key event's fields onto Alacer's [`KeyEvent`].
///
/// The mapper takes the fields, not winit's `KeyEvent` struct: the struct is
/// not constructible outside winit, and tests must be able to drive the
/// translation without a windowing system (A4: headless everything).
#[must_use]
pub fn map_key(
    logical_key: &WinitKey,
    text: Option<&str>,
    state: ElementState,
    modifiers: Modifiers,
) -> KeyEvent {
    let key_state = match state {
        ElementState::Pressed => KeyState::Pressed,
        ElementState::Released => KeyState::Released,
    };
    let synthesis = text.and_then(|t| t.chars().next()).map(String::from);
    KeyEvent {
        key: map_logical_key(logical_key, synthesis.as_deref()),
        state: key_state,
        modifiers,
        // Editing acts on presses only; releases carry no text.
        text: (state == ElementState::Pressed)
            .then_some(synthesis)
            .flatten(),
    }
}

/// Maps a winit logical key onto Alacer's [`Key`].
#[must_use]
pub fn map_logical_key(key: &WinitKey, text: Option<&str>) -> Key {
    match key {
        WinitKey::Named(named) => map_named_key(*named),
        WinitKey::Character(_) => {
            // The platform resolved the character; the text synthesis carries
            // the printable value (the IME-free path).
            match text.and_then(|t| t.chars().next()) {
                Some(ch) => Key::Char(ch),
                None => Key::Unknown,
            }
        }
        WinitKey::Dead(_) | WinitKey::Unidentified(_) => Key::Unknown,
    }
}

fn map_named_key(named: NamedKey) -> Key {
    match named {
        NamedKey::Enter => Key::Enter,
        NamedKey::Backspace => Key::Backspace,
        NamedKey::Delete => Key::Delete,
        NamedKey::Tab => Key::Tab,
        NamedKey::Space => Key::Space,
        NamedKey::Escape => Key::Escape,
        NamedKey::ArrowLeft => Key::ArrowLeft,
        NamedKey::ArrowRight => Key::ArrowRight,
        NamedKey::ArrowUp => Key::ArrowUp,
        NamedKey::ArrowDown => Key::ArrowDown,
        NamedKey::Home => Key::Home,
        NamedKey::End => Key::End,
        _ => Key::Unknown,
    }
}

/// Maps a winit IME event onto Alacer's [`ImeEvent`], converting the
/// preedit cursor from winit's byte indexing into the contract's UTF-16
/// units (the fcitx5/ibus seam — `egui-winit`'s quirk table, generalized).
/// `Enabled`/`Disabled` carry no widget-level meaning: `None`.
#[must_use]
pub fn map_ime_event(event: &winit::event::Ime) -> Option<ImeEvent> {
    match event {
        winit::event::Ime::Enabled | winit::event::Ime::Disabled => None,
        winit::event::Ime::Preedit(text, cursor) => Some(ImeEvent::Preedit {
            text: text.clone(),
            cursor_utf16: ime_cursor_utf16(text, *cursor),
        }),
        winit::event::Ime::Commit(text) => Some(ImeEvent::Commit { text: text.clone() }),
    }
}

/// Converts winit's byte-wise preedit cursor `(begin, end)` into the
/// contract's UTF-16 offset. `None` (no cursor) stays `None`; a byte offset
/// floors to the nearest scalar-value boundary before conversion — a
/// mid-rune cursor from a buggy backend can never slice a scalar value.
#[must_use]
pub fn ime_cursor_utf16(preedit: &str, cursor: Option<(usize, usize)>) -> Option<usize> {
    cursor.map(|(begin, _)| {
        let begin = begin.min(preedit.len());
        let mut floor = begin;
        while floor > 0 && !preedit.is_char_boundary(floor) {
            floor -= 1;
        }
        utf16::utf16_offset(preedit, floor)
    })
}

#[cfg(test)]
mod tests {
    use super::{ime_cursor_utf16, map_key, map_logical_key};
    use alacer_core::events::{Key, KeyState, Modifiers};
    use winit::event::ElementState;
    use winit::keyboard::{Key as WinitKey, NamedKey, SmolStr};

    #[test]
    fn named_keys_map_onto_logical_keys() {
        let event = map_key(
            &WinitKey::Named(NamedKey::Enter),
            None,
            ElementState::Pressed,
            Modifiers::default(),
        );
        assert!(event.presses(&Key::Enter));
        assert_eq!(event.state, KeyState::Pressed);
        assert_eq!(event.text, None);
    }

    #[test]
    fn space_maps_for_the_checkbox_toggle() {
        // winit reports the space bar as a named key; the M6 checkbox's
        // keyboard contract needs it as `Key::Space`.
        let event = map_key(
            &WinitKey::Named(NamedKey::Space),
            Some(" "),
            ElementState::Pressed,
            Modifiers::default(),
        );
        assert!(event.presses(&Key::Space), "space maps onto Key::Space");
    }

    #[test]
    fn character_keys_carry_their_text_and_modifiers() {
        let modifiers = Modifiers {
            shift: true,
            ..Modifiers::default()
        };
        let event = map_key(
            &WinitKey::Character(SmolStr::new("a")),
            Some("a"),
            ElementState::Pressed,
            modifiers,
        );
        assert_eq!(event.key, Key::Char('a'));
        assert_eq!(event.text.as_deref(), Some("a"));
        assert!(event.modifiers.shift);
    }

    #[test]
    fn released_keys_drop_the_text() {
        let event = map_key(
            &WinitKey::Character(SmolStr::new("x")),
            Some("x"),
            ElementState::Released,
            Modifiers::default(),
        );
        assert_eq!(event.state, KeyState::Released);
        assert_eq!(event.text, None, "editing acts on presses only");
    }

    #[test]
    fn logical_key_falls_back_without_text() {
        let key = map_logical_key(&WinitKey::Character(SmolStr::new("é")), None);
        assert_eq!(key, Key::Unknown, "no text synthesis, no character");
    }

    #[test]
    fn preedit_cursor_converts_bytes_to_utf16() {
        // ASCII: identity.
        assert_eq!(ime_cursor_utf16("ab", Some((1, 1))), Some(1));
        // CJK: one rune = 3 UTF-8 bytes = 1 UTF-16 unit.
        assert_eq!(ime_cursor_utf16("にほ", Some((3, 3))), Some(1));
        assert_eq!(ime_cursor_utf16("にほ", Some((6, 6))), Some(2));
        // No cursor stays None.
        assert_eq!(ime_cursor_utf16("abc", None), None);
        // A byte offset inside a rune floors to the rune's start.
        assert_eq!(ime_cursor_utf16("に", Some((1, 1))), Some(0));
        // Surrogate pairs: 4 UTF-8 bytes = 2 UTF-16 units.
        assert_eq!(ime_cursor_utf16("\u{1F600}", Some((4, 4))), Some(2));
    }
}
