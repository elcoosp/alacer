//! The OS setting seam (§7.2 pass 8, ADR-0080): the OS's dark-mode
//! fact becomes an installed implementation the paint pass resolves —
//! ADR-0079's pre-registered re-open rule ("a consumer needs the OS's
//! dark-mode fact → the platform grows the setting seam the palette
//! reads, the adapter the platform owns, ADR-0009, never a poll loop
//! the widget vocabulary guesses at"). Through M73 the fact existed
//! nowhere in the pipeline: a widget that wanted it guessed or polled;
//! "follow the OS" meant rebuilding the palette by hand.
//!
//! Since M75 the seam carries its composition half (ADR-0081,
//! ADR-0080's pre-registered re-open rule): an app that wants the
//! palette to *follow* the fact installs a [`SettingComposer`] through
//! [`crate::passes::PassPipeline::set_composer`] — the pipeline
//! resolves the composed [`Palette`] against the fact that is, at
//! install time and on every flip that lands, mounting it as the
//! installed theme the roles borrow. Since M80 (ADR-0086) the
//! composition resolves by all three facts — the composer's signature
//! grew beside `dark`, the pipeline handing the composition what it
//! asks for (a palette that calms when the OS asks for reduced motion
//! answers by the fact), never the composer reaching into an adapter. The fact never munges the palette
//! on its own (the M74 rule): only the composition moves it, and the
//! app's own `set_theme` ask wins until the next flip.
//!
//! The seam's shape is the theme seam's own (ADR-0079, the copy the
//! discipline rides): the platform adapter installs a
//! [`SettingSeam`] through [`crate::passes::PassPipeline::set_settings`]
//! — on winit, the `ThemeChanged` event and the window's opening fact —
//! the door answers `false` on a flip that lands where the fact already
//! is (the idle-frame property, A1's rule at the setting tier — no
//! broadcast, no record), and lands the flip otherwise: the fact
//! resolves once per paint pass into [`crate::scene::PaintCtx`]
//! beside the palette, the vocabulary consults it for the decisions no
//! role answers, and the broadcast rides the one invalidation currency
//! — `Change::PAINT` on every node, never a private repaint channel.
//! The fact never munges the palette: the flip repaints the tree, the
//! roles the widgets borrowed re-resolve from the *installed palette*
//! exactly as they were — an app that wants palette-following installs
//! the composition door the pipeline grew (ADR-0081).
//!
//! The flip answers on the causal log: one
//! [`crate::inspect::SettingRecord`] per flip that landed — the bools
//! *are* the facts, and since M76 (ADR-0082) the broadcast's node count
//! rides beside them (the lengths rule — the count names the flip's
//! reach, the ids never riding). Since M77 (ADR-0083, ADR-0080's first
//! pre-registered re-open rule) the seam and the record grow the OS's
//! reduced-motion fact beside the dark one — one seam, one door, one
//! record per flip, never a second seam per fact. Since M79 (ADR-0085,
//! the same rule cashed a second time) the seam and the record grow the
//! OS's high-contrast fact beside the two — the facts the vocabulary
//! consults riding beside each other, never a second seam per fact —
//! the theme trail's shape (ADR-0079), bounded, drained at the tick's
//! start, silent while nobody listens (the A7 idle contract). Since
//! M81 (ADR-0087, the same rule cashed a third time) the seam and the
//! record grow the OS's accent-color fact beside the three — the fact
//! riding its presence ([`None`], the honest absence, never a guess,
//! the adapter answering the absence when its platform reports no
//! accent), and since the same cut (ADR-0086's own pre-registered
//! rule) the composition resolves by it beside the three: a palette
//! that tints its accent roles from the OS accent answers by the fact
//! at the two points the door already owned.
//!
//! ```text
//! // The adapter's door (core-only dependency):
//! pipeline.set_settings(Box::new(WinitSettings { dark: true }));
//! // The composition door (ADR-0081): the palette follows the fact.
//! pipeline.set_composer(Box::new(MyComposer));
//! // The paint half (widgets, per node):
//! if ctx.dark() { ... } // the role-independent decision
//! ```

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

use crate::color::Color;
use crate::inspect::SettingRecord;
use crate::theme::{Palette, ThemeSeam};

/// The setting trail's cap: facts carried per thread before the oldest
/// evicts — the theme trail's own bounded rule (ADR-0079, §14.3)
/// applied to the setting tier (a devtools log, not a frame-budget
/// liability; the same order the theme and route-drop trails ride).
pub const SETTING_TRAIL_CAP: usize = 256;

