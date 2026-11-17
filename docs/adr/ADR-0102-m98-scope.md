# ADR-0102 — M98 scope: the orientation tier

Milestone: **M98** (this cut). Parent contract: the scroll tier's
per-axis arithmetic (§10.2, ADR-0056; two-axis since M89, ADR-0093)
and the a11y sink's M6 mapping standing beside the fact every scroll
container has declared at construction since M0 and M89 and the tree
could never say: the axis — the single track a container declares its
scrolls to ride — AccessKit's `Orientation` property, ARIA's
`aria-orientation`.

## The ask

Every scroll door since M50 answers per axis, and since M89 the axes
are a declared fact: `List::new(height)` names the y axis the viewport's
only scrollable story, `List::two_axis(width, height)` declares both —
the x doors answering `no_scrollable_content` against a vertical-only
list's empty span, the wheel's x run reaching the same walls, the AT's
`ScrollLeft`/`ScrollRight` (ADR-0095) traveling the same unit contract.
The a11y tree read "scroll area" and did not know which: whether the
AT's arrows ride a vertical track, a horizontal one, or both. The
AT's user heard a scrollable region where the sighted user saw a
track with a direction. The declared-but-unreported gap, the M94
pattern at the orientation tier — the M95 rule (the property the
boundary maps, the widget answers) cashed at the orientation tier.

## The shape

- **The trait door** (`Widget::scroll_orientation`): the provided
  method beside `value_range()` — the answer is data
  (`ScrollOrientation::Vertical` / `ScrollOrientation::Horizontal`, the
  AccessKit `Orientation` shape mirrored in core so the DAG stays
  adapter-free, ADR-9). `None` is the honest absence (the ADR-0087
  presence rule at the orientation tier): a widget that is no scroll
  container announces nothing, and a container that declared *both*
  axes has no single-axis word to say — ARIA's `undefined` is the
  spec's own word for "both axes", so the absence is the two-axis
  answer, never a guessed half. `List` overrides; every other role
  inherits the default.
- **The declaration is the constructor's** (the M97 rule at the
  orientation tier): the viewport half the list's shared state carries
  is written by no door — the handle's doors move the offsets and the
  content spans, never the viewport the widget was built with — so the
  answer is constructor-fixed, not a guess from the content's current
  span. A scroll moves the offset and moves nothing the tree says
  about the axis (the m98 suite pins the declaration surviving the
  scroll).
- **The snapshot rides** (`TreeNode::scroll_orientation`): pass 9
  resolves the fact beside the range it resolved at M97 — the same
  snapshot, the same one-copy-per-node price.
- **The sink maps** (`AccessKitSink::access_node`): the declared axis
  becomes AccessKit's `Orientation` property; the absence sets
  nothing. The tier maps no *scroll geometry* — the bounds the tree
  already carries are the read the AT announces from, and the wheel
  conventions stay platform facts.
- **No patch tier growth**: the declaration is constructor-immutable —
  `List::new` and `List::two_axis` own their axes' invariant, there is
  no `set_orientation` door, and the `height` word the list element
  has parsed since the M0 registry builds the vertical-only
  constructor, so the DSL's lists declare through the same door with
  no new prop class to pay for. A re-render that repeats the words
  queues nothing.
- **The bench** (`orientation/snapshot_1k_vertical`): the 1,000-node
  vertical list arena against the no-fact 1,000-node toggle arena the
  M94 binary case already carries — the M95 pricing rule (the tier's
  cost measured against the same corpus, one fact richer). The corpus
  grows 109 → 110 by the coordinated rule, the
  `orientation_snapshot_overhead` ratio budget (≤ 1.25×) holding the
  declaration's cost at the measured class: the enum's option the node
  builds beside the label's own. The m-suite pins re-agree at 110.
- `examples/orientation.rs` drives the beats from the consumer side —
  the vertical list beside the honest absence, the trait door read
  back beside the tree's truth, and the scrolls that move the offsets
  while the axis words stay.

## Re-open rules (pre-registered)

- a consumer needs the *both* word carried (AccessKit grows the
  variant) → the sink's mapping is one arm beside the two it has, the
  trait door growing the variant beside the axis word — the m98
  suite's read doors re-pinning the answer beside it.
- a consumer needs the declaration *movable* (a pane re-orients at
  runtime) → the write door (`set_orientation`) joins beside the
  handle's doors, the idle-frame property answering whether the word
  moved — the same rule the placeholder write rode at M96.
- a consumer needs the axis word on other containers (a horizontal
  scroll strip role, a custom scroll container) → the trait door
  already answers them (`scroll_orientation()` is not list-shaped),
  the producers joining without a seam change.
- a consumer needs the AT to *read* the declared span beside the axis
  (orientation + the content span in one announce) → the tree already
  carries both facts beside each other; the composition is the
  adapter's story, not the vocabulary's.
