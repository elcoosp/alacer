//! The windowed clipboard seam: the engine implementation riding arboard,
//! adopted as-is (ADR-0020; architecture §12/§20 — clipboard via arboard).
//!
//! The platform owns the adapter dependency (ADR-9): widgets ask the core
//! seam, this crate answers it through the OS's own ownership protocols.
//! The clipboard opens lazily on first use — a headless box (no display,
//! a sandboxed session) answers [`ClipboardSeam::read_text`] with `None`
//! and refuses [`ClipboardSeam::write_text`] instead of panicking: a
//! clipboard that cannot open is refused, never munged into success.
//!
//! Since M42 the read side names its species too (ADR-0048): the
//! engine's reported read maps the OS's own answers onto the read
//! vocabulary — the empty clipboard and the non-text payload named
//! apart by the OS's second answer (the image question asked only when
//! the text read refused), the unclassifiable staying `os_error`.

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

    /// The OS's own answer to a read, mapped onto the seam's read
    /// species (M42, ADR-0048): an unopenable clipboard names
    /// `unsupported`, the catch-all stays `os_error` — the same
    /// never-guess-past-the-answer rule the write side rides.
    fn read_refusal_of_open(
        error: &arboard::Error,
    ) -> alacer_core::clipboard::ClipboardReadRefusal {
        use alacer_core::clipboard::ClipboardReadRefusal;
        match error {
            arboard::Error::ClipboardNotSupported => ClipboardReadRefusal::Unsupported,
            _ => ClipboardReadRefusal::OsError,
        }
    }

    /// The OS's own answer to a refused text read, mapped onto the
    /// seam's read species (M42, ADR-0048): arboard answers
    /// `ContentNotAvailable` for an empty clipboard *and* for a
    /// non-text payload — the one collapse the read story must not
    /// keep (ADR-0047's rule names the two apart) — so the engine asks
    /// the OS its second question: an image read that lands names the
    /// `non_text` payload, a second `ContentNotAvailable` is the
    /// honest `empty`, any other answer stays `os_error`. Each species
    /// is the OS's own answer to a question the engine asked — never a
    /// guess (ADR-0020's rule stands). The second question runs only
    /// when the first refused: a delivered text read never pays for it.
    fn read_refusal_of(
        error: &arboard::Error,
        clipboard: &mut arboard::Clipboard,
    ) -> alacer_core::clipboard::ClipboardReadRefusal {
        match error {
            arboard::Error::ContentNotAvailable => {
                Self::species_of_image_answer(&clipboard.get_image())
            }
            _ => alacer_core::clipboard::ClipboardReadRefusal::OsError,
        }
    }

    /// The second question's mapping: the image read's own answer
    /// names the species (the pure part, so the tests price the
    /// routing without an OS clipboard).
    fn species_of_image_answer(
        answer: &Result<arboard::ImageData<'static>, arboard::Error>,
    ) -> alacer_core::clipboard::ClipboardReadRefusal {
        use alacer_core::clipboard::ClipboardReadRefusal;
        match answer {
            Ok(_) => ClipboardReadRefusal::NonText,
            Err(arboard::Error::ContentNotAvailable) => ClipboardReadRefusal::Empty,
            Err(_) => ClipboardReadRefusal::OsError,
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

    /// The engine's reported read (M42, ADR-0048, ADR-0047's third
    /// re-open rule): the real engine names its species — the arboard
    /// error kind the OS answered, mapped onto the seam's read
    /// vocabulary, never a guess the OS did not (the unclassifiable
    /// kinds stay `os_error`, their description traced engine-side,
    /// never guessed onto the wire). The empty/non-text collapse the
    /// `Option` door carried is split by the OS's own second answer —
    /// the image question asked only when the text read refused.
    fn read_reported(&self) -> alacer_core::clipboard::ClipboardReadFact {
        use alacer_core::clipboard::{ClipboardReadFact, ClipboardReadRefusal};
        let mut slot = self.clipboard.borrow_mut();
        if slot.is_none() {
            match arboard::Clipboard::new() {
                Ok(clipboard) => *slot = Some(clipboard),
                Err(error) => {
                    tracing::warn!(%error, "os clipboard unavailable; refusing the clipboard read");
                    return ClipboardReadFact::Refused(Self::read_refusal_of_open(&error));
                }
            }
        }
        let Some(clipboard) = slot.as_mut() else {
            return ClipboardReadFact::Refused(ClipboardReadRefusal::Unsupported);
        };
        match clipboard.get_text() {
            Ok(text) => ClipboardReadFact::Delivered(text),
            Err(error) => {
                tracing::warn!(%error, "os clipboard refused the read");
                ClipboardReadFact::Refused(Self::read_refusal_of(&error, clipboard))
            }
        }
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
    use alacer_core::clipboard::{ClipboardReadRefusal, ClipboardSeam, ClipboardWriteRefusal};

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

    /// The read-side open mapping (M42, ADR-0048): an unsupported
    /// subsystem names itself, every other open failure stays
    /// `os_error` — the write side's rule, carried across the read
    /// door.
    #[test]
    fn the_engine_maps_the_open_kinds_onto_the_read_species() {
        assert_eq!(
            ArboardClipboard::read_refusal_of_open(&arboard::Error::ClipboardNotSupported),
            ClipboardReadRefusal::Unsupported,
            "no subsystem at all is the unsupported species on the read side too"
        );
        assert_eq!(
            ArboardClipboard::read_refusal_of_open(&arboard::Error::ClipboardOccupied),
            ClipboardReadRefusal::OsError,
            "an occupied clipboard is the OS's own error, unclassifiable on open"
        );
        assert_eq!(
            ArboardClipboard::read_refusal_of_open(&arboard::Error::Unknown {
                description: "the OS said no".to_string(),
            }),
            ClipboardReadRefusal::OsError,
            "the unclassifiable stays os_error, its description never guessed"
        );
    }

    /// The refused-text-read mapping (M42, ADR-0048): the OS's own
    /// second answer splits the empty clipboard from the non-text
    /// payload — an image that lands names `non_text`, a second
    /// `ContentNotAvailable` names `empty`, any other answer stays
    /// `os_error`. The mapping is priced by the engine it rides: each
    /// species is the OS's own answer to a question the engine asked —
    /// the answers standing in as data, the OS clipboard itself never
    /// assumed open (the headless rule).
    #[test]
    fn the_engine_splits_the_empty_and_the_non_text_payload() {
        use std::borrow::Cow;
        let image = arboard::ImageData {
            width: 1,
            height: 1,
            bytes: Cow::Borrowed(&[0, 0, 0, 255]),
        };
        assert_eq!(
            ArboardClipboard::species_of_image_answer(&Ok(image)),
            ClipboardReadRefusal::NonText,
            "a payload the image read answers is the non-text species"
        );
        assert_eq!(
            ArboardClipboard::species_of_image_answer(&Err(arboard::Error::ContentNotAvailable)),
            ClipboardReadRefusal::Empty,
            "a second ContentNotAvailable is the empty clipboard's honest answer"
        );
        assert_eq!(
            ArboardClipboard::species_of_image_answer(&Err(arboard::Error::ConversionFailure)),
            ClipboardReadRefusal::OsError,
            "the unclassifiable second answer stays os_error"
        );
        assert_eq!(
            ArboardClipboard::species_of_image_answer(&Err(arboard::Error::Unknown {
                description: "the OS said no".to_string(),
            })),
            ClipboardReadRefusal::OsError,
            "the catch-all second answer stays os_error, its description never guessed"
        );
    }

    /// A headless clipboard refuses instead of panicking: reads are
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
        // The reported doors answer in either environment too — the
        // species named or the payload delivered, never a panic.
        let _reported_read = seam.read_reported();
        let _reported_write = seam.write_reported("probing the session");
    }
}
