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
//! installed theme the roles borrow. The fact never munges the palette
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
//! [`crate::inspect::SettingRecord`] per flip that landed — the bool
//! *is* the fact, and since M76 (ADR-0082) the broadcast's node count
//! rides beside it (the lengths rule — the count names the flip's
//! reach, the ids never riding) — the theme trail's shape (ADR-0079),
//! bounded, drained at the tick's start, silent while nobody listens
//! (the A7 idle contract).
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
/// never polls — the seam is the fact's one installed source.
pub trait SettingSeam: Send + Sync + 'static {
    /// The OS's dark-mode fact, re-read once per paint pass — one
    /// dynamic call per frame, never per node, the palette
    /// resolution's shape.
    fn dark_mode(&self) -> bool;
}

/// The provided canon setting (§13.3, the provided-implementation rule
/// beside every seam — [`crate::theme::CanonPalette`]'s precedent):
/// light, always — the fact the blessed snapshot corpus was blessed
/// with, installable by name so a consumer that swapped away can swap
/// back through the same door it swapped in. Installing it where
/// nothing is installed (or where light is installed) is the no-op the
/// door answers `false` on: the default build already resolves light.
#[derive(Debug, Clone, Copy, Default)]
pub struct CanonSettings;

impl SettingSeam for CanonSettings {
    fn dark_mode(&self) -> bool {
        false
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
/// composed [`Palette`] against the fact that *is* at install time and
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
    /// The roles the palette resolves for the fact `dark` names —
    /// called once per install and once per landed flip (never per
    /// frame: the composed palette installs as a [`crate::theme::ThemeSeam`]
    /// the paint pass resolves in its own once-per-pass shape). The
    /// equality check prices the value it answered: a flip that
    /// composes the palette already installed re-swaps nothing.
    fn palette(&self, dark: bool) -> Palette;

    /// The composed palette's tag — the stable name the causal log's
    /// [`crate::inspect::ThemeRecord`] carries for swaps this composer
    /// resolved (the theme trail's naming rule, ADR-0079): `""`-free,
    /// app-defined (`"duet"`).
    fn name(&self) -> &'static str;
}

/// The provided canon composition (§13.3, the provided-implementation
/// rule beside every seam — [`CanonSettings`]'s precedent): the canon
/// palette for either fact — the composition that follows nothing,
/// installable by name so a consumer that composed away can compose
/// back through the same door it composed in. Installing it where the
/// canon palette is already live (the default build's own answer, or
/// the canon theme) is the no-op the door answers `false` on.
#[derive(Debug, Clone, Copy, Default)]
pub struct CanonComposer;

impl SettingComposer for CanonComposer {
    fn palette(&self, _dark: bool) -> Palette {
        Palette::CANON
    }

    fn name(&self) -> &'static str {
        "canon"
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
        CanonSettings, SETTING_TRAIL_CAP, SettingRecord, SettingSeam, note_setting,
        take_setting_trail,
    };

    /// The canon setting pins light — the fact the blessed snapshot
    /// corpus was blessed with, byte for byte, the corpus's green
    /// depends on it.
    #[test]
    fn the_canon_setting_pins_light() {
        assert!(!CanonSettings.dark_mode());
    }

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the theme trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_setting(SettingRecord {
            dark: true,
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
            take_setting_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        super::set_setting_trail_enabled(false);
    }
}
