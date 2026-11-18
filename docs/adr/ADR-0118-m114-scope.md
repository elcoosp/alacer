# ADR-0118: M114 scope — the current tier: the set's current item, said by the tree

- **Status:** Accepted (2026-11-18)
- **Implements:** the word every navigation and wizard vocabulary speaks
  about which item the reader stands in — "this one is the page we're
  on, that step is where we are" — and the tree never said. Through
  M113 the snapshot said where a member sits in its grid (the M6
  table slot's zero-based row and column), what a control *does* (the
  description, ADR-0099), what an empty field *hints* (the placeholder,
  ADR-0100), the span a value rides (the range tier, ADR-0101), the
  form's ask (the required tier, ADR-0110), the form's rejection (the
  invalid tier, ADR-0111), the collection's pick, capacity and
  disclosure (ADR-0112/ADR-0113/ADR-0114), the field's suggestion
  contract (the auto-complete tier, ADR-0115), the sorted column's
  order (the sort direction tier, ADR-0116), the nested member's depth
  (the level tier, ADR-0117) — and the fact the AT turns into "current
  page" while it walks the nav was the one navigation fact the tree
  carried nothing for: the AT announced the name, the position, the
  form facts — and never the currency. The declared-but-unreported gap
  (the M94 pattern), cashed at the current tier: "a consumer needs the
  set's current item to read as exactly that, and as an honest absence
  when the item declares none → the tree carries AccessKit's
  `AriaCurrent` property (ARIA's `aria-current`), the doors answering
  unchanged beside the word — the word prices the word and nothing
  else"
- **Companion:** ADR-2 (the one-currency discipline the word joins —
  the word rides the `A11Y` mask the snapshot pass already pays),
  ADR-0099/ADR-0100 (the a11y-only prop-class tier the word rides —
  no paint, no refusal, one mapping branch), ADR-0116/ADR-0117 (the
  tiers the word grows beside — the kind-mirroring shape the word
  copies: AccessKit's own enum values mirrored so the mapping is one
  branch with no translation, the `SortDirection` precedent at the
  enum tier), ADR-0087 (the presence rule at the tier: `None` the
  honest absence — ARIA's `false`, the attribute's absent default, the
  absence spelled by `None`, never a guessed flavor), ADR-9 (the DAG
  the change never touches), architecture §11 (the a11y contract), §17
  (the register), §18 (the M114 roadmap row)

## Decision

- **One word, the kind beside it.**
  [`Widget::current`](crates/alacer-core/src/widget.rs) answers
  `Option<CurrentKind>` — `None` the honest absence (the ADR-0087
  presence rule at the tier: an item that declared nothing announces
  no currency, never a guessed flavor), the kind
  ([`CurrentKind`](crates/alacer-core/src/widget.rs)) riding beside the
  word: `True` the set's current item with no finer flavor to name,
  `Page` the page the reader stands on, `Step` the step the reader
  stands at, `Location` the place the reader stands in, `Date` the day
  the reader stands on, `Time` the time the reader stands at —
  AccessKit's own six enum values mirrored, ARIA's `false` (the
  attribute's absent default) the absence the `Option` already spells.
  Current is not a wall: a current item's doors answer exactly as any
  item's (the word names the currency, it walls nothing — the item's
  press and focus routes the widget's own), so the tier grows no
  refusal vocabulary and prices no behavior beyond the word itself.
- **The producer joins without a seam change.** The item widget ships
  the full shape —
  [`Button::current`](crates/alacer-widgets/src/button.rs), the builder
  naming the initial currency, beside
  [`Button::set_current`](crates/alacer-widgets/src/button.rs), the
  app's live door through the mutate seam (`None` the honest absence
  restored; the write that lands where the word already is queues
  nothing, the M21 idle-frame property). The word is an item word: the
  containers and collection members answer nothing (a container
  reports its members' words, a row reports its depth — the current
  item's currency is the item's own fact), so no producer joins there.
- **The snapshot carries it; the sink maps it.** `TreeNode::current`
  rides the same pass-9 snapshot every other property rides, and the
  sink maps AccessKit's `AriaCurrent` property — one branch beside the
  level word's (ADR-0117's own mapping), no patch-tier growth (the
  word is runtime state the app's door moves, never a declarable DSL
  word).
- **The living documents re-agree at M114 and 0.17.0.** The ADR count
  (a hundred and nineteen), the architecture register (ADR-0118 scopes
  M114), the roadmap row (§18), the CHANGELOG's unreleased block, and
  the examples catalog (the `current` row) all name the tier; the
  corpus grew 131 → 133 by the coordinated rule below, and the m-suite
  corpus pins re-agree at 133.

## Consequences

- **The corpus grew 131 → 133 by the coordinated rule — the tier's own
  honest pair:** `current/snapshot_1k_currents` — 1,000 buttons, the
  current word's snapshot price — against the same 1,000-node button
  arena bare (`current/snapshot_1k_buttons_bare`, this tier's own
  honest pair; the M95 pricing rule — the word is a button word, so
  the honest comparison is buttons against buttons, the M108/M110
  denominator shape). The `current_snapshot_overhead` ratio budget
  ≤ 1.25× (the word rides the snapshot the button rides, the tier's
  price the one `Option` the node carries beside it). The m-suite pins
  re-agree at 133.
- **The AT reads the currency while it walks the set.** The word is
  the mapping the platform ATs turn into the item's "current page" —
  beside the name and the position the M6 slot already carried, the
  fact a screen reader answers "Home, current page" with no extra
  route, no extra pass. The absence is the contract's other half: an
  item that declared nothing reads as the ordinary button it is, never
  as a guessed flavor, so a plain button does not start announcing a
  currency it does not hold.
- **Consumers get the whole vocabulary for free.** The
  `examples/current.rs` beat is the shape: the nav's page beside the
  wizard's step and the calendar's date, the plain button between
  them, the AT's press landing on the plain button untouched (the word
  names a currency, it walls nothing), the live door moving the word —
  stepping forward, landing, clear — the honest absence restored.
