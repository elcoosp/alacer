# ADR-0036: M30 scope — the trail names the rider: the attribution rule cashed beside the movement

- **Status:** Accepted (2026-11-12)
- **Implements:** the re-open rule ADR-0035 pre-registered — "a consumer
  needs the rides distinguished in the causal log → grow `FocusRecord`
  beside its `movement` (a rider field), priced by a real reader — never
  a second trail"
- **Companion:** ADR-0034 (the causal trail the rider joins), ADR-0030
  (the funnel the keyboard's fallback joined), ADR-0035 (the seam the
  ring joined and the rule this ADR cashes), ADR-0029 (the
  plain-help-text rule the walk's refusals carry), ADR-0010 (the
  protocol's additive versioning rule the wire field rides),
  architecture §13.3 (the questions table the attribution answers),
  §18 (the M30 roadmap row)

## Decision

The causal trail has streamed one `FocusRecord` per focus-move door ride
since M28 — the movement, the scope, the outcome — and the keyboard's
Tab fallback and the ring's arrows have ridden the same funnel since
M24 and M29. The trail could answer "why did focus change — or why
didn't it", but not "*who* rode": the user's Tab, the D-pad's arrow,
and the app's own `focus_move` call streamed records no reader could
tell apart. A status line cannot say "the user navigated", an inspector
client cannot attribute the ride, a test cannot pin the route it
exercised. M30 cashes the rule:

- **The record grows the rider beside the movement.**
  `alacer_core::inspect::FocusRider` — `Keyboard` (the Tab/Shift+Tab
  fallback, the M24 seam), `Ring` (the four directional keys' fallback,
  the M29 seam), `Consumer` (the app's own door calls) — rides every
  `FocusRecord` as a new `rider` field, the attribution the funnel
  records when the door resolves. The rider is the *route's* name,
  declared where the route is known: the public doors
  (`focus_move`, `focus_move_in`) stream `Consumer`, and the dispatch
  fallbacks stream through the same unscoped funnel behind one private
  rider-aware door (`focus_move_via`) — `Keyboard` for Tab,
  `Ring` for the arrows. No second dispatch channel, no second trail,
  no platform-side walk (ADR-0030's rule, kept literally); the walk
  behind the doors is untouched — the same candidate list, the same
  availability filter, the same wrap, the same immediate resolution,
  the same no-mask idle refusals (the M11 rule stays).
- **The wire carries the rider, additively.** `WireEvent::Focus` grows
  a `rider` field (`keyboard` / `ring` / `consumer`, the protocol's
  `snake_case` tag vocabulary, `FocusRider::name` the single source) —
  a new optional field, compatible by the protocol's own versioning
  rule ("new event kinds and new optional fields are compatible"),
  `PROTOCOL_VERSION` stays 1. Clients that ignore unknown fields (the
  rule every client already follows) are unaffected; the embedded
  client renders the rider beside the movement (M31, this cut).
- **The trail reads as an attributed history.** A reader drains the
  focus ring and answers "who moved focus — the keyboard, the ring, or
  the app?" from the records alone: the m30 exit suite pins the three
  riders against the real grid and the real dispatch (the code's doors,
  Tab through `dispatch_key`, an arrow through `dispatch_key`), the
  wire shape carrying the field, and the demonstration reading back an
  attributed history. The scoped door's dead-scope refusal streams its
  `Consumer` rider too — the early refusal is the door's own, and the
  attribution follows the door.
- **The demonstration walks all three riders.**
  `examples/focus_riders.rs` walks the M26 window with the code's
  doors, the keyboard's Tab, and the ring's arrow through the real
  dispatch, every beat logging the streamed record's rider beside the
  answer it names, with tracing at `alacer::focus_riders`, the
  workspace's per-seam idiom. The m30 exit suite pins the contract
  headlessly on the real grid through the real layout host.
- **The living documents re-agree at M30 and 0.17.0.** The README's
  status block (M0 through M30), the ADR count (thirty-seven), the
  inspector bullet, the quickstart, and the example table; the catalog's
  row; the register (36 → 37 with this ADR); the §13.3 questions table;
  the roadmap row; the protocol document's focus section; the m-suite
  pins, which move across the m5–m29 chain in the same cut (the count
  M29 → M30, the register 36 → 37); and the CHANGELOG stages the
  milestone.

## Consequences

- The record's shape grew one field — a breaking change to
  `FocusRecord` literals, the growth the M27/M28 cuts priced the same
  way: pre-1.0, in-tree constructors only (the recorder's ring, the
  benches' read-only observers, the exit suites), the doors' signatures
  unchanged, no call-site churn outside the constructor sites.
- The hot path carries no new work: the rider is a `Copy` discriminant
  resolved at the existing emission point, the walk behind the doors is
  unchanged, and the published focus routes price the walk's cost per
  move as shipped.
- The perf corpus stands at 62: the riders ride the published routes —
  the observed route (`input/route_100_observed_moves_1k`) already
  prices the door-plus-record path the rider now attributes — the
  coordinated rule's no-growth arm (no new criterion case, no baseline
  move, the m5 bijection green as pinned).
- The m1 corpus does not grow: the demonstration shows no new visual
  arrangement (the riders demo walks the M26 window's own paint), and
  the m1 suite's 37-scenario pin stands.
- No new dependency edges: the rider is core's own vocabulary, the wire
  field is `alacer-inspect`'s, and the spans ride the `tracing` core
  every member already resolves.

## Kill criteria (re-open rules)

- A consumer needs a rider the three names cannot express (a plugin's
  walk, a test driver) → grow the vocabulary beside `FocusRider`,
  priced by a real route — never a stringly-typed rider.
- A client needs the rides grouped or filtered by rider (a trail pane
  per rider, a filter control) → grow the client's rendering beside the
  field the wire already carries (ADR-0010's additive rule), priced by
  that client — never a second wire vocabulary.
- A caller needs the walk's history across ring evictions (a multi-step
  replay, candidate distances over time) → grow the record beside
  `FocusRecord` (the ADR-0034 rule stands), priced by a real consumer —
  never a second trail.
