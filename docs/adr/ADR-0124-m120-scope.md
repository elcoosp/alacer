# ADR-0124: M120 scope — the live tier: the declared live region, said by the tree

- **Status:** Accepted (2026-11-19)
- **Implements:** the word every streaming vocabulary speaks about the
  region whose changes the AT speaks — "this region is live, announce
  what lands here" — and the tree never carried for a region the app
  *declared*. Through M119 the snapshot said the member's place and
  the collection's size (the set tier, ADR-0123), what a control is
  read as (ADR-0120), the subtree's updating fact (ADR-0121), the
  routing outcome the harness answered as data (ADR-0122), the
  collection facts, the form facts — and the pipeline's own reserved
  region (§11.6, ADR-0024) carried the imperative half alone: the
  framework reserved the node, the app pushed announcements through
  it, and a region the *app* wanted the AT to watch had no word — the
  AT spoke what the framework's channel pushed and never what the app
  declared — the declared-but-unreported gap (the M94 pattern), cashed
  at the live tier: "a consumer needs the AT to watch a region the app
  names → the tree carries AccessKit's `Live` property (ARIA's
  `aria-live`), the politeness the announcement seam's own vocabulary
  carries, the doors answering unchanged beside the word — the word
  prices the word and nothing else"
- **Companion:** ADR-0024 (the announcement seam the word extends —
  the reserved region §11.6 builds is the *imperative* half, the app
  pushing one announcement through the framework's channel; the live
  word is the *declarative* half, the app saying which region the AT
  watches: the counterpart, never a second channel), ADR-0022 (the
  `Politeness` vocabulary the word rides — the same polite/assertive
  pair the imperative act asks with, the tier reusing the seam's own
  enum, never a second politeness type), ADR-0021 (the busy tier the
  word sits beside — busy withholds what the AT would announce while
  a region settles; live names the region the AT watches as it
  streams; the two words answer the two halves of the streaming
  contract on one snapshot), ADR-0087 (the presence rule at the tier:
  `None` the honest absence — a container that declared nothing
  announces no region, never a guessed politeness), ADR-2 (the
  one-currency discipline the word joins — the word rides the `A11Y`
  mask the snapshot pass already pays), ADR-9 (the DAG the change
  never touches), architecture §11 (the a11y contract), §17 (the
  register), §18 (the M120 roadmap row)

## Decision

- **One word, `None` the honest absence.**
  [`Widget::live`](crates/alacer-core/src/widget.rs) answers
  `Option<Politeness>` — the politeness the announcement seam's own
  vocabulary (ADR-0022's pair, `Polite` the AT waits its turn,
  `Assertive` it interrupts), `None` the honest absence (the ADR-0087
  presence rule at the tier, the busy shape: a container that declared
  nothing announces no region, never a guessed politeness), the word
  riding beside the announcement machinery it extends: the AT watches
  the region the app named and speaks what lands there, while the
  imperative channel the framework owns keeps the contract it always
  had. The word prices the word and nothing else: a declared region's
  doors answer exactly as they always did — the word names the region
  the AT watches, it walls nothing, the containers' press and focus
  routes the widgets' own (a tree that refused on the word's behalf
  would price a contract it does not own — the AT decides what it
  speaks, it is never the tree's to silence).
- **The producers join without a seam change.** The container
  vocabulary ships the full shape —
  [`List::live`](crates/alacer-widgets/src/list.rs), the builder naming
  the declared region from the first frame (the feed the AT watches),
  beside [`List::set_live`](crates/alacer-widgets/src/list.rs), the
  app's live door through the mutate seam (`None` restoring the honest
  absence — the region withdrawn; the write that lands where the word
  already is queues nothing, the M21 idle-frame property) — and
  [`Table::live`](crates/alacer-widgets/src/table.rs) beside
  [`Table::set_live`](crates/alacer-widgets/src/table.rs) joining by
  the M107 rule: the streaming ledger joins without a seam change, its
  doors the doors they always were. The word is a container word: the
  members and plain widgets answer nothing (a row reports its depth, a
  button its press — the watched region is the container's own), so no
  producer joins there.
- **The snapshot carries it; the sink maps it — one field, two
  riders.** `TreeNode::live` has carried the pipeline's reserved
  region's politeness since M18; the tier grows the same field to
  carry the app's declared regions beside it (`slot.widget().live()`
  the pass-9 fill), and the sink's existing mapping serves both — the
  `Live` property riding the same pass-9 snapshot every other property
  rides. One contract re-states where the pairing lives: the
  Status/Alert role pairing (polite speaks as "status", assertive
  interrupts as "alert") is the pipeline's own region's — the role its
  announcement contract names — so a container that *declares* the
  word keeps the role it already was (the word names the region the AT
  watches, it re-roles nothing). No patch-tier growth (the word is
  runtime state the app's door moves, never a declarable DSL word).
- **The living documents re-agree at M120 and 0.17.0.** The ADR count
  (a hundred and twenty-five), the architecture register (ADR-0124
  scopes M120), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `live` row) all name the tier;
  the corpus grew 139 → 140 by the coordinated rule below, and the
  m-suite corpus pins re-agree at 140.

## Consequences

- **The corpus grew 139 → 140 by the coordinated rule — the M113
  precedent's own shape:** `live/snapshot_1k_lists_live` — 1,000
  lists, the declared region's snapshot price (the politeness cycling
  down the stream, the way a feed's regions alternate) — against the
  busy route's published bare arena (`busy/snapshot_1k_lists_bare`,
  the M117 case the tier reuses by the M111 precedent; the M95 pricing
  rule — the word is a container word, so the honest comparison is
  lists against lists, the M108/M109 denominator shape). The
  `live_snapshot_overhead` ratio budget ≤ 1.25× (the word rides the
  snapshot the container rides, the tier's price the one `Option` the
  node carries beside it). The m-suite pins re-agree at 140.
- **The AT watches what the app names.** The word is the mapping the
  platform ATs turn into "speak what lands here" — beside the
  imperative region the framework reserved, a screen reader streaming
  a feed speaks the rows the app declared live and holds what it does
  not. The absence is the contract's other half: a container that
  claims no region announces none, never a guessed watch, so a settled
  table does not start carrying a region it does not own.
- **Consumers get the whole vocabulary for free.** The
  `examples/live.rs` beat is the shape: the polite feed and the
  assertive ledger under the word, the settled archive and the plain
  scroller the honest absence between them, every door answering as it
  always did, the live door moving the word — the feed's region
  withdrawn, the archive declaring its own, clear — the honest absence
  restored.
