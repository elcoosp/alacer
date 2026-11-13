# ADR-0049: M43 scope — the value record names its control

- **Status:** Accepted (2026-11-12)
- **Implements:** ADR-0046's first re-open rule — "a consumer needs the
  *who* — the control's identity on the record — → grow the record
  beside the action's routing (the dispatch funnel knows the target
  node; the widget does not), priced by a real reader — never a node id
  guessed at the note site (the M39 record's own minimalism stands)" —
  cashed as the additive field the versioning rule names: "grow the
  record's fields additively, priced by that reader — never a second
  observation channel"
- **Companion:** ADR-0046 (the value trail whose record carries the
  target), ADR-0048 (the clipboard read facts that shipped beside this
  milestone), ADR-0026/ADR-0027 (the value tier whose AT actions the
  funnel routes), ADR-0028 (the availability tier the refusals name),
  ADR-0010 (the protocol's additive versioning — the new field is
  compatible at version 1), architecture §13.3 (the observability
  questions), §18 (the M43 roadmap row)

## Decision

- **One additive field, stamped by the funnel that knows.**
  `alacer-core/src/inspect.rs`: `ValueRecord::target` — the
  AccessKit-stable id hash the dispatch funnel routed, `None` when the
  write rode no routed action. The stamp side lives where the
  knowledge lives: `alacer-core/src/value.rs` grows a `pub(crate)`
  `ActionTargetGuard` the funnel holds for the length of one widget
  call (`PassPipeline::dispatch_a11y_action` stamps before
  `on_a11y_action` answers and clears after), and `note_write` stamps
  a `None`-targeted record with the id the thread's guard held — the
  widget never guesses its own identity (the rule's own words), the
  unit tests pinning the caller's own answer, the honest absence
  outside a guard, and the guard's drop leaving no stale identity on
  the thread.
- **The identity is the id the action arrived under.** The funnel
  receives the AT's id hash and the AT's client correlates by it — the
  vocabulary the sink's tree updates and the wire's `node` fields
  already speak — so the record carries that hash rather than an arena
  id the AT cannot correlate. A screen reader's client, an inspector
  pane, or a test answers "which control answered" without a second
  naming channel.
- **The wire grows one field, additively at version 1.**
  `alacer-inspect/src/wire.rs`: the `value` event grows `target` (the
  id as a number, `null` where the fact is absent) — the additive
  field the versioning rule admits at version 1 (the clipboard
  `engine` field's rule), clients that ignore unknown fields
  unaffected. The recorder's ring carries the record verbatim (no new
  ring — the same 512 slots), the embedded client renders the control
  beside the record, and `docs/inspector-protocol.md` documents the
  field.
- **The widget's own doors stay off the value log.** The keyboard's
  step and the drag ride the widget's own doors (the key route, the
  pointer route), which stream no value fact at all — the input tier's
  records carry those rides until a consumer prices them (the second
  kill rule's open arm). The attribution stays exclusive to the AT
  tier: every value fact on the log names the control the funnel
  routed, the m43 suite pinning the keyboard's silence beside the
  routed ride's attribution.
- **The writes are untouched byte-for-byte.** The funnel's
  clamp-and-snap answers as it always answered, the refusals stream
  their species as they always streamed, and the trail's shape is
  untouched — the same 256-bounded queue, the same tick's start drain,
  one branch per write while nobody listens (the A7 idle contract,
  pinned by the exit suite's idle test).
- **The corpus stands at 63.** The stamp is one `Cell` read per noted
  fact on a chord-path tier — no new hot loop (the coordinated rule's
  no-growth arm, the M31/M33/M36/M37/M39/M40/M41/M42 rule); the
  published `progress` route measures the same tier it always
  measured.
- **The living documents re-agree at the cut.** The README's status
  block, the ADR count (fifty), the quickstart, and the example table;
  the catalog; the register; the roadmap; the m-suite pins moving
  M41 → M43 and 48 → 50 across the chain; the protocol document's
  `target` field; the CHANGELOG.

## Consequences

- The workspace stays at one milestone per minor cut: the next release
  (0.18.0) carries M43 beside M42.
- The corpus is 63 cases; the m-suite's corpus pin stands (the
  no-growth arm).
- A status line, an inspector client, or a test answers "which control
  answered" from the log: two sliders on one page, the facts tellable
  apart; the read-only bar's refusal naming the door-less control; a
  multi-control form's trail attributable — the devtools surface
  routing on the identity, never a second observation channel.
- A widget note site that someday learns its own identity keeps
  compiling and answering: the caller's own target is never
  overwritten, the stamp filling only the honest absence — the
  additive rule's whole point (ADR-0010).

## Kill criteria (re-open rules)

- A consumer needs the keyboard's and the drag's value rides on the
  log *attributed* → grow the record beside the funnel's other doors
  (the key route, the pointer route — ADR-0042's rhythm), priced by a
  real reader — never a per-pointer-move flood on a devtools surface
  (ADR-0046's second kill rule stands).
- A consumer needs the *requested* value beside the landed one (a
  clamp the trail cannot show) → grow the record's fields additively
  (ADR-0010's rule), priced by that reader — never a second
  observation channel.
- A consumer needs the control's *label* on the record (the prose a
  devtools pane shows) → grow the record beside the sink's snapshot
  vocabulary the client already renders — never a stringly-typed tag
  guessed at the note site.
