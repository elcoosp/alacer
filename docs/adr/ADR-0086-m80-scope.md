# ADR-0086: M80 scope — the composition follows the facts

- **Status:** Accepted (2026-11-15)
- **Implements:** ADR-0083's second pre-registered re-open rule, grown
  by ADR-0085's own cut — "a consumer needs the composition to follow
  the motion fact (a palette that calms with it) → the composer's
  signature grows beside `palette(dark)` only when the fact exists to
  hand (ADR-0081's own rule), priced by a real reader — the pipeline
  handing the facts the composition asks for, never the composer
  reaching into an adapter": through M79 the composition resolved
  against the dark fact alone — a palette that wanted to calm when the
  OS asks for reduced motion, or raise when it asks for high contrast,
  could not answer by the facts without reaching into the platform's
  adapter, the reach ADR-0081 forbids — the composer's signature grows
  the facts beside `dark`
  ([`SettingComposer::palette`](crates/alacer-core/src/settings.rs))
- **Companion:** ADR-0081 (the composition door the signature rides),
  ADR-0083 (the motion fact the first grow pre-registered),
  ADR-0085 (the contrast fact the second grow rides beside),
  ADR-0080 (the setting seam the facts come from), ADR-0001 (A1 — the
  one invalidation currency the re-resolution rides), ADR-0011 (the
  coordinated rule the corpus grows by), ADR-0010 (the additive rule —
  no wire change: the theme tag's vocabulary already carries the
  composer's own `name()`), architecture §7.2 pass 8, §18 (the M80
  roadmap row)

## Decision

- **The composer's signature grows the facts beside `dark` — the
  pipeline hands the composition what it asks for.**
  [`SettingComposer::palette`](crates/alacer-core/src/settings.rs)
  answers `palette(&self, dark, reduced_motion, high_contrast)` — the
  facts the setting seam holds at the composition's own resolution
  points (the install, every landed flip), handed by the pipeline, the
  composer never reaching into an adapter (ADR-0081's own rule, the
  reason the signature grows at the door instead). Required like the
  door it rides: a composer that cannot answer by the facts is a
  composer that cannot follow them. [`CanonComposer`] answers the
  canon palette for any facts — the composition that follows nothing,
  unchanged in behavior, grown in signature.
- **Every landed flip re-resolves the composition against the facts
  that landed.** [`PassPipeline::set_settings`](crates/alacer-core/src/passes.rs)'s
  flip path already re-resolves the composition on the flip's own
  record — the growth is the *inputs*: a motion-only or contrast-only
  flip (the landings the M77 and M79 cuts admitted) now hands those
  facts to the composer, so a palette that calms or raises re-resolves
  when the fact moves, riding the flip's own broadcast (one flip, one
  count — ADR-0084's rule unchanged). A flip whose facts compose the
  palette already installed still re-swaps nothing — the theme tier's
  idle rule guards the swap half, the flip itself still landed.
- **The install resolves against the facts that are.**
  [`PassPipeline::set_composer`](crates/alacer-core/src/passes.rs)'s
  install path reads the installed seam's three facts (the canon facts
  where nothing is installed) and resolves the composed palette the
  way it always did — the install's own broadcast, the effective-
  palette equality, the honest no-op.
- **The wire carries nothing new — the tag names the composition, not
  the palette.** The composed swap streams the composer's own
  [`name()`](crate::settings::SettingComposer::name) tag (ADR-0079's
  naming rule): the M75 *duet* composition already answers two
  palettes under one tag (the canon light, the duet dark), and the
  fact-following composition answers by the facts under its own —
  the log naming which composition resolved, the palette the tree
  paints carrying the fact's resolution, no per-fact tag vocabulary
  (ADR-0010 — no new field, no new event kind).
- **The demo drives the fact-following composition from the consumer
  side.** [`examples/follow_os.rs`](crates/alacer/examples/follow_os.rs)'s
  *Duet* composition grows its calm answer: the dark palette that
  follows the OS's flips resolves its quieter roles when the motion
  fact asks, the composed swap's record naming the composition that
  resolved and the flip's own record naming the fact it followed —
  the composition tier's own story, the M75 demo's beats grown beside
  it.
- **The corpus carries the composition's pricing unchanged.** The
  composition's route is not new — the flip that re-resolves it is the
  flip the corpus already prices (`settings/door_flip_composed`, the
  bench composer grown in signature beside the door), the same-shape
  precedent the a11y echo set ("no new corpus case — the route is not
  new"), the gate re-agreeing against the published baseline in the
  same cut; the m-suite pins re-agree at 85.

## Re-open rules (pre-registered)

- a consumer needs the composition to answer a *third* fact beside the
  three (accent color, text scale) → the signature grows the fact
  beside the three by this cut's own shape, priced where the
  composition resolves — never the composer reaching into an adapter.
- a consumer prices the resolution's own maintenance on the overhead
  ledger → the re-resolution rides the flip's own priced walk (the
  composer call beside the effective-palette equality, the shape
  `composer_install_no_op` and `door_flip_composed` already price),
  the ledger's row unchanged by the signature's growth.
- a consumer needs the composition to follow a fact the seam does not
  hold → the fact joins the seam first (ADR-0083's rule), the
  signature growing beside it in this cut's shape — never the
  composition polling what the seam does not answer.
