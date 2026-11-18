# ADR-0114: M110 scope — the expanded tier: the collection member's disclosure, said by the tree

- **Status:** Accepted (2026-11-18)
- **Implements:** the word every collection vocabulary speaks about the
  member that opens and closes — "this one is expanded, that one
  collapsed" — and the tree never said. Through M109 the snapshot said
  which member the collection picked (the selected tier, ADR-0112)
  beside the capacity the multiselectable tier named (ADR-0113), the
  position the M6 slot carried, the form's ask and the form's rejection
  beside them — and the fact the AT turns into "expanded" / "collapsed"
  while the user walks the rows was the one collection fact the tree
  carried nothing for: the AT announced the picks, the positions, the
  names — and never the disclosure. The declared-but-unreported gap
  (the M94 pattern), cashed at the expanded tier: "a consumer needs the
  collection member's disclosure to read as exactly that, and as an
  honest absence when the member opens and closes not → the tree
  carries AccessKit's `Expanded` property (ARIA's `aria-expanded`), the
  doors answering unchanged beside the word — the word prices the word
  and nothing else"
- **Companion:** ADR-2 (the one-currency discipline the word joins —
  the word rides the `A11Y` mask the snapshot pass already pays),
  ADR-0099/ADR-0100 (the a11y-only prop-class tier the word rides —
  no paint, no refusal, one mapping branch), ADR-0112 (the tier the
  word grows beside — the pick, and now the disclosure; the honest
  trio the word copies: `None` the member that opens and closes not,
  the absence ARIA spells `undefined`, never a guessed `false`),
  ADR-0087 (the presence rule at the tier, the selected shape),
  ADR-9 (the DAG the change never touches), architecture §11 (the a11y
  contract), §17 (the register), §18 (the M110 roadmap row)

## Decision

- **One word, the honest trio beside it.**
  [`Widget::expanded`](crates/alacer-core/src/widget.rs) answers
  `Option<bool>` — `None` the honest absence (the ADR-0087 presence
  rule at the expanded tier, the selected shape: a disclosure word
  that answered `false` for every non-member would make `undefined`
  and "collapsed" indistinguishable, the guess the presence rule
  exists to prevent), `Some(false)` the member standing collapsed,
  `Some(true)` the member standing open. AccessKit's own contract
  names the mapping verbatim ("setting this to `false` means the node
  is collapsed; omitting it means this state isn't applicable"), so
  the sink's mapping is one branch with no translation. Expanded is
  not a wall: an expanded member's doors answer exactly as any
  member's (the word names state, it walls nothing — the open and
  close routes are the widget's own), so the tier grows no refusal
  vocabulary and prices no behavior beyond the word itself.
- **The producer joins without a seam change.** The data-table canon's
  collection member ships the full shape —
  [`TableRow::expanded`](crates/alacer-widgets/src/table.rs), the
  builder naming the initial word, beside
  [`TableRow::set_expanded`](crates/alacer-widgets/src/table.rs), the
  app's live door through the mutate seam (`None` the honest absence
  restored; the write that lands where the word already is queues
  nothing, the M21 idle-frame property). The word is a member word,
  never an inherited one: the cells beside it answer nothing (the
  disclosure is the row's), and a `Table` — the container — carries no
  disclosure to say, so no producer joins there.
- **The snapshot carries it; the sink maps it.**
  `TreeNode::expanded` rides the same pass-9 snapshot every other
  property rides, and the sink maps AccessKit's `Expanded` property —
  one branch beside the multiselectable word's (ADR-0113's own
  mapping), no patch-tier growth (the word is runtime state the app's
  door moves, never a declarable DSL word).
- **The living documents re-agree at M110 and 0.17.0.** The ADR count
  (a hundred and fifteen), the architecture register (ADR-0114 scopes
  M110), the roadmap row (§18), the CHANGELOG's unreleased block, and
  the examples catalog (the `expanded` row) all name the tier; the
  corpus grew 125 → 127 by the coordinated rule below, and the
  m-suite corpus pins re-agree at 127. The campaign also re-agreed the
  stale fragments earlier cuts trailed: the README's tier-vocabulary
  table (four rows the M105–M109 campaigns left behind — read-only,
  required, selected, multiselectable) and its tier paragraph now read
  the M110 cut's truth — the coordinated rule prices the whole chain,
  not the newest suite.

## Consequences

- **The corpus grew 125 → 127 by the coordinated rule — the tier's own
  honest pair:** `expanded/snapshot_1k_expanded` — 1,000 data-table
  rows, the disclosure word's snapshot price — against the same
  1,000-node row arena bare (`expanded/snapshot_1k_rows_bare`, this
  tier's own denominator: the same corpus with the word absent, the
  M95 pricing rule — the word is a member word, so the honest
  comparison is rows against rows, the selected tier's own shape). The
  `expanded_snapshot_overhead` ratio budget ≤ 1.25× (the word rides
  the snapshot the row rides, the tier's price the one `Option<bool>`
  the node carries beside it). The m-suite pins re-agree at 127.
- **The AT reads the disclosure while the user walks the collection.**
  The word is the mapping the platform ATs turn into the member's
  "expanded" / "collapsed" — beside the picks the selected tier
  already carried, the fact a screen reader answers "Archive, expanded"
  with no extra route, no extra pass. The absence is the contract's
  other half: a member that opens and closes not reads as ARIA's
  `undefined`, never as a guessed `false`, so a row that is merely
  *there* does not start announcing itself collapsed.
- **Consumers get the whole vocabulary for free.** The
  `examples/expanded.rs` beat is the shape: the row standing open
  beside the row standing collapsed and the wordless row, the AT's
  press landing on the button beside the expanded table untouched
  (the word names state, it walls nothing), the live door moving the
  word — open, collapse, clear — the honest absence restored.
