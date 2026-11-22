# ADR-0144: M140 scope — the replay cursor: the ask skips the history the client already carried

- **Status:** Accepted (2026-11-22)
- **Implements:** the growth path ADR-0143 pre-registered first among its
  kill criteria: a client needs the burst to skip what it saw live. The
  M139 ask answered with the rings' whole worth — the right answer for the
  first reconnect, and a growing one for every reconnect after it: a
  panel that re-attaches in a long session re-reads the history its own
  earlier sessions already streamed, the folding contract pricing the
  repeats client-side forever. The wire's ask had no door that says
  "start from where I left off". [`ClientMessage::Replay`]'s optional
  `after` field (crates/alacer-inspect/src/wire.rs) is that door, and
  [`CausalRecorder::replay_after`] (crates/alacer-inspect/src/recorder.rs)
  is the answer: only the facts recorded *after* the cursor answer, the
  history at-or-below it staying silent.
- **Companion:** ADR-0143 (the replay ask the cursor grows — the additive
  rule at version 1, the rings-are-durability distinction), ADR-0010 (the
  inspector's protocol and its additive versioning rule — an optional
  field joins beside the rider field's precedent), ADR-0069 (the
  look-reads-without-disturbing rule the cursor inherits unchanged),
  architecture §13.3 (the questions table), §17 (the register), §18 (the
  M140 roadmap row)

## Decision

- **The greeting names the count; the ask rides it back.** The cursor's
  source is the greeting's new `facts` field — the recorder's total fact
  count at greeting time, monotonic and never reset (evictions drop ring
  contents, `clear` empties them, and the count moves through both
  unchanged, the stability the cursor needs across a clear-reconnect
  window). A client records the number its *first* greeting named; on
  reconnect it asks `{"kind":"replay","after":N}`, and the burst answers
  exactly the facts recorded since that greeting — the pre-history the
  client was never connected for stays silent. Facts the client streamed
  live above its cursor may still repeat in the burst (the folding
  contract stands, the escape hatch it always was); the cursor's own
  guarantee is the skip, not the dedup — "priced by that client", the
  pre-registered rule's own words.
- **Every ring entry rides the fact number it arrived under.** The
  cursor filters, so the rings must know where each entry sits in the
  global count: every ring's entries became `(fact_number, record)`
  pairs, the number assigned once per observation under the same lock
  the rings write under. The stamps are private (the `Recording`'s own
  shape, never a wire field) — the wire events stay exactly the shapes
  M139 pinned, no per-event sequence bloat, and the readers answer the
  same records they always answered. A poisoned mutex's loss is named on
  the dropped counter and never counted: a fact that never landed names
  no number.
- **The ask is additive at version 1.** `PROTOCOL_VERSION` stays 1 (the
  rider field's rule). The `after` field is optional and defaults to
  absent — the bare ask (`{"kind":"replay"}`) parses byte-for-byte as
  M139 shipped it and replays the rings whole; a cursor above every
  recorded fact answers an empty burst (the honest absence), the session
  keeps serving either way. A server older than the field ignores
  nothing — it parses the ask it knows and answers the whole worth; a
  client that asks with a cursor of an older server reads repeats, the
  folding contract absorbing the version skew without a probe.
- **The connect-time price stays a connect-time price.** The filter
  walks every stamped entry either way — the stamps are the cursor's own
  currency — so `replay_after` prices beside `replay`, never below it
  (`benches/replay.rs`: the cursor case rides the same filled recorder,
  the delta it answers strictly inside the whole worth). The A7 idle
  contract is untouched: the ask runs on the session thread, once per
  connect.

## Consequences

- **A reconnecting panel starts from its own past.** The second
  operator's window — the panel re-attached to an already-running app —
  reads the delta the rings hold above its first greeting, and the
  pre-history stops repeating. `examples/inspector_cursor.rs` is the
  consumer demonstration: the counter runs, panel A connects a few
  seconds in and streams live until the window closes, panel B asks with
  A's cursor, and the walkthrough prints the skip beside the delta.
- **The protocol document re-agrees.** The `hello` section names the
  `facts` field and its monotonicity; the client → server section names
  the cursor ask and its contract (at-or-below stays silent, the empty
  burst above every fact, the bare ask unchanged).
- **The m-suite pins re-agree at the M140 cut** — the same one-cut move
  the chain has made at every tier (ADR-0143's own account), done here
  beside M141's re-agreement as one scripted pass. The replay door's
  contract tests live where the protocol document's other sections pin
  them: the inspector protocol suite (`crates/alacer-inspect/tests/protocol.rs`),
  beside the wire-shaped integration tests.
