# ADR-0085: M79 scope — the setting seam grows the contrast fact

- **Status:** Accepted (2026-11-15)
- **Implements:** ADR-0083's first pre-registered re-open rule, cashed a
  second time — "a consumer needs more OS facts than these two (high
  contrast, accent color) → the trait grows the fact beside the two
  (the same one dynamic call per pass, priced by a real reader) — never
  a second seam per fact; the record and the wire grow the field beside
  the facts by this cut's own shape": through M78 a widget that wanted
  the OS's high-contrast preference guessed or polled — the fact the
  vocabulary consults when *contrast* itself is the question (a
  hairline's weight, a focus ring's width, a muted ink raised to
  black) — existed nowhere in the pipeline — the fact rides the seam
  the dark and motion facts already have
  ([`SettingSeam::high_contrast`](crates/alacer-core/src/settings.rs)),
  the door's flip check growing beside it
- **Companion:** ADR-0083 (the motion fact — the cut this one's shape
  copies, the rule it cashes again), ADR-0080 (the setting seam the
  facts ride), ADR-0009 (the adapter the platform owns), ADR-0001 (A1 —
  the one invalidation currency the broadcast rides), ADR-0011 (the
  coordinated rule the corpus grows by), ADR-0010 (the additive wire
  rule — the field grows at version 1), architecture §7.2 pass 8, §18
  (the M79 roadmap row)

## Decision

- **The trait grows the fact beside the two — never a second seam per
  fact.** [`SettingSeam`](crates/alacer-core/src/settings.rs) answers
  `high_contrast()` beside `dark_mode()` and `reduced_motion()`,
  required like the facts it sits beside ("a setting that cannot answer
  the fact is a setting that cannot consult it" — ADR-0080's own
  line): one seam, one door, one record per flip.
  [`CanonSettings`](crates/alacer-core/src/settings.rs) answers the
  canon absence (light, motion allowed, standard contrast) — the facts
  the blessed snapshot corpus was blessed with, the default build's own
  resolution unchanged.
- **The adapter answers the honest absence.** winit reports no
  high-contrast fact — the OS's contrast preference lives below the
  windowing layer's vocabulary — so
  [`WinitSettings`](crates/alacer-platform/src/lib.rs) answers `false`
  where it cannot read: a platform that reports no fact answers the
  absence, never a guess (ADR-0080's adapter rule, the M77 line the
  second read rides beside). An app that reads the fact through its own
  channels installs its own seam through the same door — the demo's
  simulated OS is that app.
- **The door lands the flip when any fact moves.**
  [`PassPipeline::set_settings`](crates/alacer-core/src/passes.rs)'s
  equality check compares all three facts beside each other — the M77
  door's two-fact check would have swallowed a contrast-only flip (the
  fact moved, the door answered `false`, the vocabulary never saw it).
  A flip that lands where all three facts already are answers `false`
  and queues nothing (the idle-frame property, A1's rule at the setting
  tier), and a landed flip broadcasts `Change::PAINT` on the tree — the
  mask's own documented semantics, never a private repaint channel.
- **The fact resolves beside the palette and the other facts, once per
  pass.** [`PaintCtx`](crates/alacer-core/src/scene.rs) carries
  `high_contrast` beside `dark` and `reduced_motion` — one dynamic call
  per pass, never per node, the resolution riding the facts' own
  once-per-pass overhead — and the vocabulary consults it for the
  decisions no palette role answers (a hairline's weight, a focus
  ring's width). The fact never munges the palette: the roles
  re-resolve from the installed palette exactly as they were.
- **The record grows the fact beside the facts it already names.**
  [`SettingRecord`](crates/alacer-core/src/inspect.rs) carries
  `high_contrast` beside `dark`, `reduced_motion`, and `repainted` —
  one record per flip, the facts the vocabulary consults riding beside
  each other, never a second event per fact. The wire grows additively
  at version 1 (ADR-0010): the `setting` event carries `high_contrast`
  beside the facts, the embedded client rendering the fact in the row's
  headline and detail line, the protocol document pinning the field's
  stable meaning.
- **The demo drives the fact from the consumer side.**
  [`examples/high_contrast.rs`](crates/alacer/examples/high_contrast.rs)
  walks the beats — the canon build (light, motion allowed, standard
  contrast), the OS asking for high contrast where the other facts are
  still (the landing the two-fact check swallowed), the second ask the
  honest no-op, the fact restored beside a dark flip — a contrast probe
  painting from `ctx.high_contrast()` directly, the *trail* label
  mirroring what the log streamed.
- **The corpus grows 84 → 85 by the coordinated rule.** The grown
  fact's flip joins the corpus (`settings/door_flip_contrast` — the
  contrast-only flip landing where the other facts are still, all three
  facts resolved where the broadcast resolves, the record streamed, the
  real reader naming the contrast fact) — bench, gate, and baseline in
  the same cut; the m-suite pins re-agree at 85.

## Re-open rules (pre-registered)

- a consumer needs more OS facts than these three (accent color, text
  scale) → the trait grows the fact beside the three (the same one
  dynamic call per pass, priced by a real reader) — never a second seam
  per fact; the record and the wire grow the field beside the facts by
  this cut's own shape.
- a consumer needs the composition to follow the contrast fact (a
  palette that raises with it) → the composer's signature grows beside
  the facts it hands (ADR-0081's own rule), priced by a real reader —
  the pipeline handing the facts the composition asks for, never the
  composer reaching into an adapter.
- a consumer prices the fact's per-pass resolution on the overhead
  ledger → the resolution joins the pass-overhead corpus (the third
  dynamic call riding the facts' own priced calls), the same
  coordinated rule the M77 cut rode.
- a platform grows the read (winit exposing the OS's contrast
  preference) → the adapter maps the fact where the platform knows it,
  the honest absence retired by the read — never a poll loop the
  adapter guesses at.
