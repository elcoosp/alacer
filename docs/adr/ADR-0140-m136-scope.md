# ADR-0140: M136 scope — the tooltip tier: the named help bubble the pointer pair summons, said by the tree

- **Status:** Accepted (2026-11-20)
- **Implements:** the word the hover-help vocabulary speaks — and the
  bubble never said. Through M135 the snapshot said the tree tier
  (ADR-0139), the dialog tier (ADR-0138), the meter tier (ADR-0137),
  the search field (ADR-0136), the named pane (ADR-0135) — and the
  help words had two shapes: the always-on description tier (M95,
  ADR-0099, the help text *beside a name*) and the modal window (M134,
  ADR-0138, the conversation *in a window*). The *transient* help
  bubble had no word of its own: an app wiring a hover-help reached
  for a second label wearing no container word (the bubble the AT
  cannot name is a bubble it cannot offer on demand — the ADR-5 rule
  the named roles carry) or for a dialog wearing the wrong word (a
  dialog is a modal window the walk bounds itself into; a tooltip is a
  transient bubble the pointer pair summons — the wrong word wearing
  the wrong transient). [`Role::Tooltip`]
  (crates/alacer-core/src/widget.rs) carries the bubble word
  AccessKit's own `Tooltip` role spells — the member-and-container
  shape (ADR-0128) grown to the hover tier.
- **Companion:** ADR-0128 (the member-and-container discipline the
  word grows), ADR-0133/0135/0138/0139 (the M129/M131/M134/M135
  named-container precedents the tier follows — the strip, the pane,
  the window, the collection), ADR-0099 (the description tier — the
  always-on help word the bubble never replaces), ADR-0031 (the focus
  scope the modal window bounds itself with — the door the bubble
  never asks), ADR-5 (the required-name rule the bubble joins), ADR-2
  (the one-currency discipline the change never leaves), ADR-9 (the
  DAG the change never touches), architecture §7.2 pass 9 (the
  snapshot the word rides), §17 (the register), §18 (the M136 roadmap
  row)

## Decision

- **One word, the bubble's own.** [`Role::Tooltip`]
  (crates/alacer-core/src/widget.rs) carries the "reads as a
  hover-help bubble" fact — AccessKit's own `Tooltip` role spells it.
  The role joins the required-name set (an unnamed bubble reads as a
  bare box, the help the AT cannot name is a help it cannot offer on
  demand — the ADR-5 rule the named roles carry) and never joins the
  focusable set (a bubble is not a control: it holds no focus and
  answers no activation — the M126/M129 container rule, at the hover
  tier; the controls the app mounts around it hold the focus). The
  role is *not* the description tier's word worn twice: the M95
  description is the help text beside a name, always on; the tooltip
  is the bubble itself, present exactly while the app mounts it. And
  it is *not* the dialog's word worn down: a dialog is a modal window
  the walk bounds itself into (ADR-0031's focus scope); a tooltip is a
  transient bubble the pointer pair summons, asking no scope and
  answering no action. **Pre-registered re-open rule:** a consumer
  needing the bubble to hold focus or answer activation grows it
  beside the funnel, priced by a real tooltip — never a second walk,
  and never by mutating what this word means.
- **The bubble is the help's layout, never a second channel to it.**
  [`Tooltip`](crates/alacer-widgets/src/tooltip.rs) — the children
  ride the arena exactly a pane's do (a vertical run, the shape a
  bubble is), each child keeping its role, its label, and its doors:
  the label reads, the bubble naming what it helps. No door behind the
  word changes — the tier prices the word and nothing else; the help's
  behaviour is the behaviour it always was.
- **Presence is the app's wiring, never a widget flag.** Whether the
  bubble is up is the view's own mount and unmount — the reconciler's
  shape since M1 (a conditional child, the subtree removed when the
  next patch says so). A `visible` flag would be a second truth the
  tree would have to guess at: the snapshot says what is mounted, and
  what is mounted is what the app declared this frame. The word's one
  programmatic write is [`Tooltip::set_name`]
  (crates/alacer-widgets/src/tooltip.rs) — view reconciliation, the
  write that lands where the word already is queueing nothing (the M21
  idle-frame property, at the tooltip tier).
- **The living documents re-agree at M136 and 0.17.0** (the README's
  status block, register line, widget vocabulary, and example counts;
  the catalog; the register; the roadmap; the m-suite pins moving
  M135 → M136 across the chain; the CHANGELOG). The corpus pin
  re-agrees at 144 — the tier grows no snapshot route (the word is the
  container's own, the shape the M124/M127/M129/M131/M134/M135 named
  containers rode), and its pricing door is the widgets crate's own
  bench ([`benches/tooltip.rs`](crates/alacer-widgets/benches/tooltip.rs),
  `tooltip/name_write_256` against `tooltip/idle_write_256`), outside
  the published corpus by ADR-0131's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The word's cost is priced in
  [`crates/alacer-widgets/benches/tooltip.rs`]
  (crates/alacer-widgets/benches/tooltip.rs): `tooltip/name_write_256`
  — the moving rename (the `String`'s write the bubble's word takes)
  against `tooltip/idle_write_256`, the write that lands where the
  word already is (the honest floor of the same write at the widget
  seam, the M21 idle path priced). The pair rides outside the
  published baseline: the coordinated rule grows the corpus at the
  next published cut, never at a widget bench's. The m-suite pins
  re-agree at 144, the m136 suite's pin documenting the same shape.
- **The AT offers the help on demand, at last.** A screen reader
  encountering a mounted tooltip announces the bubble by name — the
  hover-help the vocabulary could not say before — and the bubble's
  children read as themselves, the help text keeping the word it
  always carried. The word walls nothing: the controls around the
  bubble answer exactly as they always did, and the focus the app's
  controls need stays the walk the framework already publishes.
- **Consumers get the whole vocabulary for free.** The
  `examples/tooltip.rs` beat is the shape: the control the bubble
  helps, the toggle mounting and unmounting the bubble (presence the
  app's wiring, the reconciler's own shape), the help text keeping its
  own word, the reads reading back the way the app wired them — the
  neighbours untouched.
