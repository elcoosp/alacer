# ADR-0090: M86 scope — the shaped composition lock, the lock with shape

- **Status:** Accepted (2026-11-16)
- **Implements:** ADR-0089's own pre-registered re-open rule cashed:
  "a consumer needs a lock with *shape* (an accent that never moves
  but a surface that follows dark, a motion-locked kiosk beside an
  accent-free one) → the provided-composition rule grows the second
  provided composer beside the lock (the canon and the kiosk are two
  names the §13.3 rule already priced) — never a lock flag the facts
  interpret." Through M85 a palette the OS's facts could not move
  followed nothing at all: [`KioskComposer::locked`]
  (crates/alacer-core/src/settings.rs) answers one palette for every
  fact permutation, and an app's own composition follows every fact by
  how it is written — the between was impossible, so a signage display
  whose surface follows dark while its brand accent never moves had no
  composition to install. The composition tier grows the shaped lock
  ([`ShapedComposer::over`](crates/alacer-core/src/settings.rs) with
  [`FactShape`](crates/alacer-core/src/settings.rs), the second
  provided composer beside the kiosk one)
- **Companion:** ADR-0079 (the theme seam the swap rules ride),
  ADR-0080 (the setting seam the facts land through), ADR-0081 (the
  composition door the lock rides, the equality check the pinned
  fact's idle answer is), ADR-0084 (the reach count the followed
  fact's swap names), ADR-0087 (the accent fact's presence rule the
  pinned absence holds), ADR-0001 (A1 — the one invalidation currency
  the swap's broadcast rides), architecture §7.2 pass 8, §18 (the M86
  roadmap row)

## Decision

- **The shape holds facts, the base answers palettes.** One seam, one
  door: [`SettingComposer`](crates/alacer-core/src/settings.rs) keeps
  its shape, and [`ShapedComposer::over(base, shape)`]
  (crates/alacer-core/src/settings.rs) composes the two the framework
  cannot conflate — the app's own fact → palette mapping rides as the
  base composition (the palettes the framework cannot know), the
  [`FactShape`](crates/alacer-core/src/settings.rs) holds the facts it
  pins, and the composition answers `base.palette(shape.substitute(facts))`:
  a pinned fact is substituted before the base resolves, every
  unpinned fact rides to the base the way it always has. The shape
  builds const from [`FactShape::FOLLOW_ALL`](crates/alacer-core/src/settings.rs)
  one pin at a time (`pin_dark`, `pin_reduced_motion`,
  `pin_high_contrast`, `pin_accent`, `pin_absent_accent` — the honest
  absence pinned, ADR-0087's presence rule held at the shape's door),
  so the lock ships beside the facts it holds, the kiosk lock's const
  precedent.
- **The lock's flip is the equality check's idle answer wherever the
  shape holds.** A flip lands (the facts are the seam's, the door
  answers `true`, one [`SettingRecord`](crates/alacer-core/src/inspect.rs)
  streams), the shape substitutes, the base resolves, and the composed
  answer is the palette the pinned facts already named — the swap the
  equality check refuses (the idle-frame property at the theme tier,
  ADR-0081, through the shape; the theme tier streams no record). Every
  followed fact rides the base's answer the way the composition tier
  always has — the swap lands, one
  [`ThemeRecord`](crates/alacer-core/src/inspect.rs) tagged the
  shape's own name (`"shaped"`) rides the flip's own broadcast, the
  reach beside it (ADR-0084's rule). The composer never reaches into
  the platform — it reads no adapter, holds no window, and the base
  composition is the app's own (ADR-0081's rule, cashed a third time).
- **No pipeline plumbing: the shape is a composition.** The door
  resolves whatever composition is installed, at the install and on
  every landed flip — the shaped lock rides that contract unchanged
  (`PassPipeline::set_composer`, one [`Box<dyn SettingComposer>`]),
  the equality check, the `Change::PAINT` broadcast, and the records
  all the door already owned. The corpus grows 91 → 92 by the
  coordinated rule (ADR-0011: bench, gate, and baseline in one cut,
  `settings/shaped_flip_pinned_no_op` — the flip landing where the
  shape pins the fact that moved, the shape's substitution and the
  base's resolution priced beside `kiosk_flip_no_op`, the idle answer
  the pinned fact buys), and the m-suite pins re-agree at 92.
- **The beats pin the story the pixels tell.** The m86 suite runs the
  real pipeline (A4) with the two-probe shape the pixels price — the
  accent probe the fact the beats pin, the surface probe the fact they
  follow: the install resolves the shape over the facts that are (the
  pinned accent riding the base's light answer); the dark flip lands
  and the surface follows (one theme record tagged `shaped`, the
  accent probe unmoved); the OS's accent flip lands and the accent
  does not move (one setting record streams, the theme tier silent —
  the lock's idle answer through the shape); the install with the dark
  fact already held resolves the dark answer beside the pinned accent;
  and the follow-all shape over the canon composition installs as the
  honest no-op the door answers `false` on (the M75 rule at the
  shape's own door). [`examples/shaped_lock.rs`](crates/alacer/examples/shaped_lock.rs)
  drives the same beats from the consumer side, headless, the probes'
  pixels and the log's tags printed as the demo's lines.

## Re-open rules (pre-registered)

- a consumer needs the shape's answer to tell *which* facts it held
  (a reader pricing a palette against the shape that produced it) →
  the shape is data the composer holds; a record field or a reader
  API naming the shape is priced by a real reader first (ADR-0084's
  rule) — never a record per resolution.
- a consumer needs the shape itself composed (a pin that follows a
  *different* fact's landing — an accent that holds while motion is
  asked) → the shape grows a second-axis pin beside the per-fact
  ones, the substitution algebra it already owns — never a second
  composer type per axis.
- a consumer needs the same shape over a base the OS cannot know
  (the base composition itself swaps at runtime) → the app composes
  through the same door it locked in (`set_composer` with the new
  base shaped — the composition tier's swap-back rule, ADR-0089's
  release rule cashed beside it), never a mutable shape the facts
  interpret.
- a consumer prices the shape's install on the overhead ledger → the
  install rides `composer_install_no_op`'s own priced shape and the
  flip rides `shaped_flip_pinned_no_op`'s, the ledger's rows
  unchanged; a lock whose substitution costs more than four `unwrap_or`s
  and one base call is a lock with a bug.
