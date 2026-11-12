//! The windowed clipboard seam: the engine implementation riding arboard,
//! adopted as-is (ADR-0020; architecture §12/§20 — clipboard via arboard).
//!
//! The platform owns the adapter dependency (ADR-9): widgets ask the core
//! seam, this crate answers it through the OS's own ownership protocols.
//! The clipboard opens lazily on first use — a headless box (no display,
//! a sandboxed session) answers [`ClipboardSeam::read_text`] with `None`
//! and refuses [`ClipboardSeam::write_text`] instead of panicking: a
//! clipboard that cannot open is refused, never munged into success.

use alacer_core::clipboard::ClipboardSeam;
use std::cell::RefCell;

/// The arboard-backed clipboard seam — the windowed assembly's
/// implementation ([`crate::AlacerApp::new`] installs it beside the text
/// seam).
pub struct ArboardClipboard {
    /// The OS clipboard, opened on first use. `None` while it has never
    /// been needed, or while every open attempt failed (each call retries
    /// — a display that arrives later heals the seam).
    clipboard: RefCell<Option<arboard::Clipboard>>,
}

impl ArboardClipboard {
    /// An unopened seam: the clipboard opens on first use.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            clipboard: RefCell::new(None),
        }
    }

    /// The open clipboard, or `None` when this process has none (the
    /// honest escape: traced once per failed attempt, never a panic).
    fn with_open<R>(&self, f: impl FnOnce(&mut arboard::Clipboard) -> R) -> Option<R> {
        let mut slot = self.clipboard.borrow_mut();
        if slot.is_none() {
            match arboard::Clipboard::new() {
                Ok(clipboard) => *slot = Some(clipboard),
                Err(error) => {
                    tracing::warn!(%error, "os clipboard unavailable; refusing the clipboard op");
                    return None;
                }
            }
        }
        slot.as_mut().map(f)
    }

    /// The OS's own answer, mapped onto the seam's refusal species (M41,
    /// ADR-0047): the kinds arboard names map onto the vocabulary the
    /// record carries; a kind the write story does not carry stays
    /// `os_error` — the platform answered, the engine does not guess
    /// past its answer (ADR-0020's rule stands).
    fn refusal_of(error: &arboard::Error) -> alacer_core::clipboard::ClipboardWriteRefusal {
        use alacer_core::clipboard::ClipboardWriteRefusal;
        match error {
            arboard::Error::ClipboardNotSupported => ClipboardWriteRefusal::Unsupported,
            arboard::Error::ClipboardOccupied => ClipboardWriteRefusal::Occupied,
            arboard::Error::ConversionFailure => ClipboardWriteRefusal::Conversion,
            // `Error` is `#[non_exhaustive]`: every kind the write story
            // does not carry — today `ContentNotAvailable` and the
            // catch-all `Unknown` — stays `os_error`, the OS having
            // answered with an error the engine does not classify past.
            _ => ClipboardWriteRefusal::OsError,
        }
    }
}

impl Default for ArboardClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardSeam for ArboardClipboard {
    fn read_text(&self) -> Option<String> {
        self.with_open(|clipboard| clipboard.get_text().ok())
            .flatten()
    }

    fn write_text(&self, text: &str) -> bool {
        self.with_open(|clipboard| clipboard.set_text(text).is_ok())
            .unwrap_or(false)
    }

    /// The engine's reported write (M41, ADR-0047, ADR-0045's own
    /// re-open rule): the real engine names its species — the arboard
    /// error kind the OS answered, mapped onto the seam's refusal
    /// vocabulary, never a guess the OS did not (the unclassifiable
    /// kinds stay `os_error`, their description traced engine-side,
    /// never guessed onto the wire).
    fn write_reported(&self, text: &str) -> alacer_core::clipboard::ClipboardWriteFact {
        use alacer_core::clipboard::{ClipboardWriteFact, ClipboardWriteRefusal};
        let mut slot = self.clipboard.borrow_mut();
        if slot.is_none() {
            match arboard::Clipboard::new() {
                Ok(clipboard) => *slot = Some(clipboard),
                Err(error) => {
                    tracing::warn!(%error, "os clipboard unavailable; refusing the clipboard write");
                    return ClipboardWriteFact::Refused(Self::refusal_of(&error));
                }
            }
        }
        let Some(clipboard) = slot.as_mut() else {
            return ClipboardWriteFact::Refused(ClipboardWriteRefusal::Unsupported);
        };
        match clipboard.set_text(text) {
            Ok(()) => ClipboardWriteFact::Accepted,
            Err(error) => {
                tracing::warn!(%error, "os clipboard refused the write");
                ClipboardWriteFact::Refused(Self::refusal_of(&error))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ArboardClipboard;
    use alacer_core::clipboard::{ClipboardSeam, ClipboardWriteRefusal};

    /// The engine's species mapping (M41, ADR-0047): the kinds arboard
    /// names map onto the record's vocabulary, the unclassifiable stays
    /// `os_error` — the mapping is the adapter's own, priced by the
    /// engine it rides.
    #[test]
    fn the_engine_maps_the_os_kinds_onto_the_seams_species() {
        assert_eq!(
            ArboardClipboard::refusal_of(&arboard::Error::ClipboardNotSupported),
            ClipboardWriteRefusal::Unsupported,
            "no subsystem at all is the unsupported species"
        );
        assert_eq!(
            ArboardClipboard::refusal_of(&arboard::Error::ClipboardOccupied),
            ClipboardWriteRefusal::Occupied,
            "another party holding the clipboard is the occupied species"
        );
        assert_eq!(
            ArboardClipboard::refusal_of(&arboard::Error::ConversionFailure),
            ClipboardWriteRefusal::Conversion,
            "a payload the format cannot carry is the conversion species"
        );
        assert_eq!(
            ArboardClipboard::refusal_of(&arboard::Error::ContentNotAvailable),
            ClipboardWriteRefusal::OsError,
            "a kind the write story does not carry stays os_error"
        );
        assert_eq!(
            ArboardClipboard::refusal_of(&arboard::Error::Unknown {
                description: "the OS said no".to_string(),
            }),
            ClipboardWriteRefusal::OsError,
            "the unclassifiable stays os_error, its description never guessed"
        );
    }

    /// The engine answers without panicking on a headless box: reads are
    /// `None`, writes are refused — the escape story the seam's contract
    /// pins. On a desktop session the same calls round-trip the OS
    /// clipboard (the fcitx5 matrix's honesty rule: the contract never
    /// assumes a display it does not have).
    #[test]
    fn a_headless_clipboard_refuses_instead_of_panicking() {
        let seam = ArboardClipboard::new();
        // Both answers are honest in either environment: `None`/`false`
        // when the OS clipboard cannot open, real values when it can.
        let _read = seam.read_text();
        let _wrote = seam.write_text("probing the session");
    }
}
