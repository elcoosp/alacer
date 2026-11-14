# ADR-0073: M67 scope — the adapter's full-tree echo joins the causal log

- **Status:** Accepted (2026-11-14)
- **Implements:** ADR-0065's second kill rule — a consumer prices the
  adapter activation's timing or the a11y sink's full-tree fallbacks
  (the AT connected or reconnected; the app answered with the whole
  resolved tree — "why did the app just push the whole tree, and how
  big was it" unanswerable from the log alone) → the tier that resolves
  them grows its own record beside its route
- **Companion:** ADR-0015 (the windowed a11y path the echo rides — the
  sink's own builder, the activation response), ADR-0063 (the
  route-drop trail whose shape the echo trail adopts), ADR-0065 (the
  kill rule this ADR cashes), A7 (the idle contract: the sink's one
  branch when nobody listens is unchanged), architecture §13.3, §18
  (the M67 roadmap row)

## Decision

- **The a11y tier grows its own trail.**
  [`A11yEchoRecord`](crates/alacer-core/src/inspect.rs) carries one
  `nodes: usize` — the resolved tree's size, the count the full-tree
  build answered with. The payload itself never rides: a resolved tree
  carries the app's labels, and a label can carry a secret (the
  lengths-not-payload rule at every tier, the clipboard trail's own).
  `0` never rides — an activation before the first frame resolves no
  tree, and no fact streams for it (the honest absence; the trail
  never invents a size).
- **The note site is the exact place the echo resolves.**
  [`AccessKitSink::full_tree_update`](crates/alacer-a11y/src/lib.rs)
  notes the fact where the build happens — the activation response the
  adapter forwards, the same builder the deltas use. The build's own
  purity is unchanged: no bookkeeping changes, no update enters the
  ledger (the m9 suite's pin stands — the note is the trail's one
  branch, not a ledger write), and the platform's activation route
  ([`AlacerApp::push_initial_tree`](crates/alacer-platform/src/lib.rs))
  rides the note without its own changes — the echo is the sink's
  fact, the platform merely the courier.
- **The trail is the route-drop trail's shape** (ADR-0063): a bounded
  thread queue ([`A11Y_ECHO_TRAIL_CAP`] = 256, oldest-first eviction),
  the note a no-op while no observer listens (the A7 idle contract —
  one branch per echo), the dispatching pipeline draining it at the
  tick's start into [`InspectEvent::A11yEcho`].
- **The echo reads first.** The drain runs ahead of the frame's
  routes: a connect predates the asks the frame carries, and the echo
  explains the tree those asks answer into — a frame's trail reads
  a11y-echo → input → the doors' facts → frame.
- **The wire grows additively at version 1.** The flat `a11y_echo`
  event carries the `nodes` field (a new event kind, the additive rule
  ADR-0010 names — clients that ignore unknown kinds are unaffected);
  the embedded client renders "the AT answered with the full tree
  (N nodes)"; the protocol document's section carries the field, and
  the bounds list re-agrees (the M56–M66 rings it had lagged ride the
  same line).
- **No new corpus case.** The echo rides no measured route — a connect
  is rare, event-driven, never a frame-rate companion (the flood rule
  at a tier whose events are rare); the note costs one branch and a
  bounded push off every hot path, the same no-growth arm M57's drops,
  M59's silence, and M65's riders rode. The corpus stands at 68, the
  m5/m6 pins re-agree unchanged.
- **The demo walks the echo beat.** `examples/a11y_echo.rs` routes the
  activation echo the way the platform's route rides it — the erased
  sink downcast, the full-tree build — beside a reconnect beat, the
  trail label mirroring what the log streamed through the M64 tee, one
  fact per build.
