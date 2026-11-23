# ADR-0146: M142 scope — the subscribe ask: the session's live stream, gated by tier

- **Status:** Accepted (2026-11-23)
- **Implements:** the blind spot the replay ask (ADR-0143), the cursor
  (ADR-0144) and the stats ask (ADR-0145) left on the *live* half of the
  wire: a panel can ask for the rings' history and its bound, but the
  live stream it rides has exactly one shape — every tier, always. A
  panel that reads a replay burst while the session streams (the M139
  folding contract prices the repeats client-side), a panel that only
  wants the frame ledger, a panel that wants silence while it digests a
  burst — each got facts it did not ask for, the folding contract
  pricing them forever. The `{"kind":"subscribe","tiers":[…]}` control
  is the ask, and the session's fan-out (crates/alacer-inspect/src/server.rs)
  is the door: the facts whose ring the ask names answer the session's
  live stream, the unasked tiers staying silent — the panel's own
  silence, never a loss.
- **Companion:** ADR-0143 (the replay ask — the ask that made the rings
  wire-readable and the folding contract's own rule), ADR-0144 (the
  cursor — the additive optional field's precedent at version 1),
  ADR-0145 (the stats ask — the row vocabulary the tiers' names ride),
  ADR-0010 (the inspector's protocol and its additive versioning rule),
  ADR-0029 (the observability rule — the loss named, never swallowed;
  the gate drops nothing the rings hold), architecture §13.3 (the
  questions table), §17 (the register), §18 (the roadmap row)

## Decision

- **The gate is the session's fan-out, never the recorder's capture.**
  The ask lives in `run_session`: one `TierFilter` per session, the
  broadcast drain admitted event by event, the ask re-set from that
  frame's drain onward. The rings keep recording every tier — the
  replay and stats asks answer whole regardless, and a later bare ask
  (`{"kind":"subscribe"}`) restores the whole stream without a single
  fact having been lost to the gate. The eviction counters (ADR-0145)
  name the losses the caps cause; a muted tier is not one of them.
- **One vocabulary, two doors.** The ask's tier names are the stats
  answer's row names (`Tier` — the twenty-two trail rings, `snake_case`
  through serde's rename and `Tier::name()` alike): M141's rows are
  M142's ask's own list, the two lists unable to drift apart (the
  protocol document's bounds sentence names both). `broadcast` is
  deliberately absent — the stream is the medium, never a tier of
  itself; a panel asking for it sent a malformed control frame, and the
  session's documented report-and-close behavior answers (the unknown
  control's own rule, at the name tier). The closed vocabulary is the
  parse: an unknown name fails deserialization, never a silent munge.
- **The three shapes ride one optional field.** The absent list is the
  whole stream (the shape every pre-M142 session rode — the bare ask
  parses byte-for-byte as nothing new), a list gates the fan-out to the
  named tiers, an empty list is the mute (the stream goes quiet while
  the rings keep recording — the replay burst's reading panel's ask).
  Additive at version 1 (ADR-0010's rule, the `Replay` ask's
  precedent): a server older than the control closes the session on
  the unknown control.
- **The answers ride whole.** The filter gates the captured fact
  vocabulary only: the events no ring captures — the `stats` occupancy
  answer among them — always admit (`WireEvent::tier()` naming `None`
  for them), so a gated panel's asks are still answered. The ask is a
  live-stream ask, never an answer-silencer (the replay and stats
  asks' own rule at the fan-out tier).
- **The gate's price is one tier word per event.** The drain is the
  session's existing price (the queue it has always taken); what the
  ask adds is the per-event admission — `benches/replay.rs` prices it
  on the same filled recorder's steady-state batch
  (`subscribe/broadcast_512x3_filtered`), a per-drain price bounded by
  the queue's own 4 096 cap, never a per-frame tax on the app.

## Consequences

- **A panel prices what it reads, and silences what it does not.** The
  replay burst's reading panel mutes the live noise while it digests
  (`{"kind":"subscribe","tiers":[]}`), asks the burst, and restores the
  stream when it is done — the folding contract's repeats shrink to
  the facts it actually asked for. The frame-ledger panel gates to
  `["frames"]` and stops paying for the input storm's fan-out. The
  vocabulary's growth (a new trail tier) joins the ask automatically —
  the tier joins the enum beside the ring, the names one list.
- **The mute is honest silence, not a loss.** A panel reading the stats
  rows under the mute sees the facts the gate silenced right where the
  rings hold them (`facts` climbing, the rows' occupancy climbing) —
  the panel's own ask explains the quiet stream, and the protocol
  document says so. A consumer that needs the history answers it from
  the rings (the replay ask), never from the gate.
- **Kill-switch:** a consumer needs the gate to persist across
  reconnects → grow the greeting beside the cursor's `facts` precedent
  (the server remembering a panel is a second channel, priced first);
  a consumer needs the gate to *throttle* instead of dropping (the
  muted tiers queuing for later delivery) → that is a second queue, the
  pre-registered rule the folding contract refuses — the rings are the
  durable history, the gate a live-stream shape.
