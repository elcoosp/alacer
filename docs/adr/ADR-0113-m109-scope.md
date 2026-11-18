# ADR-0113: M109 scope — the multiselectable tier: the collection's capacity, said by the tree

- **Status:** Accepted (2026-11-18)
- **Implements:** the word every collection vocabulary speaks about the
  container — "this one allows more than one pick" — and the tree never
  said. Through M108 the snapshot said which member the collection
  picked (the selected tier, ADR-0112) beside the position the M6 slot
  carried, the form's ask and the form's rejection beside them — and
  the fact the AT turns into "you can pick more than one" before the
  user ever reaches the rows was the one collection fact the tree
  carried nothing for: the AT announced the picks, the positions, the
  names — and never the capacity. The declared-but-unreported gap (the
  M94 pattern), cashed at the multiselectable tier: "a consumer needs
  the collection's capacity to read as exactly that → the tree carries
  AccessKit's `Multiselectable` property (ARIA's `aria-multiselectable`),
  the doors answering unchanged beside the word — the word prices the
  word and nothing else"
- **Companion:** ADR-2 (the one-currency discipline the word joins —
  the word rides the `A11Y` mask the snapshot pass already pays),
  ADR-0099/ADR-0100 (the a11y-only prop-class tier the word rides —
  no paint, no refusal, one mapping branch), ADR-0112 (the tier the
  word grows beside — the member's pick, and now the container's
  capacity; the constructor-plus-live-door shape the tier copies),
  ADR-0087 (the presence rule at the tier, the read-only shape: `false`
  the honest absence), ADR-9 (the DAG the change never touches),
  architecture §11 (the a11y contract), §17 (the register), §18 (the
  M109 roadmap row)

## Decision

- **One word, one flag.**
  [`Widget::multiselectable`](crates/alacer-core/src/widget.rs) answers
  `bool` — `false` the honest absence (the ADR-0087 presence rule at
  the tier, the read-only shape: a container word with no third state
  to protect — the member word's trio exists because `undefined` and
  "unpicked" must stay distinguishable; the container word carries no
  such ambiguity, so the flag is the whole truth). Multiselectable is
  not a wall: a multiselectable container's doors answer exactly as
  any container's (the word names capacity, it walls nothing — the
  members' press, toggle and focus routes are the widgets' own), so
  the tier grows no refusal vocabulary and prices no behavior beyond
  the word itself.
- **The producer joins without a seam change.** The data-table canon's
  collection container ships the full shape —
  [`Table::multiselectable`](crates/alacer-widgets/src/table.rs), the
  builder naming the capacity, beside
  [`Table::set_multiselectable`](crates/alacer-widgets/src/table.rs),
  the app's live door through the mutate seam (`false` the honest
  absence restored; the write that lands where the word already is
  queues nothing, the M21 idle-frame property). The word is a container
  word, never a member or widget word: the rows and cells beside it
  answer nothing (the ADR-0112 trio is theirs), and a `List` — the
  tree says `ScrollArea` for it — carries no collection role to size,
  so no producer joins there.
- **The snapshot carries it; the sink maps it.**
  `TreeNode::multiselectable` rides the same pass-9 snapshot every
  other property rides, and the sink maps AccessKit's `Multiselectable`
  property — one branch beside the selected word's (ADR-0112's own
  mapping), no patch-tier growth (the word is runtime state the app's
  door moves, never a declarable DSL word).
- **The living documents re-agree at M109 and 0.17.0.** The ADR count
  (a hundred and fourteen), the architecture register (ADR-0113 scopes
  M109), the roadmap row (§18), the CHANGELOG's unreleased block, and
  the examples catalog (the `multiselectable` row) all name the tier;
  the corpus grew 123 → 125 by the coordinated rule below, and the
  m-suite corpus pins re-agree at 125.

## Consequences

- **The corpus grew 123 → 125 by the coordinated rule — the tier's own
  honest pair:** `multiselectable/snapshot_1k_multiselectable` — 1,000
  data-table containers, the capacity word's snapshot price — against
  the same 1,000-node table arena bare
  (`multiselectable/snapshot_1k_tables_bare`, this tier's own
  denominator: the same corpus with the word absent, the M95 pricing
  rule — the word is a container word, so the honest comparison is
  tables against tables). The `multiselectable_snapshot_overhead`
  ratio budget ≤ 1.25× (the word rides the snapshot the container
  rides, the tier's price the one bool the node carries beside it).
  The m-suite pins re-agree at 125.
- **The AT reads the capacity before the user reaches the rows.** The
  word is the mapping the platform ATs turn into the collection's
  "multiple" — beside the member picks the selected tier already
  carried, the fact a screen reader answers "multi-select" on entering
  the collection with no extra route, no extra pass. The absence is
  the contract's other half: a container that declared nothing reads
  as ARIA's absent `aria-multiselectable`, never as a guessed
  capacity — a single-pick table does not start announcing itself
  multi-pick.
- **Consumers get the whole vocabulary for free.** The
  `examples/multiselectable.rs` beat is the shape: the multi-pick
  table (its first row the pick, the two tiers' words beside each
  other in one snapshot) beside the single-pick table, the AT's press
  landing on the button beside the multiselectable table untouched
  (the word names capacity, it walls nothing), the live door moving
  the word — declare, clear — the honest absence restored.
