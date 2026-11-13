# ADR-0050: M44 scope — the value record shows the clamp

- **Status:** Accepted (2026-11-13)
- **Implements:** ADR-0049's second re-open rule — "a consumer needs the
  *requested* value beside the landed one (a clamp the trail cannot
  show) → grow the record's fields additively (ADR-0010's rule), priced
  by that reader — never a second observation channel"
- **Companion:** ADR-0046 (the value trail), ADR-0049 (the attribution
  whose ask this joins), ADR-0010 (the wire's additive versioning —
  the new field is compatible at version 1), architecture §13.3, §18
  (the M44 roadmap row)

## Decision

- **One additive field, carried where the knowledge lives.**
  `alacer-core/src/inspect.rs`: `ValueRecord::requested` — the finite
  scalar the AT asked before the control's clamp-and-snap moved it.
  Unlike M43's `target` there is no stamp guard: the widget's own door
  computes the ask (the parse for `SetValue`, the step arithmetic for
  the step tier), so the note sites in the `Slider` and the read-only
  `Progress` carry it directly — never a value reconstructed from the
  landing.
- **The absence is the honest answer, per species.** `None` when the
  ask never became a finite scalar: the unparsable payload, the door
  refused before arithmetic (the availability tier, the door-less
  bar), the ask itself non-finite — JSON cannot carry an infinite ask
  honestly, `null` would claim an absence that never was, and the
  refusal species names the wall.
- **The wire grows one field, additively at version 1.**
  `alacer-inspect/src/wire.rs`: the `value` event grows `requested`
  (`null` where the fact is absent); the embedded client renders
  "asked 150, landed 100" as data; `docs/inspector-protocol.md`
  documents the field.
- **The writes are untouched byte-for-byte.** The funnel's
  clamp-and-snap answers as it always answered; the trail's shape, the
  256-bounded queue, the tick's start drain, and the A7 idle contract
  (one branch per write while nobody listens) are unchanged.
- **The corpus stands at 63.** No new hot loop (the coordinated rule's
  no-growth arm); the published `progress` route measures the same
  tier it always measured.
- **The living documents re-agree at the cut.** The README's status
  block, the ADR count (fifty-two), the register count, the protocol
  document's `requested` field, the CHANGELOG (M44 and M45 staged),
  and the m-suite pins moving M43 → M45 and 50 → 52 across the chain.

## Kill criteria (re-open rules)

- A consumer needs the snap's lattice visible separately from the
  clamp → grow the record beside the step arithmetic, priced by a
  real reader — never a second channel.
- A consumer needs the drag's rides on the log → ADR-0051's flood rule
  governs; the pointer route stays off until priced.
