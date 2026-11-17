# ADR-0099 — M95 scope: the description tier

Milestone: **M95** (this cut). Parent contract: the `Widget` trait's own
a11y reporting set (§11.1) and the a11y sink's M6 mapping standing beside
the vocabulary the boundary declares and no widget produced: AccessKit's
`Description` property — ARIA's `aria-description` — the help text a
screen reader reads when the name alone does not answer "what does this
control do?".

## The ask

The a11y tree has always carried the name and the value — the two answers
the AT speaks first — and never the *description*. The property is
declared at the boundary (every platform adapter maps it; the sink's
`Node` vocabulary names it), and the tree could never say it: a consumer
composing a form had only the label, so the control either explained
itself in its name (the name is the identity, not the instruction) or
dropped the semantics entirely. The state the vocabulary declares is the
state the widget must produce — the M94 rule at the description tier:
the property the boundary maps, the widget answers.

## The shape

- **The trait door** (`Widget::description`): the provided method beside
  `label()` — `None` is the honest absence (the ADR-0087 presence rule at
  the description tier): a widget with nothing to explain announces
  nothing, never a guess. Actionable controls override; containers and
  reports inherit the default.
- **The snapshot rides** (`TreeNode::description`): pass 9 resolves the
  fact beside the value and the toggle state it already resolves — the
  same snapshot every other property rides, one `String` per node on the
  rebuild the label's own copy already prices.
- **The sink maps** (`AccessKitSink::access_node`): the help text becomes
  AccessKit's `Description` property; the absence sets nothing, and the
  AT reads the node exactly as it read the tree M94 handed over.
- **The producers**: `Button::description`, `Checkbox::description`, and
  `TextInput::description` mount the help text; each carries the one
  programmatic write (`set_description(Option<String>)`) answering
  whether it moved — the write that lands where the text already is
  queues nothing (the M21 idle-frame property, at the description tier).
- **The view DSL** (`description:` word): the button, checkbox, and
  text-input elements parse the vocabulary at create and patch. The
  patch tier grows the **a11y-only prop class**: a changed `description`
  word queues exactly `Change::A11Y` — the re-announcement without the
  repaint — and counts in the patch report (the M72 rule: one fact per
  patch that moved something). The pixels never pay for a fact the
  painter did not write.
- **The bench** (`describe/snapshot_1k_described`): the same 1,000-node
  checkbox arena the toggle route's binary case carries, every node
  described — the corpus grows 103 → 104 by the coordinated rule, the
  `describe_snapshot_overhead` ratio budget (≤ 1.25×) holding the
  description's cost at the measured class: the help text rides the
  snapshot as the second `String` the node builds beside the label's
  own, one bounded copy per node per rebuilt frame.
- `examples/descriptions.rs` drives the beats from the consumer side —
  the described canon beside the honest absences, and the
  description-only patch the pixels never pay for.

## Re-open rules (pre-registered)

- a consumer needs the *description* on other producers (a container's
  group description, a slider's format hint) → the trait door already
  answers them (`description()` is not control-shaped), the producers
  joining without a seam change.
- a consumer needs a *derived* widget to carry the description → the
  derive grows the attribute beside `role`/`label` (the M92 form's own
  vocabulary), the generated code calling the same builder.
- a consumer needs the description *reactive* (a help text that follows
  a signal) → the `Binding` prop class already carries reactive text,
  the sink resolving the bound string the label's own rule rides.
