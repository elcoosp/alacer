# ADR-0089: M85 scope — the kiosk lock, the composition that follows nothing

- **Status:** Accepted (2026-11-16)
- **Implements:** ADR-0081's own pre-registered re-open rule cashed
  beside the provided-implementation rule (§13.3): "a consumer needs
  the composition to answer a *shape* the facts cannot move → the
  door grows the provided composition beside the canon one, priced by
  a real reader — never the composer reaching into an adapter."
  Through M84 a palette the OS's facts could not move had two
  half-doors: the manual
  [`PassPipeline::set_theme`](crates/alacer-core/src/passes.rs) ask,
  which wins *until the next flip* (the composition owns the palette
  when the OS moves — the M75 rule), and
  [`CanonComposer`](crates/alacer-core/src/settings.rs), which locks
  the *canon* palette — light, motion-allowing — and no other. A
  kiosk, a signage display, an embedded panel cannot follow the OS: a
  dark flip at 7pm must not repaint the boarding gate, and the
  operator's desktop accent must not tint the checkout line — so the
  composition tier grows the lock over any palette
  ([`KioskComposer::locked`](crates/alacer-core/src/settings.rs), the
  provided implementation beside the canon one)
- **Companion:** ADR-0079 (the theme seam the swap rules ride),
  ADR-0080 (the setting seam the facts land through), ADR-0081 (the
  composition door the lock installs through, the equality check the
  lock's answer rides), ADR-0084 (the reach count the win-back's swap
  names), ADR-0001 (A1 — the one invalidation currency the flip's
  broadcast rides), ADR-0011 (the coordinated rule the corpus grows
  by), architecture §7.2 pass 8, §13.3, §18 (the M85 roadmap row)

## Decision

- **The lock is a provided composition, not a new seam.** One seam,
  one door: [`SettingComposer`](crates/alacer-core/src/settings.rs)
  keeps its shape, and
  [`KioskComposer::locked(palette)`](crates/alacer-core/src/settings.rs)
  — const, so the lock ships beside the palette it holds — answers
  that one [`Palette`](crates/alacer-core/src/theme.rs) for every fact
  permutation, all sixteen bit-flips pinned by the settings tier's own
  unit test. The facts are the seam's (the flip lands, the door
  answers `true`, one
  [`SettingRecord`](crates/alacer-core/src/inspect.rs) streams); the
  palette is the lock's. The composer never reaches into the
  platform — it reads no adapter, holds no window (ADR-0081's own
  rule, cashed a second time).
- **The lock's flip is the equality check's idle answer.** The
  composed answer *is* the palette already installed, so a landed flip
  re-swaps nothing — the theme tier streams no record, the flip's own
  broadcast carries the facts' repaint and nothing else (the
  idle-frame property at the theme tier, ADR-0081). The corpus grows
  90 → 91 by the coordinated rule (ADR-0011: bench, gate, and
  baseline in one cut, `settings/kiosk_flip_no_op` — the door's fact
  check beside the composition's resolution beside the refused swap,
  priced between `door_no_op_flip` and `door_flip_composed` where the
  route's cost lives).
- **The manual door wins while it holds; the next flip is where the
  lock wins it back.** The app's own `set_theme` ask lands over the
  composed answer exactly as the M75 rule has it — and on the next
  landed flip the composition resolves again, the answer is the locked
  palette, the swap rides the flip's own broadcast, and the
  [`ThemeRecord`](crates/alacer-core/src/inspect.rs) is tagged the
  composer's own name (`"kiosk"`), the reach beside it (ADR-0084's
  rule). A kiosk that must re-assert its look after any operator
  override gets that as the flip's own consequence, never a
  re-assertion poll.
- **The beats pin the story the pixels tell.** The m85 suite runs the
  real pipeline (A4) with the role probe the m80 shape prices: the
  install resolves the lock over the facts that are (the dark fact
  already held changes nothing — the lock is the resolution, not a
  fact-driven answer it then abandons); the dark flip lands, the
  palette holds, one setting record streams and no theme record beside
  it; the manual ask lands and the probe paints it; the next flip
  wins the lock back, one theme record tagged `kiosk`; and the lock
  over the canon palette itself installs as the honest no-op the door
  answers `false` on (the M75 rule at the lock's own door).
  [`examples/kiosk.rs`](crates/alacer/examples/kiosk.rs) drives the
  same three beats from the consumer side, headless, the probe's
  pixels and the log's records printed as the demo's lines.

## Re-open rules (pre-registered)

- a consumer needs a lock with *shape* (an accent that never moves but
  a surface that follows dark, a motion-locked kiosk beside an
  accent-free one) → the provided-composition rule grows the second
  provided composer beside the lock (the canon and the kiosk are two
  names the §13.3 rule already priced) — never a lock flag the
  facts interpret.
- a consumer needs the lock's refusal to *tell* (which flip asked,
  which facts moved) → the setting tier already streams the fact's
  record; a theme-tier record for a refused swap is the equality
  check's idle answer grown a voice — priced by a real reader first
  (ADR-0084's rule), never a record per idle frame.
- a consumer prices the lock's install on the overhead ledger → the
  install rides `composer_install_no_op`'s own priced shape (the
  composer call beside the effective-palette equality), the ledger's
  rows unchanged; a lock whose composition costs more than a copy of
  one palette is a lock with a bug.
- a consumer needs the lock released (an exit-kiosk flow) → the app
  composes back through the same door it locked in (`set_composer`
  with its own composition, or the canon composer by name — the
  provided rule's swap-back), never a lock flag.
