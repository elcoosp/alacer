# ADR-0112: M108 scope — the selected tier: the collection's pick, said by the tree

- **Status:** Accepted (2026-11-18)
- **Implements:** the word every collection vocabulary speaks about the
  member — "this one is the pick" — and the tree never said. Through
  M107 the snapshot said where a member sits (the M6 table slot), what
  a control *does* (the description, ADR-0099), what an empty field
  *hints* (the placeholder, ADR-0100), the span a value rides (the
  range tier, ADR-0101), the form's ask (the required tier, ADR-0110),
  the form's rejection (the invalid tier, ADR-0111) — and the fact the
  AT turns into "selected" while the user walks the rows was the one
  collection fact the tree carried nothing for: the AT announced the
  position, the name, the form facts — and never the pick. The
  declared-but-unreported gap (the M94 pattern), cashed at the selected
  tier: "a consumer needs the collection's pick to read as exactly
  that, and as an honest absence when the member belongs to no
  selection → the tree carries AccessKit's `Selected` property (ARIA's
  `aria-selected`), the doors answering unchanged beside the word — the
  word prices the word and nothing else"
- **Companion:** ADR-2 (the one-currency discipline the word joins —
  the word rides the `A11Y` mask the snapshot pass already pays),
  ADR-0099/ADR-0100 (the a11y-only prop-class tier the word rides —
  no paint, no refusal, one mapping branch), ADR-0111 (the tier the
  word grows beside — the form's rejection, and now the collection's
  pick; the constructor-plus-live-door shape the tier copies), ADR-0087
  (the presence rule at the tier, grown one honest step: the tier's
  word is a *trio* — `None` the member of no selection collection, the
  absence ARIA spells `undefined`, never a guessed `false`),
  ADR-9 (the DAG the change never touches), architecture §11 (the a11y
  contract), §17 (the register), §18 (the M108 roadmap row)

## Decision

- **One word, the honest trio beside it.**
  [`Widget::selected`](crates/alacer-core/src/widget.rs) answers
  `Option<bool>` — `None` the honest absence (the ADR-0087 presence
  rule at the tier, grown one step: a selection word that answers
  `false` for every non-member would make `undefined` and "unpicked"
  indistinguishable, the guess the presence rule exists to prevent),
  `Some(false)` the member standing unpicked beside its picked
  siblings, `Some(true)` the pick. Selected is not a wall: a selected
  member's write and focus doors answer exactly as any member's (the
  word names state, it walls nothing — the pick's click, toggle and
  focus routes are the widget's own), so the tier grows no refusal
  vocabulary and prices no behavior beyond the word itself.
- **The producers join without a seam change.** The data-table canon's
  collection members ship the full shape —
  [`TableRow::selected`](crates/alacer-widgets/src/table.rs), the
  builder naming the initial word, beside
  [`TableRow::set_selected`](crates/alacer-widgets/src/table.rs), the
  app's live door through the mutate seam (`None` the honest absence
  restored; the write that lands where the word already is queues
  nothing, the M21 idle-frame property) — with
  [`TableCell::selected`] and [`TableCell::set_selected`] riding beside
  it (the cell tier beside the row tier, the trait door already
  answering both). The word is a member word, never an inherited one:
  a cell that declared no word of its own announces nothing, the row's
  word never bleeds into it.
- **The snapshot carries it; the sink maps it.** `TreeNode::selected`
  rides the same pass-9 snapshot every other property rides, and the
  sink maps AccessKit's `Selected` property — one branch beside the
  invalid word's (ADR-0111's own mapping), no patch-tier growth (the
  word is runtime state the app's door moves, never a declarable DSL
  word).
- **The living documents re-agree at M108 and 0.17.0.** The ADR count
  (a hundred and thirteen), the architecture register (ADR-0112 scopes
  M108), the roadmap row (§18), the CHANGELOG's unreleased block, and
  the examples catalog (the `selected` row) all name the tier; the
  corpus grew 121 → 123 by the coordinated rule below, and the
  m-suite corpus pins re-agree at 123. The campaign also re-agreed the
  stale fragments earlier cuts trailed: the m9 suite's corpus doc
  comment (read "48 criterion cases" since M10) and the "94-case
  corpus" panic messages nine suites carried now read the M108 cut's
  truth — the coordinated rule prices the whole chain, not the newest
  suite.

## Consequences

- **The corpus grew 121 → 123 by the coordinated rule — the tier's own
  honest pair:** `selected/snapshot_1k_selected` — 1,000 data-table
  rows, the pick word's snapshot price — against the same 1,000-node
  row arena bare (`selected/snapshot_1k_rows_bare`, this tier's own
  denominator: the same corpus with the word absent, the M95 pricing
  rule — the word is a member word, so the honest comparison is rows
  against rows, the input arena the earlier tiers reused having no
  member to say the word). The `selected_snapshot_overhead` ratio
  budget ≤ 1.25× (the word rides the snapshot the row rides, the
  tier's price the one `Option<bool>` the node carries beside it).
  The m-suite pins re-agree at 123.
- **The AT reads the pick while the user walks the collection.** The
  word is the mapping the platform ATs turn into the collection's
  "selected" — beside the position the M6 slot already carried, the
  fact a screen reader answers "row three, selected" with no extra
  route, no extra pass. The absence is the contract's other half: a
  member of no selection collection reads as ARIA's `undefined`, never
  as a guessed `false`, so a control that is merely *there* does not
  start announcing itself unpicked.
- **Consumers get the whole vocabulary for free.** The
  `examples/selected.rs` beat is the shape: the picked row beside the
  row standing unpicked and the wordless row, the AT's press landing
  on the button beside the selected table untouched (the word names
  state, it walls nothing), the live door moving the word — pick,
  unpick, clear — the honest absence restored.
