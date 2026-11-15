# ADR-0083: M77 scope — the setting seam grows the motion fact

- **Status:** Accepted (2026-11-15)
- **Implements:** ADR-0080's first pre-registered re-open rule — "a
  consumer needs more OS facts than dark mode (reduced motion, high
  contrast, accent color) → the trait grows the fact beside
  `dark_mode` (the same one dynamic call per pass, priced by a real
  reader) — never a second seam per fact": through M76 a widget that
  wanted the OS's motion preference guessed or polled — the fact the
  vocabulary consults when motion itself is the question (a spinner's
  spin, a thumbnail's autoplay) existed nowhere in the pipeline — the
  fact rides the seam the dark fact already has
  ([`SettingSeam::reduced_motion`](crates/alacer-core/src/settings.rs)),
  the door's flip check growing beside it
- **Companion:** ADR-0080 (the setting seam the fact rides — the
  pre-registered re-open rule this cut cashes), ADR-0009 (the adapter
  the platform owns), ADR-0001 (A1 — the one invalidation currency the
  broadcast rides), ADR-0011 (the coordinated rule the corpus grows
  by), ADR-0010 (the additive wire rule — the field grows at version
  1), architecture §7.2 pass 8, §18 (the M77 roadmap row)

## Decision

- **The trait grows the fact beside `dark_mode` — never a second seam
  per fact.** [`SettingSeam`](crates/alacer-core/src/settings.rs)
  answers `reduced_motion()` beside `dark_mode()`, required like the
  dark fact it sits beside ("a setting that cannot answer the fact is
  a setting that cannot consult it" — ADR-0080's own line): one seam,
  one door, one record per flip. [`CanonSettings`](crates/alacer-core/src/settings.rs)
  answers the canon absence (light, motion allowed) — the facts the
  blessed snapshot corpus was blessed with, the default build's own
  resolution unchanged.
- **The adapter answers the honest absence.** winit reports no
  reduced-motion fact — the OS's motion preference lives below the
  windowing layer's vocabulary — so
  [`WinitSettings`](crates/alacer-platform/src/lib.rs) answers `false`
  where it cannot read: a platform that reports no fact answers the
  absence, never a guess (ADR-0080's adapter rule). An app that reads
  the fact through its own channels installs its own seam through the
  same door — the demo's simulated OS is that app.
- **The door lands the flip when either fact moves.**
  [`PassPipeline::set_settings`](crates/alacer-core/src/passes.rs)'s
  equality check compares both facts beside each other — the M74
  door's dark-only check would have swallowed a motion-only flip (the
  fact moved, the door answered `false`, the vocabulary never saw it).
  A flip that lands where both facts already are answers `false` and
  queues nothing (the idle-frame property, A1's rule at the setting
  tier), and a landed flip broadcasts `Change::PAINT` on the tree —
  the mask's own documented semantics, never a private repaint
  channel.
- **The fact resolves beside the palette and the dark fact, once per
  pass.** [`PaintCtx`](crates/alacer-core/src/scene.rs) carries
  `reduced_motion` beside `dark` — one dynamic call per pass, never
  per node, the resolution riding the dark fact's own once-per-pass
  overhead — and the vocabulary consults it for the decisions no
  palette role answers. The fact never munges the palette: the roles
  re-resolve from the installed palette exactly as they were.
- **The record grows the fact beside the facts it already names.**
  [`SettingRecord`](crates/alacer-core/src/inspect.rs) carries
  `reduced_motion` beside `dark` and `repainted` — one record per
  flip, the facts the vocabulary consults riding beside each other,
  never a second event per fact. The wire grows additively at version
  1 (ADR-0010): the `setting` event carries `reduced_motion` beside
  `dark`, the embedded client rendering the fact in the row's headline
  and detail line, the protocol document pinning the field's stable
  meaning.
- **The demo drives the fact from the consumer side.**
  [`examples/reduced_motion.rs`](crates/alacer/examples/reduced_motion.rs)
  walks the beats — the canon build (light, motion allowed), the OS
  asking for reduced motion where the dark fact is still (the landing
  the dark-only check swallowed), the second ask the honest no-op, the
  fact restored beside a dark flip — a motion probe painting from
  `ctx.reduced_motion()` directly, the *trail* label mirroring what
  the log streamed.
- **The corpus grows 82 → 83 by the coordinated rule.** The grown
  fact's flip joins the corpus (`settings/door_flip_motion` — the
  motion-only flip landing where the dark fact is still, both facts
  resolved where the broadcast resolves, the record streamed, the real
  reader naming the motion fact) — bench, gate, and baseline in the
  same cut; the m-suite pins re-agree at 83.

## Re-open rules (pre-registered)

- a consumer needs more OS facts than these two (high contrast, accent
  color) → the trait grows the fact beside the two (the same one
  dynamic call per pass, priced by a real reader) — never a second
  seam per fact; the record and the wire grow the field beside the
  facts by this cut's own shape.
- a consumer needs the composition to follow the motion fact (a
  palette that calms with it) → the composer's signature grows beside
  `palette(dark)` only when the fact exists to hand (ADR-0081's own
  rule), priced by a real reader — the pipeline handing the facts the
  composition asks for, never the composer reaching into an adapter.
- a consumer prices the fact's per-pass resolution on the overhead
  ledger → the resolution joins the pass-overhead corpus (the second
  dynamic call riding the dark fact's own priced call), the same
  coordinated rule this cut rode.
- a platform grows the read (winit exposing the OS's motion
  preference) → the adapter maps the fact where the platform knows it,
  the honest absence retired by the read — never a poll loop the
  adapter guesses at.
