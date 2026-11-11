# ADR-0034: M28 scope — the walk joins the causal log: the focus trail as streamed events, the observed route priced, the introspection the M25 species priced narrowed

- **Status:** Accepted (2026-11-11)
- **Implements:** the re-open rule ADR-0033 pre-registered (and ADR-0031
  stated first) — "a consumer needs to observe the walk's refusals as
  *streamed events* (the inspector's causal log: 'why did focus change /
  why didn't it') → grow the observation seam beside `InspectEvent`,
  priced by that consumer" — plus the micro-debt the sweep priced: the
  `FrameReport::pass_overhead_nanos`/`pass_overhead_ratio` pair whose
  only callers were the crate's own M1 gate test (the M25 species:
  a public accessor no consumer rides)
- **Companion:** ADR-0033 (the outcome vocabulary the record carries
  verbatim), ADR-0030/ADR-0031/ADR-0032 (the funnel, the scope, the
  directions the trail observes), ADR-0029 (the plain-help-text rule the
  species keep on the `alacer::focus` target), ADR-0010 (the inspector's
  wire and its additive versioning rule), ADR-0011 (the coordinated
  corpus rule the observed route lands under), architecture §13.3 (the
  causal questions the log answers — now six), §18 (the M28 roadmap row)

## Decision

M27 made the doors' answer data; the data still reached only the caller
that asked. A status line, an inspector trail, or a test that wants "why
did focus change — or why didn't it" had to re-implement the observation
at the call site — exactly the client-side reimplementation the ADR-0031
rule forbids. M28 cashes the rule:

- **The doors stream their answers beside `InspectEvent`.**
  `focus_move` and `focus_move_in` forward one `InspectEvent::Focus`
  the moment they resolve, carrying a `FocusRecord` — `movement` (the
  direction the caller asked for), `scope` (the subtree root that
  bounded the walk, `None` for the unscoped funnel), and `outcome` (the
  door's own `FocusMoveOutcome`, verbatim: the `FocusLanding`'s
  endpoints and wrap, or the named `FocusRefusal` species). The record
  is `Copy` — two words over `NodeId`s and a flag, the M27 consequence's
  budget holding. The emission rides one helper (`notify_focus`) beside
  the pipeline's `notify_input`/`notify_frame`, behind the same
  installed-observer branch: without devtools the doors pay one branch
  per call — the A7 idle contract is unchanged.
