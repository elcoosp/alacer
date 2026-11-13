# ADR-0052: M46 scope — the drag's story joins the log, per-gesture

- **Status:** Accepted (2026-11-13)
- **Implements:** ADR-0046's second kill rule's open arm and ADR-0051's
  first kill rule — "a consumer needs the drag's rides *attributed* on
  the log → price a bounded pointer-tier record first (per-gesture,
  not per-move) — never a per-move flood on a devtools surface"
- **Companion:** ADR-0025 (the capture the stamp reads), ADR-0049
  (the stamp discipline this route adopts), ADR-0046/ADR-0050 (the
  value tier whose funnel the drag shares), ADR-0028 (the availability
  tier whose wall arms nothing), architecture §13.3, §18 (the M46
  roadmap row)

## Decision

- **The drag's record is per-gesture, never per-move.**
  `alacer-core/src/inspect.rs`: one `GestureRecord` —
  `phase` (`Started`/`Ended`), the value the drag found (`from`), the
  value it left the control holding (`to`, `None` on the beginning —
  the landing is the gesture's answer, not its beginning's fact), and
  the control (`target`). One hundred moves inside one gesture stream
  exactly two records; the flood rule holds by construction.
- **The trail is the value trail's shape.** `alacer-core/src/gesture.rs`:
  the bounded thread queue (`GESTURE_TRAIL_CAP`, oldest-first
  eviction), the enabled `Cell` (A7: one branch per gesture phase
  while nobody listens), the note site at the exact place the phase
  resolves, and the tick's-start drain into
  `InspectEvent::Gesture` (the value drain's shape, running after it —
  a gesture explains the value the pointer moved).
- **The pointer route holds the stamp the action and key routes hold.**
  `alacer-core/src/passes.rs`: the hit-test route and the captured
  route hold `gesture::GestureTargetGuard::for_target(id_hash(target))`
  for the length of one routed event's walks — the deepest hit that
  captured (ADR-0025) is the *who* the drag's door cannot know (the
  M43 rule's words at the pointer tier). The idle contract is the key
  route's own: one `Cell` read while nobody listens, no id-path walk.
- **The widget's door notes its own phases.**
  `alacer-widgets/src/slider.rs`: the press arms the drag and notes
  `Started` (the value it found, before the press's own write lands);
  the release and the leave note `Ended` (the landing beside the
  remembered beginning — each record self-contained). The wall
  (M22, ADR-0028) arms nothing and streams nothing: a gesture that
  never was is the honest absence.
- **The wire grows the event additively at version 1.**
  `alacer-inspect/src/wire.rs`: the flat `gesture` event (the value
  event's rule — phase, from, to, target); the recorder's
  `gestures` ring (512, the trails' order); the embedded client
  rendering the phase beside the found and landed values.
- **The corpus grows 63 → 64** (`gestures/route_100_drag_moves_1k`,
  measured, published, and gated at this run — the coordinated rule);
  **the living documents re-agree at the cut** (the README, the
  register, the protocol document's gesture section, the catalog, the
  CHANGELOG, the m-suite pins).

## Kill criteria (re-open rules)

- A consumer needs the drag's *mid-flight* story (the value at each
  stride) → price a bounded per-gesture trajectory first (a sampled
  curve, never a per-move record) — the flood rule standing.
- A second gesture kind needs the log (a drag-selection's span, a
  scroll's glide) → grow the record's vocabulary beside the phases —
  never a per-move record on a devtools surface.
