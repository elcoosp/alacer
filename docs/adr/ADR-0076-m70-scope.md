# ADR-0076: M70 scope — the activation names its rider

- **Status:** Accepted (2026-11-14)
- **Implements:** ADR-0075's own kill rule — a consumer cannot tell the
  AT's press from the keyboard's Space/Enter activation on the log the
  action trail streams (the record answered "did the press land — or
  why not", never "who pressed") → the record grows the rider beside
  the ask, never a second trail
- **Companion:** ADR-0036 (the M30 attribution rule the rider joins at
  the activation tier), ADR-0016 (the M10 parity the keyboard's
  activation rides — the focused button answering Space/Enter like a
  click), ADR-0051 (the key route's stamp the keyboard's ride carries —
  M45), ADR-0028 (the availability wall whose keyboard silence stays
  the honest absence), ADR-0010 (the additive wire rule), architecture
  §10.3, §13.3, §18 (the M70 roadmap row)

## Decision

- **The record grows the rider.**
  [`ActionRecord`](crates/alacer-core/src/inspect.rs) carries
  [`ActionRider`](crates/alacer-core/src/inspect.rs) beside the ask —
  the M30 attribution rule at the activation tier.
  `ActionRider::ActionRoute` is the dispatch funnel's own note site
  (the AT's ask the platform's action route drives);
  `ActionRider::Keyboard` is the M10 parity the widgets answer like a
  click — the focused button's Space/Enter (the GTK release rule: the
  click fires on key release, so an auto-repeat hold cannot
  machine-gun the handler) and the focused checkbox's toggle-on-press
  (HTML semantics).
- **The attribution is declared where the route is known.** The
  funnel's notes are the action route's by construction; the widgets'
  key doors name their own rider at the exact place the activation
  resolves — one fact per activation (the flood rule at the activation
  tier: the button's pressed phase arms nothing on the log, the
  release is where the door answered). The keyboard's ride carries the
  key route's stamp (M45, ADR-0051): the focused control's
  AccessKit-stable id, the guard the key visit holds — the identity
  never guessed at the note site.
- **The wire grows additively at version 1.** The `action` event
  carries the new `rider` field (`action_route` / `keyboard`,
  `ActionRider::name` the single source) — the additive rule ADR-0010
  names; clients that ignore unknown fields are unaffected; the
  embedded client renders "the AT's press landed" and "the keyboard's
  press landed" on one log.
- **The held-down control's keyboard silence stays honest.** A
  disabled control never holds focus (the M22 derivation) and a routed
  activation key is ignored — the M22 keyboard silence is the wall's
  own contract, not a dropped fact; the AT route's wall is the priced
  fact (`ActionRefusal::Unavailable`, the M69 species).
- **No new corpus case.** The keyboard's activation rides the routes
  the press bench already prices (the coordinated rule's no-growth
  arm, the M62 precedent); the corpus stands at 69.
- **The demo walks the rider beat.** `examples/action_trail.rs`
  routes the keyboard's Space activation beside the AT beats — the
  same log, the riders tellable apart.