- **One funnel, two doors, one trail.** The emission lives in the
  doors, not the walk: whoever rides the funnel streams the same
  records. The keyboard's Tab fallback routes through `focus_move`
  (ADR-0030's rule kept literally), so its rides stream exactly what
  the consumer's code streams — a key event's trail reads focus →
  input → frame, the record written when the door resolves, *before*
  the input record the dispatch forwards and the frame summary the
  tick forwards. The scoped door's early refusal
  (the dead scope) streams too: the M25 refusal is not only named, it
  is observable.
- **The recorder grows the bounded focus ring.** The
  `CausalRecorder`'s rings gain the focus trail beside frames/edges/
  writes/inputs: `FOCUS_RING_CAP` (512, the input ring's size — the
  walk answers per ride, not per event), oldest dropped first (§14.3's
  eviction discipline, pinned by the recorder's own test), the `clear`
  control emptying it, and `CausalRecorder::focus()` as the read door.
  The wire grows `WireEvent::Focus` (the documented flat shape:
  `movement`, `scope`, `moved`, `from`, `to`, `wrapped`, `refusal`,
  `node`, `direction`) with `wire_focus`/`refusal_name` as the
  converters — a new event kind, additive by the protocol's own
  versioning rule (ADR-0010: `PROTOCOL_VERSION` stays 1; clients ignore
  unknown kinds; `docs/inspector-protocol.md` documents the shape).
- **The walk behind the doors is untouched.** The same pre-order
  candidate list, the same cycle-time `enabled` filter (M22,
  ADR-0028), the same wrap, the same immediate resolution, the same
  `FOCUS | PAINT` currency on the two endpoints, the same no-mask idle
  refusals — the M11 rule stays. The trail observes; it never aims.
- **The demonstration reads the trail.** `examples/focus_trail.rs`
  walks the M26 window — the two-by-two grid and the modal's form —
  one beat per `FocusMove` with the causal observer riding, and logs
  what the walk answered as data the moment each door resolved:
  landings naming their endpoints and wrap, refusals naming their
  species, the held-down trap's empty list among them. Tracing at
  `alacer::focus_trail`, the workspace's per-seam idiom. The m28 exit
  suite pins the records headlessly on the real grid and the real trap
  through the real layout host: the door's answer and the record are
  the same value, the keyboard's ride streams what the door's ride
  streams, the scoped record names its scope (and its dead scope), the
  refusals stream their species, the trail ring is bounded and orders
  before the frame, and the unobserved doors answer identically.
- **The observed route joins the published corpus.**
  `input/route_100_observed_moves_1k` — the same tree-order funnel as
  `route_100_focus_moves_1k` with the observer installed and the
  records consumed — prices the observation seam's own cost (the
  branch, the record, the observer call) against the unobserved
  funnel on the same 1k arena. The corpus pins move 61 → 62 across the
  m5–m27 chain in the same cut (the coordinated rule: bench, gates,
  and baseline in the same cut; CONTRIBUTING's publishing rule).
- **The micro-debt the sweep priced rides the cut.** The
  `FrameReport::pass_overhead_nanos`/`pass_overhead_ratio` pair narrows
  to `pub(crate)`: their only callers were the crate's own M1 gate test
  (passes.rs's §7.2 churn workload), the M25 species — a public
  accessor whose callers were all in-crate tests. The gate keeps
  measuring; the public surface stops advertising an API nobody rides.
  (The sweep also priced `Flux::counters` and found it alive — the
  flux bench resets the counters under measurement — so it stays
  public, the honest negative.)
- **The living documents re-agree at M28 and 0.17.0.** The README's
  badge moves to 0.17.0 — the release shipped, the badge lagging was
  drift — with the status block (M0 through M28), the ADR count
  (thirty-five), the inspector bullet, the quickstart, the perf bullet
  (62 cases), and the example table; the catalog's row; the register
  (34 → 35 with this ADR); the §13.3 questions table (the sixth row);
  the roadmap row; the m-suite pins, which move across the m5–m27 chain
  in the same cut — the corpus pins to 62, the document pins to M28 and
  thirty-five, and the badge pins to 0.17.0 (the m20–m26 badge pins
  riding the convention); and the CHANGELOG stages the milestone.

## Consequences

- The observer sink's vocabulary grows by one variant — the additive
  change the trait's consumers were built for. Custom observers match
  the new arm; the recorder, the protocol, and the dashboard handle it
  in this cut.
- The hot path carries no new work without devtools: one branch per
  door call. With the observer installed, the route's measured delta is
  the seam's own cost — published and gated like every route before it.
- The perf corpus grows 61 → 62 at this cut's measurement run by the
  coordinated rule — measured, published, and gated in the same cut.
- The m1 corpus does not grow: the demonstration shows no new visual
  arrangement (the trail demo walks the M26 window's own paint), and
  the m1 suite's 37-scenario pin stands.
- No new dependency edges: the record is core's own vocabulary, the
  wire shape is `alacer-inspect`'s, and the spans ride the `tracing`
  core every member already resolves.

## Kill criteria (re-open rules)

- A consumer needs the walk's *history* — a multi-step replay,
  candidate distances recorded over time, the stops between two doors'
  answers → grow the vocabulary beside `FocusRecord`, priced by a real
  consumer — never a second walk or a client-side reimplementation of
  the candidate list.
- A client needs the trail's species rendered or filtered → the wire
  vocabulary grows beside `WireEvent::Focus` (new fields are additive,
  the protocol's rule; reshaping existing fields bumps `version`),
  never a second channel.
- A consumer needs the walk's *candidate list* (not just its answer) →
  that is a different question than "why did focus change"; it grows as
  its own read door beside `focusables()`, priced by a real UI — the
  trail stays the doors' answers (ADR-0030's rule stands).
