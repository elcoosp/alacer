# ADR-0018: M12 scope — the pointer joins the editing contract

- **Status:** Accepted (2026-11-09)
- **Implements:** the §10.3 input contract's pointer half at the text
  boundary: the caret — until now keyboard- and AT-addressable only —
  becomes **pointer-addressable**, the click-to-caret seam M11 recorded
  as its follow-up
- **Companion:** ADR-0017 (the selection model this drives), ADR-0014
  (the frame-batched queue the gestures arrive through), ADR-5 (the a11y
  contract), architecture §10.1–§10.3, §18 (the M12 roadmap row)

## Decision

M11's "what this does not do" recorded the deferred seam: "no
caret-from-click mapping: a pointer press collapses the selection but
does not move the caret, because placing a caret from an x-coordinate
needs the bounds-aware hit-test seam (the widget's resolved bounds do
not reach `on_pointer_event`)". M12 closes it in the same "join the
contract" shape as M8/M9/M10/M11 — one contract, three consumers:

- **The event contract carries the resolved frame.** `EventCtx` grows
  `bounds()` — the handling node's resolved layout rectangle in window
  coordinates — and the pass-1 router fills it from the arena before the
  capture walk (`with_phase_and_bounds`). Window-coordinate pointer
  events are only mappable to widget-local math against the frame layout
  resolved; keyboard and IME events are focus-routed, not positioned,
  and carry no frame. Direct dispatch (probes, replay) keeps the `None`
  fallback: the x is already field-local.
- **The text seam inverts what paint measures.** `TextSeam` grows
  `hit_test(text, font_size, x) -> usize` — the UTF-16 offset whose
  boundary is nearest to the pixel x, in the contract unit §10.3 already
  fixed. The provided implementation is a binary search over the run's
  scalar boundaries on the prefix widths `layout_glyphs` already
  measures — exactly the widths the caret and the M11 highlight paint
  from, inverted — with ties breaking to the earlier boundary (a click
  inside a cluster never lands past it). Engines may override with a
  cluster-precise mapping; the inversion stays honest for the
  single-line, left-to-right runs the input covers because prefix
  advances are non-decreasing along them.
- **The editor consumes it.** A press maps its x through the seam and
  collapses anchor+caret onto the hit point (the pre-M12 honest fallback
  — collapse at the stale caret — is deleted); a move with the press
  held extends the selection from the anchor, the M11 anchor state
  serving as the drag anchor with no new state beyond the press flag; a
  release ends the gesture and the range survives as a state; an
  unpressed move is hover and never moves the caret. During a
  composition the committed text is what offsets map to — the preedit is
  overlay spliced at the caret, not a click target. Every request rides
  the existing masks: `PAINT | A11Y` for caret and selection moves,
  `FOCUS` for the click-to-focus seam, unchanged.
- **The a11y tree carries it, direction included.** The drag's
  selection reaches the AT through the M9/M11 machinery untouched —
  anchor at the press, focus at the drag end, in the drag's own
  direction, dropping while a preedit composes. A leftward drag reports
  anchor > focus; the AT reads the gesture's shape, not a normalized
  range.
- **The AT tier and the pointer write the same state.** `SetTextSelection`
  (M11) and a drag (M12) are interchangeable upstream: the range either
  writes is consumed by the same edits, painted by the same highlight,
  reported by the same `selection()`.

### 1. Why this is the milestone

The recorded unblocked seams after M11 were announcements/live regions,
clipboard, caret-from-click, word-wise movement, multi-line selection,
mouse-drag extension, `supports_*` advertisement, and a `Slider`. The
caret-from-click seam is the one the code itself calls its follow-up
(the press handler's comment named it "the ADR's recorded follow-up"),
and it crosses three subsystems at once — the event contract (a
window-coordinate x is useless to a widget that cannot see its own
frame), the text seam (mapping pixels to offsets is a text-engine
question the widget must never answer directly), and the editing
contract (the M11 selection was keyboard-complete but pointer-inert: a
mouse user could not place the caret at all). That is exactly the shape
of milestone this roadmap ratifies: one contract widens, every consumer
rides the same currency, the corpus grows by the coordinated rule. The
keyboard-only analogies are exhausted too: word-wise movement and
double-click selection need this exact seam and arrive next, cheaply.
The alternative candidates (a hit-test spatial index for the routing
walk, atlas eviction) remain single-pass performance repairs — they
move a number, not a contract.

### 2. The seams that did not move

- **The corpus grows by the coordinated rule, nothing else moves.** One
  bench case (`editing/route_100_caret_clicks_1k` — one hundred queued
  caret placements routed through the pipeline on a 1,000-node arena,
  measured on the shipped `TextInput`, not a probe), one regression
  contract — 49 → 50 cases; the M5/M6/M7/M8/M9/M10/M11 pin suites move
  together.
- **Pass 1's shape is untouched.** The router still walks the arena
  linearly per event (§10.1's spatial index remains the recorded
  follow-up); the frame delivery is one `Option<Rect>` copy per visit.
  The capture/bubble order, the click-to-focus seam, and the M8 queue
  semantics are exactly as they were.
- **The DAG is untouched.** `bounds()` lives in `alacer-core`'s event
  vocabulary and `hit_test` in its text seam; neither the widgets nor
  the text engine learn a new dependency; the platform needs no change —
  the gestures already arrive through the M8 queue.

### 3. What this does not do

- No drag beyond the field's frame: the routing seam's contains-test is
  the delivery contract, so a move outside the resolved bounds is not
  delivered — extending a selection past the field needs the W3C
  pointer-capture seam (§10.1's recorded follow-up), not a widget
  workaround.
- No double-click word selection, no word-wise movement (Alt+arrows),
  no multi-line: they need the same seam plus a word-boundary contract
  the seam does not carry yet, and the input is single-line.
- No clipboard (`Copy`/`Cut`/`Paste` remain the arboard milestone's
  vocabulary, ADR-0017 §3 unchanged).
- No cluster-precise engine override: parley carries cluster indexes,
  and a `ThreadLocalTextSeam::hit_test` override is the honest upgrade
  path; the provided prefix-width inversion is exact for the
  single-line runs the contract covers and test-enforced round-trip.
- No caret blink, no IME-cursor-rect change: the caret rect still
  reports the field's frame (the recorded simplification), and the
  caret's visibility contract is focus, not time.
- No announcements (`SelectionChanged` is the live-region milestone's
  vocabulary, ADR-0016 §3 unchanged).

## Consequences

- The workspace stays at one milestone per minor cut: the next release
  (0.10.0) carries M12.
- The corpus is 50 cases; the M5–M11 suites pin the number together and
  the M12 suite asserts all seven pins plus the published baseline/
  contract pair.
- The baselines are re-published from the collecting host (the
  ADR-0016 procedure): the new case's median rides the same snapshot,
  and the same-host regression gates re-arm against it.
- A pointer user can finally *edit* like a keyboard user already could:
  click to place the caret, drag to select the range, Backspace to
  consume it — and an AT reading the tree sees the drag's own anchor
  and focus, direction included, because the selection stays one piece
  of tree vocabulary no matter which input modality wrote it.
