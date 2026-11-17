# ADR-0103 — M99 scope: the numeric tier

Milestone: **M99** (this cut). Parent contract: the value tier's own
funnel (§10.3, ADR-0026/0027), the range tier's declared span
(ADR-0101) and the a11y sink's M6 mapping standing beside the fact the
value widgets have reported since M3 and the tree could only say as
prose: the number behind the value's spoken word — AccessKit's
`NumericValue` property, ARIA's `aria-valuenow`.

## The ask

Every value-tier widget since M3 has reported its scalar through the
`value()` contract, and since M97 the tree carries the span the writes
funnel through. But the value itself rode as *text*: the AT read "40"
(`aria-valuetext`) and never the number 40 (`aria-valuenow`) — the
automation and voice-control tier that reads range widgets as numbers
read prose, and a report like "42" of 0..100 was a string to parse, not
a fact. The declared-but-unreported gap, the M94 pattern at the numeric
tier — ADR-0101's own re-open rule pre-registered this cash verbatim
("a consumer needs the *numeric now* … the sink's mapping is one setter
beside the walls (`set_numeric_value`), the trait door growing the
answer beside `value_range` where the value widgets already report
their scalar — the m97 suite's read doors re-pinning the answer beside
it"), and the rule is cashed here at the numeric tier.

## The shape

- **The trait door** (`Widget::numeric_value`): the provided method
  beside `value_range()` — `None` is the honest absence, and a widget
  whose value is prose announces no number, never a parsed guess (the
  ADR-0087 presence rule at the numeric tier): a text field's "16
  digits on the front" is not 16, and a valueless role announces
  nothing. The answer is the scalar the funnel holds (`f64`); the
  widening `f32` → `f64` is exact, the same widening the walls rode at
  M97. `Slider` and `Progress` override; every other role inherits the
  default.
- **The snapshot rides** (`TreeNode::numeric_value`): pass 9 resolves
  the fact beside the range it resolved at M97 — the same snapshot,
  the same one-copy-per-node price. The text value rides exactly as it
  always has: the number is *beside* the spoken word, never instead of
  it (`aria-valuenow` and `aria-valuetext` are a pair; AccessKit's
  `Value` property stays the AT's announcement source).
- **The sink maps** (`AccessKitSink::access_node`): the number becomes
  AccessKit's `NumericValue` property beside the walls and the step the
  M97 mapping already sets — one setter, exactly the shape the re-open
  rule named. The absence sets nothing.
- **No patch tier growth**: the number is derived, not declared —
  `Slider`'s and `Progress`'s scalars move through the doors they have
  answered since M20 and M21 (the funnel, the app's report write), and
  the tree's number follows those landings. There is no
  `set_numeric_value` door, no new prop class, and a re-render that
  repeats the same `value` word queues nothing (an idle frame; the m20 suite's own
  property, re-pinned by the m99 suite).
- **The bench** (`numeric/snapshot_1k_valued`): the 1,000-node value
  arena whose every node answers the number against the no-fact
  1,000-node toggle arena the M94 binary case already carries — the
  M95 pricing rule (the tier's cost measured against the same corpus,
  one fact richer). The corpus grows 110 → 111 by the coordinated
  rule, the `numeric_snapshot_overhead` ratio budget (≤ 1.25×) holding
  the number's cost at the measured class: the `f64` option the node
  builds beside the label's own. The m-suite pins re-agree at 111.
- `examples/numeric.rs` drives the beats from the consumer side — the
  slider and the report whose numbers ride the funnel beside the text,
  and the *Snap to max* write that moves both shapes of the answer
  together.

## Re-open rules (pre-registered)

- a consumer needs the *large step* (page increments,
  `NumericValueJump`) → the `ValueRange` shape grows the field the
  same way the step did at M97, the sink mapping it where the walls
  map — the m99 suite's read doors re-pinning beside it.
- a consumer needs the *value text derived* (a slider whose
  announcement reads "42 of 0..100") → the `value()` contract's answer
  is the widget's own prose door; the tree carries what the widget
  says, and a composed valuetext is the widget's story, not the
  vocabulary's.
- a consumer needs the numeric now on other scalars (a scroll
  container's offset, a date field's epoch day) → the trait door
  already answers them (`numeric_value()` is not slider-shaped), the
  producers joining without a seam change.
- a consumer needs the number to *move* without the funnel (an
  automation harness pinning a report) → the report's write doors are
  the app's (`Progress`'s bridge and mutate seam, ADR-0027); the AT's
  write tier stays refused on the read half — the no-write-door rule
  the M21 wall named, never a munged back door.
