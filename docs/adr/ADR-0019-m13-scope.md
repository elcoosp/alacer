# ADR-0019: M13 scope — the word joins the editing contract

- **Status:** Accepted (2026-11-09)
- **Implements:** the §10.3 editing contract's word half: the caret —
  scalar-addressable since M1, keyboard-extended since M11,
  pointer-placed since M12 — becomes **word-addressable**: word-wise
  movement (Alt/Ctrl+arrows), word-wise deletion (Ctrl/Alt+Backspace and
  Delete), and double-click word selection
- **Companion:** ADR-0018 (the pointer seam this rides), ADR-0017 (the
  selection model every writer shares), ADR-0014 (the frame-batched queue
  the clicks arrive through), architecture §10.3, §18 (the M13 roadmap
  row)

## Decision

M12's "why this is the milestone" recorded the exhausted seam: "The
keyboard-only analogies are exhausted too: word-wise movement and
double-click selection need this exact seam and arrive next, cheaply."
M13 is that arrival, in the same "join the contract" shape as
M8/M9/M10/M11/M12 — one contract widens, every consumer rides the same
currency:

- **The text seam carries the word segmentation.** `TextSeam` grows
  `word_segments(text) -> Vec<WordSegment>` — the run's UAX #29 word
  segmentation in the contract unit §10.3 already fixed (UTF-16 offsets,
  half-open segments, `word_like` classifying letters — CJK included —
  and numbers as words, whitespace and punctuation as separators). There
  is **no provided implementation**, deliberately: segmentation is a
  Unicode-table question, and a whitespace-and-ASCII approximation would
  silently mis-hop `don't` and every multi-script run — exactly the
  documented guess the honesty rules forbid. A seam that cannot answer
  fails at compile time, not in front of a user. The workspace's engine
  implementation rides ICU's `WordSegmenter` (`alacer-text`), the same
  segmenter parley's line breaking already adopts, dictionary-backed for
  Thai and friends.
- **The hop rule is one definition, in core.** `word_hop(segments, from,
  direction)` — pure arithmetic over segment positions, living next to
  the contract it serves — fixes the convention: a hop lands on the far
  edge of the word-like segment that spans the caret, or on the near edge
  of the next word-like segment in the hop's direction; separators are
  never landings from outside them, so `", "` takes one hop; the run's
  edges clamp. No engine can grow its own dialect: the widget asks the
  seam for data and core for the rule.
