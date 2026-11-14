# ADR-0063: M57 scope — the boundary's drops join the causal log

- **Status:** Accepted (2026-11-14)
- **Implements:** ADR-0061's third kill rule — "the platform's
  defensive drops on the log — the `SetValue` without a payload, the
  `SetTextSelection` aimed at another node, the actions the funnel
  traces and drops before the widget's door (`map_action_request`'s
  arms) → the platform route grows its own refusal vocabulary (a
  route-side record the guard stamps), never a munged widget-door
  fact" — the reader prices them now: a trail reader cannot answer
  "the AT asked; why did no door ever answer" from the log alone
- **Companion:** ADR-0016 (the faithful-mapping rule the unknown-action
  drop enforces), ADR-0017 (the single-node selection contract the
  off-target drop enforces, and the payload rule the no-payload drop
  enforces), ADR-0046 (the refusal-naming rhythm the species follow),
  ADR-0061 (the ADR whose kill rule this cashes), ADR-0010 (the
  additive wire rule), architecture §13.3, §18 (the M57 roadmap row)

## Decision

- **The route names its own refusals.** The platform boundary's
  mapping (`crates/alacer-platform/src/lib.rs`, `map_action_request`)
  dropped three species of ask before any widget's door — the action
  no widget contract answers, the `SetValue`/`ReplaceSelectedText`
  whose payload the request never carried, the `SetTextSelection`
  aimed at another node — and the dispatch funnel dropped a fourth
  (`crates/alacer-core/src/passes.rs`, `dispatch_a11y_action`'s
  unknown-target refusal). Every one was traced and gone; the
  consumed `false` verdicts collapsed four different facts into one
  silence (the M36 rule's own shape, at the boundary). The vocabulary
  (`crates/alacer-core/src/inspect.rs`, the trail in
  `crates/alacer-core/src/route_drop.rs`): one `RouteDropRecord` per
  drop — the dropped action named by its stable `snake_case` tag
  (`A11yAction::name` in core, the boundary's `action_tag` beside it —
  the two vocabularies sharing the tags the actions share), the
  species naming why the boundary refused (`unknown_action`,
  `no_payload`, `off_target_selection`, `unknown_target`), the aimed
  id riding verbatim — the boundary's aim, never re-resolved against
  the tree (a re-resolved aim would be the funnel's answer, not the
  AT's ask). A drop never rides a widget-door record: no door ever
  answered, and a munged one would be the lie the kill rule refuses.
- **The facts queue where the drop resolves and stream at the tick's
  start.** The trail is the text-write trail's shape (ADR-0061 — the
  value trail's rhythm): thread-local, bounded
  (`ROUTE_DROP_TRAIL_CAP` = 256, oldest-first eviction), enabled by
  `PassPipeline::set_observer` beside the other trails, silent while
  nobody listens (the A7 idle contract — one branch per drop; the
  drops themselves are the cold path, outside the frame's hot
  loops). The drain runs after the text-write drain (a drop explains
  the ask the boundary refused; it streams beside the doors' facts),
  into `InspectEvent::RouteDrop`.
- **The wire and the client speak it at version 1.** The wire's
  `route_drop` event carries the record flat (the additive rule
  ADR-0010 names): the action's tag beside the species and the aimed
  id — a new event kind, additive by the protocol's own versioning
  rule, `PROTOCOL_VERSION` stays 1, clients that ignore unknown kinds
  unaffected. The embedded client renders the echo — "dropped:
  unknown_target (aimed at 3735928559)" — and
  `examples/route_drops.rs` is the consumer demonstration (tracing at
  `alacer::route_drops`): the funnel's unknown-target drop driven
  through the real dispatch route, the delivered ask beside it
  streaming nothing (the honest absence).
- **No new corpus case: the corpus stands at 64.** The record rides no
  measured route and runs outside the frame's hot paths — the drops
  are the boundary's refusals, one branch per drop while nobody
  listens (the coordinated rule's no-growth arm, the M51 precedent).
  The living documents re-agree at the cut (the README, the register,
  the protocol document's `route_drop` vocabulary, the catalog, the
  CHANGELOG, the m-suite pins).

## Kill criteria (re-open rules)

- A consumer prices the *platform's* other silences — the adapter
  activation's timing, the a11y sink's full-tree fallbacks, the
  present path's drops — on the log → the tier that resolves them
  grows its own record beside its route (the value trail's shape,
  where the answer resolves), priced by a real reader, never a
  boundary-wide echo channel.
- A consumer needs the *dropped payload's shape* — an audit that must
  name how large the refused ask was — → the record grows the length
  behind an explicit opt-in seam (the clipboard trail's own re-open
  rule, ADR-0045's words), never a default-on wire field.
- A consumer needs the *funnel's unknown-target refusals attributed*
  to the rider who aimed them (the M30 rule at the boundary tier) →
  the record grows the rider beside the aim, the routes declaring
  where the knowledge lives, never a guessed attribution.
