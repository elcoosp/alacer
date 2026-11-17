# ADR-0094 — M90 scope: the handle grows the x half

Milestone: **M90** (this cut). Parent contract: ADR-0093 (M89), whose
third re-open rule pre-registered this cut in its own words: "a
consumer needs the app's own *two-axis handle door* — `scroll_to_x`
beside `scroll_to` — → the handle grows the x half with the same
named-wall record, priced by a real reader first."

## The ask

M89 grew the container's axis: the wheel runs the x half, the
bring-into-view door prices both halves, the content door declares the
width — and the record grew the axis beside the action. The app's own
programmatic ask still had only the vertical scalar: a consumer whose
app code pans a wide feed (a horizontal pager snapping to a column, a
kiosk drifting a banner back into frame, the same real reader the M89
ask priced) reaches the x offset only through geometry (a synthetic
bring-into-view for a node that is already visible) or the wheel (a
synthetic event for an ask the app itself owns). The honest door is
the handle's: the ask is the app's own scalar, the door is the same
one `scroll_to` answers by, and the record is the same named-wall
record.

## The shape

- **The door** ([`ListHandle::scroll_to_x`]): the x half beside
  [`ListHandle::scroll_to`], answered by the same
  `answer_horizontal_ask` clamp arithmetic every x door here answers
  by — the declared width span, the same named walls
  (`no_scrollable_content` against the undeclared span,
  `non_finite_ask` before the arithmetic), the state moved only on a
  delivery. A vertical-only list answers `no_scrollable_content`
  against its undeclared span — the honest wall the M89 rule names,
  met at the handle's own door.
- **The record** (ADR-0056's axis rule, cashed again): the same
  [`ScrollRecord`] the vertical door streams under
  [`ScrollAction::ScrollTo`] — the x story riding its horizontal half
  (the outcome, the landed offset, the ask, the wall the clamp
  pinned) beside the vertical fields riding the *identity answer*:
  delivered, the held offset, no ask beside it (the same convention
  the M89 content door's width half answers by — the axis unasked
  holds the offset it had, never a guessed y scroll). The door rides
  no route: the stamps carry the honest absence, the app door's own
  rule (the M43 discipline at the x half).
- **The frame**: the handle knows no event context, so the visible
  motion composes at the seam every programmatic scroll already
  surfaces through — the mutate-guard `request` seam
  (`arena.mutate(node)` + `request(PAINT | A11Y)`, the M23 shape and
  ADR-0093's call-site composition rule). The suite composes the
  landing there and reads the pixels where the shipped renderer paints
  them; the record half is pinned bare, the M50 suite's own shape for
  the handle doors.

## Exit criteria

- The app's x ask lands: the offset moves, the record's horizontal
  half carries the ask's whole story beside the vertical identity, and
  the pixels flip where the shipped renderer paints them (the m90
  exit suite, both halves of the contract).
- The wall pins a past run (`min` below, `max` above); the ask that
  wanted the wall exactly pins nothing.
- The non-finite ask refuses before the arithmetic
  (`non_finite_ask`), the offset keeping what it held.
- The vertical-only container refuses the x ask honestly
  (`no_scrollable_content`), the ask the app priced riding beside the
  refusal.
- The stamps ride their honest absence (`target`/`aimed`/`unit`/
  `lines` all `None` — the app door's rule).
- The corpus grows 95 → 96 by the coordinated rule:
  `scroll/route_100_handle_scroll_x_1k` — the app's own x door driven
  the way apps drive it (the mutate seam's PAINT | A11Y merge), one
  hundred wall-to-wall asks, each landing moving the whole visible
  band. The m-suite pins re-agree at 96.
- `examples/handle_scroll_x.rs` drives the beats from the consumer
  side.

## Re-open rules (pre-registered)

- a consumer needs the *deeper* containers to follow the handle's x
  ask the way the bring-into-view walk follows — → the handle grows a
  cascade beside the nearest-ancestor answer, each container's ask
  priced by its own clamp (ADR-0093's first rule, standing).
- a consumer needs the AT's *horizontal* vocabulary — `ScrollLeft`/
  `ScrollRight` beside the unit scrolls — → the action enum grows the
  pair, every variant gaining its consuming widget in the same cut
  (the enum's own rule; the M91 cut pre-registers it).
- a consumer needs the handle's x ask to carry a *unit* (snap by
  item, page by viewport width) — → the door grows the unit beside
  the scalar, the record's `unit` field the fact it already is.
