//! The keymap seam: the app's shortcuts ride the dispatch contract
//! (§10.3, ADR-0041).
//!
//! A GUI toolkit's keyboard story has two halves. The focused widget owns
//! the *editing* half — a caret's arrows, a slider's step, the input's
//! chords (§10.3; the M29 rule that the chords are the widgets'
//! vocabulary). But an application also names *window-level* actions —
//! save, find, export — that must answer wherever the focus sits, and
//! until now every consumer had to smuggle that vocabulary through widget
//! handlers. Rather than dragging a settings file, a menu bar, or a
//! global-hotkey crate into every app's dependency set, the assembly site
//! installs one [`KeymapSeam`] implementation and the dispatch funnel
//! consults it — the same thread-local discipline the clipboard
//! ([`crate::clipboard`]) and text ([`crate::text_seam`]) seams impose,
//! for the same reason (handlers cannot reach the pipeline, axiom A1).
//!
//! ```text
//! // Assembly (an app's main, before the first frame):
//! let mut keymap = alacer_core::keymap::Keymap::new();
//! keymap.bind(
//!     alacer_core::keymap::Chord::new(alacer_core::Key::Char('s')).control(),
//!     "save",
//!     move || save_the_document(),
//! );
//! alacer_core::keymap::install(std::rc::Rc::new(keymap));
//!
//! // The funnel (already wired): a chord the focused widget did not
//! // answer is the keymap's to name, before the framework's own
//! // fallbacks (Tab cycling, the directional ring) take it.
//! ```
//!
//! # Precedence
//!
//! The dispatch funnel's keyboard order is, and stays (§10.3):
//!
//! 1. the focused widget's own vocabulary — a focused input's Space must
//!    type a space, never fire a bound action;
//! 2. the app's shortcuts — the explicit beats the implicit, so an app
//!    that binds `Tab` in a modal means it;
//! 3. the framework's fallbacks — Tab cycling (M24) and the directional
//!    ring (M29), which stand exactly where the app stayed silent.
//!
//! A chord fires on the press only ([`Chord::matches`]): a held key's
//! auto-repeat is one run per physical press, and a release is never a
//! second firing (the GTK activation rule, at the window tier).
//!
//! # The uninstalled window
//!
//! The seam is *optional* — a window with no shortcuts is a legitimate
//! application, not a wiring bug. [`dispatch_installed`] answers `false`
//! when nothing was installed and the funnel falls through honestly (the
//! M11 rule at the dispatch seam); there is no panic door, unlike the
//! clipboard's, whose absence *is* a wiring bug.

use std::cell::RefCell;
use std::rc::Rc;

use crate::events::{Key, KeyEvent, KeyState, Modifiers};

/// A keyboard chord: one logical [`Key`] plus the modifier state it
/// requires. The dispatch funnel consults the keymap with the *event's*
/// modifiers, so a chord matches only its own exact modifier shape —
/// `Ctrl+S` does not answer `Ctrl+Shift+S` (that is a different chord,
/// free for another action).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    key: Key,
    modifiers: Modifiers,
}

impl Chord {
    /// A chord of `key` with no modifiers held.
    #[must_use]
    pub const fn new(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: false,
            },
        }
    }

    /// A chord of `key` with exactly `modifiers` held.
    #[must_use]
    pub const fn with_modifiers(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    /// Requires Control (Linux/Windows) or Command (macOS) — the
    /// platform's primary accelerator modifier (`Modifiers::primary`,
    /// the input's command-chord convention).
    ///
    /// The chord stores the *event's* modifier shape it will match, and
    /// events carry the physical flags, so `primary()` records Control
    /// here; an app that must distinguish Command from Control across
    /// platforms builds the chord with [`Chord::with_modifiers`] instead.
    #[must_use]
    pub fn control(mut self) -> Self {
        self.modifiers.control = true;
        self
    }

    /// Requires Shift.
    #[must_use]
    pub fn shift(mut self) -> Self {
        self.modifiers.shift = true;
        self
    }

    /// Requires Alt / Option.
    #[must_use]
    pub fn alt(mut self) -> Self {
        self.modifiers.alt = true;
        self
    }

    /// Requires Super / Command.
    #[must_use]
    pub fn meta(mut self) -> Self {
        self.modifiers.meta = true;
        self
    }

    /// The chord's key.
    #[must_use]
    pub const fn key(&self) -> &Key {
        &self.key
    }

    /// The chord's modifier shape.
    #[must_use]
    pub const fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// Whether the event *is* this chord: the same logical key, the same
    /// modifier shape, on the press only — a release is never a firing
    /// (the GTK activation rule, at the window tier), and a held key's
    /// auto-repeat is one run per physical press.
    #[must_use]
    pub fn matches(&self, event: &KeyEvent) -> bool {
        event.state == KeyState::Pressed
            && event.key == self.key
            && event.modifiers == self.modifiers
    }
}

