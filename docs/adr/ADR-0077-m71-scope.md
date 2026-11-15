# ADR-0077: M71 scope — the frame's refusals join the causal log

- **Status:** Accepted (2026-11-15)
- **Implements:** the frame tier's own kill rule — a consumer cannot
  answer "why did nothing paint?" from the log the causal trail
  streams: the frame, the tier every other tier's work rides, answered
  a `tracing::error!` line and a skipped frame when it went unpainted
  (the tick's layout refusal, the backends' present ladder), no record
  at all → the frame grows its own named fact beside the tiers that
  already price themselves, never a munge onto a door that refused
- **Companion:** ADR-0023 (the `LayoutError` shape the record's node
  vocabulary rides), ADR-0063 (the route-drop trail's shape the record
  copies), ADR-0040 (the backend feature matrix the present ladder's
  note sites compile under), ADR-0029 (the frame ledger that prices
  the frames that *painted* — the refusal trail prices the ones that
  did not), ADR-0010 (the additive wire rule), architecture §9.3, §13.3,
  §18 (the M71 roadmap row)

## Decision

- **The frame tier grows its own record.**
  [`FrameRefusalRecord`](crates/alacer-core/src/inspect.rs) carries
  [`FrameRefusalSpecies`](crates/alacer-core/src/inspect.rs) beside the
  failing node when one is the fact. The species are the ladder's own
  stages — the stage that refused is the fact, never a collapsed
  `false`: `LayoutRefused` (the tick's layout stage answered
  `LayoutError` — the engine seam's own failure or the no-root wiring
  bug, ADR-0023's error shape), `RenderFailed` (the backend's renderer
  refused a draw the frame asked for), `PartialRepaintFailed` (the
  damage regions' scissored draw failed; the framebuffer survives),
  `FramebufferLost` (the software path's persistent framebuffer could
  not be built), `PresentFailed` (the pixels were drawn and the present
  refused — the frame the user never saw).
- **The note site is the exact place the refusal resolves.** The tick's
  layout block notes the engine seam's `Err` before propagating it —
  the `?`'s propagation is spelled out so the fact queues exactly
  where the refusal resolves, the failing node riding verbatim
  ([`NodeId::raw`](crates/alacer-core/src/arena.rs), ADR-0023's
  vocabulary — the id a consumer resolves against its own arena). The
  present ladder's stages note their species where each fails
  (`alacer-platform`, both compiled backends; the no-backend build has
  no ladder and no note sites). The record rides *beside* the
  refusal's own door — the tick still answers `Err(LayoutError)`, the
  ladder still skips the frame — never instead of it; the engines'
  prose summaries stay on the error types and the trail's trace lines
  (the payload rule's shape: a fact, not a message).
- **One silent arm is named at the same cut.** The softbuffer ladder's
  full re-rasterization answered `None` and *kept the stale framebuffer
  without logging anything* — the one swallow the present ladder hid
  even from its own log. The note site names it (`render_failed`,
  the `tracing::error!` beside the record); the stale-frame fact a
  trail reader wants is no longer a silent loss.
- **The drain rides the tick's start.** The frame-refusal trail is the
  route-drop trail's shape (ADR-0063): a fact queues where the refusal
  resolves, the dispatching pipeline drains it one frame later into
  [`InspectEvent::FrameRefusal`](crates/alacer-core/src/inspect.rs).
  The queue is bounded (256 facts per thread, oldest-first eviction —
  the newest refusals are the ones a consumer can still act on), the
  disabled path costs one branch per note (the A7 idle contract), the
  drain empties the queue whether an observer listens or not.
- **One fact per refusal, never per frame.** The frames that painted
  stream no refusal fact — the frame summary the pipeline already
  forwards (ADR-0037) prices those; a frame that refused once queues
  exactly one record, and the recovered frames after it stream none
  (the flood rule at the frame tier).
- **The wire grows additively at version 1.** The `frame_refusal` event
  is a new kind (`species` / `node`, `FrameRefusalSpecies::name` the
  single source) — the additive rule ADR-0010 names; clients that
  ignore unknown kinds are unaffected; the embedded client renders the
  refused stage's row beside the other trails.
- **The demo drives the seam from the consumer side.**
  `examples/frame_refusals.rs` arms a refusing layout engine behind
  the real pipeline, reads the `Err` the door still answers, and
  streams the record on the same log — one fact per refused frame.
- **The corpus grows by the note-cost gates.** The refusal rides the
  routes the existing benches price (the coordinated rule's no-growth
  arm for the routes), but the note site's own cost joins the corpus:
  `frame_refusal/note_disabled` (the A7 contract's one disabled branch
  per note) and `frame_refusal/note_enabled_bounded` (the enabled
  steady state's bounded push, the eviction priced) — 69 → 71 cases,
  bench, gate, and baseline in the same cut.
