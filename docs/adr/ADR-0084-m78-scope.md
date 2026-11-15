# ADR-0084: M78 scope — the swap names its repaint count

- **Status:** Accepted (2026-11-15)
- **Implements:** ADR-0082's second pre-registered re-open rule
  reaching the theme tier — "a consumer needs the count on other
  tiers' records → each tier's record grows its own lengths beside its
  fact by its own coordinated cut, never a shared side-channel":
  through M77 a swap's [`ThemeRecord`](crates/alacer-core/src/inspect.rs)
  named the tag alone while the flip's `SettingRecord` beside it
  carried its count (the M76 cut) — the asymmetry the two doors' rows
  show side by side on one log — the record grows the broadcast's node
  count beside the tag, the log answering "which palette is live now,
  when did it move, and how much did the swap repaint?" from one row
- **Companion:** ADR-0082 (the lengths rule this cut carries to the
  theme tier), ADR-0079 (the theme seam the record rides), ADR-0081
  (the composition door whose swaps stream beside the counted
  installs), ADR-0001 (A1 — the one invalidation currency the
  broadcast rides), ADR-0011 (the coordinated rule the corpus grows
  by), ADR-0010 (the additive wire rule — the field grows at version
  1), architecture §7.2 pass 8, §18 (the M78 roadmap row)

## Decision

- **The record grows the count beside the tag — the lengths rule.**
  [`ThemeRecord`](crates/alacer-core/src/inspect.rs) carries
  `repainted: usize` beside `palette`: how many nodes the swap's
  `Change::PAINT` merge touched, computed where the swap's broadcast
  resolves ([`PassPipeline::set_theme`](crates/alacer-core/src/passes.rs)'s
  own pre-order walk — the walk's length *is* the swap's reach, never
  re-counted at the reader), never the ids (the payload rule's shape:
  a count is not an identity). The count is the fact of the swap's own
  moment: a tree that grows later never rewrites a queued record. The
  no-op swap queues no record — there is no count to name, the
  idle-frame property unchanged (A1's rule at the theme tier).
- **One flip, one count — the composed swap names the flip's reach.**
  On a landed flip the composed palette mounts riding the flip's own
  broadcast (ADR-0081: no second walk over the arena), so the composed
  swap's record names the same count the flip's `SettingRecord`
  carries — one flip, one count, however many palettes the composition
  resolved (ADR-0082's companion rule). An install that composes
  outside a flip (`PassPipeline::set_composer`) walks its own
  broadcast and names that walk's length — the count always the reach
  of the broadcast the record's own moment rode.
- **The wire grows additively at version 1.** The `theme` event
  carries `repainted` beside `palette` (ADR-0010's additive field rule
  — clients that ignore unknown fields are unaffected), the embedded
  client rendering the count in the row's detail line ("the swap
  repainted N nodes"), the protocol document pinning the field's
  stable meaning.
- **The demo names the reach.**
  [`examples/theme_swap.rs`](crates/alacer/examples/theme_swap.rs)'s
  exit read-back prints the count beside the tag — the consumer-side
  shape the m78 suite pins.
- **The corpus grows 83 → 84 by the coordinated rule.** The counted
  swap read end to end joins the corpus (`theme/door_swap_counted` —
  the swap that lands with its broadcast count, the tick's start drain
  streaming it into the trail reader, the reader naming the count
  beside the tag — the real reader the lengths rule priced) — bench,
  gate, and baseline in the same cut; the m-suite pins re-agree at 84.

## Re-open rules (pre-registered)

- a consumer prices the count on a *third* tier's record (the action
  tier's aim, the clipboard's round trip) → that tier's record grows
  its own lengths by its own coordinated cut (this cut's shape), never
  a shared side-channel — the M72 four-counts precedent at the view
  tier.
- a consumer prices the count's own maintenance → the count is the
  broadcast walk's length collected where the walk already runs: zero
  added work on the door path beyond the record field, priced by
  `theme/door_swap_counted` beside the door's own cases.
- a consumer prices *which nodes* the swap repainted (the per-node
  ask) → the counts stay counts (the payload rule — the ids never
  ride); the ask is the pass-overhead corpus's per-node pricing, the
  record's shape untouched by it.
