# ADR-0029: M23 scope — the frame's ledger (the journal's own stats seam, the repaired scroll trio, the micro-debt the sweep priced)

- **Status:** Accepted (2026-11-10)
- **Implements:** the observability candidate the M22 cut left
  undemonstrated — the scene journal, the M2 headline whose economics
  every other seam spends, is the last major seam without a consumer
  demonstration, and its introspection accessors were dead code
  precisely because no consumer existed — plus the surface work the
  M16–M22 pattern leaves open: the blessed trio that blessed one image
  three times, the 0.14.0 release's living-documents remainder, and the
  weak errors the ADR-0025 species still had left
- **Companion:** ADR-0008 (the journal replay the ledger accounts),
  ADR-0011 (the budgets-as-contracts discipline the ledger makes
  legible), ADR-0022 (the observability idiom the example rides),
  ADR-0025 (the diagnostic species the macro remainder joins),
  ADR-0028 (the cut whose sweep this one continues), architecture §9.1
  (the journal the seam exposes), §18 (the M23 roadmap row)

## Decision

Every seam since M16 paired with a consumer demonstration, and every one
of those demonstrations made its seam's dead accessors alive or priced
them dead. The scene journal skipped both: `SceneJournalImpl` shipped
`commits()`, `drops()`, `layer_count()`, and a slice-view `layer()`
duplicate with no caller anywhere — dead introspection the M21/M22 sweeps
could not see because the accessors looked like surface, not debt. M23
makes the journal legible the way the other seams became legible:

- **The journal reports its own ledger.** `SceneJournal::stats()` is a
  required trait method answering `JournalStats { commits, drops,
  live_layers }` — the frame's cost as the journal itself accounts it.
  `PassPipeline::journal_stats()` is the pipeline's one read door; the
  consumer never touches journal storage. A journal that keeps no books
  answers the zero value — the honest default, documented at the struct.
- **The demonstration walks the beats.** `examples/frame_ledger.rs`
  mounts a static label and a bound ticker, then reads the ledger at the
  two beats that matter: the mount frame (every layer commits once) and
  one ticker write batch (exactly one layer re-commits; the static
  layer's clean pixels replay from the journal untouched). In the window,
  the *Tick* press rides the same one-batch route, and the ledger rides
  the `alacer::frame_ledger` tracing target — the workspace's per-seam
  idiom, `RUST_LOG=alacer=info` to watch. The m23 exit suite pins the
  delta headlessly: one write batch ⇒ one commit, no churn, no shape
  change, idle frames write nothing.
- **The scroll trio's brokenness is repaired at the source.** The m1
  corpus's `list_scrolled_120` and `list_scrolled_400` scenarios never
  called `set_content_height`, so `scroll_to`'s clamp folded every offset
  to zero — and three distinct scroll positions blessed one byte-identical
  image (the `chore: bless broken pngs` commit blessed the brokenness
  honestly, and every re-bless reproduced it). The shared scenario helper
  now derives the content height from the laid-out children (the m0
  pattern's app-side duty), and `list_scrolled_120` grows to twelve
  panels so its 120 px target is inside the clamp's real range. The next
  bless run re-blesses the trio distinct — CONTRIBUTING's
  diff-is-the-review rule — and the m23 suite pins the three images
  pairwise-distinct so the failure mode cannot return silently.
- **The living documents re-agree at 0.14.0.** The 0.14.0 release
  bumped `[workspace.package]` and nothing else: the README's badge and
  status block, both migration guides' family line, and the m-suite's
  own version pins stayed at 0.13.0 — the M20/M21 pattern's remainder,
  back again at the release seam. The README, the catalog, the
  architecture register (28 → 29 with this ADR), the roadmap, the
  migration guides, the widgets manifest's shipped vocabulary (the
  slider and progress the M16 discipline demands the manifest name), the
  facade's shipped-surface rustdoc, and this changelog move with the cut,
  and the m20/m21/m22 pins move with them in the same commit — the chain
  cannot drift.
- **The micro-debt the sweep priced rides the cut.** The journal's dead
  inherent accessors go — replaced by the seam that gives their data a
  caller. The todo canon's `footer_node` getter goes with its write-only
  field. The `rsx!` body-cardinality error names its cardinality ("found
  0" is a missing element, "found 2" a collision) with the miette-class
  `help:` the ADR-0025 species ships — as plain diagnostic text, because
  the workspace's no-new-dependency rule outranks adopting the miette
  crate for strings the compiler already renders. The two no-op `let _`
  discards in the macro's `for` branch go with it. And the inspector's
  accept loop reports the session errors its `let _ = serve_connection(…)`
  swallowed silently, through the adopted `tracing` core — the one
  dependency edge the cut adds, an intra-workspace edge to a crate every
  other member already resolves.

## Consequences

- The ledger is a read seam, not a write channel: nothing in the paint
  pass consults it, no mask names it, and the hot path carries no new
  work — the counters already existed; the seam only lets a consumer read
  them.
- The corpus does not grow: the demonstration shows no new visual
  arrangement (the catalog's own rule for when an example needs a
  scenario), and the m1 suite's 37-scenario pin stands. The repaired trio
  re-blesses in place at the next bless run.
- The perf contracts do not grow either: a ledger case joins at a
  measurement run, by the coordinated rule, if a milestone ever prices
  one — CONTRIBUTING's publishing rule keeps the checker's uncontracted
  failure ahead of any half-published bench.
- The m-suite's version pins now live in three suites; the next release
  moves all of them together or the m23 suite fails loudly.

## Kill criteria (re-open rules)

- A consumer needs the ledger per frame on the hot path → that is the
  damage report's job; grow `FrameReport`, never a second channel.
- A journal shape the `JournalStats` vocabulary cannot express (per-node
  ledgers, layer age) → grow the struct as a snapshot field priced by a
  real consumer, never a second read channel.
- The zero default proves too dishonest for a real journal (a backend
  that must account) → make `stats()` answer an explicit "no books"
  variant instead of zeros, one priced implementation at a time.
