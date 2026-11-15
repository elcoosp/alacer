# ADR-0081: M75 scope — the composition door

- **Status:** Accepted (2026-11-15)
- **Implements:** the palette that *follows* the OS's dark-mode fact —
  ADR-0080's pre-registered re-open rule cashes ("a consumer needs the
  palette to *follow* the OS fact → the pipeline grows the composition
  door, the installed palette re-swapped on the flip's own record,
  priced by a real reader — never a palette that reaches into the
  platform's adapter"): through M74 an app that wanted palette-
  following subscribed to its own OS events and called `set_theme`
  beside every flip it watched — the composition installs as one
  [`SettingComposer`](crates/alacer-core/src/settings.rs)
  (`PassPipeline::set_composer`) and the pipeline does the re-swap
  itself, the theme tier's own swap rules riding unchanged
- **Companion:** ADR-0080 (the setting seam whose flips resolve the
  composition — the pre-registered re-open rule this cut cashes),
  ADR-0079 (the theme seam the composed palette mounts into), ADR-0001
  (A1 — the one invalidation currency the install's broadcast rides),
  ADR-0011 (the coordinated rule the corpus grows by), architecture
  §7.2 pass 8, §18 (the M75 roadmap row)

## Decision

- **The composition is one
  [`SettingComposer`](crates/alacer-core/src/settings.rs) the pipeline
  owns.** The trait answers a [`Palette`](crates/alacer-core/src/theme.rs)
  for the fact the pipeline hands it plus the tag the theme trail
  carries — a pure function: the composer reads no platform state,
  holds no window, never reaches into an adapter (the re-open rule's
  own bound). `Send + Sync` like the seams it sits between.
  [`CanonComposer`](crates/alacer-core/src/settings.rs) is the provided
  implementation: the canon palette for either fact — the composition
  that follows nothing, installable by name so a consumer that
  composed away can compose back through the same door it composed
  in.
- **The install resolves now.**
  [`PassPipeline::set_composer`](crates/alacer-core/src/passes.rs)
  resolves the composed palette against the fact that *is* (the
  installed setting seam's, or the canon light fact where nothing is
  installed) and swaps it in the way `set_theme` swaps — the
  `PartialEq` idle check against the *effective* palette (the canon
  constants where no theme is installed, the paint pass's own
  resolution), one [`ThemeRecord`](crates/alacer-core/src/inspect.rs)
  tagged with the composer's name, `Change::PAINT` on the tree. An
  install that composes the palette already live answers `false` and
  queues nothing — the canon composition over the canon default *is*
  that no-op — and the composer still rides for the next flip.
- **The flip resolves the composition.** Every flip that lands through
  `PassPipeline::set_settings` re-resolves the installed composer —
  the composed palette mounting as the installed theme the roles
  borrow, riding the flip's own broadcast (no second walk over the
  arena, no second currency). A flip that composes the palette already
  installed re-swaps nothing: the swap half is the no-op (the idle
  rule at the theme tier), the flip itself still landed (the fact
  moved, the broadcast repaints). The composed swap streams one
  `InspectEvent::Theme` tagged with the composer's name — behind the
  flip's own `InspectEvent::Setting` at the tick's start drain (the
  M74 ordering: the fact first, the palette that fact painted into
  second).
- **The manual door wins while the fact is still.** The app's own
  `set_theme` ask lands over a composed palette and holds — the
  composition owns the palette only when the OS moves, the next flip
  re-resolving over whatever the manual door installed. No uninstall
  door: installing [`CanonComposer`](crates/alacer-core/src/settings.rs)
  by name composes nothing further (the canon answer for either fact
  is the canon palette the default build resolves — the swap half
  idles), the swap-back path the provided-implementation rule keeps
  beside every seam.
- **The demo drives the door from the consumer side.**
  [`examples/follow_os.rs`](crates/alacer/examples/follow_os.rs) walks
  the beats — the *Duet* composition installing over the canon default
  (the honest no-op), the OS flipping dark and the composed dark
  palette landing beside the fact, the second dark ask the honest
  no-op, light landing beside them — a role probe painting from
  `ctx.palette().accent` (the palette the widgets borrow, the pixel
  evidence the composition moved it), a fact probe painting from
  `ctx.dark()` directly (the fact beside it, never munging the
  palette), the *trail* label mirroring what the log streamed.
- **The corpus grows 79 → 81 by the coordinated rule.** The
  composition tier's own costs join the corpus
  (`settings/composer_install_no_op` — the composer's resolution
  beside the effective-palette equality, the no-op install's whole
  cost; `settings/door_flip_composed` — the fact that lands, the
  composer's resolution, the composed palette's mount, the real reader
  the re-open rule priced) — bench, gate, and baseline in the same
  cut; the m-suite pins re-agree at 81.

## Re-open rules (pre-registered)

- a consumer prices *which nodes* the flip repainted → the setting
  record grows the broadcast's node count beside the fact (the lengths
  rule — the ids never ride), priced by a real reader (ADR-0080's own
  second rule, still open).
- a consumer needs more OS facts than dark mode → the trait grows the
  fact beside `dark_mode` (the same one dynamic call per pass, priced
  by a real reader) — never a second seam per fact; the composition
  reads the facts the pipeline hands it, the composer's signature
  growing beside `palette(dark)` only when a fact exists to hand.
- a consumer prices the composition's per-pass resolution on the
  overhead ledger → the composed palette's own once-per-pass shape is
  already the theme seam's (the paint pass resolves the *installed*
  theme; the composition moved the installation, never the
  resolution), and the resolution joins the pass-overhead corpus by
  the same coordinated rule this cut rode.
- a consumer needs the composition to follow more than the OS (a
  time-of-day tint, a remote config) → the composer is the app's own
  code: it may read whatever it owns inside `palette(dark)` — the
  pipeline's contract is the fact hand-off, never the composer's
  inputs; the door grows only when the *fact* tier grows.
