# ADR-0128: M124 scope — the radio tier: the one-of-N control and the group it reads as

- **Status:** Accepted (2026-11-19)
- **Implements:** the control every settings vocabulary speaks as "one
  of these, not many" — and the tree never carried. Through M123 the
  snapshot said which member a collection picks (the selected tier,
  ADR-0112), the set's span and the member's place (the set tier,
  ADR-0123), the form facts, the collection facts, the declared live
  region, the chord, the capacity, the language — and exclusive choice
  had no control: an app wiring one-of-N reached for a checkbox that
  toggles itself off (the wrong verb — a radio selects, it never
  deselects itself) or for a bare button wearing no selection fact at
  all — the declared-but-unreported gap (the M94 pattern), cashed at
  the radio tier: "a consumer needs the one-of-N control whose
  selection the AT announces and whose group the AT names → the tree
  carries `Role::Radio` (the selection the M6 `toggled` contract
  announces, never the mixed summary) and `Role::RadioGroup` (the
  named container, the set tier's span word its carrier), the user's
  doors selecting and never deselecting, the redundancy the idle
  frame, the exclusivity the app's wiring — the control prices the
  control and nothing else"
- **Companion:** ADR-0098 (the tri-state summary the radio refuses —
  a checkbox's word, never a radio's third bit), ADR-0023 (the
  availability wall every door answers behind), ADR-0123 (the set tier
  whose words the radio rides instead of growing second ones), ADR-5
  (the required-name contract the member and the group both carry),
  ADR-2 (the one-currency discipline the selection joins — the move
  rides `PAINT | A11Y` the toggle's own masks), ADR-9 (the DAG the
  change never touches), architecture §10.3 (the activation parity),
  §17 (the register), §18 (the M124 roadmap row)

## Decision

- **Two roles, one tier.** [`Role::Radio`](crates/alacer-core/src/widget.rs)
  carries the selection: the M6 `toggled` contract announces it exactly
  as the checkbox's — `On` or `Off`, never `Mixed` (the ADR-0098 rule
  restated at the tier: the summary is the select-all parent's word, a
  radio has no children to summarize).
  [`Role::RadioGroup`](crates/alacer-core/src/widget.rs) is the named
  container the set reads as — the group word the AT announces before
  it walks the members, and the set tier's own span word
  ([`Widget::size_of_set`], ADR-0123) its natural carrier: the radio
  tier grows no second set vocabulary, it rides the one published
  since M119. Both roles join the required-name set (ADR-5): an
  unnamed member is a bare mark, an unnamed group a bare box — the
  set the AT cannot name is a set it cannot walk.
- **The user's doors select and never deselect.**
  [`Radio::select`](crates/alacer-widgets/src/radio.rs) is the one
  select path — the click, the Space key, and the AccessKit default
  action all land in it, exactly the checkbox's three-doors-one-funnel
  shape (M6). A checked member asked again is the M21 idle frame: the
  selection does not move, the handler does not fire, and the
  widget's own doors request no masks — the a11y door answers `true`
  (the ask is the radio's own tier) and the re-announcement it costs
  rides the dispatch funnel's coarse `consumed` contract, the one
  coarse corner, named at the module docs where the consumer can
  read it. The one deselect door is the programmatic write
  ([`Radio::set_checked`], view reconciliation) — programmatic
  state, never a user door, the checkbox's own rule.
- **The exclusivity is the app's wiring.** The framework never
  rewrites widgets it does not own (A1), and a radio set is N widgets,
  not one: clearing the sibling the selection left is the app's write,
  through the [`Radio::on_select`] handler and the view's programmatic
  door (the `radio` example's beat). The keyboard's activation names
  itself (M70, ADR-0076) — the Space-select streams one `ActionRecord`
  with `Keyboard` beside it, answered only when the selection moved:
  the redundant ask streams nothing (one fact per trail, the M39 rule).
- **The snapshot carries it; the sink maps it.** `TreeNode::toggled`
  already announces the selection and `TreeNode::position_in_set` /
  `size_of_set` already say the set words — the pass-9 fill needs no
  new field, and the sink maps `Role::Radio` onto AccessKit's
  `RadioButton` (the `Checked` state the toggle tier already carries)
  and `Role::RadioGroup` onto AccessKit's `RadioGroup`. No patch-tier
  growth beyond the DSL door the widget set shares (the `radio` and
  `radio_group` elements: `name`, `checked`, `position`, `size`,
  `on_select`), no second channel: the selection re-announces exactly
  the member's node.
- **The living documents re-agree at M124 and 0.17.0.** The ADR count
  (a hundred and twenty-nine), the architecture register (ADR-0128
  scopes M124), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `radio` row) all name the tier;
  the m-suite corpus pins re-agree at 144 — the tier grew no snapshot
  route (its words are the set tier's), and its pricing door is the
  widgets crate's own bench, outside the published corpus by the
  coordinated rule below.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The radio's cost is priced in
  [`crates/alacer-widgets/benches/radio.rs`](crates/alacer-widgets/benches/radio.rs):
  `radio/select_route_256` — the moving select (the state write, the
  mask request, the handler call) against `radio/redundant_ask_256`,
  the M21 idle path (the consumed ask that moves nothing). The pair
  rides outside the published baseline: the corpus prices the a11y
  snapshot machinery (the M95 rule), the radio grew no snapshot route,
  and the coordinated rule grows the corpus at the next published cut,
  never at a widget bench's. The m-suite pins re-agree at 144, the
  m124 suite's pin documenting the same shape.
- **The AT reads the set the toolkit always owed it.** A screen reader
  walking a mounted set announces the group's name, the member's name,
  the member's place and the span ("2 of 3", the set tier's words), and
  the selection — every fact the ARIA settings canon speaks, from
  controls the framework now grows instead of shapes the app faked. The
  absence is the contract's other half: a member that declared no place
  and a group that declared no size announce neither, never a guessed
  count (the ADR-0087 presence rule at the tier).
- **Consumers get the whole vocabulary for free.** The
  `examples/radio.rs` beat is the shape: the named group under the
  word, the members mounting checked and cleared from the signal, the
  select handler writing the one write the exclusivity rides, the
  status line reading the selection back the way the tree carries it —
  every door answering as it always did, the neighbours untouched.