/// The keymap services seam. One implementation per thread — installed at
/// assembly time, before the first frame.
///
/// There **is** a provided implementation: [`Keymap`], the owned
/// dispatch table the settings story (bind, rebind, unbind) rides. A
/// consumer whose shortcuts come from somewhere else — a settings file
/// re-read on change, a menu bar's accelerator table — implements the
/// trait instead; the funnel never knows the difference.
pub trait KeymapSeam: 'static {
    /// Runs the bound action when `event` names a bound chord and
    /// answers whether the key is the keymap's now. `false` is the
    /// honest "nobody's key" — the funnel keeps it for the framework's
    /// fallbacks (Tab cycling, the directional ring), exactly as an
    /// ignored widget answer does (the M11 rule at the dispatch seam).
    fn dispatch(&self, event: &KeyEvent) -> bool;
}

/// One binding in the owned table: the chord that names it, the action
/// it runs, and the name the trail and the settings UI read. The name is
/// the binding's identity in prose ("save", "export-pdf") — a fired
/// action's `tracing` event carries it, so a `RUST_LOG` reader sees
/// *which* shortcut answered, not merely that one did.
pub struct Binding {
    chord: Chord,
    name: String,
    action: Rc<dyn Fn()>,
}

/// The provided [`KeymapSeam`]: an owned dispatch table. Bindings ride
/// [`Rc<dyn Fn()>`], so the table dispatches through the seam's
/// `&self` while actions capture the app's own state — the same
/// shared-ownership shape a button's `on_click` handler rides.
///
/// The table's rules:
///
/// - **One binding per chord.** [`Keymap::bind`] over a chord the table
///   already names replaces that binding in place — the settings story
///   ("the user changed the shortcut") is a move, not a shadow; the
///   displaced action is gone, never both.
/// - **Insertion order.** With the one-binding-per-chord invariant the
///   walk is at most one match; order is traversal stability, not
///   priority.
/// - **The press only.** A chord matches pressed events
///   ([`Chord::matches`]); a release falls through to the framework's
///   fallbacks, and nothing double-fires.
#[derive(Default)]
pub struct Keymap {
    bindings: Vec<Binding>,
}

impl Keymap {
    /// An empty table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds `chord` to `action` under `name`. Rebinding a chord the
    /// table already names replaces that binding in place — the
    /// settings story is a move, never a shadow (the struct's rules).
    pub fn bind(
        &mut self,
        chord: Chord,
        name: impl Into<String>,
        action: impl Fn() + 'static,
    ) -> &mut Self {
        let name = name.into();
        let action = Rc::new(action);
        if let Some(existing) = self
            .bindings
            .iter_mut()
            .find(|binding| binding.chord == chord)
        {
            existing.name = name;
            existing.action = action;
        } else {
            self.bindings.push(Binding {
                chord,
                name,
                action,
            });
        }
        self
    }

    /// Removes the chord's binding, answering whether the table carried
    /// one. After an unbind the chord belongs to the framework's
    /// fallbacks again — unbinding `Tab` returns focus cycling (the
    /// settings story's way back).
    pub fn unbind(&mut self, chord: &Chord) -> bool {
        let before = self.bindings.len();
        self.bindings.retain(|binding| &binding.chord != chord);
        self.bindings.len() != before
    }

    /// How many bindings the table carries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Whether the table carries no bindings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// The binding names, in table order — the introspection a settings
    /// read-back (or a test) walks.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.bindings.iter().map(|b| b.name.as_str()).collect()
    }
}

