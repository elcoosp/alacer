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

    /// Writes `text` as the clipboard's text payload and reports whether
    /// the platform accepted it. `false` is the permission/escape story
    /// made observable: a refused write is never munged into success, so
    /// a cut whose write was refused does not delete — the selection
    /// survives a clipboard that failed it.
    fn write_text(&self, text: &str) -> bool;
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
}

#[cfg(test)]
mod tests {
    use super::{
        ClipboardSeam, MemoryClipboard, install, is_installed, paste_text, with_clipboard,
    };
    use std::rc::Rc;

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
}
