# ADR-0105 — M101 scope: the scroll position tier

Milestone: **M101** (this cut). Parent contract: the scroll doors'
named-wall arithmetic (ADR-0056; the two-axis split, ADR-0093) standing
beside the fact the doors have resolved since M50 and the tree could
never say: where a scroll container *is* — the resolved position and
extent — AccessKit's `ScrollY` / `ScrollYMin` / `ScrollYMax` properties
(the `ScrollX` halves beside them on a two-axis container), the
arithmetic every platform AT turns into "scrolled 200 of 0..600".

## The ask

Since M50 the scroll doors land offsets and clamp them against
declared spans; since M89 the container keeps both axes' stories; since
M98 the tree says which axis the scrolls ride. What the tree never said
is where the container *was*: the AT read the axis word and the walls
the actions answered, never the position and extent the doors landed —
a screen reader could name the track but not the carriage on it. The
declared-but-unreported gap, the M94 pattern at the scroll position
tier — the M95 rule (the fact the doors resolve, the tree says) cashed
at the scroll position tier.

## The shape

- **The trait door** (`Widget::scroll_position`): the provided method
  beside `scroll_orientation()` — each axis rides its own honesty (the
  ADR-0087 presence rule at the scroll position tier): an axis whose
  span is empty (the content fits the viewport, or the width story was
  never declared) carries no position — the same wall every scroll
  door answers `no_scrollable_content` with — never a guessed zero. A
  container with no live axis on either side answers `None`. The two
  facts ride one type (`ScrollPosition`, `ScrollAxisPosition` — the
  offset beside the wall it clamps to), mirrored in core so the DAG
  stays adapter-free (ADR-9).
- **The snapshot carries it** (`TreeNode::scroll_position`): pass 9
  resolves the pair beside the axis word it already resolved — the
  state the doors move read at the a11y pass, no second channel.
- **The sink maps** (`AccessKitSink::access_node`): a live vertical
  axis maps `ScrollY` / `ScrollYMin` / `ScrollYMax`; a live horizontal
  axis maps the `ScrollX` halves beside them. The floor the funnel
  clamps to is carried, not guessed: the clamp arithmetic's own zero
  (the span every door answers by), the same discipline the range
  tier's walls ride (ADR-0101). The honest absence maps to the
  absence: a container with no live axis sets nothing, and the AT
  reads the node exactly as it read the tree M100 handed over.
- **The tree's currency rule is the wheel's own**: the doors that pay
  the a11y currency (the wheel, the AT's asks, bring-into-view) merge
  `PAINT | A11Y` on the container, so the position re-announces the
  frame the offset moved; the app's own handle door rides the next
  paid pass — the M90 rule the trail half already answers by, now
  stated for the tree half too.
- **No patch-tier growth**: the position is runtime state the doors
  resolve, not a declarable word — no DSL prop class to pay for, the
  way M98 priced the orientation (the declaration is the
  constructor's) and M99 priced the number (the value the funnel
  holds).
- **The bench** (`scroll_position/snapshot_1k_scrolled`): the
  1,000-node scrolled list arena against the same 1,000-node list
  arena carrying only the axis word (`orientation/snapshot_1k_vertical`,
  the M98 case) — the M95 pricing rule (the tier's cost measured
  against the same corpus, one fact richer). The corpus grows 112 →
  113 by the coordinated rule, the `scroll_position_snapshot_overhead`
  ratio budget (≤ 1.25×) holding the word's cost at the measured
  class: the two axis options the node builds beside the orientation
  word's own. The m-suite pins re-agree at 113.
- `examples/scroll_position.rs` drives the beats from the consumer
  side — the scrolled list beside the bare one, the AT's own page ask
  landing the position the status line reads back, and the wheel
  paying the currency natively.

## Re-open rules (pre-registered)

- a consumer needs the position on *other* containers (a custom scroll
  host beyond the list) → the trait door already answers them
  (`scroll_position()` is not list-shaped beyond the span arithmetic),
  the producers joining without a seam change.
- a consumer needs the *proportion* named on the tree (a
  percentage-bearing word beside the raw offsets) → the composition
  is the AT's story — every platform adapter computes the percentage
  from the position and extent the tree now carries; the vocabulary
  maps no derived word (the M99 rule: never a parsed guess beside the
  numbers the tree says).
- a consumer needs the offset's story on the log too (a trail record
  naming the tree's re-announcement) → the trail already names every
  scroll ask (ADR-0056); the tree half rides the same asks, and a
  second record per ask would double-count one fact (the ADR-0046
  kill rule).
- a consumer needs scroll *sub*-facts the snapshot cannot carry (the
  per-child visibility the bring-into-view door prices) → AccessKit's
  text-position model is the documented simplification's own seam
  (ADR-0017); the vocabulary grows no per-child geometry until a
  consumer prices it (the ADR-0046 kill rule).