impl KeymapSeam for Keymap {
    fn dispatch(&self, event: &KeyEvent) -> bool {
        for binding in &self.bindings {
            if binding.chord.matches(event) {
                // The fired action names itself: a RUST_LOG reader sees
                // which shortcut answered, on the keymap's own target
                // (the house tracing idiom — full-path macro calls).
                tracing::debug!(target: "alacer::keymap", chord = binding.name.as_str(), "the chord's action fires");
                (binding.action)();
                return true;
            }
        }
        false
    }
}

/// Installs the thread's keymap seam. Re-installing replaces the
/// previous one (tests may swap implementations; production installs
/// once).
pub fn install(seam: Rc<dyn KeymapSeam>) {
    KEYMAP.with(|cell| *cell.borrow_mut() = Some(seam));
}

/// Whether a keymap seam is installed on this thread.
#[must_use]
pub fn is_installed() -> bool {
    KEYMAP.with_borrow(Option::is_some)
}

/// The dispatch funnel's total read: runs the installed seam's
/// [`KeymapSeam::dispatch`] when one is installed and answers `false`
/// when none is — a window with no shortcuts is a legitimate
/// application, never a wiring bug, so the read has no panic door (the
/// clipboard's `with_clipboard` rule, inverted for an optional seam).
#[must_use]
pub fn dispatch_installed(event: &KeyEvent) -> bool {
    KEYMAP.with_borrow(|seam| seam.as_ref().is_some_and(|seam| seam.dispatch(event)))
}

thread_local! {
    static KEYMAP: RefCell<Option<Rc<dyn KeymapSeam>>> = const { RefCell::new(None) };
}

#[cfg(test)]
mod tests {
    use super::{Chord, Keymap, KeymapSeam, dispatch_installed, install, is_installed};
    use crate::events::{Key, KeyEvent, KeyState, Modifiers};

    /// A pressed event for `key` with `modifiers`.
    fn pressed(key: Key, modifiers: Modifiers) -> KeyEvent {
        KeyEvent {
            key,
            state: KeyState::Pressed,
            modifiers,
            text: None,
        }
    }

    /// The event's own shape as a chord (what a binding would carry).
    fn chord_of(event: &KeyEvent) -> Chord {
        Chord::with_modifiers(event.key.clone(), event.modifiers)
    }

    #[test]
    fn a_chord_matches_its_own_key_and_modifiers_only() {
        let ctrl_s = Chord::new(Key::Char('s')).control();
        let event = pressed(
            Key::Char('s'),
            Modifiers {
                control: true,
                ..Modifiers::default()
            },
        );
        assert!(ctrl_s.matches(&event), "the chord answers its own shape");

        // A modifier's absence or addition is a different chord: Ctrl+Shift+S
        // is free for another action.
        let with_shift = pressed(
            Key::Char('s'),
            Modifiers {
                control: true,
                shift: true,
                ..Modifiers::default()
            },
        );
        assert!(
            !ctrl_s.matches(&with_shift),
            "an extra modifier is a different chord, never a superset match"
        );

        // The key is the chord's identity too.
        let ctrl_d = pressed(
            Key::Char('d'),
            Modifiers {
                control: true,
                ..Modifiers::default()
            },
        );
        assert!(!ctrl_s.matches(&ctrl_d), "another key is another chord");

        // And a release is never a firing — a held key's auto-repeat is
        // one run per physical press-and-release.
        let released = KeyEvent {
            key: Key::Char('s'),
            state: KeyState::Released,
            modifiers: Modifiers {
                control: true,
                ..Modifiers::default()
            },
            text: None,
        };
        assert!(!ctrl_s.matches(&released), "a release is nobody's firing");
    }