- **The editor consumes it, the M11 semantics riding along.** Alt or
  Ctrl/Meta+Left/Right hop word-wise — Shift extends from the anchor (the
  M11 anchor state is the word hop's anchor too), a plain press over a
  live selection collapses to its near edge first (the GTK rule the
  arrows already follow), and only a second press steps. Ctrl/Alt+
  Backspace and Delete delete **exactly the span the caret would hop** —
  the word, with the separator following in the next chord — and an
  active selection is consumed whole first, the M11 edit rule unchanged.
  Word chords are keys like any other: inert while a preedit composes.
- **The event contract carries the click chain.** `PointerEvent::Down`
  grows `click_count: u8` — 1 a single click, 2 the double-click's second
  press — computed by the platform (`alacer-platform`'s `ClickCounter`:
  presses within 400 ms and 4 px of the previous one chain, GTK's
  defaults), because a double-click is a fact about the user's hand that
  only the platform's clock can witness, and widgets must never carry a
  clock. The M12 frame growth set the shape: one `Option`/count copy per
  routed event, pass 1's walk untouched.
- **The editor answers it.** A `click_count > 1` press selects the word
  under the hit — anchor at the word's start, caret at its end, the
  click's own reading order; a hit on a separator collapses at the hit
  point (a double-click never wanders to the nearest word). During a
  composition the committed text is what offsets map to (the M12 rule),
  and the selection report drops while it composes (the M11 rule).
- **The a11y tree rides unchanged.** A word selection is the same
  anchor+caret tree vocabulary every writer shares — keyboard, drag,
  `SetTextSelection`, and now the double-click — and the AT reads it
  through the M9/M11 machinery untouched, direction included.

### 1. Why this is the milestone

The recorded unblocked seams after M12 were word-wise movement,
double-click selection, clipboard, announcements/live regions, the
pointer-capture seam, `supports_*` advertisement, and a `Slider`. The
word seam is the one the code itself calls its follow-up — ADR-0018 §1
names word-wise movement and double-click selection as arriving "next,
cheaply" through the caret seam M12 just built — and it crosses the
contract at every tier at once: the text seam (a word boundary is a
Unicode-table question the widget must never answer directly), the event
contract (a double-click is a gesture fact the widget cannot time), and
the editing contract (the M11 selection was scalar-complete but
word-inert: no chord moved by more than one scalar value). That is the
roadmap's ratified milestone shape: one contract widens, every consumer
rides the same currency, the corpus grows by the coordinated rule. The
alternative candidates (clipboard, live regions, the pointer-capture
seam) are their own milestones' vocabulary — each needs an adopted core
arboard does not share with the editing contract, and none is unblocked
by a seam the code itself names as its next move.

### 2. The seams that did not move

- **The corpus grows by the coordinated rule, nothing else moves.** One
  bench case (`editing/route_100_word_moves_1k` — one hundred
  direction-alternating word hops routed through the pipeline on a
  1,000-node arena, measured on the shipped `TextInput`, not a probe),
  one regression contract — 50 → 51 cases; the M5–M12 pin suites move
  together.
- **Pass 1's shape is untouched.** The router still walks the arena
  linearly per pointer event; the click count is one `u8` copy per
  delivered `Down`, the keyboard path stays focus-routed with no arena
  walk. The capture/bubble order, the click-to-focus seam, and the M8
  queue semantics are exactly as they were.
- **The DAG is untouched.** `word_segments` lives in `alacer-core`'s
  text seam, `word_hop` beside it, the editor's consumers in
  `alacer-widgets`; the ICU tables ride `alacer-text`, where parley's
  engine already carries `icu_segmenter` in the graph — the workspace
  adopts it explicitly rather than growing a new dependency edge. The
  platform needs no new dependency — the `ClickCounter` is `std`-only.

### 3. What this does not do

- No word-wise drag extension: a drag after a double-click extends
  scalar-wise from the double-click's anchor — the W3C pointer-capture
  seam (§10.1's recorded follow-up) plus a gesture state machine is the
  honest upgrade path, not a widget workaround.
- No triple-click select-all: `click_count` carries the chain (3 = the
  triple), but the field consumes only the word tier; select-all stays
  Ctrl/Cmd+A and the AT's action tier.
- No clipboard (`Copy`/`Cut`/`Paste` remain the arboard milestone's
  vocabulary, ADR-0017 §3 unchanged); no announcements (`SelectionChanged`
  remains the live-region milestone's vocabulary, ADR-0016 §3 unchanged).
- No macOS line-navigation special cases: Meta participates in the word
  chord set (it is the command modifier), and Cmd+Left's line-start role
  is not implemented — the input is single-line and Home/End already
  cover line navigation; the macOS polish is recorded here as debt, not
  hidden.
- No BiDi/RTL hop semantics: the segmentation is visual-run honest for
  the single-line, left-to-right runs the contract covers (the same
  domain statement `hit_test` makes); multi-directional runs need the
  layout-aware seam, not a widget heuristic.
- No cluster-precise engine override for `hit_test`, no caret blink, no
  IME-cursor-rect change: ADR-0018 §3's simplifications stand.

## Consequences

- The workspace stays at one milestone per minor cut: the next release
  (0.10.0) carries M12 and M13.
- The corpus is 51 cases; the M5–M12 suites pin the number together and
  the M13 suite asserts all eight pins plus the published baseline/
  contract pair.
- The baselines are re-published from the collecting host (the
  ADR-0016 procedure): the new case's median rides the same snapshot,
  and the same-host regression gates re-arm against it.
- `TextSeam` grew a required method at 0.9.x: every implementation — the
  engine, the widget tests' stubs — gained `word_segments` in this
  milestone, and external engine implementors follow at the next minor
  (ADR-11's internals-break-at-minors rule, stated in the CHANGELOG).
- A keyboard user moves and deletes by word; a pointer user selects a
  word with the gesture their hand already knows from every other text
  field; and an AT reading the tree sees one more writer of the same
  selection vocabulary it has read since M11 — because the word stayed
  one piece of the one invalidation currency no matter which input
  modality wrote it.
