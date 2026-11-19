# ADR-0121: M117 scope — the busy tier: the subtree's updating fact, said by the tree

- **Status:** Accepted (2026-11-19)
- **Implements:** the word every bulk-update vocabulary speaks
  about the region under construction — "this list is still
  loading, hold what you would announce" — and the tree never
  said. Through M116 the snapshot said what a control is read as
  (the role-description tier, ADR-0120), what it opens (ADR-0119),
  the set's current item (ADR-0118), the nested member's depth
  (ADR-0117), the sorted column's order (ADR-0116), the field's
  suggestion contract (ADR-0115), the member's disclosure
  (ADR-0114), the container's capacity (ADR-0113), the member's
  pick (ADR-0112), the form's rejection (ADR-0111), the form's ask
  (ADR-0110) — and the fact the AT defers its speech on while the
  stream writes the subtree was the one updating fact the tree
  carried nothing for: the AT announced every half-settled
  intermediate the stream wrote in, even where the app knew the
  region was still filling. The declared-but-unreported gap (the
  M94 pattern), cashed at the busy tier: "a consumer needs the AT
  to hold what it would announce while a region settles → the tree
  carries AccessKit's `Busy` property (ARIA's `aria-busy`), the
  doors answering unchanged beside the word — the word prices the
  word and nothing else"
- **Companion:** ADR-0024 (the announcement seam the word paces —
  the reserved live-region node §11.6 reserves is the *imperative*
  half, the app pushing one announcement; the busy word is the
  *declared* half, the app saying which region is settling: the
  counterpart, never a second channel), ADR-2 (the one-currency
  discipline the word joins — the word rides the `A11Y` mask the
  snapshot pass already pays), ADR-0113 (the container-word tier
  the word copies — the M109 multiselectable shape: a plain bool on
  the containers, the members and plain widgets answering nothing),
  ADR-0087 (the presence rule at the tier: `false` the honest
  absence — the container that declared nothing announces nothing,
  never a guessed "updating"), ADR-9 (the DAG the change never
  touches), architecture §11 (the a11y contract), §17 (the
  register), §18 (the M117 roadmap row)

## Decision

- **One word, `false` the honest absence.**
  [`Widget::busy`](crates/alacer-core/src/widget.rs) answers `bool`
  — `false` the honest absence (the ADR-0087 presence rule at the
  tier, the multiselectable shape: a container that declared nothing
  announces nothing, never a guessed "updating"), the word riding
  beside the announcement machinery it paces: the AT holds what it
  would announce while the region settles, and speaks what it held
  when the word clears. The word is the declared counterpart of the
  reserved region — the app says *which* subtree is being updated;
  it never silences the tree's routes (a tree that refused on the
  word's behalf would price a contract it does not own — the AT
  defers its own speech, it is never the tree's to silence).
- **The producers join without a seam change.** The collection
  containers ship the full shape —
  [`List::busy`](crates/alacer-widgets/src/list.rs), the builder
  naming the initial updating fact (the scroller the feed fills),
  beside [`List::set_busy`](crates/alacer-widgets/src/list.rs), the
  app's live door through the mutate seam (`false` the honest
  absence restored — the bulk update finished, the AT speaks; the
  write that lands where the word already is queues nothing, the
  M21 idle-frame property) — and
  [`Table::busy`](crates/alacer-widgets/src/table.rs) joining by the
  M107 rule: the streaming ledger joins without a seam change, its
  doors the doors they always were. The word is a container word:
  the members and plain widgets answer nothing (a row reports its
  depth, a button its press — the updating fact is the region's own),
  so no producer joins there.
- **The snapshot carries it; the sink maps it.**
  `TreeNode::busy` rides the same pass-9 snapshot every other
  property rides, and the sink maps AccessKit's `Busy` property —
  one branch beside the multiselectable word's (ADR-0113's own
  mapping), no patch-tier growth (the word is runtime state the
  app's door moves, never a declarable DSL word).
- **The living documents re-agree at M117 and 0.17.0.** The ADR count
  (a hundred and twenty-three), the architecture register (ADR-0121
  scopes M117), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `busy` row) all name the
  tier; the corpus grew 135 → 137 by the coordinated rule below, and
  the m-suite corpus pins re-agree at 137.

## Consequences

- **The corpus grew 135 → 137 by the coordinated rule — the M109
  precedent's own shape:** `busy/snapshot_1k_lists_busy` — 1,000
  lists, the updating word's snapshot price (cycling truthfully down
  the stream, the way the app declares it) — against the same
  1,000-node list arena bare (`busy/snapshot_1k_lists_bare`, the
  tier's own honest pair; the M95 pricing rule — the word is a
  container word, so the honest comparison is lists against lists,
  the M108/M109 denominator shape). The `busy_snapshot_overhead`
  ratio budget ≤ 1.25× (the word rides the snapshot the container
  rides, the tier's price the one bool the node carries beside it).
  The m-suite pins re-agree at 137.
- **The AT holds its speech while the region settles.** The word is
  the mapping the platform ATs turn into "wait" — beside the
  announcement region the framework already reserves, the fact a
  screen reader streaming a feed does not read every half-settled
  row aloud. The absence is the contract's other half: a container
  that claims no updating announces nothing, never a guessed wait,
  so a settled list does not start withholding announcements it does
  not hold.
- **Consumers get the whole vocabulary for free.** The
  `examples/busy.rs` beat is the shape: the loading feed and the
  streaming ledger under the word, the settled archive and the plain
  scroller the honest absence between them, every door answering as
  it always did, the live door moving the word — the feed finishing,
  the archive starting its own refresh, clear — the honest absence
  restored.
