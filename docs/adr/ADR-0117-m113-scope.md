# ADR-0117: M113 scope — the level tier: the nested member's depth, said by the tree

- **Status:** Accepted (2026-11-18)
- **Implements:** the word every nested-collection vocabulary speaks
  about how deep a member stands — "this one is a level-2 row under
  the level-1 one" — and the tree never said. Through M112 the snapshot
  said where a member sits in its grid (the M6 table slot's zero-based
  row and column), what a control *does* (the description, ADR-0099),
  what an empty field *hints* (the placeholder, ADR-0100), the span a
  value rides (the range tier, ADR-0101), the form's ask (the required
  tier, ADR-0110), the form's rejection (the invalid tier, ADR-0111),
  the collection's pick, capacity and disclosure (ADR-0112/ADR-0113/
  ADR-0114), the field's suggestion contract (the auto-complete tier,
  ADR-0115), the sorted column's order (the sort direction tier,
  ADR-0116) — and the fact the AT turns into "level 2" while it walks
  the rows was the one nested-collection fact the tree carried nothing
  for: the AT announced the grid position, the name, the form facts —
  and never the depth. The declared-but-unreported gap (the M94
  pattern), cashed at the level tier: "a consumer needs the nested
  member's depth to read as exactly that, and as an honest absence
  when the member declares none → the tree carries AccessKit's `Level`
  property (zero-based, beside ARIA's one-based `aria-level`), the
  doors answering unchanged beside the word — the word prices the word
  and nothing else"
- **Companion:** ADR-2 (the one-currency discipline the word joins —
  the word rides the `A11Y` mask the snapshot pass already pays),
  ADR-0099/ADR-0100 (the a11y-only prop-class tier the word rides —
  no paint, no refusal, one mapping branch), ADR-0116 (the tier the
  word grows beside — the sorted column's order, and now the member's
  depth), ADR-0087 (the presence rule at the tier: `None` the honest
  absence — the flat row the absence keeps, never a guessed `0`),
  ADR-9 (the DAG the change never touches), architecture §11 (the a11y
  contract), §17 (the register), §18 (the M113 roadmap row)

## Decision

- **One word, the zero-based count beside it.**
  [`Widget::level`](crates/alacer-core/src/widget.rs) answers
  `Option<usize>` — `None` the honest absence (the ADR-0087 presence
  rule at the tier: a member that declared nothing announces no depth,
  never a guessed `0`), the count riding the word: `0` the root-level
  member, `1` the member under it, `2` the member under that —
  AccessKit's own base, the contract naming it verbatim ("this
  property is zero-based", beside ARIA's one-based `aria-level` the
  property maps), so the mapping is one branch with no translation and
  the base difference is the documentation's, never a silent cast.
  Level is not a wall: a nested member's doors answer exactly as any
  member's (the word names the depth, it walls nothing — the member's
  press and focus routes the widget's own), so the tier grows no
  refusal vocabulary and prices no behavior beyond the word itself.
- **The producer joins without a seam change.** The table canon's
  member ships the full shape —
  [`TableRow::level`](crates/alacer-widgets/src/table.rs), the builder
  naming the initial depth, beside
  [`TableRow::set_level`](crates/alacer-widgets/src/table.rs), the
  app's live door through the mutate seam (`None` the honest absence
  restored, the flat row; the write that lands where the word already
  is queues nothing, the M21 idle-frame property). The word is a
  member word: the headers beside it answer nothing (a header reports
  the column's order, not the row's depth), so no producer joins
  there.
- **The snapshot carries it; the sink maps it.** `TreeNode::level`
  rides the same pass-9 snapshot every other property rides, and the
  sink maps AccessKit's `Level` property — one branch beside the sort
  word's (ADR-0116's own mapping), no patch-tier growth (the word is
  runtime state the app's door moves, never a declarable DSL word).
- **The living documents re-agree at M113 and 0.17.0.** The ADR count
  (a hundred and eighteen), the architecture register (ADR-0117 scopes
  M113), the roadmap row (§18), the CHANGELOG's unreleased block, and
  the examples catalog (the `level` row) all name the tier; the corpus
  grew 130 → 131 by the coordinated rule below, and the m-suite corpus
  pins re-agree at 131.

## Consequences

- **The corpus grew 130 → 131 by the coordinated rule — the M111
  precedent's reuse:** `level/snapshot_1k_level` — 1,000 data-table
  rows, the depth word's snapshot price — against the same 1,000-node
  row arena bare (`expanded/snapshot_1k_rows_bare`, the M110 case the
  tier reuses by the M111 precedent; the M95 pricing rule — the word
  is a row word, so the honest comparison is rows against rows, the
  M108/M110 denominator shape). The `level_snapshot_overhead` ratio
  budget ≤ 1.25× (the word rides the snapshot the row rides, the
  tier's price the one `Option` the node carries beside it). The
  m-suite pins re-agree at 131.
- **The AT reads the depth while it walks the rows.** The word is the
  mapping the platform ATs turn into the member's "level 2" — beside
  the grid position the M6 slot already carried, the fact a screen
  reader answers "row, level 2" with no extra route, no extra pass.
  The absence is the contract's other half: a member that declared
  nothing reads as the flat row it is, never as a guessed `0`, so a
  plain row does not start announcing a nesting it does not hold.
- **Consumers get the whole vocabulary for free.** The
  `examples/level.rs` beat is the shape: the root-level row beside the
  level-1 row and the level-2 row, the flat row between them, the AT's
  press landing on the button beside the nested rows untouched (the
  word names a depth, it walls nothing), the live door moving the word
  — deeper, higher, clear — the honest absence restored.
