# ADR-0101 — M97 scope: the range tier

Milestone: **M97** (this cut). Parent contract: the value tier's own
clamp arithmetic (§10.3, ADR-0026/0027) and the a11y sink's M6 mapping
standing beside the fact the value widgets have carried since M20 and
M21 and the tree could never say: the range — the declared span every
write funnels through — AccessKit's `MinNumericValue` /
`MaxNumericValue` properties beside the `NumericValueStep`, ARIA's
`aria-valuemin` / `aria-valuemax` / the step.

## The ask

Every value-tier write since M20 has clamped into a span the widget
declares at construction: `Slider::new(name, min, max)` and
`Progress::new(name, min, max)` refuse a degenerate range outright, and
every door — the drag, the arrows, Home/End, the AT's own `SetValue`,
`Increment`, `Decrement` — lands where the clamp and the snap put it.
The a11y tree read the value ("42") and did not know the span: whether
that was the middle of 0..100 or the top of 0..50. The AT's user heard
a bare number where the sighted user saw the track's walls. The
declared-but-unreported gap, the M94 pattern at the range tier — the
M95 rule (the property the boundary maps, the widget answers) cashed at
the range tier.

## The shape

- **The trait door** (`Widget::value_range`): the provided method
  beside `placeholder()` — `None` is the honest absence, and a widget
  with no span announces nothing, never a guessed default (the ADR-0087
  presence rule at the range tier). The answer is data
  (`ValueRange::new(min, max)` / `.with_step(step)`): walls plus the
  step one act moves, the step `Option` because the report half
  declares where its value lives, not how far one act moves it.
  `Slider` and `Progress` override; every other role inherits the
  default. The `f32` → `f64` widening at the door is exact.
- **The snapshot rides** (`TreeNode::value_range`): pass 9 resolves the
  fact beside the placeholder it resolved at M96 — the same snapshot,
  the same one-copy-per-node price.
- **The sink maps** (`AccessKitSink::access_node`): the walls become
  AccessKit's `MinNumericValue` / `MaxNumericValue`, the declared step
  becomes `NumericValueStep`; the absence sets nothing. The tier maps
  no numeric *now* — the value string the tree already carries is the
  read the AT announces from, and the numeric read is pre-registered
  below, not smuggled in here.
- **No patch tier growth**: the range is constructor-immutable —
  `Slider::new` owns the span's invariant (`min < max`, both finite),
  there is no `set_range` door, and the `min` / `max` / `step` words the
  slider and progress elements have parsed since M32 land where every
  other declared fact lands: the tree, at mount. A re-render that
  repeats the same words queues nothing (an idle frame; the m97 suite
  pins the counters' absence).
- **The bench** (`range/snapshot_1k_ranged`): the ranged 1,000-node
  slider arena against the no-range 1,000-node toggle arena the M94
  binary case already carries — the M95 pricing rule (the tier's cost
  measured against the same corpus, one fact richer). The corpus grows
  108 → 109 by the coordinated rule, the `range_snapshot_overhead`
  ratio budget (≤ 1.25×) holding the span's cost at the measured class:
  the copy the node builds beside the label's own.
- `examples/ranges.rs` drives the beats from the consumer side — the
  ranged slider beside the honest absence, and the value write that
  moves inside the walls the tree carries.

## Re-open rules (pre-registered)

- a consumer needs the *numeric now* (the value the automation and the
  voice-control tier read as a number, `aria-valuenow`) → the sink's
  mapping is one setter beside the walls (`set_numeric_value`), the
  trait door growing the answer beside `value_range` where the value
  widgets already report their scalar — the m97 suite's read doors
  re-pinning the answer beside it.
- a consumer needs the range on other editables (a scroll area's
  content span, a date field's bounds) → the trait door already answers
  them (`value_range()` is not slider-shaped), the producers joining
  without a seam change.
- a consumer needs the *large step* (page increments,
  `NumericValueJump`) → the `ValueRange` shape grows the field the same
  way the step did, the sink mapping it where the walls map.
- a consumer needs the range to *move* (a zoom re-ranging its slider) →
  the write door (`set_range`) joins beside `set_value`, the idle-frame
  property answering whether the span moved — the same rule the
  placeholder write rode at M96.