/// The setting seam: implemented by the platform adapter (the adapter
/// the platform owns, ADR-0009) or by the app in tests and demos,
/// installed through [`crate::passes::PassPipeline::set_settings`],
/// resolved by the paint pass once per frame. `Send + Sync` like
/// [`crate::theme::ThemeSeam`] — the pipeline owns the implementation
/// across the platform loop.
///
/// The trait's contract is total and per-flip: every read answers the
/// fact the adapter holds *now*. The adapter pushes the OS's flip
/// through the door (the event the platform owns); the vocabulary
/// never polls — the seam is the fact's one installed source. Since
/// M77 (ADR-0083, ADR-0080's pre-registered re-open rule) the trait
/// carries the OS's reduced-motion fact beside the dark fact, and
/// since M79 (ADR-0085, the same rule cashed a second time) the OS's
/// high-contrast fact beside the two, and since M81 (ADR-0087, the
/// same rule cashed a third time) the OS's accent-color fact beside
/// the three — one seam, the facts the vocabulary consults riding
/// beside each other, never a second seam per fact. The accent fact
/// rides its presence: an adapter whose platform reports no accent
/// answers [`None`] — the honest absence, never a guess.
pub trait SettingSeam: Send + Sync + 'static {
    /// The OS's dark-mode fact, re-read once per paint pass — one
    /// dynamic call per frame, never per node, the palette
    /// resolution's shape.
    fn dark_mode(&self) -> bool;

    /// The OS's reduced-motion fact, re-read beside `dark_mode` once
    /// per paint pass (ADR-0083) — the same one dynamic call per frame,
    /// the vocabulary consulting it when motion itself is the question
    /// (a spinner's spin, a thumbnail's autoplay). A setting that
    /// cannot answer the fact is a setting that cannot consult it: the
    /// method is required, the adapter answering the absence when its
    /// platform reports no motion fact (`false` — motion allowed).
    fn reduced_motion(&self) -> bool;

    /// The OS's high-contrast fact, re-read beside the two once per
    /// paint pass (ADR-0085) — the same one dynamic call per frame,
    /// the vocabulary consulting it when contrast itself is the
    /// question (a hairline's weight, a focus ring's width, a muted
    /// ink raised). Required like the facts it sits beside, the
    /// adapter answering the absence when its platform reports no
    /// contrast fact (`false` — standard contrast).
    fn high_contrast(&self) -> bool;

    /// The OS's accent-color fact, re-read beside the three once per
    /// paint pass (ADR-0087) — the same one dynamic call per frame, the
    /// vocabulary consulting it when tint itself is the question (an
    /// accent that follows the OS's pick, a selection riding it), the
    /// composition resolving its accent roles by it beside the facts.
    /// Required like the facts it sits beside, and answerable by the
    /// absence: [`None`] is the honest absence — the fact riding its
    /// presence, a platform that reports no accent answering `None`
    /// (never a guessed color).
    fn accent_color(&self) -> Option<Color>;
}

/// The provided canon setting (§13.3, the provided-implementation rule
/// beside every seam — [`crate::theme::CanonPalette`]'s precedent):
/// light, motion allowed, standard contrast, no accent, always — the
/// facts the blessed snapshot corpus was blessed with, installable by
/// name so a consumer that swapped away can swap back through the same
/// door it swapped in. Installing it where nothing is installed (or
/// where the canon facts are installed) is the no-op the door answers
/// `false` on: the default build already resolves them.
#[derive(Debug, Clone, Copy, Default)]
pub struct CanonSettings;

impl SettingSeam for CanonSettings {
    fn dark_mode(&self) -> bool {
        false
    }

    fn reduced_motion(&self) -> bool {
        false
    }

    fn high_contrast(&self) -> bool {
        false
    }

    fn accent_color(&self) -> Option<Color> {
        None
    }
}

