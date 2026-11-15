# ADR-0075: M69 scope — the activation ask answers on the log

- **Status:** Accepted (2026-11-14)
- **Implements:** the action tier's own pre-registered kill rule — a
  consumer prices the activation ask (the AT's press, the default
  accessibility action the platform's action route aims at a control:
  the ask arrived, the target's door answered, and the answer was a
  bare `bool` whose `false` collapsed the two facts a trail reader
  wants apart — the availability tier's wall and the doorless target's
  silence; "did the AT's press land — or why not" unanswerable from the
  log alone) → the action tier grows its own trail, never a munge onto
  a door that answered
- **Companion:** ADR-0016 (the action tier the ask rides — the faithful
  routing the `bool` contract of record answered since M10), ADR-0028
  (the availability tier whose `enabled` truth names the wall's
  species), ADR-0045 (the one-fact-per-trail rule the activation tier
  keeps), ADR-0063 (the boundary route whose species the un-resolved
  asks keep — never a munged action record), ADR-0049 (the M43 stamp
  vocabulary at the activation tier), A7 (the idle contract: the note
  rides the funnel's own branch), architecture §11.5, §13.3, §18 (the
  M69 roadmap row)

## Decision

- **The action tier grows its own trail.**
  [`ActionRecord`](crates/alacer-core/src/inspect.rs) — `ask`, `target`,
  `outcome` — streams one fact per activation ask the funnel routes,
  carried in [`InspectEvent::Action`](crates/alacer-core/src/inspect.rs)
  at the tick's start drain (the preedit drain's shape, one frame after
  the ask), bounded at 256 on the thread trail (the journal's own
  bounded rule), 512 on the recorder's ring. The note site is the exact
  place the ask resolves: the dispatch funnel
  (`PassPipeline::dispatch_a11y_action`), the route-drop trail's shape
  (ADR-0063) — the route holds the ask, the aimed id, the door's
  answer, and the tree's one `enabled` truth.
- **The species is computed where both facts live.** A refused ask
  names [`ActionRefusal::Unavailable`](crates/alacer-core/src/inspect.rs)
  when the availability flag held the control down (M22, ADR-0028 —
  the tree's `enabled`, never a widget-private guess) and
  [`ActionRefusal::NoDoor`](crates/alacer-core/src/inspect.rs) when
  nothing did (a press aimed at a heading, a door no widget grew). The
  collapsed `false` splits into two named walls; a landing keeps its
  `Delivered` verdict and the mask-queueing ride the `bool` contract of
  record always drove — the trail rides beside it, never instead of it.
- **One fact per trail.** The asks whose tiers already price
  themselves — the value tier's writes (ADR-0046), the scroll tier's
  asks (ADR-0056), the text-write tier's rewrites (ADR-0061) — stream
  no action fact: the funnel notes the activation ask only, a question
  outside the tier is not a tier fact (the ADR-0046 rule the scroll
  tier rides). An ask the boundary dropped before any door (no target
  to resolve) stays the route-drop trail's fact (ADR-0063's species),
  never a double-priced action record.
- **The wire grows additively at version 1.** The flat `action` event
  (`ask`, `target`, `outcome`, `refusal`) joins the protocol — the
  additive rule ADR-0010 names; clients that ignore unknown kinds are
  unaffected; the embedded client renders the row the other trails
  render, the species beside the aim.
- **The corpus grows by the coordinated rule.** The activation route
  joins the published bench (`a11y_action/route_100_presses_1k` — the
  worst-case walk, the door's refusal, the trail's one disabled branch
  under the A7 contract), the regression gate beside it in
  `contracts.json`, the baseline re-measured, the m-suite corpus pins
  moving with it (68 → 69, the coordinated rule's own chain).
- **The demo walks the activation beat.**
  `examples/action_trail.rs` routes the three presses — the landing,
  the wall, the doorless silence — at startup, the value tier's step
  between them streaming no action fact, the trail label mirroring
  what the log streamed.
- **Kill rules pre-registered for the next consumer:** a trail reader
  who cannot tell the AT's press from the keyboard's Space/Enter
  activation → the record grows the rider beside the ask (the M30
  attribution rule at the activation tier); a consumer who prices the
  out-of-tier asks (the increment aimed at a button) → the ask
  vocabulary grows beside the tiers' own refusals, never a munge onto
  the activation ask.
