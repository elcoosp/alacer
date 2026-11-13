//! The clipboard seam: how the editing contract's copy/cut/paste tier
//! reaches the OS clipboard without any widget depending on the platform
//! (§10.3, ADR-0020).
//!
//! Widgets live two layers below the platform stack, yet a focused field
//! must answer Ctrl/Cmd+C, X, and V. Rather than dragging an OS clipboard
//! dependency into every widget's dependency set, the assembly site
//! installs one [`ClipboardSeam`] implementation and widgets call it
//! through a thread-local indirection — the same discipline the text
//! seam ([`crate::text_seam`]) imposes, for the same reason.
//!
//! ```text
//! // Assembly (facade/platform) — the engine rides arboard, in alacer-platform:
//! alacer_core::clipboard::install(std::rc::Rc::new(
//!     alacer_platform::clipboard::ArboardClipboard::new(),
//! ));
//!
//! // Tests and headless harnesses — the honest in-memory seam:
//! alacer_core::clipboard::install(std::rc::Rc::new(
//!     alacer_core::clipboard::MemoryClipboard::default(),
//! ));
//!
//! // Widgets (core-only dependencies):
//! use alacer_core::clipboard::with_clipboard;
//! with_clipboard(|clipboard| clipboard.write_text(&selected));
//! ```

//! Since M39 the round trips ride the causal log too (ADR-0045): every
//! clipboard round trip the editing chords answer — a copy, a cut, a
//! paste, and every honest refusal — streams one
//! [`crate::inspect::ClipboardRecord`] onto the thread's trail below,
//! and the dispatching pipeline drains it at the tick's start into
//! [`crate::inspect::InspectEvent::Clipboard`] — the edit trail's shape
//! (ADR-0037), dev-mode and bounded, silent while nobody listens. The
//! record carries the payload's scalar count, never the payload: a
//! clipboard can carry a secret, and the log is a devtools surface, not
//! a shadow clipboard.
//!
//! Since M41 the engine names its platform-side facts too (ADR-0047,
//! ADR-0045's own re-open rule): the seam's engine contract grows
//! beside [`ClipboardSeam`] — one *provided* [`ClipboardSeam::write_reported`]
//! door answering [`ClipboardWriteFact`], the M37 provided-method
//! discipline (the provided default maps the `bool` door and names its
//! absence honestly, a real engine names its species). The round trip's
//! record carries the engine's species beside the `write_refused`
//! verdict — the log answers "which clipboard failed it", not only
//! "it failed".
//!
//! Since M42 the read side names its facts too (ADR-0048, ADR-0047's
//! third re-open rule): the paste's `None` collapsed three facts — the
//! empty clipboard, the non-text payload, the unopenable clipboard —
//! into one silence, so one *provided* [`ClipboardSeam::read_reported`]
//! door grows beside the `Option` door, answering [`ClipboardReadFact`]
//! with the same provided-method discipline (the provided default maps
//! the `Option` door and names its absence honestly, a real engine
//! names its species). The round trip's record carries the read's
//! species beside the paste's verdict — the M39 `empty_clipboard`
//! collapse splits at last: a paste that refused for the *read* names
//! the engine's species, a paste that refused for the *field* carries
//! the read's absence. The log answers "did the paste refuse for the
//! read, or for the field".
//!
//! [`crate::passes::PassPipeline::announce`]: crate::passes::PassPipeline::announce

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// The clipboard services seam. One implementation per process — installed
/// at assembly time, before the first frame.
///
/// There is **no provided implementation**, deliberately: the OS
/// clipboard is a platform fact — the honest answer needs the platform's
/// own ownership protocols, exactly what the adopted cores carry. A
/// widget-internal buffer would silently break the round trip every other
/// application shares, exactly the documented guess the honesty rules
/// forbid. The workspace's engine implementation rides arboard (the
/// audit's adopted clipboard core, architecture §12/§20 — ADR-9 keeps the
/// adapter dependencies in the platform crate), and tests drive the
/// in-memory [`MemoryClipboard`] so headless CI never races the
/// operator's actual clipboard.
pub trait ClipboardSeam: 'static {
    /// The clipboard's text payload, if it currently carries one. `None`
    /// is the honest answer for an empty clipboard, a non-text payload,
    /// and a clipboard this process cannot open — a paste refuses on
    /// `None` and queues no masks (the refuse-don't-munge rule, now at
    /// the clipboard tier).
    fn read_text(&self) -> Option<String>;

    /// The engine's reported read (M42, ADR-0048, ADR-0047's third
    /// re-open rule): the read-side fact the `Option` door cannot
    /// carry — the payload delivered, or refused with the species the
    /// engine itself answered. *Provided*, so a custom engine that
    /// overrides nothing keeps answering its `Option` door
    /// ([`ClipboardReadRefusal::Unnamed`] naming the absence honestly —
    /// the M37 `Option::None` discipline, as a species), and a real
    /// engine names its species. The `read_text` door stays the
    /// contract of record (the M35 `dispatch_installed` rule): a read
    /// through either door is the same read, and a refusal through
    /// either inserts nothing.
    fn read_reported(&self) -> ClipboardReadFact {
        match self.read_text() {
            Some(text) => ClipboardReadFact::Delivered(text),
            None => ClipboardReadFact::Refused(ClipboardReadRefusal::Unnamed),
        }
    }

    /// Writes `text` as the clipboard's text payload and reports whether
    /// the platform accepted it. `false` is the permission/escape story
    /// made observable: a refused write is never munged into success, so
    /// a cut whose write was refused does not delete — the selection
    /// survives a clipboard that failed it.
    fn write_text(&self, text: &str) -> bool;

    /// The engine's reported write (M41, ADR-0047, ADR-0045's own
    /// re-open rule): the platform-side fact the `bool` door cannot
    /// carry — the write accepted, or refused with the species the
    /// engine itself answered. *Provided*, so a custom engine that
    /// overrides nothing keeps answering its `bool` door
    /// ([`ClipboardWriteRefusal::Unnamed`] naming the absence honestly —
    /// the M37 `Option::None` discipline, as a species), and a real
    /// engine names its species. The `write_text` door stays the
    /// contract of record (the M35 `dispatch_installed` rule): a write
    /// through either door is the same write, and a refusal through
    /// either deletes nothing (ADR-0020's rule stands).
    fn write_reported(&self, text: &str) -> ClipboardWriteFact {
        if self.write_text(text) {
            ClipboardWriteFact::Accepted
        } else {
            ClipboardWriteFact::Refused(ClipboardWriteRefusal::Unnamed)
        }
    }
}

