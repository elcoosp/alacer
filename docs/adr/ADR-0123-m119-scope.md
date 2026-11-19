# ADR-0123: M119 scope — the set tier: the member's place and the collection's size, said by the tree

- **Status:** Accepted (2026-11-19)
- **Implements:** the two numbers every collection vocabulary speaks
  about its members and its span — "row 3 of 500" — and the tree
  never carried. Through M118 the snapshot said what a control is read
  as (the role-description tier, ADR-0120), the subtree's updating fact
  (the busy tier, ADR-0121), the routing outcome the harness answered
  as data (ADR-0122), what a control opens (ADR-0119), the set's
  current item (ADR-0118), the nested member's depth (ADR-0117), the
  sorted column's order (ADR-0116), the field's suggestion contract
  (ADR-0115), the member's disclosure (ADR-0114), the container's
  capacity (ADR-0113), the member's pick (ADR-0112) — and the place a
  member stands in the collection's own span, and the span itself, were
  the two collection facts the tree carried nothing for: the AT
  announced the grid position the M6 slot carried (a zero-based row and
  column, a table's geometry), the depth, the name — and never the
  place or the size — the declared-but-unreported gap (the M94
  pattern), cashed at the set tier: "a consumer needs the AT to read
  where a member stands in the collection and how wide the collection
  stands → the tree carries AccessKit's `PositionInSet` property
  (ARIA's `aria-posinset`) and `SizeOfSet` property (ARIA's
  `aria-setsize`), the doors answering unchanged beside the words —
  the words price the words and nothing else"
- **Companion:** ADR-0112 (the member-word tier the place copies — the
  M108 selected shape: a constructor word and a live door on the
  collection member), ADR-0113 (the container-word tier the size
  copies — the M109 multiselectable shape: the word on the collection
  container), ADR-0117 (the level tier the place rides beside — the
  zero-based count the AT reads beside the place, two numbers one
  snapshot apart), ADR-0087 (the presence rule at the tier: `None` the
  honest absence — a member that declared no place and a collection
  that declared no size announce neither, never a guessed `0`), ADR-2
  (the one-currency discipline the words join — the words ride the
  `A11Y` mask the snapshot pass already pays), ADR-9 (the DAG the
  change never touches), architecture §11 (the a11y contract), §17
  (the register), §18 (the M119 roadmap row)

## Decision

- **Two words, one tier, `None` the honest absence.**
  [`Widget::position_in_set`](crates/alacer-core/src/widget.rs)
  answers `Option<usize>` and
  [`Widget::size_of_set`](crates/alacer-core/src/widget.rs) answers
  `Option<usize>` — `None` the honest absence (the ADR-0087 presence
  rule at the tier, the level shape: a member that declared no place
  and a collection that declared no size announce neither, never a
  guessed count), the words riding beside the collection machinery
  they name: the AT reads "row 3 of 500" off the same snapshot every
  other fact rides. The words price the words and nothing else: a
  collection's doors answer exactly as they always did — the words
  name a place and a span, they wall nothing, the rows' and the
  table's press and focus routes the widgets' own (a tree that refused
  on the words' behalf would price a contract it does not own).
- **AccessKit's own contracts ride — the two differences the
  documentation carries, one branch with no translation.** AccessKit
  names the size's placement verbatim ("`aria-setsize` is set on each
  item, while this property is set on the container") — so the size
  word is the container's ([`Table::size_of_set`](crates/alacer-widgets/src/table.rs),
  the M109 shape) and the place word the member's
  ([`TableRow::position_in_set`](crates/alacer-widgets/src/table.rs),
  the M108 shape); AccessKit names the place's base verbatim
  ("`aria-posinset` is one-based, while this property is zero-based") —
  so the trait answers the zero-based count AccessKit's property maps,
  the base difference the trait's documentation carries, never a
  silent cast.
- **The producers join without a seam change.** The collection
  vocabulary ships the full shape — [`TableRow::position_in_set`](crates/alacer-widgets/src/table.rs),
  the builder naming the member's place from the first frame, beside
  [`TableRow::set_position_in_set`](crates/alacer-widgets/src/table.rs),
  the app's live door through the mutate seam (`None` restoring the
  honest absence — the row left the set; the write that lands where
  the word already is queues nothing, the M21 idle-frame property) —
  and [`Table::size_of_set`](crates/alacer-widgets/src/table.rs) beside
  [`Table::set_size_of_set`](crates/alacer-widgets/src/table.rs)
  joining by the M107 rule: the collection's span joins without a seam
  change, its doors the doors they always were. The words are
  collection words: the plain widgets answer nothing (a button reports
  its press, a field its value — the place and the span are the
  collection's own), so no other producer joins there.
- **The snapshot carries them; the sink maps them.**
  `TreeNode::position_in_set` and `TreeNode::size_of_set` ride the
  same pass-9 snapshot every other property rides, and the sink maps
  AccessKit's `SizeOfSet` and `PositionInSet` properties — one branch
  pair beside the level word's (ADR-0117's own mapping), no patch-tier
  growth (the words are runtime state the app's doors move, never
  declarable DSL words).
- **The living documents re-agree at M119 and 0.17.0.** The ADR count
  (a hundred and twenty-five), the architecture register (ADR-0123
  scopes M119), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `set` row) all name the tier;
  the corpus grew 137 → 139 by the coordinated rule below, and the
  m-suite corpus pins re-agree at 139.

## Consequences

- **The corpus grew 137 → 139 by the coordinated rule — the M117
  precedent's own shape:** `set/snapshot_1k_rows_set` — 1,000 rows
  under a table declaring the span, every row carrying its place (the
  place cycling down the rows, the way a collection lays them) —
  against the same shapes bare (`set/snapshot_1k_rows_bare`, the
  tier's own honest pair; the M95 pricing rule — the words are
  collection words, so the honest comparison is rows against rows, the
  M117 denominator shape). The `set_snapshot_overhead` ratio budget
  ≤ 1.25× (the words ride the snapshot the nodes ride, the tier's
  price the two `Option`s the nodes carry beside them). The m-suite
  pins re-agree at 139.
- **The published contracts re-agree with the corpus.** The M119 cut
  re-publishes the baseline and dashboard with the set route's pair
  (the M5 bijection: every case gated, every gate measured) — and
  re-states two contracts the published data could no longer hold: the
  range and numeric snapshot budgets (M97/M99's ≤ 1.25×) measured
  1.31–1.33× the minimal arena on this host, the same violation the
  committed data already carried — the gate re-states at 1.35×, the
  measured ratio with headroom, the value-tier words' snapshot price
  being the value work the snapshot rides, not the words' own. The
  honest re-measurement is the coordinated rule's own work: the
  baseline is a snapshot, regenerated, never hand-tuned.
- **The AT reads "row 3 of 500" beside the facts it already read.**
  The words are the mapping the platform ATs turn into position
  announcements — beside the depth the level tier carried and the grid
  slot the M6 canon carried, a screen reader walking a virtualized
  collection reads the member's place and the collection's span off
  the tree. The absence is the contract's other half: a collection
  that claims no size and a row that claims no place announce
  neither, never a guessed count, so a bare table does not start
  carrying positions it does not own.
- **Consumers get the whole vocabulary for free.** The
  `examples/set.rs` beat is the shape: the sized ledger and its placed
  rows beside the unsized archive and the plain widgets, every door
  answering as it always did, the live doors moving the words — the
  ledger's third row taking its place, the archive starting its own
  count, clear — the honest absences restored.
