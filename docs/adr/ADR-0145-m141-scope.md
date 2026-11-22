# ADR-0145: M141 scope — the rings' stats ask: the replay burst's own bound, named

- **Status:** Accepted (2026-11-22)
- **Implements:** the blind spot ADR-0143's replay ask left beside the
  cursor ADR-0144 grew: the burst answers what the rings still hold, and
  nothing tells a panel what they *cannot* — the caps evicted the older
  history silently, the loss the trail reader's own `dropped` counter
  names in-process but no wire event carried. A panel reading a replay
  burst of a long session could not tell a complete history from a
  capped one, and the operator debugging "where did the early frames
  go?" had no answer on the wire. The `{"kind":"stats"}` control is the
  ask, and the `stats` event (crates/alacer-inspect/src/wire.rs) is the
  answer: the fact count beside one row per ring — the occupancy, the
  cap, and the evictions the cap caused — the burst's own bound, named.
- **Companion:** ADR-0143 (the replay ask the stats bounds — the
  additive control's precedent at version 1), ADR-0144 (the cursor the
  fact count rides — the same monotonic counter answers both), ADR-0010
  (the inspector's protocol and its additive versioning rule), ADR-0029
  (the observability rule the eviction counters ride — the loss named,
  never swallowed), architecture §13.3 (the questions table), §17 (the
  register), §18 (the M141 roadmap row)

## Decision

- **One row per ring, the stream's bound beside the trails.** The answer
  carries the recorder's fact count and twenty-three rows: `broadcast`
  first (the stream's own 4 096 bound — a slow session's window is a
  loss the panel can price beside the trails'), then the twenty-two
  trail rings in the event vocabulary's order, `frames` last, each row
  the flat `{name, held, cap, evicted}` shape the wire's other rows
  ride. The rows are the bounds sentence's own contract on the wire —
  the protocol document's ring list and the stats answer can no longer
  drift apart, one naming the other.
- **The eviction counters are the loss, named where it happens.** Every
  ring carries its eviction count beside its stamped queue (`Ring<T>` —
  the queue, the cap, the counter): a push at the cap evicts the oldest
  and counts it, the counter surviving `clear` (a reset empties the
  contents, it does not un-name the loss). The broadcast queue's drop
  counts too — the push the recorder handle performs counts the oldest
  it makes way for. A poisoned mutex answers the empty row list, the
  readers' honest absence (the other readers' own rule).
- **The look reads without disturbing.** The ask drains nothing, queues
  nothing, counts nothing new — the replay ask's own rule at the stats
  tier (ADR-0069's discipline): no ring drains, the broadcast queue is
  untouched, and the live stream beside the answer continues untouched.
  The answer is one event, answered once per ask — a connect-time price
  like the burst's (`benches/replay.rs` prices it on the same filled
  recorder), never a per-frame one.
- **The control is additive at version 1.** `PROTOCOL_VERSION` stays 1
  (ADR-0010's rule, the `Replay` ask's precedent): a new client control
  joins beside the ask the version already carries, and a server older
  than the ask closes the session on the unknown control — the
  documented unknown-frame behavior, the client that needs the rows
  probing or reconnecting.

## Consequences

- **A panel prices the history it reads.** The rows tell the operator
  which trails are complete (held < cap, evicted 0) and which are capped
  (held == cap, evicted climbing) — the replay burst's completeness is
  data, not a guess. `examples/inspector_stats.rs` is the consumer
  demonstration: the counter runs, the window closes, a scripted
  std-lib client connects to the same server, sends the ask, and prints
  the rows.
- **The protocol document re-agrees.** The client → server section
  names the ask and its contract; the bounds sentence names `stats` as
  the read that does not disturb beside `replay`, the row list and the
  document's ring list one list.
- **The m-suite pins re-agree at the M141 cut** — the one-cut move
  beside M140's (ADR-0143's account of the chain), the replay and stats
  doors' contract tests living where the protocol document's other
  sections pin them: the inspector protocol suite
  (`crates/alacer-inspect/tests/protocol.rs`).
