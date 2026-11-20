# ADR-0143: M139 scope — the replay ask: the rings' durable history reaches the late client

- **Status:** Accepted (2026-11-20)
- **Implements:** the wire door the rings never had. Through M138 the
  recorder's twenty-two bounded rings were the durable causal history —
  and the only channel that carried them to a client was the shared
  broadcast queue: one queue, drained by whichever session polls first,
  bounded at 4 096 with the oldest dropped. A panel that connects after a
  first panel's drain took the queue's contents saw a silent inspector
  until the next interaction; a panel that reconnects after a long quiet
  window never sees what the rings still hold. The wire's control
  vocabulary (`clear`, `bye`) had no door that asks for history — the
  gap this tier closes. [`ClientMessage::Replay`]
  (crates/alacer-inspect/src/wire.rs) is the ask, and
  [`CausalRecorder::replay`] (crates/alacer-inspect/src/recorder.rs) is
  the answer: every ring converted through its wire shape, oldest first
  per trail, tiers in the event vocabulary's order, frames last.
- **Companion:** ADR-0010 (the inspector's protocol and its additive
  versioning rule — the ask is a new control at version 1, the rider
  field's precedent), ADR-0029 (the observability rule the burst is
  named on the log beside), ADR-0069 (the trail reader's look-reads-
  without-disturbing rule the ask inherits), ADR-0034 (the first ring's
  own growth precedent — every later tier's ring rides the same
  discipline), architecture §13.3 (the questions table), §17 (the
  register), §18 (the M139 roadmap row)

## Decision

- **One ask, one burst, nothing drained.** The ask is a client control
  (`{"kind":"replay"}`); the answer is the rings' contents as one burst
  of wire events, per trail oldest first, tiers in the vocabulary's own
  order — edge, write, input, focus, edit, announce, key, clipboard,
  value, gesture, scroll, text_write, route_drop, key_edit, preedit,
  a11y_echo, action, frame_refusal, view_patch, theme, setting, frame.
  The look reads without disturbing (ADR-0069's rule at the wire tier):
  no ring drains, the broadcast queue is untouched, and the live stream
  beside the burst continues untouched. Facts the session saw live may
  repeat in the burst — the client folds the duplicates, the way the
  `clear` control already makes the client own its buffer. An empty
  history answers an empty burst: the session keeps serving, the next
  live fact arrives whole.
- **The rings are the durability; the queue is the stream.** The
  distinction is the design: the broadcast queue is the per-tick stream
  (bounded at 4 096, oldest dropped, drained by whichever session
  polls), the rings are the bounded history (per-tier caps, eviction
  named). The replay ask reads the rings, not the queue — so a second
  panel gets the whole held history, not the delta the first panel's
  drain left, and a reconnection after a quiet window gets what the
  rings still hold, not nothing. The rings' caps are the burst's bound:
  the answer is never larger than the history the recorder is allowed
  to hold, and the connect-time cost is measured, not believed
  (`benches/replay.rs` — a 1 280-event burst converts in tens of
  microseconds, a connect-time price, never a per-frame one).
- **The ask is additive at version 1.** `PROTOCOL_VERSION` stays 1
  (ADR-0010's versioning rule — new client controls join beside the
  rider field's rule). A server older than the ask closes the session
  on the unknown control — the documented unknown-frame behavior — so
  a client that needs the ask can probe or reconnect; a client that
  never asks is unaffected, the wire's own shape unchanged for it.
- **The `clear` door clears every ring — the micro-debt the ask's
  own test priced.** The first `clear` implementation cleared the
  sixteen rings the first doors grew; the six later tiers' rings
  (a11y_echo, action, frame_refusal, view_patch, theme, setting)
  survived a clear that documented itself as "clears every ring" —
  the door's sentence lying by omission. M139's replay-after-clear
  test priced the omission (`a_replay_ask_after_a_clear_answers_
  nothing_and_lives`), and the fix rides the cut: every ring answers
  the reset, the protocol document's ring list names all twenty-two
  (the drift the list had carried since the M71–M74 tiers shipped —
  frame_refusals, view_patches, themes, settings were missing from
  the bounds sentence), and `replay` reads the same rings the clear
  empties.

## Consequences

- **A reconnecting panel catches up.** The second operator's window —
  the re-opened browser tab, the panel attached to an already-running
  app — reads the history the rings hold, then streams live beside it.
  `examples/inspector_replay.rs` is the consumer demonstration: the
  counter runs, the window closes, a scripted std-lib client connects
  to the same server, sends the ask, and prints the burst.
- **The protocol document re-agrees.** The client → server section
  names the ask and its contract; the bounds sentence names all
  twenty-two rings and names `replay` as the read that does not
  disturb.
- **The m-suite pins re-agree at the M139 cut.** The living-documents
  pins (the status line's cut, the register's record count, the
  cross-suite chain reads) are pinned by fifty-seven exit-criteria
  suites, and the M139 re-agreement of the README, the architecture,
  and the register moves those pins with the documents they read —
  the same one-cut move the chain has made at every tier since the
  pins were introduced, done here as one scripted pass rather than
  one file per commit. The replay door's own contract tests live
  where the protocol document's other sections pin them: the
  inspector protocol suite (`crates/alacer-inspect/tests/protocol.rs`),
  beside the wire-shaped integration tests.
