# ADR-0061: M55 scope — the AT's text write joins the causal log

- **Status:** Accepted (2026-11-14)
- **Implements:** ADR-0046's second kill rule — "the input tier
  carries those rides until a consumer prices them" — the reader
  prices them now: a trail reader cannot answer "did the screen
  reader rewrite this field — what shape, where did the caret land,
  or why not" from the log alone, so the editing tier's AT door
  grows its own record (the value trail's shape, ADR-0046, at the
  tier whose door the rule held off the log)
- **Companion:** ADR-0017 (the editing vocabulary the door answers —
  `SetValue`/`ReplaceSelectedText`/`SetTextSelection`), ADR-0028
  (the availability tier whose closed door the refusal names),
  ADR-0021 (the journal whose reset rides beneath the whole-value
  write), ADR-0049 (the stamp the record carries, M43), ADR-0050
  (the ask beside the landing, M44 — at the text tier), ADR-0045
  (the payload-privacy rule the lengths honor), ADR-0010 (the
  additive wire rule), architecture §13.3, §18 (the M55 roadmap row)

## Decision

- **The write's own fact streams from the door.** The input's AT
  actions (M11, ADR-0017) answered the widget's `bool`, whose `false`
  collapsed the closed door (M22) into the same silence as an
  answered write — and the answered write itself left no fact a
  trail reader could price (the journal's reset and record echoes
  rode, but they are the journal's facts about the depths, not the
  door's facts about the ask). The vocabulary
  (`crates/alacer-core/src/text_write.rs`, `crates/alacer-widgets/src/text_input.rs`):
  one `TextWriteRecord` per AT text write — the action beside the
  outcome (`Delivered` or `Refused`), the write's shape named by
  **lengths** (`replaced`/`inserted`, UTF-16 units — the whole value
  for a `set_value`, the selection's span for a
  `replace_selection`), the payload itself never riding the log (a
  field's value can carry a secret — the clipboard trail's rule,
  ADR-0045), the selection the field carries as the record resolves
  beside them, and the ask beside the landing on the selection door
  (the M44 rule, ADR-0050, at the text tier — the forgiving read
  side's floor and clamp visible as data, never reconstructed). The
  refusal streams too: the unavailable field's write door names its
  species (`unavailable` — the same fact the value tier names, one
  concept spoken identically wherever a door names it, ADR-0028's
  tier).
- **The stamp and the drain ride the house disciplines.** The
  record's `target` fills from the routing funnel's guard (M43,
  ADR-0049 — the same `ACTION_TARGET` stamp the value trail's writes
  and the keyboard's steps stamp from; one guard, the shared routes,
  the identity never guessed at the note site), and the facts ride
  the tick's start drain into `InspectEvent::TextWrite` (the scroll
  drain's shape, ADR-0056) — dev-mode, bounded
  (`TEXT_WRITE_TRAIL_CAP` = 256, oldest-first eviction), silent
  while nobody listens (the A7 idle contract, one branch per write).
  The journal's own echoes are untouched: a `set_value` still resets
  the journal (ADR-0021's rule — the AT's write is an app write, undo
  never fights it) and the reset still streams — the journal's fact
  about the depths, beside the door's fact about the ask.
- **The wire and the client speak it at version 1.** The wire's
  `text_write` event carries the record flat (the additive rule
  ADR-0010 names): the action and outcome tags (`set_value`/
  `replace_selection`/`set_selection`, `delivered`/`refused`), the
  refusal species, the lengths, the selection as one `(anchor,
  focus)` object, the ask beside the landing, the stamp beside them
  — clients that ignore unknown kinds are unaffected. The embedded
  client renders the echo — "the text write landed (11 replaced by
  21) — the selection at 21..21" — the refusals rendered the way the
  other tiers' refusals are, and `examples/at_text_writes.rs` is the
  consumer demonstration (tracing at `alacer::at_text_writes`).
- **No new corpus case: the corpus stands at 64.** The record rides
  the route the write already rides — the action route is measured —
  and the lengths are two `usize`s carried in a `Copy` record
  computed once per answered ask, inside the budgets the contracts
  already gate. The living documents re-agree at the cut (the
  README, the register, the protocol document's `text_write`
  vocabulary, the catalog, the CHANGELOG, the m-suite pins).

## Kill criteria (re-open rules)

- A consumer prices the *payload's content* — a reader the privacy
  rule cannot serve (an audit that must name what was written) → the
  record grows the payload behind an explicit opt-in seam (the
  clipboard trail's own re-open rule, ADR-0045's words), never a
  default-on wire field.
- A consumer needs the *app's own* `set_value` and the view
  reconciliation's writes attributed on the log — the door's facts
  the AT's asks ride alone today → the app door grows its own record
  beside the reset echo (today the reset names the depths; the
  attribution is the AT door's fact).
- A consumer needs the *platform's defensive drops* on the log — the
  `SetValue` without a payload, the `SetTextSelection` aimed at
  another node, the actions the funnel traces and drops before the
  widget's door (`map_action_request`'s arms) → the platform route
  grows its own refusal vocabulary (a route-side record the guard
  stamps), never a munged widget-door fact.
