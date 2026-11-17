# ADR-0095 — M91 scope: the AT's vocabulary grows the horizontal pair

Milestone: **M91** (this cut). Parent contract: ADR-0093 (M89), whose
second re-open rule pre-registered this cut in its own words: "a
consumer needs the AT's *horizontal* vocabulary — `ScrollLeft`/
`ScrollRight` beside the unit scrolls — → the action enum grows the
pair, every variant gaining its consuming widget in the same cut (the
enum's own rule)."

## The ask

M89 grew the container's axis, and M90 grew the app's own handle
beside it — but the assistive-technology reader of a wide feed still
had only the vertical vocabulary: a screen reader walking a two-axis
table column by column, a magnifier panning a wide canvas, the same
real reader the M89 ask priced, reaches the x offset through no action
at all. AccessKit's vocabulary has carried the pair all along
(`Action::ScrollLeft`/`Action::ScrollRight`, units and all); the
boundary's mapping dropped them unmapped — the route-drop trail the
only place the ask was ever named. The enum grows the pair: the AT's
horizontal ask becomes a first-class action the scroll area consumes,
the same cut (the enum's own rule — every variant gaining its
consuming widget at once, no variant stranded unmapped).

## The shape

- **The action pair** ([`A11yAction::ScrollLeft(ScrollUnit)`]/
  [`A11yAction::ScrollRight(ScrollUnit)`]): the same unit contract the
  vertical pair travels by — an item is one line (the platform's 40 px
  wheel notch, §10.2), a page is the viewport, the viewport's *width*
  on the x axis. The wire tags (`scroll_left`/`scroll_right`) ride
  [`A11yAction::name`], the boundary's stable `snake_case` rule.
- **The record species** ([`ScrollAction::ScrollLeft`]/
  [`ScrollAction::ScrollRight`]): the ask's own species on the scroll
  log, the wire tags flowing additively (ADR-0010's rule — clients
  that ignore unknown values are unaffected). The record's horizontal
  fact carries the ask's story (outcome, landed offset, ask, wall)
  beside the vertical fields riding the identity answer — the axis
  unasked holds the offset it had, the same convention the M89 content
  door and the M90 handle door answer by. The unit rides the record
  (`Some(unit)`) exactly as the vertical pair's asks ride theirs
  (M52/M54's attribution rule, spoken on the x axis).
- **The consuming widget** ([`List::on_a11y_action`]): the scroll area
  consumes the pair by the same clamp arithmetic every door answers by
  — the declared width span, the same named walls, the state moved
  only on a delivery. A vertical-only list answers
  `no_scrollable_content` against its undeclared span, the AT's ask
  riding beside the refusal where the reader can read it.
- **The boundary mapping** (the platform's `map_action_request`):
  AccessKit `Action::ScrollLeft`/`Action::ScrollRight` map onto the
  core pair like their vertical siblings — the request's unit carried
  through (defaulting to one item, the granularity an AT means when it
  sends no unit). The boundary never drops the pair again.
- **The tier boundary**: the value-tier widgets (the slider, the
  progress bar, the text field) refuse the pair in the same cut — a
  question outside the tier is not a tier fact (the ADR-0046 rule),
  the refusal arms growing with the enum so no variant strands unmapped
  at a widget door.

## Exit criteria

- The AT's horizontal ask lands by its unit: one item moves 40 px, one
  page moves the viewport's width, the landing clamped at the declared
  span's edge — the record's species naming the ask, the horizontal
  fact beside the vertical identity, the pixels flipping where the
  shipped renderer paints them (the m91 exit suite; the a11y route's
  PAINT | A11Y merge composing the landing).
- The walls pin per axis: a run past the ceiling lands at `max`, a run
  past the floor at `min`, the mid-range ask pinning nothing.
- The vertical-only container refuses the horizontal ask honestly
  (`no_scrollable_content`), the vertical vocabulary beside it landing
  untouched — the pair grew beside the vertical vocabulary, never over
  it (the M89 contract re-pinned at this cut).
- The value tier refuses the pair (the slider's route: refused, never
  munged into a value write).
- The corpus grows 96 → 97 by the coordinated rule:
  `a11y_action/route_100_horizontal_actions_1k` — the horizontal pair
  routed through pass 10 on the same 1,000-node arena at the same
  worst-case target, the gate keeping it in the mixed route's cost
  class. The m-suite pins re-agree at 97.
- `examples/at_horizontal.rs` drives the beats from the consumer side.

## Re-open rules (pre-registered)

- a consumer needs the AT's *set-scroll-offset* vocabulary — the AT
  setting the offset directly — → the action enum grows the setter
  beside the pair, the payload tier's own contract naming it (the
  AccessKit `SetScrollOffset` action's own shape).
- a consumer needs the *deeper* containers to follow the AT's
  horizontal ask — a node nested two scrolls deep — → the walk grows a
  cascade beside the nearest-ancestor answer (ADR-0093's first rule,
  standing).
- a consumer needs the handle's x door to carry a *unit* — the app
  snapping by item or paging by viewport width — → the door grows the
  unit beside the scalar (ADR-0094's third rule, standing).
