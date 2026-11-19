# ADR-0132: M128 scope — the spin-button tier: the bounded number field the AT walks by arrows

- **Status:** Accepted (2026-11-19)
- **Implements:** the control every quantity-and-count vocabulary
  speaks as "one bounded number, discretely written" — and the tree
  never carried. Through M127 the snapshot said the page selector and
  its strip (ADR-0131), the section title the document outline walks
  (ADR-0130), the hyperlink and its open verb (ADR-0129), the set
  words, the form facts, the collection facts, the declared live
  region, the chord, the capacity, the language — and the number field
  had no tier: an app wiring one reached for a slider wearing text
  (the wrong shape — a slider's write is the *continuous drag* the
  pointer speaks, a spin button's write is the *discrete step* the
  arrows speak) or for a bare text input carrying no value words at
  all. The tier ships one role beside one machinery:
  [`Role::SpinButton`](crates/alacer-core/src/widget.rs) carrying the
  "writes one bounded number" fact AccessKit's own `SpinButton` role
  spells, the numbers riding the value tier's own vocabulary — never a
  second one
- **Companion:** ADR-0026 (the M20 value tier — the write funnel the
  field's every door rides, `SetValue` parsed then snapped like every
  other write), ADR-0101 (the M97 range word — the span the AT reads
  before it asks, the step riding beside the walls), ADR-0103 (the
  M99 numeric word — the number behind the value's spoken word),
  ADR-0051/ADR-0053/ADR-0055 (the M45 keyboard route, the M47 named
  jump species, the M49 pinned-bound rule — the value log's discipline
  the field inherits whole), ADR-0005 (the ADR-5 required-name rule —
  an unnamed number field a bare number), ADR-0023 (the availability
  wall every door answers behind), ADR-2 (the one-currency discipline
  the step joins), ADR-9 (the DAG the change never touches),
  architecture §10.3 (the activation parity), §17 (the register), §18
  (the M128 roadmap row)

## Decision

- **One role, the value tier's own words.**
  [`Role::SpinButton`](crates/alacer-core/src/widget.rs) carries the
  "writes one bounded number" fact AccessKit's `SpinButton` role
  spells — not a slider wearing text, not a bare input wearing
  nothing: the AT announces a spin button as a number field it walks
  by arrows, and the numbers ride the words the tree already carries —
  the value the M3 contract ([`Widget::value`]), the span the M97
  range word ([`Widget::value_range`], the step beside the walls), the
  number the M99 numeric word ([`Widget::numeric_value`]). The sink
  maps `Role::SpinButton` onto AccessKit's `SpinButton` role, one
  branch beside the tab pair's; the value properties ride the
  M97/M99 mappings, no new branch. The role joins the required-name
  set (ADR-5).
- **Four doors, one funnel, one contract.** An arrow key (the slider's
  own key vocabulary — the arrows the value tier already speaks),
  Home/End (the M47 named jump species, never a munged `SetValue`),
  a stepper click (the discrete write a spin button is — the release
  over the stacked pair steps by its sign, the funnel's
  clamp-and-snap deciding where it lands), and the AccessKit value
  tier's `SetValue`/`Increment`/`Decrement` all reach the value
  through the one clamp-snap-commit funnel the slider published —
  `total_cmp` equality, never `f32::eq`; a write that lands where the
  value already is queues no masks and fires no handler (the M21
  idle frame); `DefaultPress` is refused, never munged into a write
  (the M11 rule — a value control has no press). The availability
  tier guards every door (M22, ADR-0028): an unavailable field
  refuses, never munges. There is no drag and no gesture log: a
  click that never was a gesture streams nothing.
- **The value log's discipline, inherited whole.** Every *AT* write
  and every *keyboard* step streams one
  [`ValueRecord`](crates/alacer-core/src/value.rs) at the exact place
  the answer resolves — the step's own species
  (`Increment`/`Decrement`), the jump's own species
  (`JumpToMin`/`JumpToMax`), the ask beside the landing (M44),
  the wall a pinned landing names (M49) — while the stepper *clicks*
  stay off the value log (the flood rule at the discrete tier: the
  pointer's facts ride the dispatch funnel's own record). No patch
  route grows: the field's words are the value tier's, published
  since M3/M97/M99.
- **The living documents re-agree at M128 and 0.17.0.** The ADR count
  (a hundred and thirty-three), the architecture register (ADR-0132
  scopes M128), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `spin_button` row) all name
  the tier; the m-suite corpus pins re-agree at 144 — the tier grew no
  snapshot route, and its pricing door is the widgets crate's own
  bench, outside the published corpus by ADR-0131's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The field's cost is priced in
  [`crates/alacer-widgets/benches/spin_button.rs`](crates/alacer-widgets/benches/spin_button.rs):
  `spin_button/step_route_256` — the moving step (the state write, the
  mask request, the handler call, the record the M45 rule streams)
  against `spin_button/redundant_ask_256`, the M21 idle path (the
  arrow the wall held, the write that landed where the value already
  was). The pair rides outside the published baseline: the coordinated
  rule grows the corpus at the next published cut, never at a widget
  bench's. The m-suite pins re-agree at 144, the m128 suite's pin
  documenting the same shape.
- **The AT names the number field the toolkit always owed it.** A
  screen reader walking a mounted field announces "spin button,
  Quantity, 3" — the value words the vocabulary reserves for the
  number, the span and the step the AT reads before it asks. The wall
  is the contract's other half: an unavailable field reads as the
  muted text it is, every door refusing before the request is parsed.
  The field's own frame joins the snapshot corpus (`spin_button.png`,
  blessed by the m128 suite) — the tier's visual arrangement held by
  the same regression net every styled widget rides.
- **Consumers get the whole vocabulary for free.** The
  `examples/spin_button.rs` beat is the shape: the named field under
  the word, the value mounted from the signal, every write — arrow,
  stepper, AT act — landing through the funnel the constructor
  mounted, and the status line reading the value back the way the
  tree carries it — the neighbours untouched.