/// The composition seam (§7.2 pass 8, ADR-0081): the palette the OS's
/// fact resolves into — ADR-0080's pre-registered re-open rule cashing
/// ("a consumer needs the palette to *follow* the OS fact → the
/// pipeline grows the composition door, the installed palette
/// re-swapped on the flip's own record, priced by a real reader —
/// never a palette that reaches into the platform's adapter").
/// Implemented by the app, installed through
/// [`crate::passes::PassPipeline::set_composer`]: the door resolves the
/// composed [`Palette`] against the facts that *are* at install time
/// (since M80 the signature carries them beside `dark`, ADR-0086, and
/// since M81 the accent fact beside the three, ADR-0087) and
/// re-resolves it on every flip that lands — the theme tier's own swap
/// rules (the `PartialEq` idle check, the `Change::PAINT` broadcast,
/// the [`crate::inspect::ThemeRecord`] tagged with the composer's own
/// [`name`](SettingComposer::name)) riding unchanged.
///
/// The composer never reaches into the platform: it reads no adapter,
/// holds no window — the pipeline hands it the fact and the composer
/// answers a palette, the pure function the composition tier owns.
/// While a composer is installed the palette follows the fact; the
/// app's own [`crate::passes::PassPipeline::set_theme`] ask lands over
/// the composed answer and holds until the next flip (the manual door
/// wins while the fact is still — the composition owns the palette
/// only when the OS moves).
pub trait SettingComposer: Send + Sync + 'static {
    /// The roles the palette resolves for the facts the seam holds —
    /// called once per install and once per landed flip (never per
    /// frame: the composed palette installs as a [`crate::theme::ThemeSeam`]
    /// the paint pass resolves in its own once-per-pass shape). The
    /// equality check prices the value it answered: a flip that
    /// composes the palette already installed re-swaps nothing. Since
    /// M80 (ADR-0086) the bool facts ride beside `dark` — the pipeline
    /// handing the composition what it asks for, never the composer
    /// reaching into an adapter (ADR-0081's own rule) — and since M81
    /// (ADR-0087, the signature's own pre-registered growth) the accent
    /// fact rides beside the three, the fact riding its presence: a
    /// palette that tints its accent roles from the OS accent answers
    /// by the fact at the same two resolution points the door already
    /// owned, and answers the absence with its own.
    fn palette(
        &self,
        dark: bool,
        reduced_motion: bool,
        high_contrast: bool,
        accent: Option<Color>,
    ) -> Palette;

    /// The composed palette's tag — the stable name the causal log's
    /// [`crate::inspect::ThemeRecord`] carries for swaps this composer
    /// resolved (the theme trail's naming rule, ADR-0079): `""`-free,
    /// app-defined (`"duet"`).
    fn name(&self) -> &'static str;
}

/// The provided canon composition (§13.3, the provided-implementation
/// rule beside every seam — [`CanonSettings`]'s precedent): the canon
/// palette for any facts — the composition that follows nothing,
/// installable by name so a consumer that composed away can compose
/// back through the same door it composed in. Installing it where the
/// canon palette is already live (the default build's own answer, or
/// the canon theme) is the no-op the door answers `false` on.
#[derive(Debug, Clone, Copy, Default)]
pub struct CanonComposer;

impl SettingComposer for CanonComposer {
    fn palette(
        &self,
        _dark: bool,
        _reduced_motion: bool,
        _high_contrast: bool,
        _accent: Option<Color>,
    ) -> Palette {
        Palette::CANON
    }

    fn name(&self) -> &'static str {
        "canon"
    }
}

/// The locked composition (§13.3, ADR-0089): one palette, every fact,
/// every flip — the kiosk's answer. A kiosk, a signage display, an
/// embedded panel cannot follow the OS: a dark flip at 7pm must not
/// repaint the boarding gate, and the operator's desktop accent must
/// not tint the checkout line. Through M84 a palette the facts could
/// not move had two half-doors: the manual
/// [`crate::passes::PassPipeline::set_theme`] ask, which wins *until
/// the next flip* (the composition owns the palette when the OS moves),
/// and [`CanonComposer`], which locks the *canon* palette — light,
/// motion-allowing — and no other. [`KioskComposer::locked`] is the
/// lock over any palette: the composition answers the same
/// [`Palette`] for every fact permutation, so a flip lands (the facts
/// are the seam's, the door answers `true`), the door resolves the
/// composition, the composed answer *is* the palette already installed
/// — and the equality check swaps nothing (the idle-frame property at
/// the theme tier, ADR-0081). The app's own `set_theme` ask still wins
/// while it holds; the next flip is where the lock wins it back, the
/// swap riding the flip's own broadcast, the record tagged the
/// composer's own [`name`](SettingComposer::name).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KioskComposer {
    /// The palette the lock holds — the one answer, every fact.
    palette: Palette,
}

