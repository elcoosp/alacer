# ADR-0129: M125 scope — the link tier: the hyperlink control and its activation verb

- **Status:** Accepted (2026-11-19)
- **Implements:** the control every document vocabulary speaks as
  "this goes somewhere" — and the tree never carried. Through M124 the
  snapshot said the one-of-N control and the group it reads as
  (ADR-0128), the set words, the form facts, the collection facts, the
  declared live region, the chord, the capacity, the language — and
  the hyperlink had no control: an app wiring one reached for a button
  (the wrong verb — a button's activation is a *press*, a link's is an
  *open*, and the AT's vocabulary names the difference) or for styled
  text wearing no activation fact at all — the declared-but-unreported
  gap (the M94 pattern), cashed at the link tier: "a consumer needs
  the control whose activation the AT names an open → the tree carries
  `Role::Link` (AccessKit's own `Link` role, the fact that is not a
  button wearing styled text), the activation door answering the open
  act the app's navigation rides, the keyboard's verb the platform
  convention names — the control prices the control and nothing else"
- **Companion:** ADR-0023 (the availability wall every door answers
  behind), ADR-0016 (the M10 activation parity — the link's keys are
  the control's own, not every key's), ADR-0076 (the keyboard
  activation naming itself — the Enter-open streams the `Keyboard`
  rider), ADR-0099 (the description tier the destination word rides —
  the tree says "opens", never where), ADR-2 (the one-currency
  discipline the activation joins), ADR-9 (the DAG the change never
  touches), architecture §10.3 (the activation parity), §17 (the
  register), §18 (the M125 roadmap row)

## Decision

- **One role, one verb.**
  [`Role::Link`](crates/alacer-core/src/widget.rs) carries the
  "opens something" fact AccessKit's `Link` role spells — not a button
  wearing styled text: the AT announces a link as something that
  navigates, and the navigation is the app's
  ([`Link::on_open`](crates/alacer-widgets/src/link.rs), the handler
  the destination rides). The tree says "opens", never where — the
  name and the description (M95, ADR-0099) carry the words the app
  declared. The role joins the required-name set (ADR-5): an unnamed
  link reads as bare styled text — the "goes somewhere" the AT cannot
  announce.
- **The activation verb is the link's own.** The click opens on
  release (the button's GTK rule, M6: an auto-repeat hold cannot
  machine-gun the navigation), the AT's default action is the open ask
  the door answers, and the keyboard's key is the platform
  convention's — **Enter** opens, Space is handed on (the scroller's
  key, the framework's to take: the M10 parity at the link tier — the
  activation keys are the control's own, not every key's; the radio's
  Space select is the checkbox vocabulary's own verb, and the link's
  is not). The Enter-open streams one `ActionRecord` with `Keyboard`
  beside it (M70, ADR-0076), the attribution declared where the route
  is known. The availability tier guards every door (M22, ADR-0028):
  an unavailable link refuses, never munges (the M11 rule).
- **The snapshot carries it; the sink maps it.** The pass-9 fill needs
  no new field — the role is the fact, the tree carries it the way it
  carries every role — and the sink maps `Role::Link` onto AccessKit's
  `Link` role. No patch-tier growth beyond the DSL door the widget set
  shares (the `link` element: `name`, `description`, `on_open`), no
  second channel: the activation re-announces exactly the link's node.
- **The living documents re-agree at M125 and 0.17.0.** The ADR count
  (a hundred and thirty), the architecture register (ADR-0129 scopes
  M125), the roadmap row (§18), the CHANGELOG's unreleased block, and
  the examples catalog (the `link` row) all name the tier; the
  m-suite corpus pins re-agree at 144 — the tier grew no snapshot
  route (the role is the fact), and its pricing door is the widgets
  crate's own bench, outside the published corpus by ADR-0128's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The link's cost is priced in
  [`crates/alacer-widgets/benches/link.rs`](crates/alacer-widgets/benches/link.rs):
  `link/open_route_256` — the activation (the handler call, the mask
  request) against `link/refusal_wall_256`, the M22 refusal (the door
  that answers before the request is even parsed). The pair rides
  outside the published baseline: the coordinated rule grows the
  corpus at the next published cut, never at a widget bench's. The
  m-suite pins re-agree at 144, the m125 suite's pin documenting the
  same shape.
- **The AT names the navigation the toolkit always owed it.** A screen
  reader walking a mounted document announces "link, Read the
  architecture" — the activation verb the vocabulary reserves for
  navigation, the destination the description carries — and the app's
  handler rides the open wherever the app says. The wall is the
  contract's other half: an unavailable link reads as the muted text
  it is, every door refusing before the request is parsed.
- **Consumers get the whole vocabulary for free.** The
  `examples/link.rs` beat is the shape: the described link under the
  word, the plain link the honest absence beside it, every activation
  routing through the same door — the status line reading the
  destination back the way the app named it, the neighbours untouched.
