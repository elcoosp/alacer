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
//!
//! # The consult as data
//!
//! The bare `bool` the funnel read left its own answer unrecorded: a
//! fired shortcut, a table that named no binding, and a window that
//! never installed one all answered the same way. [`consult`] is the
//! read's full answer as data ([`KeymapConsult`] — `Uninstalled`,
//! `FellThrough`, `Fired`), [`dispatch_installed`] stays the fired
//! verdict, and the dispatch funnel streams one
//! [`crate::inspect::KeyRecord`] (the chord beside the species) per
//! consult the moment it resolves — ADR-0041's pre-registered re-open
//! rule, the causal log's additive rule (never a second trail).

use std::cell::RefCell;
use std::rc::Rc;

use crate::events::{Key, KeyEvent, KeyState, Modifiers};

/// A keyboard chord: one logical [`Key`] plus the modifier state it
/// requires. The dispatch funnel consults the keymap with the *event's*
/// modifiers, so a chord matches only its own exact modifier shape —
/// `Ctrl+S` does not answer `Ctrl+Shift+S` (that is a different chord,
/// free for another action).
///
/// `Copy` since M36 (ADR-0042): the chord is the record vocabulary the
/// funnel's consult streams beside its answer
/// ([`crate::inspect::KeyRecord`]), and a chord is two small
/// by-value fields — the causal log copies it the way the consult
/// already carried it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
///
/// Since M36 this is [`consult`]'s fired verdict
/// ([`KeymapConsult::fired`]) — the funnel's `bool` unchanged, the full
/// answer now data ([`KeymapConsult`]) the funnel streams beside it
/// (ADR-0042).
#[must_use]
pub fn dispatch_installed(event: &KeyEvent) -> bool {
    consult(event).fired()
}

/// The consult's answer as data (M36, ADR-0042): what the funnel's read
/// resolved to — the three species the optional seam's total read
/// carries. ADR-0041 shipped the consult as a bare `bool`, and its own
/// kill criteria named the cost: a fired shortcut, a table that named
/// no binding, and a window that never installed one all collapsed into
/// the same `false` — the trail reader's "why didn't my shortcut fire?"
/// was unanswerable from the answer. The species name themselves with
/// [`KeymapConsult::name`] (the wire's `snake_case` tags, the
/// protocol's stable, documented field values — the refusal species'
/// rule at the keymap tier).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeymapConsult {
    /// No seam is installed — the window carries no shortcuts at all
    /// (a legitimate application, never a wiring bug; the M35 rule the
    /// record now names instead of guessing).
    Uninstalled,
    /// A seam answered and named no binding for this chord — the key
    /// falls through to the framework's fallbacks (Tab cycling, the
    /// directional ring) exactly as an ignored widget answer does.
    FellThrough,
    /// A binding fired: the action ran, the key is the keymap's now,
    /// and the funnel answers `Handled` before the fallbacks run.
    Fired,
}

impl KeymapConsult {
    /// Whether a shortcut took the key — the M35 total read's verdict
    /// ([`dispatch_installed`] is this accessor at the funnel).
    #[must_use]
    pub const fn fired(self) -> bool {
        matches!(self, Self::Fired)
    }

    /// The answer's name, the wire vocabulary's `snake_case` tag
    /// (the protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Uninstalled => "uninstalled",
            Self::FellThrough => "fell_through",
            Self::Fired => "fired",
        }
    }
}

/// The consult as data (M36, ADR-0042): runs the installed seam's
/// [`KeymapSeam::dispatch`] when one is installed and answers the
/// [`KeymapConsult`] species the read resolved to — `Uninstalled` when
/// no seam is on this thread, `Fired` when a binding ran, `FellThrough`
/// when the seam named none. [`dispatch_installed`] is the fired
/// verdict at the funnel, so the M35 contract is this function's
/// `fired()` accessor and the consult is the read's full answer.
///
/// The seam handle leaves the thread-local before `dispatch` runs (the
/// handle is an [`Rc`]): a fired action that re-installs or uninstalls
/// mutates its own thread's table without re-entering the borrow — the
/// total read stays re-entrancy-shaped.
#[must_use]
pub fn consult(event: &KeyEvent) -> KeymapConsult {
    let seam = KEYMAP.with_borrow(std::clone::Clone::clone);
    match seam.as_ref() {
        Some(seam) if seam.dispatch(event) => KeymapConsult::Fired,
        Some(_) => KeymapConsult::FellThrough,
        None => KeymapConsult::Uninstalled,
    }
}