    #[test]
    fn the_table_fires_the_action_and_names_it() {
        let fired = std::rc::Rc::new(std::cell::Cell::new(0_u32));
        let counter = std::rc::Rc::clone(&fired);
        let mut keymap = Keymap::new();
        keymap.bind(Chord::new(Key::Char('s')).control(), "save", move || {
            counter.set(counter.get() + 1);
        });
        assert_eq!(keymap.len(), 1);
        assert_eq!(keymap.names(), vec!["save"]);

        let event = pressed(
            Key::Char('s'),
            Modifiers {
                control: true,
                ..Modifiers::default()
            },
        );
        assert!(
            keymap.dispatch(&event),
            "the bound chord is the keymap's key"
        );
        assert_eq!(fired.get(), 1, "the action ran exactly once");

        // An unbound chord of the same family falls through honestly.
        let unbound = pressed(Key::Char('s'), Modifiers::default());
        assert!(
            !keymap.dispatch(&unbound),
            "the plain 's' is nobody's binding here"
        );
        assert_eq!(fired.get(), 1, "the fall-through ran nothing");
    }

    #[test]
    fn rebinding_moves_and_unbinding_returns_the_key() {
        let first = std::rc::Rc::new(std::cell::Cell::new(0_u32));
        let second = std::rc::Rc::new(std::cell::Cell::new(0_u32));

        let mut keymap = Keymap::new();
        keymap.bind(Chord::new(Key::Tab), "cycle", {
            let first = std::rc::Rc::clone(&first);
            move || first.set(first.get() + 1)
        });
        // The settings story: the same chord rebinds in place — the
        // displaced action is gone, never both.
        keymap.bind(Chord::new(Key::Tab), "next pane", {
            let second = std::rc::Rc::clone(&second);
            move || second.set(second.get() + 1)
        });
        assert_eq!(keymap.len(), 1, "the rebind moved, it did not shadow");
        assert_eq!(keymap.names(), vec!["next pane"]);

        let tab = pressed(Key::Tab, Modifiers::default());
        assert!(keymap.dispatch(&tab));
        assert_eq!(first.get(), 0, "the displaced action never ran");
        assert_eq!(second.get(), 1, "the replacement runs");

        // Unbinding returns the chord to the framework's fallbacks.
        assert!(keymap.unbind(&chord_of(&tab)), "the unbind moved one");
        assert!(!keymap.unbind(&chord_of(&tab)), "nothing left to move");
        assert!(keymap.is_empty(), "the table is empty after the unbind");
        assert!(
            !keymap.dispatch(&tab),
            "the unbound chord is nobody's binding"
        );
    }

    #[test]
    fn the_installed_seam_answers_and_replacement_wins() {
        // A fresh thread carries a fresh thread-local: the uninstalled
        // window is a legitimate shape, not a wiring bug — the read is
        // total, there is no panic door.
        std::thread::spawn(|| {
            assert!(!is_installed(), "a fresh thread carries no keymap");

            let unbound = pressed(Key::Char('q'), Modifiers::default());
            assert!(
                !dispatch_installed(&unbound),
                "no seam: the read answers false and the funnel falls through"
            );

            let mut keymap = Keymap::new();
            keymap.bind(Chord::new(Key::Char('q')), "quit", || {});
            install(std::rc::Rc::new(keymap));
            assert!(is_installed(), "the install landed");
            assert!(
                dispatch_installed(&pressed(Key::Char('q'), Modifiers::default())),
                "the bound chord is the keymap's key"
            );

            // Re-installing replaces the previous seam (tests swap
            // implementations; production installs once).
            let empty = Keymap::new();
            install(std::rc::Rc::new(empty));
            assert!(
                !dispatch_installed(&pressed(Key::Char('q'), Modifiers::default())),
                "the replacement's silence is the answer now"
            );
        })
        .join()
        .expect("the thread-local story is thread-shaped");
    }

    #[test]
    fn a_custom_seam_implementation_rides_the_same_funnel() {
        // The trait's own story: a consumer whose shortcuts come from
        // somewhere else implements the seam, and the funnel never
        // knows the difference.
        struct OnlyEscape;
        impl KeymapSeam for OnlyEscape {
            fn dispatch(&self, event: &KeyEvent) -> bool {
                event.presses(&Key::Escape)
            }
        }
        install(std::rc::Rc::new(OnlyEscape));
        assert!(
            dispatch_installed(&pressed(Key::Escape, Modifiers::default())),
            "the custom seam answers its own vocabulary"
        );
        assert!(
            !dispatch_installed(&pressed(Key::Enter, Modifiers::default())),
            "the custom seam's silence falls through"
        );
    }
}
