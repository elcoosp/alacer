# ADR-0087: M81 scope — the setting seam grows the accent fact, the composition follows it

- **Status:** Accepted (2026-11-16)
- **Implements:** ADR-0085's own pre-registered re-open rule, cashed a
  third time — "a consumer needs more OS facts than these two (high
  contrast, accent color) → the trait grows the fact beside the two —
  never a second seam per fact; the record and the wire grow the field
  beside the facts by this cut's own shape" — grown by ADR-0086's own
  rule beside it: "a consumer needs the composition to answer a third
  fact beside the three (accent color, text scale) → the signature
  grows the fact beside the three by this cut's own shape, priced by a
  real reader — the pipeline handing the composition what it asks for,
  never the composer reaching into an adapter." Through M80 a widget
  that wanted the OS's accent preference guessed or polled, and a
  composition that wanted to tint by it had to reach for the
  platform's adapter, the reach ADR-0081 forbids — the seam grows the
  fact, and the composition resolves by it at the two points the door
  already owns
  ([`SettingSeam::accent_color`](crates/alacer-core/src/settings.rs),
  [`SettingComposer::palette`](crates/alacer-core/src/settings.rs))
- **Companion:** ADR-0080 (the setting seam the fact joins), ADR-0083
  (the adapter rule the absence rides), ADR-0085 (the fact rule cashed
  the first and second times), ADR-0086 (the composition rule the
  signature grows by), ADR-0001 (A1 — the one invalidation currency
  the flip's broadcast rides), ADR-0011 (the coordinated rule the
  corpus grows by), ADR-0010 (the additive rule — the wire's `accent`
  field at version 1, never a second event per fact), architecture
  §7.2 pass 8, §18 (the M81 roadmap row)

## Decision

- **The seam grows the accent fact beside the three — the fact rides
  its presence.**
  [`SettingSeam::accent_color`](crates/alacer-core/src/settings.rs)
  answers `Option<Color>` — required like the door it rides, one
  dynamic call per frame beside the three bool facts, never per node.
  `None` is the honest absence (ADR-0083's adapter rule): a platform
  that reports no accent answers `None`, never a guessed color.
  [`CanonSettings`](crates/alacer-core/src/settings.rs) answers the
  canon absence — the facts the blessed snapshot corpus was blessed
  with. One seam, one door, never a second seam per fact.
- **The door's equality check compares all four facts.**
  [`PassPipeline::set_settings`](crates/alacer-core/src/passes.rs)
  compares the dark, motion, contrast, and accent answers — the
  accent-only flip (the landing the M79 three-fact check swallowed)
  lands, broadcasts `Change::PAINT` on the tree, and resolves the fact
  once per paint pass into
  [`PaintCtx::accent_color`](crates/alacer-core/src/scene.rs) beside
  the palette and the facts. A flip that lands where all four facts
  already are is the no-op the door answers `false` on (the
  idle-frame property, A1's rule at the setting tier).
- **The record and the wire grow the field beside the facts by this
  cut's own shape.**
  [`SettingRecord`](crates/alacer-core/src/inspect.rs) grows `accent:
  Option<Color>` beside the three bool facts and the reach count; the
  wire's `setting` event grows the additive `accent` field at version
  1 (ADR-0010 — clients that ignore unknown or unexpected fields
  unaffected, never a second event per fact): the documented
  `#RRGGBBAA` hex, `null` the honest absence, the embedded client
  rendering it beside the facts it sits beside, and
  [`docs/inspector-protocol.md`](docs/inspector-protocol.md) pinning
  the field value.
- **The composition resolves by the accent fact beside the three.**
  [`SettingComposer::palette`](crates/alacer-core/src/settings.rs)
  answers `palette(&self, dark, reduced_motion, high_contrast, accent:
  Option<Color>)` — ADR-0086's own shape: the pipeline hands the
  composition what it asks for at the two resolution points the door
  already owned (the install, every landed flip), never the composer
  reaching into an adapter. A palette that tints its accent roles from
  the OS accent answers by the fact; the absence resolves the
  composition's own answer.
  [`CanonComposer`](crates/alacer-core/src/settings.rs) answers the
  canon palette for any facts — behavior unchanged, signature grown.
- **The adapter answers the honest absence where winit reports no
  read.** [`WinitSettings`](crates/alacer-platform/src/lib.rs) answers
  `None` — the OS's accent lives below the windowing layer's
  vocabulary, the same below-the-vocabulary position the motion and
  contrast facts hold (ADR-0083's adapter rule); an app that reads the
  accent through its own channels installs its own seam through the
  same door.
- **The demo drives the fact from the consumer side.**
  [`examples/accent_follow.rs`](crates/alacer/examples/accent_follow.rs)
  installs an *accent-follow* composition — the palette whose accent
  roles tint from the OS accent when the fact is present, its own
  answer when it is not — the fact probe painting the resolved accent
  beside the palette probe, the beats walked before the window opens,
  the log's facts read back on exit.
- **The corpus carries the fact's pricing.** The accent-only flip is a
  new route (the fact the M79 check swallowed, landing where the other
  facts are still) — the corpus grows 85 → 86 by the coordinated rule
  (ADR-0011: bench, gate, and baseline in one cut,
  `settings/door_flip_accent` read end to end through the trail
  reader), the m-suite pins re-agreeing at 86.

## Re-open rules (pre-registered)

- a consumer needs a *fifth* fact the seam does not hold → the fact
  joins the seam first by this cut's own shape (ADR-0085's rule), the
  composition growing beside it only when a real reader prices it —
  never the composer reaching into an adapter.
- a consumer prices the accent flip's own maintenance on the overhead
  ledger → the flip rides the door's own priced walk (the equality
  check beside the three, the shape `door_no_op_flip` already prices),
  the ledger's rows unchanged by the fact's growth.
- a consumer needs the accent's presence logged where no flip lands
  (an install-time read, a poll) → the fact stays the flip record's —
  the door is the fact's one resolution point; a read outside the door
  is the app's own vocabulary, never a second setting tier.
- a platform reports the accent through winit's vocabulary → the
  adapter grows the read beside `from_theme` (ADR-0009 — the adapter
  the platform owns), the seam's shape unchanged.
