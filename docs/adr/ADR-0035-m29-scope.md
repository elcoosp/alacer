# ADR-0035: M29 scope — the directional keys ride the walk: the D-pad rule cashed at the dispatch seam

- **Status:** Accepted (2026-11-11)
- **Implements:** the re-open rule ADR-0032 pre-registered (and
  ADR-0033 re-stated) — "a platform key needs to ride a direction (a
  D-pad button, a TV remote's ring) → map the key onto `FocusMove` at
  the platform's key-fallback seam, priced by a real backend — never a
  second dispatch channel (ADR-0032's rule stands)"
- **Companion:** ADR-0030 (the funnel the arrows ride), ADR-0032 (the
  directions and the never-wrap rule the arrows answer), ADR-0033 (the
  outcome vocabulary the dispatch reads), ADR-0034 (the causal trail
  the arrows' rides stream into), ADR-0028 (the availability tier the
  walk's filter keeps), ADR-0029 (the plain-help-text rule the walk's
  refusals carry), architecture §10.3 (the keyboard contract the
  fallback belongs to), §18 (the M29 roadmap row)

## Decision

The four directional moves have ridden two doors since M26 — the
keyboard's Tab fallback and the consumer's code. The platform's own
directional keys could not: a D-pad button or a TV remote's ring sends
`ArrowLeft/Right/Up/Down`, and a focused widget that does not answer
arrows (a button, a panel) left the key ignored — the window's own
navigation vocabulary, refused at the seam the M24 cut built for
exactly this shape. M29 cashes the rule:

- **The dispatch funnel maps the four arrows onto the walk.** At
  `PassPipeline::dispatch_key`'s fallback — the seam Tab and Shift+Tab
  ride since M24 — a plain directional key the focused widget did not
  answer maps onto its `FocusMove` (`ArrowLeft → Left`, `ArrowRight →
  Right`, `ArrowUp → Up`, `ArrowDown → Down`) and rides the same
  funnel (`focus_move`). The platform's key map already carries the
  arrows (`alacer-platform/src/keys.rs`: winit's `NamedKey` arrows map
  onto the framework's arrow keys), so the seam is one `match` — no
  second dispatch channel, no platform-side walk, the order never
  duplicated at a call site (ADR-0030's rule, kept literally).
- **The widget's answer always takes precedence.** The fallback fires
  only when the focused widget's response is `Ignored` — or nothing
  holds focus. A focused input's caret, a slider's step, the word-hop
  and selection chords: the widget's vocabulary is the widget's
  business (the m20 suite's arrow-step contract and the m11/m13
  caret contracts stand untouched; the m29 suite pins the slider's
  precedence headlessly). From no focus, a directional key takes the
  walk's entry contract (the first candidate, direction-agnostic —
  ADR-0032's repair).
- **A modifier held is a chord, and the chords are the widgets'.** The
  fallback fires on the plain four arrows only — shift, control, alt,
  or meta held means the key belongs to whatever vocabulary answers
  it (a caret's Shift+Arrow, a word hop's Alt+Arrow, a platform
  accelerator); the window's walk is never a chord's rider. The D-pad
  story stays honest: a remote's ring sends plain arrows.
- **The refusal falls through honestly.** A direction the walk refuses
  (the M26 edge: nothing fully beyond the focused node's edge; space
  never wraps) leaves the response `Ignored` — the key is nobody's,
  exactly the shape of a refused Tab — and the M28 causal trail
  carries the arrows' rides like every ride on the funnel: the record
  streams when the door resolves, the movement the key named, the
  landing's endpoints or the refusal species. One funnel, now three
  riders (Tab, the code, the ring); one trail.
- **The demonstration walks the ring.**
  `examples/dpad_navigation.rs` walks the M26 window — the two-by-two
  grid whose tree order and spatial reading disagree, plus a slider
  the widget's own arrows answer — with arrow presses through the
  real `dispatch_key` at startup, before the window opens: the entry
  from nothing, the geometry's stride, the edge's honest fall-through,
  and the focused slider's precedence. Tracing at
  `alacer::dpad_navigation`, the workspace's per-seam idiom. The m29
  exit suite pins the contract headlessly on the real grid through the
  real layout host: the arrows land the geometry's answer, the
  widget's answer takes precedence (the slider steps, the focus
  stays), a refused direction falls through with the refusal recorded,
  the arrows' rides stream to the causal log, and the corpus stands at
  62 — the arrows ride the published routes, no new criterion case.
- **The living documents re-agree at M29 and 0.17.0.** The README's
  status block (M0 through M29), the ADR count (thirty-six), the
  features bullet, the quickstart, and the example table; the catalog's
  row; the register (35 → 36 with this ADR); the §10.3 keyboard
  paragraph; the roadmap row; the m-suite pins, which move across the
  m5–m28 chain in the same cut (the count M28 → M29, the register 35 →
  36); and the CHANGELOG stages the milestone.

## Consequences

- The dispatch funnel grows one fallback arm — the additive change the
  M24 seam was built for. Call sites routing on Tab's verdict keep
  reading the door's answer; the arrows ride behind the same response
  contract (`Handled` when the walk lands, `Ignored` when it refuses).
- The hot path carries no new work: the fallback is a `match` on the
  key the dispatch already routed, the walk behind it unchanged, the
  published focus routes pricing the walk's cost per move as shipped.
- The perf corpus stands at 62: the arrows ride the walk the published
  routes already measure — the coordinated rule's no-growth arm (no new
  criterion case, no baseline move, the m5 bijection green as pinned).
- The m1 corpus does not grow: the demonstration shows no new visual
  arrangement (the D-pad demo walks the M26 window's own paint), and
  the m1 suite's 37-scenario pin stands.
- No new dependency edges: the arrows are the framework's own key
  vocabulary, the mapping is core's, and the spans ride the `tracing`
  core every member already resolves.

## Kill criteria (re-open rules)

- A platform needs directions the four arrows cannot name (a ring's
  diagonals, a jog dial's steps) → grow the mapping beside the arrow
  `match` at the same seam, priced by a real backend — never a second
  dispatch channel (ADR-0030's rule stands).
- A widget needs the window's arrow (not its own vocabulary) — a
  control that wants the walk to move focus while it holds focus →
  that is a shape the `Ignored` response cannot express; grow the
  response vocabulary beside `EventResponse`, priced by a real widget —
  never a bypass of the funnel.
- A consumer needs the arrows' rides distinguished from the code's
  rides in the causal log → grow `FocusRecord` beside its `movement`
  (a rider field), priced by a real reader — never a second trail.
