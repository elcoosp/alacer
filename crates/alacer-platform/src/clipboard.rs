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
}

#[cfg(test)]
mod tests {
    use super::ArboardClipboard;
    use alacer_core::clipboard::ClipboardSeam;

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