thread_local! {
    static KEYMAP: RefCell<Option<Rc<dyn KeymapSeam>>> = const { RefCell::new(None) };
}

#[cfg(test)]
mod tests {
    use super::{
        Chord, Keymap, KeymapConsult, KeymapSeam, consult, dispatch_installed, install,
        is_installed,
    };
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
        Chord::with_modifiers(event.key, event.modifiers)
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

    #[test]
    fn the_consult_answers_the_three_species_and_the_total_read_agrees() {
        // A fresh thread: the uninstalled consult names its species
        // instead of collapsing into the same `false` (M36, ADR-0042)
        // — and the M35 total read stays the fired verdict.
        std::thread::spawn(|| {
            let event = pressed(Key::Char('s'), Modifiers::default());
            assert_eq!(consult(&event), KeymapConsult::Uninstalled);
            assert!(!consult(&event).fired());
            assert!(!dispatch_installed(&event));

            let ran = std::rc::Rc::new(std::cell::Cell::new(0_u32));
            let mut keymap = Keymap::new();
            let counter = std::rc::Rc::clone(&ran);
            keymap.bind(Chord::new(Key::Char('s')).control(), "save", move || {
                counter.set(counter.get() + 1);
            });
            install(std::rc::Rc::new(keymap));

            let ctrl_s = pressed(
                Key::Char('s'),
                Modifiers {
                    control: true,
                    ..Modifiers::default()
                },
            );
            assert_eq!(consult(&ctrl_s), KeymapConsult::Fired);
            assert_eq!(ran.get(), 1, "the consult ran the binding exactly once");
            assert!(consult(&ctrl_s).fired());
            assert!(
                dispatch_installed(&ctrl_s),
                "the M35 read rides the consult"
            );

            let miss = pressed(Key::Char('x'), Modifiers::default());
            assert_eq!(consult(&miss), KeymapConsult::FellThrough);
            assert!(!consult(&miss).fired());
            assert!(!dispatch_installed(&miss));

            // The species name themselves — the wire's snake_case tags.
            assert_eq!(KeymapConsult::Fired.name(), "fired");
            assert_eq!(KeymapConsult::FellThrough.name(), "fell_through");
            assert_eq!(KeymapConsult::Uninstalled.name(), "uninstalled");
        })
        .join()
        .expect("the thread-local story is thread-shaped");
    }

    #[test]
    fn a_fired_action_may_reinstall_without_reentering_the_borrow() {
        // The consult hands the seam handle out of the thread-local
        // before dispatch runs: a fired action that installs a new
        // table mutates its own thread's seam instead of panicking on
        // the borrow (the re-entrancy shape the funnel's consult keeps).
        std::thread::spawn(|| {
            struct SelfReplacing(std::rc::Rc<std::cell::Cell<bool>>);
            impl KeymapSeam for SelfReplacing {
                fn dispatch(&self, _event: &KeyEvent) -> bool {
                    if !self.0.get() {
                        self.0.set(true);
                        install(std::rc::Rc::new(Keymap::new()));
                    }
                    true
                }
            }
            let replaced = std::rc::Rc::new(std::cell::Cell::new(false));
            install(std::rc::Rc::new(SelfReplacing(std::rc::Rc::clone(
                &replaced,
            ))));
            assert!(
                consult(&pressed(Key::Char('r'), Modifiers::default())) == KeymapConsult::Fired,
                "the first consult fires and swaps the table mid-flight"
            );
            assert!(replaced.get(), "the action ran");
            assert_eq!(
                consult(&pressed(Key::Char('r'), Modifiers::default())),
                KeymapConsult::FellThrough,
                "the replacement's silence is the answer now"
            );
        })
        .join()
        .expect("the thread-local story is thread-shaped");
    }
}