impl KioskComposer {
    /// The lock over `palette`: the composition that answers it for
    /// every fact permutation — the flip the facts land, the palette
    /// never moves. Const, so the lock ships beside the palette it
    /// holds (the examples' `const DUET_DARK` shape).
    #[must_use]
    pub const fn locked(palette: Palette) -> Self {
        Self { palette }
    }
}

impl SettingComposer for KioskComposer {
    fn palette(
        &self,
        _dark: bool,
        _reduced_motion: bool,
        _high_contrast: bool,
        _accent: Option<Color>,
    ) -> Palette {
        self.palette
    }

    fn name(&self) -> &'static str {
        "kiosk"
    }
}

/// The composed palette the composition door mounts (ADR-0081): a
/// [`ThemeSeam`] the pipeline itself installed from a composer's
/// answer — the roles the composer resolved beside the tag it named.
/// Not an app-facing type: the door owns the mount, the theme trail's
/// [`crate::inspect::ThemeRecord`] names it, and the app's own
/// [`crate::passes::PassPipeline::set_theme`] ask replaces it the way
/// any swap does.
pub(crate) struct ComposedPalette {
    /// The roles the composer resolved for the fact that landed.
    palette: Palette,
    /// The composer's own tag — the theme trail's fact.
    name: &'static str,
}

impl ComposedPalette {
    /// The composition door's mount: the roles the composer resolved
    /// beside the tag it named — constructed only where the swap
    /// resolves ([`crate::passes::PassPipeline`]'s doors).
    pub(crate) fn new(palette: Palette, name: &'static str) -> Self {
        Self { palette, name }
    }
}