/// Why one engine write refused (M41, ADR-0047): the engine's own
/// refusal species — the platform-side facts ADR-0045's re-open rule
/// pre-registered, named by the engine that answered them, never a
/// guess the OS did not (ADR-0020's rule stands). The species ride the
/// round trip's record beside the `write_refused` verdict
/// ([`crate::inspect::ClipboardRecord::engine`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardWriteRefusal {
    /// The engine answered only its `bool` (the provided door's
    /// default) — the write's platform-side fact is honest about its
    /// absence, the M37 `Option::None` discipline as a species.
    Unnamed,
    /// The platform offers no clipboard subsystem at all: a headless
    /// box, a sandboxed session, an unsupported selection.
    Unsupported,
    /// The native clipboard was held by another party while the write
    /// ran.
    Occupied,
    /// The payload could not be converted to the platform's format.
    Conversion,
    /// The platform answered with an error the engine cannot classify —
    /// its description traced engine-side, never guessed onto the wire.
    OsError,
}

impl ClipboardWriteRefusal {
    /// The refusal's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unnamed => "unnamed",
            Self::Unsupported => "unsupported",
            Self::Occupied => "occupied",
            Self::Conversion => "conversion",
            Self::OsError => "os_error",
        }
    }
}

/// What the engine answered one write (M41, ADR-0047): the engine's own
/// fact — the write accepted, or refused with its named species
/// ([`ClipboardWriteRefusal`]). The engine-tier answer the round trip's
/// record carries beside its own verdict: the round trip says *what*
/// moved, the engine fact says *which platform answered*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardWriteFact {
    /// The write landed — the platform accepted the payload.
    Accepted,
    /// The write moved nothing — the engine names its species.
    Refused(ClipboardWriteRefusal),
}

impl ClipboardWriteFact {
    /// The fact's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Refused(_) => "refused",
        }
    }
}

/// Why one engine read refused (M42, ADR-0048): the engine's own
/// refusal species — the read-side facts ADR-0047's third re-open rule
/// pre-registered, named by the engine that answered them, never a
/// guess the OS did not (ADR-0020's rule stands). The species ride the
/// round trip's record beside the paste's verdict
/// ([`crate::inspect::ClipboardRecord::read`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardReadRefusal {
    /// The engine answered only its `Option` (the provided door's
    /// default) — the read's platform-side fact is honest about its
    /// absence, the M37 `Option::None` discipline as a species.
    Unnamed,
    /// The platform offers no clipboard subsystem at all, or the
    /// clipboard cannot open: a headless box, a sandboxed session.
    Unsupported,
    /// The clipboard carries nothing the OS counts as content — the
    /// empty clipboard, named.
    Empty,
    /// The clipboard carries content, but not text (an image, a
    /// custom format) — the payload the text format cannot carry,
    /// named at last.
    NonText,
    /// The platform answered with an error the engine cannot classify —
    /// its description traced engine-side, never guessed onto the wire.
    OsError,
}

