# ADR-0039: M33 scope — the announcement rides the causal log: the announcement trail

- **Status:** Accepted (2026-11-12)
- **Implements:** the hook the M18 contract's own refusals pre-registered
  — "why did the live region speak — or refuse" was answerable only by
  re-implementing the observation at the call site (`announce.rs`, the
  push route's honest refusals; `PassPipeline::announce`, the direct
  door's contract) — cashed as a real reader: "a status line, an
  inspector trail, or a test needs the speech acts' facts → every act
  the pipeline answers streams one record the moment the answer
  resolves — never a second announcement channel, never a re-implementation
  at the call site"
- **Companion:** ADR-0037 (the edit trail whose shape the facts ride),
  ADR-0034 (the causal trail the announcement tier joins), ADR-0024
  (the announcement contract whose refusals are the vocabulary),
  ADR-0020 (the thread-local discipline the push trail borrows),
  ADR-0010 (the protocol's additive versioning rule the wire event
  rides), architecture §13.3 (the questions table the echo answers),
  §18 (the M33 roadmap row)

## Decision

The causal log answers "why did this render", "who wrote this signal",
"why did focus change — or why didn't it", and "what did this field's
journal do". The app's voice had no route: a speech act landed on the
reserved live region and left no trace a trail reader could route on —
while the contract's own refusals (empty text, no root) are exactly the
facts a trail reader wants. M33 cashes it:

- **The pipeline speaks for its answers.** Every act the pipeline
  answers streams one `alacer_core::inspect::AnnounceRecord` the moment
  the answer resolves — the focus doors' shape at the speech-act tier:
  the direct door's acceptance and both refusals at the door
  (`PassPipeline::announce`), the push-side empty refusal on the
  thread's trail (`announce::announce` notes the fact where the
  clipboard-seam discipline lives, ADR-0020), and the tick's start
  drain answering the queued batch. The record carries the politeness
  the act asked for (`Politeness::name`, the wire's tag) and the
  outcome (`AnnounceOutcome::name`): `delivered`, `superseded` — the
  last-write-wins rule visible as data instead of a silent drop (the
  batch drains whole now, `take_pending` returning every act) — or
  `refused` with the named species (`AnnounceRefusal::EmptyText`,
  `AnnounceRefusal::NoRoot`). The refusals stream too: the M28 honesty
  at the speech-act tier.
- **The trail is dev-mode and bounded, like everything on the seam.**
  The queue notes a fact only while the trail is enabled — one branch
  per act otherwise, the A7 idle contract — and `PassPipeline::set_observer`
  enables it when a devtools observer installs (beside the edit
  trail's enable, ADR-0037). The bound is the §14.3 eviction rule
  applied to the echo: `ANNOUNCE_TRAIL_CAP` (256 facts per thread,
  oldest dropped first), and the drain empties the queue whether an
  observer listens or not: a fact streams once, an absent reader just
  never sees it.
- **The wire carries the echo, additively.** `WireEvent::Announce`
  (`politeness`, `outcome`, `refusal` — the focus event's flat shape,
  the refusal species beside the outcome) — a new event kind,
  compatible by the protocol's own versioning rule ("new event kinds
  and new optional fields are compatible"), `PROTOCOL_VERSION` stays 1.
  The recorder grows the bounded announce ring beside its
  frames/edges/writes/inputs/focus/edits (512, the §14.3 discipline,
  `CausalRecorder::announces()` the read door), and the embedded
  client renders the echo in the log pane beside the focus and edit
  trails — the politeness and the outcome named, the refusal rendered
  the way the walk's refusals are.
- **The demonstration drives the seam with the observer riding.**
  `examples/announce_trail.rs` walks the M18 contract's own routes — a
  delivered polite act, an assertive interrupt, the empty speech act's
  refusal, a three-act storm whose superseded facts stream beside the
  survivor — the app mirroring its own introspection while the *trail*
  label mirrors what the log streamed, with tracing at
  `alacer::announce_trail`, the workspace's per-seam idiom. The m33
  exit suite pins the contract headlessly: the real pipeline, real
  speech acts, the delivered record's politeness beside the tree's own
  answer (the region label the AT reads), both refusals at the door
  and at the push, the no-root refusal at the drain, the
  last-write-wins shape visible as data, and the no-observer idle
  contract.
- **The living documents re-agree at M33 and 0.17.0.** The README's
  status block (M0 through M34, this cut and M34 together), the ADR
  count (forty-one), the inspector bullet, the quickstart, and the
  example table; the catalog's row; the register; the §13.3 questions
  table; the roadmap row; the protocol document's `announce` section;
  the m-suite pins moving M32 → M34 and 39 → 41 across the chain; and
  the CHANGELOG stages the milestone under `[Unreleased]`.

## Consequences

- The announcement contract's rules are untouched: the always-present
  region, last write wins, and the honest refusals answer exactly as
  before, the m18 suite's pins stand, and `announce`'s signature is
  unchanged — the note is the push route's own side effect, priced at
  one branch when the trail is off. The drain's new answers ride the
  observer path only.
- `AnnounceRecord`, `AnnounceOutcome`, and `AnnounceRefusal` are new
  public core vocabulary — the growth the M27/M28/M31 cuts priced the
  same way: pre-1.0, additive, the wire carrying the flat shape.
- The hot path carries no new work by default: the disabled trail
  costs one branch per act, the drain runs only while an observer is
  installed, and the announcement drain's shape is unchanged for every
  consumer that never installs one.
- The perf corpus stands at 62: the echo rides no hot loop the corpus
  does not already price (the speech acts ride the published
  announcement route, re-measured at this cut: 18.62 ms median against
  the published 18.81 ms, -1.0% — inside the same-host gate), the
  coordinated rule's no-growth arm (no new criterion case, no baseline
  move, the m5 bijection green as pinned; the ADR-0037 precedent).
- The m1 corpus does not grow: the demonstration shows no new visual
  arrangement (the trail label rides the announcements example's
  window shape), and the m1 suite's 37-scenario pin stands.
- No new dependency edges: the echo is core's own vocabulary, the wire
  event is `alacer-inspect`'s, and the spans ride the `tracing` core
  every member already resolves.

## Kill criteria (re-open rules)

- A consumer needs the echo's *content* (the act's text, the region's
  label at the moment it changed) → grow the record beside
  `AnnounceRecord`, priced by a real reader — never a second
  announcement channel.
- A consumer needs per-act attribution (which handler queue streamed
  the fact, the direct door or the ambient route) → grow the record
  beside the outcome, priced by a real multi-route reader — never a
  stringly-typed tag.
- A client needs the echo grouped or filtered by politeness or outcome
  (a speech-act pane, a replay control) → grow the client's rendering
  beside the event the wire already carries (ADR-0010's additive
  rule), priced by that client — never a second wire vocabulary.
