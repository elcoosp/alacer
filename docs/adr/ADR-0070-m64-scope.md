# ADR-0070: M64 scope — the observer's tee joins the seam

- **Status:** Accepted (2026-11-14)
- **Implements:** the second-door ask the single slot pre-registered —
  the pipeline's observer slot is one (`PassPipeline::set_observer`),
  and two devtools at once (the inspector's recorder on the wire
  beside the trail's provided reader in-process, ADR-0069's) needed
  either a hand-rolled wrapper (the benches' own `Share` shape, again)
  or one door silently losing the other — a consumer prices the
  *second door* → the seam grows the combinator the slot's own
  contract implies, never a door that loses the other by omission
- **Companion:** ADR-0069 (the provided reader the tee fans out to),
  ADR-0010 (the recorder the wire keeps riding), A7 (the idle contract:
  the slot's single `None` branch is unchanged — the tee prices only
  what a consumer asks for), architecture §13.3, §18 (the M64 roadmap
  row)

## Decision

- **The combinator is provided beside the trait.**
  [`Tee`](crates/alacer-core/src/inspect.rs) lives in `alacer-core`
  beside [`InspectObserver`](crates/alacer-core/src/inspect.rs) — two
  boxed observers behind one, each event forwarded first to `first`
  then to `second` (the arrival order the causal reads rely on), a
  third reader nesting (`Tee::new(a, Tee::new(b, c))`) — the
  composition the slot cannot hold on its own.
- **The fan-out is priced, not assumed.** The bench prices the tee on
  the route the reader case already runs (`trail/tee_announce_100_1k`,
  the announce shape with two readers behind the slot) — gated by the
  `trail_tee_overhead` budget (≤ 2.5× the unobserved
  `announce/route_100_announcements_1k`: each door rides the observe
  branch the slot already pays, the fan-out never a second pass over
  the route) and its own same-host regression gate; the m5/m6 corpus
  pins re-agree to 68 beside ADR-0069's additions.
- **The demo shares the slot.** `examples/inspector_demo.rs` wires the
  recorder and the reader behind one tee — the recorder keeps the
  wire, the app holds the in-process reading handle, and the window
  closes on a read-back that names what the tee forwarded (the
  demonstration the docs' §13.3 sentence rides).
- **No wire change.** The tee forwards the events the routes already
  stream — no new protocol kind, no version bump, the inspector's wire
  untouched.
