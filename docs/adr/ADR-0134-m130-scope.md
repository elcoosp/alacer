# ADR-0134: M130 scope — the switch tier: the instant on/off control, said by the tree

- **Status:** Accepted (2026-11-19)
- **Implements:** the control every settings vocabulary speaks as "flip
  this, now" — and the tree never named. Through M129 the snapshot
  said the tool strip (ADR-0133), the bounded number field
  (ADR-0132), the page selector and the strip it reads as
  (ADR-0131), the section title (ADR-0130), the hyperlink
  (ADR-0129), the one-of-N control and its group (ADR-0128), the set
  words, the form facts, the collection facts — and the instant
  toggle had no word: an app wiring one reached for a checkbox
  wearing the wrong word (a checkbox's write resolves a *form
  contract* — checked, unchecked, the mixed summary the select-all
  parent needs, ADR-0098; a switch's write is the instant on/off the
  settings vocabulary speaks, and a switch has no summary to hold) or
  for a button wearing no state at all (a press with no after-state
  — the M127 rule, at the settings tier).
  [`Role::Switch`](crates/alacer-core/src/widget.rs) carries the
  "writes one on/off fact" fact AccessKit's `Switch` role spells —
  the checkbox's member shape (ADR-0099's description tier joining
  beside the name) at the instant-toggle tier.
- **Companion:** ADR-0098 (the tri-state summary is the select-all
  parent's word, never a switch's — the binary pair the switch
  carries), ADR-0028 (the availability wall every door refuses
  behind), ADR-0076 (the keyboard's activation naming itself — the
  Space record the funnel streams), ADR-0005 (the ADR-5 required-name
  rule — an unnamed switch a bare pill), ADR-2 (the one-currency
  discipline the word rides), ADR-9 (the DAG the change never
  touches), architecture §10.3 (the activation parity), §17 (the
  register), §18 (the M130 roadmap row)

## Decision

- **One role, the control's own word.**
  [`Role::Switch`](crates/alacer-core/src/widget.rs) carries the
  "writes one on/off fact" fact AccessKit's `Switch` role spells —
  not a checkbox wearing the wrong word, not a button wearing no
  state: the AT announces the on/off beside the name, and the write
  resolves the binary pair. The sink maps `Role::Switch` onto
  AccessKit's `Switch`, one branch beside the checkbox's; the binary
  pair rides the `Toggled` property the sink has mapped since the
  toggle tier (the M6 contract's `toggled` half — never a second
  state vocabulary). The role joins the required-name set (ADR-5) —
  and the focusable set: a switch is a control, the click, the Space
  key and the AT's ask the doors it answers (the checkbox's own
  rule, at the instant tier).
- **The binary pair, never a summary.** A switch carries `On` or
  `Off` and nothing else — ADR-0098's resolution rule restated at the
  switch tier, from the other side: the mixed summary is the
  select-all parent's word, a summary the switch does not hold and
  the tree never guesses. Every user door moves the tree's truth — a
  switch toggles, and toggles again (the radio's never-deselect rule
  is the radio's own; the switch's write is the flip).
- **Three doors, one funnel.** The
  [`Switch`](crates/alacer-widgets/src/switch.rs) control answers the
  pointer (the press arms, the release toggles — the checkbox's
  route), the keyboard (Space while focused — the M70 activation
  record streaming `Delivered` or the availability wall's
  `Refused(Unavailable)`, never a munged answer), and the AT's
  default action (the dispatch funnel queueing `PAINT | A11Y` when
  the door consumes) — the checkbox's three-doors-one-funnel shape,
  at the instant tier. The availability tier guards the funnel's
  single entry (M22, ADR-0028): an unavailable switch refuses every
  door honestly, holds no focus, paints the muted track — the M11
  discipline (refused, never munged) at the switch tier.
- **The programmatic door reports the move.**
  [`Switch::set_on`] (view reconciliation) fires no handler —
  programmatic state, never a user toggle — and answers whether the
  flag moved, the M95 description door's own shape: the caller
  requests `PAINT | A11Y` only when the write moved, so a write that
  lands where the on/off already is queues nothing (the M21
  idle-frame property, at the switch tier).
- **The living documents re-agree at M130 and 0.17.0.** The ADR count
  (a hundred and thirty-five), the architecture register (ADR-0134
  scopes M130), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `switch` row) all name the
  tier; the m-suite corpus pins re-agree at 144 — the tier grew no
  snapshot route (the binary pair riding the `Toggled` mapping the
  sink has carried since M6, the role branch the only sink growth),
  and its pricing door is the widgets crate's own bench, outside the
  published corpus by ADR-0131's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The switch's cost is priced in
  [`crates/alacer-widgets/benches/switch.rs`](crates/alacer-widgets/benches/switch.rs):
  `switch/toggle_route_256` — the moving toggle (the pointer's full
  press-and-release, the state write, the mask request) against
  `switch/idle_write_256`, the programmatic write that lands where
  the on/off already is (the honest floor of a write at the widget
  seam, the M21 idle path). The pair rides outside the published
  baseline: the coordinated rule grows the corpus at the next
  published cut, never at a widget bench's. The m-suite pins re-agree
  at 144, the m130 suite's pin documenting the same shape.
- **The AT names the instant toggle the toolkit always owed it.** A
  screen reader walking a settings page announces "switch, Wi-Fi, on"
  — the role word beside the name, the fact beside the label — and
  its default action flips the control the way the pointer's click
  does. The switch walls nothing: the availability, focus and
  description doors answer exactly as they always did — the word
  names the on/off, it walls nothing.
- **Consumers get the whole vocabulary for free.** The
  `examples/switch.rs` beat is the shape: the switches the app wires,
  the flags one signal carries, the status line reading the facts
  back the way the tree holds them — the neighbours untouched.
