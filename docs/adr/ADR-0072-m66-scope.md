# ADR-0072: M66 scope — the journal's echo names its field

- **Status:** Accepted (2026-11-14)
- **Implements:** ADR-0037's second kill rule — a consumer needs
  per-field attribution (a real multi-field reader: two inputs, two
  journals, one log — the depths streamed but the field unnamed left
  "whose journal moved" unanswerable from the rows alone) → the record
  grows the field beside the depths, priced by that reader — never a
  second journal channel
- **Companion:** ADR-0037 (the edit trail the record rides), ADR-0049
  (the M43 stamp rule the echo generalizes), A7 (the idle contract:
  the journal's one branch when nobody listens is unchanged),
  architecture §13.3, §18 (the M66 roadmap row)

## Decision

- **The record grows the M43 stamp.**
  [`EditRecord`](crates/alacer-core/src/inspect.rs) carries one
  additive `target: Option<u64>` — the AccessKit-stable id the routed
  event arrived under, stamped where the echo resolves
  ([`EditHistory`](crates/alacer-core/src/history.rs)'s note: a
  record the journal queued with a `None` target is stamped with
  [`crate::value::action_target()`]'s answer — the same guard the key
  route has held since M45 and the AT action route since M43, the
  identity never guessed at the note site).
- **The direct rides carry the honest absence.** The journal driven
  outside any route — the app's own `record`/`undo`/`redo`/`reset`
  calls, the demo's buttons — answers `None`: the stamp fills only
  what the route knows (the M43 rule's own word), and a dropped guard
  leaves no stale identity behind.
- **The wire grows additively at version 1.** The `edit` event carries
  the `target` field (the route-drop `rider` field's rule — clients
  that ignore unknown fields are unaffected); the embedded client
  renders "— the field N" beside the depths; the protocol document's
  section carries the field.
- **No new corpus case.** The field rides the routes the journal
  already rides — the editing route the key-edit bench prices, the
  stamp a `Cell` read while nobody listens — the corpus stands at 68,
  the m5/m6 pins re-agree unchanged (the coordinated rule's no-growth
  arm).
- **The demo walks the two stamps.** `examples/edit_trail.rs` routes
  one committed composition through the real IME route into a live
  `TextInput` at startup — its echo carries the field's id beside the
  depths, the direct journal's echoes beside it carrying none: one
  log, both stamps tellable apart.
