# ADR-0051: M45 scope — the keyboard's ride joins the value log, attributed

- **Status:** Accepted (2026-11-13)
- **Implements:** ADR-0049's first kill rule — "a consumer needs the
  keyboard's … value rides on the log *attributed* → grow the record
  beside the funnel's other doors (the key route …), priced by a real
  reader — never a per-pointer-move flood on a devtools surface
  (ADR-0046's second kill rule stands)"
- **Companion:** ADR-0049 (the stamp discipline this route adopts),
  ADR-0046/ADR-0050 (the record vocabulary the ride streams), ADR-0026
  (the value tier whose funnel the arrows share), ADR-0028 (the
  availability tier the keyboard's wall belongs to), architecture
  §13.3, §18 (the M45 roadmap row)

## Decision

- **The key route holds the stamp the action route holds.**
  `alacer-core/src/passes.rs`: `visit_key` — the funnel's key visit —
  holds `value::ActionTargetGuard::for_target(Some(id_hash(node)))`
  for the length of the focused widget's key call; a value-tier
  widget's key door notes its ride and the record carries the focused
  control's AccessKit-stable id, the same vocabulary the action route
  stamps (ADR-0049).
- **The idle contract is a Cell read, never an id-path walk.** The
  guard is held only when the trail is enabled (`value::
  value_trail_enabled`, one `Cell` read per routed key); while nobody
  listens no walk runs and no fact can queue (A7).
- **The widget's door notes its own fact.** `alacer-widgets/src/
  slider.rs`: the arrows route through `step_note`, which streams the
  step's own action species (`Increment`/`Decrement`), the landed
  value, M44's ask beside it, and a `None` target the funnel stamps —
  the AT tier's door and the keyboard's door now answer on one log,
  attributable by the route that routed them.
- **Home/End stay off the log.** The record vocabulary has no bound
  action; a jump munged into a `SetValue` would be the M11 rule's own
  munging. **The drag stays off.** Per-pointer-move facts are the
  flood the kill rule refuses; the input tier carries the ride.
- **The M43 silence pin re-agrees.** The m43 suite's keyboard-silence
  test becomes the M45 attribution pin — the supersedence the M40
  scope decision pre-registered as its own open arm.
- **The corpus stands at 63; the living documents re-agree at the
  cut** (the README, the register, the protocol document's keyboard
  sentence, the CHANGELOG, the m-suite pins).

## Kill criteria (re-open rules)

- A consumer needs the drag's rides *attributed* on the log → price a
  bounded pointer-tier record first (per-gesture, not per-move) —
  never a per-move flood on a devtools surface.
- A consumer needs Home/End on the log → grow the action vocabulary
  with a named bound species beside `SetValue` — never a munged jump.