impl ClipboardReadRefusal {
    /// The refusal's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unnamed => "unnamed",
            Self::Unsupported => "unsupported",
            Self::Empty => "empty",
            Self::NonText => "non_text",
            Self::OsError => "os_error",
        }
    }
}

/// What the engine answered one read (M42, ADR-0048): the engine's own
/// fact — the clipboard's text payload delivered, or refused with its
/// named species ([`ClipboardReadRefusal`]). The read-tier answer the
/// round trip's record carries beside its own verdict: the round trip
/// says *what the field did*, the read fact says *what the clipboard
/// answered*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardReadFact {
    /// The read landed — the clipboard's text payload.
    Delivered(String),
    /// The read moved nothing — the engine names its species.
    Refused(ClipboardReadRefusal),
}

impl ClipboardReadFact {
    /// The fact's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values). Not `const`: the
    /// delivered payload is a `String`, and a `const` match could not
    /// drop it.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Delivered(_) => "delivered",
            Self::Refused(_) => "refused",
        }
    }
}

/// The single-line paste rule the editing contract fixes (§10.3,
/// ADR-0020): the clipboard carries arbitrary platform text — multi-line,
/// CRLF, orphan CR — and the field is single-line. Every line break (CR,
/// LF, or a CRLF pair, counted once) pastes as exactly one space, the
/// convention single-line entries across the platform stack share.
///
/// This is *pure text arithmetic* — the rule lives here, next to the seam
/// it serves, so every consumer shares one definition and no widget grows
/// its own dialect (the `word_hop` discipline, applied to paste).
#[must_use]
pub fn paste_text(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push(' ');
            }
            '\n' => out.push(' '),
            other => out.push(other),
        }
    }
    out
}

/// Installs the thread's clipboard seam. Re-installing replaces the
/// previous one (tests may swap implementations; production installs
/// once).
pub fn install(seam: Rc<dyn ClipboardSeam>) {
    CLIPBOARD.with(|cell| *cell.borrow_mut() = Some(seam));
}

/// Whether a clipboard seam is installed on this thread.
#[must_use]
pub fn is_installed() -> bool {
    CLIPBOARD.with_borrow(Option::is_some)
}

/// Runs `f` with the installed clipboard seam.
///
/// # Panics
///
/// Panics with an actionable message when no seam was installed: a widget
/// reached the clipboard before assembly did — a wiring bug, not a
/// runtime condition.
pub fn with_clipboard<R>(f: impl FnOnce(&dyn ClipboardSeam) -> R) -> R {
    CLIPBOARD.with_borrow(|clipboard| {
        let Some(clipboard) = clipboard.as_ref() else {
            panic!(
                "no clipboard seam installed — call `alacer_core::clipboard::install` \
                 (e.g. with `alacer_platform::clipboard::ArboardClipboard`) before the first frame"
            );
        };
        f(clipboard.as_ref())
    })
}

thread_local! {
    static CLIPBOARD: RefCell<Option<Rc<dyn ClipboardSeam>>> = const { RefCell::new(None) };
}

