# ADR-0093 — M89 scope: the door grows the axis beside the ask

Milestone: **M89** (this cut). Parent contract: ADR-0092 (M88), whose
re-open rule pre-registered this cut in its own words: "a consumer
needs the *horizontal* axis — a container that scrolls on both axes —
→ the door grows the axis beside the ask (the vertical scalar is the
scroll vocabulary's only offset today, ADR-0056's own axis rule
standing; the ask's x half prices itself on a real reader first)."

## The ask

The M88 door priced the bring-into-view ask from the bounds ledger and
handed one scalar to the container's clamp. A wide reader — a row
wider than the viewport, a horizontally paged canvas — prices an x
half the door refuses to carry: the geometry sticks out, the container
holds an x offset (the compose translation has always been two-axes),
and the vocabulary's only ask is vertical. The door grows the axis:
the ask becomes two halves, the clamp answers each by its own
declared span, and the record grows the axis beside the action
(ADR-0056's own axis rule, cashed).

## The shape

- **The ask** ([`crate::widget::BringIntoViewAsk`]): the content-space
  target offsets per axis, priced from the same ledger read — the
  vertical half exactly the M88 ask, the horizontal half by the same
  edge-pin convention (left of the window scrolls to the left edge,
  right of it to the right edge, a rect wider than the viewport pins
  its left edge; the axis that already holds the node asks its own
  identity).
- **The container's answer** ([`crate::widget::ScrollLanding`] +
  [`crate::widget::ScrollAxisLanding`]): the vertical fields keep the
  M88 shape and meaning; the horizontal axis's own answer rides beside
  them — the same clamp arithmetic, the same named walls, the
  refusals named per axis. A vertical-only container answers the x
  half `no_scrollable_content` against its undeclared span — the
  honest wall where the consumer can read it, never a guessed x
  scroll.
- **The record** ([`crate::inspect::ScrollRecord::horizontal`]): one
  additive field carrying the horizontal half's own story (outcome,
  landed offset, ask, wall) beside the vertical fields — `None` on the
  stories with no horizontal half (the AT's unit scrolls ride their
  vertical vocabulary untouched). The wheel's x run reaches the same
  clamp (positive x scrolls right, the platform's own mapping), and
  the two-axis list grows the handles its story needs
  (`List::two_axis`, `set_content_width`, `offset_x`).
- **The wire** (ADR-0010's additive rule): the scroll event grows the
  `horizontal` object beside the vertical fields, `null` on the
  stories with no half; clients that ignore unknown fields are
  unaffected.

## Exit criteria

- The door lands a straddling row on both axes; the landing is data on
  both; the record's horizontal half carries the x story; the stamps
  name the container and the aimed node (the m89 exit suite).
- The wheel runs the x half by the declared span; the wall pins a past
  run (`max`); the pure-x run's vertical half refuses honestly.
- The vertical-only container refuses the x half honestly, the ask the
  geometry priced riding beside the refusal.
- The AT's page scroll moves the vertical half and never the x half;
  the record rides the honest absence.
- The corpus grows 94 → 95 by the coordinated rule:
  `scroll/route_100_horizontal_bring_into_view_1k` — the wide rows
  re-pin the x half on every landing, so the two-axis ask's price is
  the scrolled frame's own. The m-suite pins re-agree at 95.
- `examples/horizontal_scroll.rs` drives the beats from the consumer
  side.

## Re-open rules (pre-registered)

- a consumer needs the *deeper* containers to follow — a node nested
  two scrolls deep asking both — → the walk grows a cascade beside the
  nearest-ancestor answer, each container's ask priced by its own
  clamp (ADR-0092's second rule, standing).
- a consumer needs the AT's *horizontal* vocabulary — `ScrollLeft`/
  `ScrollRight` beside the unit scrolls — → the action enum grows the
  pair, every variant gaining its consuming widget in the same cut
  (the enum's own rule).
- a consumer needs the app's own two-axis handle door — `scroll_to_x`
  beside `scroll_to` — → the handle grows the x half with the same
  named-wall record, priced by a real reader first.
- a consumer needs the walk's landing to *follow* — the composition
  joins the two doors at the call site first (ADR-0092's third rule,
  standing).
