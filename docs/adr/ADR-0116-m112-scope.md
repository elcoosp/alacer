# ADR-0116: M112 scope — the sort direction tier: the sorted column's order, said by the tree

- **Status:** Accepted (2026-11-18)
- **Implements:** the word every sorted-column vocabulary speaks about
  the rows it orders — "this one sorts ascending, that one
  descending" — and the tree never said. Through M111 the snapshot said
  what a control *does* (the description, ADR-0099), what an empty
  field *hints* (the placeholder, ADR-0100), the span a value rides
  (the range tier, ADR-0101), the form's ask (the required tier,
  ADR-0110), the form's rejection (the invalid tier, ADR-0111), the
  collection's pick, capacity and disclosure (ADR-0112/ADR-0113/
  ADR-0114), the field's suggestion contract (the auto-complete tier,
  ADR-0115) — and the fact the AT turns into "sorted ascending" while
  it walks the rows was the one sorted-column fact the tree carried
  nothing for: the AT announced the position the M6 slot carried, the
  name, the form facts — and never the order. The
  declared-but-unreported gap (the M94 pattern), cashed at the sort
  direction tier: "a consumer needs the sorted column's order to read
  as exactly that, and as an honest absence when the column declares
  none → the tree carries AccessKit's `SortDirection` property (ARIA's
  `aria-sort`), the doors answering unchanged beside the word — the
  word prices the word and nothing else"
- **Companion:** ADR-2 (the one-currency discipline the word joins —
  the word rides the `A11Y` mask the snapshot pass already pays),
  ADR-0099/ADR-0100 (the a11y-only prop-class tier the word rides —
  no paint, no refusal, one mapping branch), ADR-0115 (the tier the
  word grows beside — the suggestion contract, and now the order; the
  kind-mirroring shape the word copies: ARIA's own values mirrored so
  the mapping is one branch with no translation, the `InvalidKind`
  precedent at the enum tier), ADR-0087 (the presence rule at the
  tier: `None` the honest absence — ARIA's `none` the absence spelled
  by `None`, never a guessed direction), ADR-9 (the DAG the change
  never touches), architecture §11 (the a11y contract), §17 (the
  register), §18 (the M112 roadmap row)

## Decision

- **One word, the kind beside it.**
  [`Widget::sort_direction`](crates/alacer-core/src/widget.rs) answers
  `Option<SortDirection>` — `None` the honest absence (the ADR-0087
  presence rule at the tier: a header that declared nothing announces
  no order, never a guessed direction), the kind
  ([`SortDirection`](crates/alacer-core/src/widget.rs)) riding beside
  the word: `Ascending` the column whose rows rise as the value grows,
  `Descending` the column whose rows fall, `Other` the order ARIA
  names no word for — AccessKit's own three enum values mirrored,
  ARIA's fourth (`none`) the absence the `Option` already spells. Sort
  is not a wall: a sorted column's doors answer exactly as any
  header's (the word names the order, it walls nothing — the column's
  press and focus routes the widget's own), so the tier grows no
  refusal vocabulary and prices no behavior beyond the word itself.
- **The producer joins without a seam change.** The table canon's
  header ships the full shape —
  [`ColumnHeader::sort`](crates/alacer-widgets/src/table.rs), the
  builder naming the initial order, beside
  [`ColumnHeader::set_sort`](crates/alacer-widgets/src/table.rs), the
  app's live door through the mutate seam (`None` the honest absence
  restored; the write that lands where the word already is queues
  nothing, the M21 idle-frame property). The word is a header word:
  the member and value roles answer nothing (a row reports its
  position, not the column's order), so no producer joins there.
- **The snapshot carries it; the sink maps it.**
  `TreeNode::sort_direction` rides the same pass-9 snapshot every
  other property rides, and the sink maps AccessKit's `SortDirection`
  property — one branch beside the auto-complete word's (ADR-0115's
  own mapping), no patch-tier growth (the word is runtime state the
  app's door moves, never a declarable DSL word).
- **The living documents re-agree at M112 and 0.17.0.** The ADR count
  (a hundred and seventeen), the architecture register (ADR-0116
  scopes M112), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `sort` row) all name the tier;
  the corpus grew 128 → 130 by the coordinated rule below, and the
  m-suite corpus pins re-agree at 130.

## Consequences

- **The corpus grew 128 → 130 by the coordinated rule — the tier's own
  honest pair:** `sort/snapshot_1k_sort` — 1,000 column headers, the
  sort word's snapshot price — against the same 1,000-node header
  arena bare (`sort/snapshot_1k_headers_bare`, this tier's own honest
  pair; the M95 pricing rule — the word is a header word, so the
  honest comparison is headers against headers, the M108/M110
  denominator shape). The `sort_snapshot_overhead` ratio budget
  ≤ 1.25× (the word rides the snapshot the header rides, the tier's
  price the one `Option` the node carries beside it). The m-suite pins
  re-agree at 130.
- **The AT reads the order while it walks the rows.** The word is the
  mapping the platform ATs turn into the column's "sorted ascending" —
  beside the position the M6 slot already carried, the fact a screen
  reader answers "column header, sorted descending" with no extra
  route, no extra pass. The absence is the contract's other half: a
  header that declared nothing reads as ARIA's `none`, never as a
  guessed direction, so a plain column does not start announcing an
  order it does not hold.
- **Consumers get the whole vocabulary for free.** The
  `examples/sort.rs` beat is the shape: the ascending column beside
  the descending column and the order ARIA names no word for, the
  plain header between them, the AT's press landing on the button
  beside the sorted columns untouched (the word names an order, it
  walls nothing), the live door moving the word — descending,
  ascending, clear — the honest absence restored.
