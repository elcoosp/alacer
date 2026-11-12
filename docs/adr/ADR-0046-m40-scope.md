# ADR-0046: M40 scope — the value tier rides the causal log

- **Status:** Accepted (2026-11-12)
- **Implements:** the ride ADR-0010's additive rule and the
  M31/M33/M36/M37/M39 rhythm pre-register for every seam that answers in
  facts — the value tier's AT actions (M20, ADR-0026, the `Slider`; M21,
  ADR-0027, the read-only `Progress`) answered the widget's own `bool`,
  whose `false` collapsed four different facts (the control the
  availability tier holds down, the payload that parsed to nothing, the
  payload the funnel cannot carry, the control that never had a write
  door) into one silence; cashed as the vocabulary growth the additive
  rule names: "grow the event vocabulary beside the wire's kinds, priced
  by a real reader — never a second log"
- **Companion:** ADR-0026 (the value tier whose writes this record
  carries), ADR-0027 (the read-only tier whose refusal is the
  `no_write_door` species), ADR-0028 (the availability tier whose
  refusal is the `unavailable` species), ADR-0045 (the clipboard trail
  whose drain shape this trail rides — the facts queue where they
  resolve, the tick's start drains them), ADR-0010 (the protocol's
  additive versioning — the new kind is compatible at version 1),
  architecture §13.3 (the causal log), §18 (the M40 roadmap row)

## Decision

- **One record, carried where the write resolves.**
  `alacer-core/src/inspect.rs`: [`ValueRecord`] — the action
  ([`ValueAction::SetValue`/`Increment`/`Decrement`]), the outcome
  ([`ValueOutcome::Delivered`], or [`ValueOutcome::Refused`] naming its
  species: `unavailable`, `invalid_payload`, `non_finite_value`,
  `no_write_door`), and the value the control holds as the record
  resolves. The widgets note the fact through
  `alacer_core::value::note_write` at the exact place the answer
  resolves — the widget knows the action and the value it holds, the
  trail module owns the queue (the journal's own rule: the fact lives
  next to the contract it echoes; `alacer-core/src/value.rs` is the
  new module, the clipboard trail's shape). A question outside the
  value tier (a press, a scroll, a text edit) is not a value fact and
  streams nothing — the clipboard trail's own scope over its chords,
  the input tier's records carrying those rides.
- **The value names the fact; it is not a secret.** A slider's value is
  the tree's own vocabulary the AT already reads (the M3 `value()`
  contract) — the record carries it verbatim, the funnel's
  clamp-and-snap result on a delivery, the untouched current value on a
  refusal (the tree keeps the value the app wrote). The clipboard's
  count-not-payload rule was a secret's rule; the value tier has no
  payload to hide.
- **The two parse facts name apart.** The widget's `bool` treated "loud"
  and "inf" identically; the record does not. A payload that is not a
  number is `invalid_payload` (a `SetValue` never invents a value from
  prose); a payload that parses but is not a finite scalar is
  `non_finite_value` (the funnel's own wall — the m20 suite's two
  refusals, named apart at last). The availability refusal
  (`unavailable`) and the read-only tier's (`no_write_door`) complete
  the four facts the `bool` collapsed.
- **The trail is the clipboard trail's shape.**
  `alacer-core/src/value.rs`: a thread-local, `VALUE_TRAIL_CAP`-bounded
  (256, oldest-first eviction) queue behind the A7 gate — one branch
  per action while no observer is installed, `PassPipeline::set_observer`
  arming it. The tick's start drain forwards every fact as
  [`InspectEvent::Value`], the drain running after the edit drain (a
  value write explains itself; it deletes nothing — there is no
  clipboard → edit causal order to preserve). A no-op delivery streams
  `delivered` at the unchanged value: the M20 idle-frame property is an
  answered request, not a refusal, and the log now names it.
- **The wire grows one kind, additively at version 1.**
  `alacer-inspect/src/wire.rs`: the `value` event — `action`, `outcome`,
  `refusal` (species or `null`), `value` — the clipboard event's flat
  rule. The recorder grows a bounded ring (512, the same order the
  input, focus, edit, announce, key, and clipboard rings ride), the
  embedded client renders the echo in the causal log pane, and
  `docs/inspector-protocol.md` documents the event. Clients that ignore
  unknown kinds are unaffected (the versioning rule).
- **The corpus stands at 63.** The trail is dev-mode and bounded — no
  new hot loop (the coordinated rule's no-growth arm, the
  M31/M33/M36/M37/M39 rule); the published `progress/route_100_value_writes_1k`
  route measures the same programmatic writes it always measured (the
  AT tier is the record's scope; the route rides the write funnel the
  record does not duplicate), and the disabled trail costs one branch
  per action (the A7 idle contract, pinned by the exit suite's idle
  test).
- **The living documents re-agree at M40.** The README's status block,
  the ADR count (forty-seven), the quickstart, and the example table
  (33 rows); the catalog; the register; the roadmap; the m-suite pins
  moving M39 → M40 and 46 → 47 across the chain; the protocol
  document's `value` section; the CHANGELOG.

## Consequences

- The workspace stays at one milestone per minor cut: the next release
  (0.18.0) carries M40.
- The corpus is 63 cases; the m-suite's corpus pin stands (the
  no-growth arm).
- An app, a status line, an inspector client, or a test reads the value
  tier without re-implementing the observation at the action site: the
  writes' history is data, oldest first, refusals included — and the
  recorder's ring carries the trail beside the other seams' rings.
- The availability tier's honesty joins the log: a locked control's
  refusal streams the same species the tree advertised (`unavailable` —
  the AT read the refusal before it happened, and now the trail names
  it after the fact too).

## Kill criteria (re-open rules)

- A consumer needs the *who* — the control's identity on the record —
  → grow the record beside the action's routing (the dispatch funnel
  knows the target node; the widget does not), priced by a real reader —
  never a node id guessed at the note site (the M39 record's own
  minimalism stands).
- A consumer needs the keyboard's and the drag's value rides on the log
  (the arrows, Home/End, the thumb) → grow the record beside the
  funnel's other doors (ADR-0042's rhythm), priced by a real reader —
  never a per-pointer-move flood on a devtools surface (the input
  tier's records carry those rides until then).
- A consumer needs the *requested* value beside the landed one (a clamp
  the trail cannot show) → grow the record's fields additively
  (ADR-0010's rule), priced by that reader — never a second
  observation channel.
