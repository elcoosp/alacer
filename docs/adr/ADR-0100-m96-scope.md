# ADR-0100 — M96 scope: the placeholder tier

Milestone: **M96** (this cut). Parent contract: the text input's own
paint vocabulary (§10.3) and the a11y sink's M6 mapping standing beside
the fact the canvas has carried since M6 and the tree could never say:
the placeholder — the hint an empty field paints — AccessKit's
`Placeholder` property, ARIA's `aria-placeholder`.

## The ask

The field has painted its hint since M6: the grey sentence an empty
input shows, the format the label cannot carry ("Find a parts number…").
The a11y tree read an empty value where the sighted user read the hint —
the declared-but-unreported gap, the M94 pattern at the placeholder tier.
The fact the painter tells is the fact the tree must carry — the M95
rule (ADR-0099's own pre-registered re-open rule, cashed one cut later):
the property the boundary maps, the widget answers.

## The shape

- **The trait door** (`Widget::placeholder`): the provided method beside
  `description()` — `None` is the honest absence, and an empty
  placeholder *is* the absence, never an empty hint (the ADR-0087
  presence rule at the placeholder tier). `TextInput` overrides; every
  other role inherits the default.
- **The snapshot rides** (`TreeNode::placeholder`): pass 9 resolves the
  fact beside the description it resolved at M95 — the same snapshot,
  the same one-copy-per-node price.
- **The sink maps** (`AccessKitSink::access_node`): the hint becomes
  AccessKit's `Placeholder` property; the absence sets nothing. The
  property stays mounted once the value lands — the value is what the
  AT reads first, the painter stops drawing the hint when the field is
  non-empty, and the AT reads the same two truths the canvas shows.
- **The write is honest** (`TextInput::set_placeholder`): the door grows
  the moved answer — the write that lands where the hint already is
  queues nothing (the M21 idle-frame property, at the placeholder tier),
  the `visual` mask the registry patcher reports riding the widget's own
  comparison, never an unconditional write.
- **The view DSL** (`placeholder:` word, already parsed): the patch tier
  grows the **a11y-*also* prop class** beside M95's a11y-only one — the
  hint the pixels *do* carry keeps the paint masks beside the
  re-announcement the a11y class adds. The tree's truth and the painted
  hint move together, each tier paying its own mask; the re-render that
  lands where the hint already is queues nothing (an idle frame).
- **The bench** (`placeholder/snapshot_1k_placeholdered` beside
  `placeholder/snapshot_1k_bare`): the hinted 1,000-node input arena
  against the same arena bare — the corpus grows 103 → 108 by the
  coordinated rule (the pair, the M95 case, and the two orphans the full
  re-publish repaired: `canvas/paint_100_canvases` and
  `inspect/tick_with_recorder_attached`, bench code the M5 rule had
  carried unpublished, gated now), the `placeholder_snapshot_overhead`
  ratio budget (≤ 1.25×) holding the hint's cost at the measured class:
  the second bounded copy the node builds beside the label's own.
- `examples/placeholders.rs` drives the beats from the consumer side —
  the hinted field beside the honest absence, and the *Swap hint* patch
  paying both masks the tier costs.

## Re-open rules (pre-registered)

- a consumer needs the *hint* on other editables (a multi-line area, a
  search-combining field) → the trait door already answers them
  (`placeholder()` is not input-shaped), the producers joining without a
  seam change.
- a consumer needs the hint to *clear* when the value lands (the tree's
  answer following the paint exactly) → the sink's mapping is one `if
  let` at the snapshot build, changed where the route is known, the
  m96 suite's read doors re-pinning the answer beside it.
- a consumer needs the hint *reactive* (a hint that follows a signal) →
  the `Binding` prop class already carries reactive text, the sink
  resolving the bound string the label's own rule rides.