/// The clipboard trail's cap: facts carried per thread before the oldest
/// evicts — the journal's own bounded rule (§14.3) applied to its echo
/// (a devtools log, not a frame-budget liability; the same order the
/// edit and announcement trails ride).
pub const CLIPBOARD_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// trail costs one branch per round trip (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side of
    /// the seam, bounded by [`CLIPBOARD_TRAIL_CAP`] with oldest-first
    /// eviction, so a round trip queued where no pipeline will ever
    /// drain still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::ClipboardRecord>> =
        const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the clipboard trail (the pipeline's side of the
/// seam: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_clipboard_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Notes one clipboard fact on this thread's trail (the clipboard's
/// side of the seam, ADR-0045): a no-op while the trail is disabled, an
/// oldest-evicting push while it rides. The editing chords call this
/// where the round trip resolves — the widget knows the action and the
/// payload it moved, the trail knows the drain — so a status line, an
/// inspector trail, or a test reads the clipboard tier without
/// re-implementing the observation at the chord site.
pub fn note_round_trip(record: crate::inspect::ClipboardRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        tracing::trace!(
            target: "alacer::clipboard",
            action = record.action.name(),
            outcome = record.outcome.name(),
            chars = record.chars,
            "clipboard fact queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= CLIPBOARD_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::Clipboard`].
pub(crate) fn take_clipboard_trail() -> Vec<crate::inspect::ClipboardRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

/// The in-memory clipboard: the tests' and headless harnesses' honest
/// seam. It *is* a clipboard — a deterministic store that answers reads
/// with exactly what the last write delivered — never a guess about the
/// OS clipboard's behavior. Headless CI must never race the operator's
/// actual clipboard, so the harness installs this; the windowed assembly
/// swaps in the arboard-backed engine.
#[derive(Default)]
pub struct MemoryClipboard {
    text: RefCell<Option<String>>,
}

impl MemoryClipboard {
    /// Sets the clipboard's text payload (a test's paste side).
    pub fn set(&self, text: impl Into<String>) {
        *self.text.borrow_mut() = Some(text.into());
    }

    /// The clipboard's current text payload (a test's copy side).
    #[must_use]
    pub fn text(&self) -> Option<String> {
        self.text.borrow().clone()
    }

    /// Clears the payload (a test's isolation boundary).
    pub fn clear(&self) {
        *self.text.borrow_mut() = None;
    }
}

impl ClipboardSeam for MemoryClipboard {
    fn read_text(&self) -> Option<String> {
        self.text()
    }

    fn write_text(&self, text: &str) -> bool {
        self.set(text);
        true
    }

    /// The engine's reported read (M42, ADR-0048): the in-memory store
    /// is a real engine whose absence is *known* emptiness — the store
    /// knows the payload it holds, so it names its own species instead
    /// of riding the provided default's honest `unnamed` absence.
    fn read_reported(&self) -> ClipboardReadFact {
        match self.text() {
            Some(text) => ClipboardReadFact::Delivered(text),
            None => ClipboardReadFact::Refused(ClipboardReadRefusal::Empty),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ClipboardSeam, ClipboardWriteFact, ClipboardWriteRefusal, MemoryClipboard, install,
        is_installed, paste_text, with_clipboard,
    };
    use std::rc::Rc;

    /// A seam that refuses every write through the `bool` door alone:
    /// the provided reported door's honest default (`unnamed`).
    struct BoolRefuser;

    impl ClipboardSeam for BoolRefuser {
        fn read_text(&self) -> Option<String> {
            None
        }

        fn write_text(&self, _text: &str) -> bool {
            false
        }
    }

    /// A seam whose engine names its own species: the kiosk's clipboard
    /// — no subsystem at all, the write refused with the fact answered,
    /// the read refused with the species the engine itself answered
    /// (the M42 read door's real-engine shape).
    struct Kiosk;

    impl ClipboardSeam for Kiosk {
        fn read_text(&self) -> Option<String> {
            None
        }

        fn write_text(&self, _text: &str) -> bool {
            false
        }

        fn read_reported(&self) -> super::ClipboardReadFact {
            super::ClipboardReadFact::Refused(super::ClipboardReadRefusal::Unsupported)
        }

        fn write_reported(&self, _text: &str) -> ClipboardWriteFact {
            ClipboardWriteFact::Refused(ClipboardWriteRefusal::Unsupported)
        }
    }

    #[test]
    fn the_seam_routes_installed_implementations() {
        assert!(!is_installed(), "nothing installed on a fresh thread");
        let clipboard = Rc::new(MemoryClipboard::default());
        install(Rc::clone(&clipboard) as Rc<dyn ClipboardSeam>);

        assert_eq!(clipboard.text(), None, "a fresh store is empty");
        assert!(
            with_clipboard(|seam| seam.write_text("held")),
            "the memory seam accepts every write"
        );
        assert_eq!(clipboard.text().as_deref(), Some("held"));
        assert_eq!(
            with_clipboard(super::ClipboardSeam::read_text),
            Some("held".to_string()),
            "the read is the write's own payload"
        );
    }

    /// The rule's edge cases: a CRLF pair is one break (one space), a
    /// lone CR or LF is one break each, and everything else rides
    /// untouched.
    #[test]
    fn paste_text_collapses_line_breaks_to_single_spaces() {
        assert_eq!(paste_text("plain"), "plain", "no breaks, no change");
        assert_eq!(paste_text(""), "", "empty stays empty");
        assert_eq!(paste_text("a\r\nb"), "a b", "a CRLF pair is one break");
        assert_eq!(paste_text("a\rb"), "a b", "a lone CR is one break");
        assert_eq!(paste_text("a\nb"), "a b", "a lone LF is one break");
        assert_eq!(
            paste_text("a\r\n\r\nb"),
            "a  b",
            "consecutive breaks are consecutive spaces"
        );
        assert_eq!(paste_text("a\n\r\nb"), "a  b", "LF then CRLF is two breaks");
        assert_eq!(
            paste_text("タブ\tkept"),
            "タブ\tkept",
            "only line breaks collapse — tabs and text ride untouched"
        );
    }

    /// Surrogate pairs and CJK paste whole: the rule walks scalars, never
    /// UTF-16 units.
    #[test]
    fn paste_text_walks_scalars_never_units() {
        assert_eq!(paste_text("a\u{1F600}\nb"), "a\u{1F600} b");
        assert_eq!(paste_text("にほ\nご"), "にほ ご");
    }

    /// The provided door maps the `bool` door honestly (M41, ADR-0047):
    /// an accepted write names `accepted`, a refusal the bool cannot
    /// name answers `unnamed` — and an engine that overrides the door
    /// names its own species. The `write_text` contract of record is
    /// untouched.
    #[test]
    fn the_reported_door_defaults_to_the_bool_door_and_names_its_absence() {
        let memory = MemoryClipboard::default();
        assert_eq!(
            memory.write_reported("held"),
            super::ClipboardWriteFact::Accepted,
            "the memory engine accepts every write"
        );
        assert_eq!(
            BoolRefuser.write_reported("held"),
            super::ClipboardWriteFact::Refused(super::ClipboardWriteRefusal::Unnamed),
            "the provided default names the absence honestly"
        );
        assert_eq!(
            Kiosk.write_reported("held"),
            super::ClipboardWriteFact::Refused(super::ClipboardWriteRefusal::Unsupported),
            "an engine that names its species names it"
        );
    }

    /// The species' wire names: the protocol's stable, documented
    /// `snake_case` tags.
    #[test]
    fn the_engine_species_carry_their_wire_names() {
        use super::ClipboardWriteRefusal;
        assert_eq!(ClipboardWriteRefusal::Unnamed.name(), "unnamed");
        assert_eq!(ClipboardWriteRefusal::Unsupported.name(), "unsupported");
        assert_eq!(ClipboardWriteRefusal::Occupied.name(), "occupied");
        assert_eq!(ClipboardWriteRefusal::Conversion.name(), "conversion");
        assert_eq!(ClipboardWriteRefusal::OsError.name(), "os_error");
        assert_eq!(super::ClipboardWriteFact::Accepted.name(), "accepted");
        assert_eq!(
            super::ClipboardWriteFact::Refused(ClipboardWriteRefusal::Occupied).name(),
            "refused"
        );
    }

    /// The provided read door maps the `Option` door honestly (M42,
    /// ADR-0048): a delivery names `delivered`, an absence the Option
    /// cannot name answers `unnamed` — and an engine that overrides the
    /// door names its own species. The `read_text` contract of record
    /// is untouched.
    #[test]
    fn the_read_reported_door_defaults_to_the_option_door_and_names_its_absence() {
        let memory = MemoryClipboard::default();
        assert_eq!(
            memory.read_reported(),
            super::ClipboardReadFact::Refused(super::ClipboardReadRefusal::Empty),
            "the memory engine is a real engine — its absence is known emptiness"
        );
        memory.set("held");
        assert_eq!(
            memory.read_reported(),
            super::ClipboardReadFact::Delivered("held".to_string()),
            "the memory engine delivers the payload the store holds"
        );
        assert_eq!(
            BoolRefuser.read_reported(),
            super::ClipboardReadFact::Refused(super::ClipboardReadRefusal::Unnamed),
            "the provided default names the absence honestly"
        );
        assert_eq!(
            Kiosk.read_reported(),
            super::ClipboardReadFact::Refused(super::ClipboardReadRefusal::Unsupported),
            "an engine that names its species names it"
        );
    }

    /// The read species' wire names: the protocol's stable, documented
    /// `snake_case` tags.
    #[test]
    fn the_read_species_carry_their_wire_names() {
        use super::ClipboardReadRefusal;
        assert_eq!(ClipboardReadRefusal::Unnamed.name(), "unnamed");
        assert_eq!(ClipboardReadRefusal::Unsupported.name(), "unsupported");
        assert_eq!(ClipboardReadRefusal::Empty.name(), "empty");
        assert_eq!(ClipboardReadRefusal::NonText.name(), "non_text");
        assert_eq!(ClipboardReadRefusal::OsError.name(), "os_error");
        assert_eq!(
            super::ClipboardReadFact::Delivered("held".to_string()).name(),
            "delivered"
        );
        assert_eq!(
            super::ClipboardReadFact::Refused(ClipboardReadRefusal::NonText).name(),
            "refused"
        );
    }
}
