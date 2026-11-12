# ADR-0045: M39 scope — the clipboard rides the causal log

- **Status:** Accepted (2026-11-12)
- **Implements:** the ride ADR-0010's additive rule and the
  M31/M33/M36/M37 rhythm pre-register for every seam that answers in
  facts — the clipboard tier's chords (ADR-0020) answered the field's
  own `bool`, whose `false` collapsed three different facts (the
  selection that was not there, the clipboard that carried nothing, the
  platform that refused the write) into one silence; cashed as the
  vocabulary growth the additive rule names: "grow the event vocabulary
  beside the wire's kinds, priced by a real reader — never a second
  log"
- **Companion:** ADR-0020 (the clipboard seam whose round trips this
  record carries), ADR-0037 (the edit trail whose drain shape this
  trail rides — the facts queue where they resolve, the tick's start
  drains them), ADR-0039 (the announcement trail's refusal-tier
  honesty — a refused round trip streams too), ADR-0010 (the
  protocol's additive versioning — the new kind is compatible at
  version 1), architecture §13.3 (the causal log), §18 (the M39
  roadmap row)

## Decision

- **One record, carried where the round trip resolves.**
  `alacer-core/src/inspect.rs`: [`ClipboardRecord`] — the action
  ([`ClipboardAction::Copy`/`Cut`/`Paste`]), the outcome
  ([`ClipboardOutcome::Delivered`], or [`ClipboardOutcome::Refused`]
  naming its species: `no_selection`, `empty_clipboard`,
  `write_refused`), and the payload's UTF-8 scalar count. The widget's
  clipboard funnels note the fact through
  `alacer_core::clipboard::note_round_trip` at the exact place the
  answer resolves — the widget knows the action and the payload it
  moved, the seam module owns the trail (the journal's own rule: the
  fact lives next to the contract it echoes). A cut never rides the
  copy funnel: the funnels were restructured so one round trip streams
  one fact, never a copy record and a cut record for the same span.
- **The count names the fact; the payload never rides.** A clipboard
  can carry a secret — a password pasted into a field would land on a
  devtools log that mirrors every payload it touched. The record
  carries the scalar count (zero on a refusal — a refusal moves
  nothing), and the wire carries the same count: the log answers "did
  the text move — and how much", never "what was the text".
- **The trail is the edit trail's shape.** `alacer-core/src/clipboard.rs`:
  a thread-local, `CLIPBOARD_TRAIL_CAP`-bounded (256, oldest-first
  eviction) queue behind the A7 gate — one branch per round trip while
  no observer is installed, `PassPipeline::set_observer` arming it.
  The tick's start drain forwards every fact as
  [`InspectEvent::Clipboard`], the drain running ahead of the edit
  drain so a cut's trail reads clipboard → edit — the causal order the
  round trip ran (what moved out, then the deletion it drove).
- **The refusals are the facts.** The three species are the editing
  contract's own refuse-don't-munge rules made observable: a copy
  without a selection never invented content (`no_selection`), an
  empty clipboard's read pastes nothing (`empty_clipboard` — an
  honest `None` or a payload that sanitizes to nothing), and a
  refused write never deletes (`write_refused` — the selection
  surviving a clipboard that failed it, ADR-0020's own example). A
  trail reader answers "why didn't my paste land" from the log, the
  same honesty the walk's refusals (ADR-0033) and the journal's
  refused chords (ADR-0037) stream.
- **The wire grows one kind, additively at version 1.**
  `alacer-inspect/src/wire.rs`: the `clipboard` event — `action`,
  `outcome`, `refusal` (species or `null`), `chars` — the announce
  event's flat rule. The recorder grows a bounded ring (512, the same
  order the input, focus, edit, announce, and key rings ride), the
  embedded client renders the echo in the causal log pane, and
  `docs/inspector-protocol.md` documents the event. Clients that
  ignore unknown kinds are unaffected (the versioning rule).
- **The corpus stands at 63.** The trail is dev-mode and bounded — no
  new hot loop (the coordinated rule's no-growth arm, the
  M31/M33/M36/M37 rule); the published `editing/route_100_clipboard_ops_1k`
  route measures the same round trips it always measured, and the
  disabled trail costs one branch per chord (the A7 idle contract,
  pinned by the exit suite's idle test).
- **The living documents re-agree at M39.** The README's status block,
  the ADR count (forty-six), the quickstart, and the example table (32
  rows); the catalog; the register; the roadmap; the m-suite pins
  moving M38 → M39 and 45 → 46 across the chain; the protocol
  document's `clipboard` section; the CHANGELOG.

## Consequences

- The workspace stays at one milestone per minor cut: the next release
  (0.18.0) carries M39.
- The corpus is 63 cases; the m-suite's corpus pin stands (the
  no-growth arm).
- An app, a status line, an inspector client, or a test reads the
  clipboard tier without re-implementing the observation at the chord
  site: the round trips' history is data, oldest first, refusals
  included — and the recorder's ring carries the trail beside the
  other seams' rings.
- The paste-side facts join the edit side: a cut's trail reads
  clipboard → edit in one frame, so "what moved, then what the journal
  did" is one log's reading, not two reconstructions.

## Kill criteria (re-open rules)

- A consumer needs the clipboard's *platform-side* facts the widget's
  round trips cannot name (the engine's own refusal diagnostics, the
  OS's change notifications) → grow the seam's engine contract beside
  `ClipboardSeam`, priced by a real engine — never a guess the OS
  did not answer (ADR-0020's rule stands).
- A consumer needs the rides grouped or filtered by outcome → grow the
  client's rendering beside the event the wire already carries
  (ADR-0010's additive rule), priced by that client.
- A consumer needs the payload on the log (a sync service, a clipboard
  history) → grow it as an explicit, opt-in consumer-side capture
  beside the seam — never a default on a devtools surface a secret can
  reach (the count-not-payload rule stands).
