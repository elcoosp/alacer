# ADR-0062: M56 scope — the app's text write joins the causal log

- **Status:** Accepted (2026-11-14)
- **Implements:** ADR-0061's second kill rule — "the app's own
  `set_value` and the view reconciliation's writes attributed on the
  log — the door's facts the AT's asks ride alone today → the app door
  grows its own record beside the reset echo" — the reader prices them
  now: a trail reader cannot answer "did the screen reader rewrite
  this field, or did the app" from the log alone (the M30 rule's own
  question, at the tier whose door the rule held off)
- **Companion:** ADR-0036 (the attribution rule the record follows —
  the rider's shape, `FocusRider` at the walk), ADR-0061 (the AT door
  whose asks the `source` joins), ADR-0021 (the journal whose reset
  still rides beneath the whole-value write), ADR-0049 (the stamp the
  app door honestly carries none of), ADR-0045 (the payload-privacy
  rule the lengths honor), ADR-0010 (the additive wire rule),
  architecture §13.3, §18 (the M56 roadmap row)

## Decision

- **The app door names itself beside the AT's.** The input's write
  vocabulary serves two doors: the AT's action door (M55, ADR-0061)
  and the app's imperative door — `TextInput::set_value`, which the
  mutate seam calls and the view reconciliation's patches call. The
  app door's writes streamed only the journal's reset echo (a fact
  about the depths, ADR-0021), never the door's fact about the write.
  The vocabulary (`crates/alacer-core/src/inspect.rs`,
  `crates/alacer-widgets/src/text_input.rs`): `TextWriteRecord` grows
  the additive `source: TextWriteSource` — `AtDoor` for the AT's asks
  (the four M55 note sites), `AppDoor` for the app's write, noted
  where the write lands. One write, one fact: the AT's `SetValue`
  rides a raw `apply_value` beneath its own note, so the app door's
  record never double-streams an AT ask. The app door rides no funnel
  route, so its `target` stays the honest absence (the stamp's own
  rule, M43 — the trail fills only what the caller cannot know: an
  identity, never a door).
- **The shape and the privacy rule ride unchanged.** The record the
  app door streams names the write by lengths (`replaced` — the value
  the field carried; `inserted` — the value the app handed over; UTF-16
  units) and never the payload (a field's value can carry a secret —
  the clipboard trail's rule, ADR-0045), the caret the write left
  beside them, the journal's reset still streaming beside the door's
  fact (the depths the journal's own story). A no-op write — the app
  writing the value the field already carries — lands as the write it
  is: the journal resets, the door's fact streams, the log naming it
  (the M20 idle-frame property, named where it applies).
- **The wire and the client speak it at version 1.** The wire's
  `text_write` event grows the additive `source` field (the rider
  field's rule, ADR-0010): `at_door` / `app_door`, the protocol's
  `snake_case` tags — clients that ignore unknown fields are
  unaffected, `PROTOCOL_VERSION` stays 1. The embedded client renders
  the door beside the record — "the app's text write landed (9
  replaced by 18) — the selection at 18..18" — the AT door's rows
  unchanged, and `examples/app_text_writes.rs` is the consumer
  demonstration (tracing at `alacer::app_text_writes`).
- **No new corpus case: the corpus stands at 64.** The record rides
  the write the app already makes — no new hot loop, no new route;
  the note is one branch per write while nobody listens (the A7 idle
  contract), inside the budgets the contracts already gate. The
  living documents re-agree at the cut (the README, the register, the
  protocol document's `text_write` vocabulary, the catalog, the
  CHANGELOG, the m-suite pins).

## Kill criteria (re-open rules)

- A consumer prices the *keyboard's and IME's* edits on the log — the
  editing tier's per-keystroke rides the widget doors stream nothing
  of today (the clipboard trail priced the round trips, the journal
  priced the depths) → the edit stream grows its own per-change
  record beside the journal's, priced by a real reader, never a
  per-keystroke flood (the flood rule's own words).
- A consumer needs the app door's *refusals* — a write the field's
  vocabulary cannot express — named on the log → the app door grows
  its own refusal species (the door's honest answers are the log's
  facts, ADR-0046's rhythm), never a munged delivery.
- A consumer needs the *reconciliation's* writes distinguished from
  the imperative ones (a binding's churn priced apart from the app's
  explicit write) → the source vocabulary grows its third species
  beside the two doors (the additive rule, ADR-0010), the doors
  staying the names the log speaks.