impl ThemeSeam for ComposedPalette {
    fn palette(&self) -> Palette {
        self.palette
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

/// The OS facts one resolution carries (ADR-0080, ADR-0083, ADR-0085,
/// ADR-0087): the pipeline's own bundle of the seam's answers — the
/// composition door reads them once per resolution point (the install,
/// every landed flip) and hands them to the composer. Not app-facing:
/// the [`SettingComposer`] trait keeps the plain per-fact signature
/// (ADR-0086's rule — the composer answers by the facts beside
/// `dark`), the bundle is the door's internal shape.
#[derive(Debug, Clone, Copy)]
pub(crate) struct OsFacts {
    /// The dark-mode fact — [`SettingSeam::dark_mode`]'s answer.
    pub(crate) dark: bool,
    /// The reduced-motion fact — [`SettingSeam::reduced_motion`]'s
    /// answer.
    pub(crate) reduced_motion: bool,
    /// The high-contrast fact — [`SettingSeam::high_contrast`]'s
    /// answer.
    pub(crate) high_contrast: bool,
    /// The accent-color fact — [`SettingSeam::accent_color`]'s answer,
    /// riding its presence (`None` is the honest absence).
    pub(crate) accent: Option<Color>,
}

impl OsFacts {
    /// The canon facts (light, motion allowed, standard contrast, no
    /// accent) — what the door resolves where nothing is installed, the
    /// facts the default build resolves.
    pub(crate) const CANON: Self = Self {
        dark: false,
        reduced_motion: false,
        high_contrast: false,
        accent: None,
    };
}

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs
    /// ([`crate::passes`] `PassPipeline::set_observer`), off
    /// otherwise — the disabled trail costs one branch per flip (the
    /// A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side
    /// of the trail, bounded by [`SETTING_TRAIL_CAP`] with
    /// oldest-first eviction, so a fact queued where no pipeline will
    /// ever drain still terminates.
    static TRAIL: RefCell<VecDeque<SettingRecord>> = const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the setting trail (the pipeline's side of the
/// trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_setting_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Notes one setting fact on this thread's trail (the note site's side
/// of the trail, ADR-0080): a no-op while the trail is disabled, an
/// oldest-evicting push while it rides. [`crate::passes`]
/// `PassPipeline::set_settings` calls this at the exact place the flip
/// lands — so a status line, an inspector trail, or a test reads the
/// setting tier without re-implementing the observation at the call
/// site. Public like [`crate::theme::note_theme`]: the note site's own
/// door, priced by the bench corpus (`settings/note_disabled`,
/// `settings/note_enabled_bounded`) and usable by a composition that
/// drives its own setting door outside the pipeline. One fact per flip
/// that landed: a flip that lands where the fact already is queues
/// nothing (the door's idle rule), and the record rides *beside* the
/// door's `bool` answer, never instead of it.
pub fn note_setting(record: SettingRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        tracing::trace!(
            target: "alacer::settings",
            dark = record.dark,
            reduced_motion = record.reduced_motion,
            high_contrast = record.high_contrast,
            accent = ?record.accent,
            repainted = record.repainted,
            "setting flip queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= SETTING_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::Setting`].
pub(crate) fn take_setting_trail() -> Vec<SettingRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::{
        CanonSettings, KioskComposer, SETTING_TRAIL_CAP, SettingComposer, SettingRecord,
        SettingSeam, note_setting, take_setting_trail,
    };
    use crate::color::Color;
    use crate::theme::Palette;

    /// The canon setting pins the canon facts — light, motion allowed,
    /// standard contrast, no accent — the facts the blessed snapshot
    /// corpus was blessed with, byte for byte, the corpus's green
    /// depends on it.
    #[test]
    fn the_canon_setting_pins_the_canon_facts() {
        assert!(!CanonSettings.dark_mode());
        assert!(!CanonSettings.reduced_motion());
        assert!(!CanonSettings.high_contrast());
        assert!(
            CanonSettings.accent_color().is_none(),
            "the canon absence is the honest absence — no guessed accent"
        );
    }

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the theme trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_setting(SettingRecord {
            dark: true,
            reduced_motion: false,
            high_contrast: false,
            accent: None,
            repainted: 0,
        });
        assert!(
            take_setting_trail().is_empty(),
            "no observer, no trail — nothing queued"
        );
    }

    /// The enabled trail queues bounded, oldest-first eviction, and the
    /// drain clears whole: a fact streams once.
    #[test]
    fn the_enabled_trail_queues_bounded_and_drains_once() {
        super::set_setting_trail_enabled(true);
        for i in 0..(SETTING_TRAIL_CAP + 8) {
            note_setting(SettingRecord {
                dark: i % 2 == 0,
                reduced_motion: i % 3 == 0,
                high_contrast: i % 5 == 0,
                accent: None,
                repainted: 1,
            });
        }
        let drained = take_setting_trail();
        assert_eq!(drained.len(), SETTING_TRAIL_CAP, "the cap bounds the trail");
        assert!(
            drained[0].dark,
            "the oldest evicted first (the ninth flip's fact — i=8 is dark)"
        );
        assert!(
            !drained[0].reduced_motion,
            "both facts ride the same record (i=8 allows motion — 8 % 3 != 0)"
        );
        assert!(
            !drained[0].high_contrast,
            "all three facts ride the same record (i=8 allows contrast — 8 % 5 != 0)"
        );
        assert!(
            take_setting_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        super::set_setting_trail_enabled(false);
    }

    /// The kiosk lock answers one palette for every fact permutation —
    /// all sixteen, bit-flipped — the composition that follows nothing,
    /// tagged by its own name (ADR-0089). Const-constructible, the lock
    /// ships beside the palette it holds.
    #[test]
    fn the_kiosk_lock_answers_one_palette_for_every_facts() {
        let locked = Palette {
            accent: Color::rgb(255, 176, 64),
            ..Palette::CANON
        };
        let lock = KioskComposer::locked(locked);
        for bits in 0_u8..16 {
            let dark = bits & 1 != 0;
            let reduced_motion = bits & 2 != 0;
            let high_contrast = bits & 4 != 0;
            let accent = (bits & 8 != 0).then_some(Color::rgb(255, 94, 58));
            assert_eq!(
                SettingComposer::palette(&lock, dark, reduced_motion, high_contrast, accent),
                locked,
                "the lock followed the facts (bits {bits})"
            );
        }
        assert_eq!(SettingComposer::name(&lock), "kiosk");
    }

    /// The lock's answers are total and pure: two resolutions of the
    /// same facts answer the same palette, and two locks hold
    /// different palettes apart (the palette is the lock's state, the
    /// facts never its input).
    #[test]
    fn two_locks_hold_two_palettes_and_one_lock_is_pure() {
        let first = KioskComposer::locked(Palette::CANON);
        let second = KioskComposer::locked(Palette {
            accent: Color::rgb(255, 176, 64),
            ..Palette::CANON
        });
        assert_eq!(
            SettingComposer::palette(&first, true, true, true, Some(Color::rgb(1, 2, 3))),
            SettingComposer::palette(&first, false, false, false, None),
            "the lock's answer moved with the facts"
        );
        assert_ne!(
            SettingComposer::palette(&first, false, false, false, None),
            SettingComposer::palette(&second, false, false, false, None),
            "two locks resolved the same palette"
        );
    }
}
