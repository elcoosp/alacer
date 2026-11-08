# ADR-0016: M10 scope — the a11y action tier joins the dispatch contract

- **Status:** Accepted (2026-11-08)
- **Implements:** the §11.5 contract's third leg at the real boundary:
  reachable, named, and **actionable** — where "actionable" finally means
  the full action vocabulary an assistive technology sends, routed
  faithfully, refused honestly
- **Companion:** ADR-0015 (the drain protocol that unblocked this tier),
  ADR-5 (the a11y contract this widens), architecture §11.5 (actions), §18
  (the M10 roadmap row)

## Decision

M9 made the pipeline's sink the only `TreeUpdate` path and explicitly
unblocked "the next tier of a11y work (actions, announcements, live
regions)". M10 delivers that tier's first half — **actions** — as a
routing contract, in the same "join the contract" shape as M8 and M9:

- `alacer-core` grows the action vocabulary from the single `DefaultPress`
  to the AT tier: `Increment`, `Decrement`, `ScrollUp(ScrollUnit)`,
  `ScrollDown(ScrollUnit)` — with `ScrollUnit` mirrored in core
  (`Item | Page`) so the DAG stays adapter-free (ADR-9). Pass 10 already
  routes any `A11yAction`; the enum *is* the seam, and it grows with a
  consuming widget in the same milestone, never speculatively.
- `alacer-platform` deletes the pre-M10 munge: every `ActionRequest` was
  forced to `DefaultPress` — a braille display's `Increment` clicked the
  button it touched, a `Focus` request would have pressed it. The mapping
  is now faithful (`Click`, `Increment`, `Decrement`, `ScrollUp`,
  `ScrollDown`, with the request's `ScrollUnit` carried through); actions
  with no widget contract drop at the boundary, traced, instead of
  masquerading as presses.
- The widget canon consumes the vocabulary: the `List` answers
  `ScrollUp/ScrollDown` by unit (an item is the platform's 40 px line, a
  page is the viewport) and `Increment/Decrement` by a page — a scrollbar
  whose value *is* the offset — clamped exactly like the wheel path,
  translation-only, zero measure calls.
- Keyboard parity rides the same milestone because it is the same
  contract at the keyboard boundary: a focused `Button` answers Space and
  Enter like a pointer press-and-release (GTK semantics — the click fires
  on key release, so an auto-repeat hold cannot machine-gun the handler).
  Today a focused button with a visible focus ring is Space/Enter-dead;
  reachable + named + visible-focus must be operable or the contract is a
  claim, not a fact.
- The harness gains `act(role, name, action)` — the full tier is drivable
  through the a11y query API, returning the consumption verdict so tests
  can assert refusals; `press` becomes `act(.., DefaultPress)` with its
  assertion.
- The bench pair grows by one: `a11y_action/route_100_actions_1k` —
  100 mixed actions routed through pass 10 on a 1,000-node arena,
  worst-case target — published with a same-host regression gate, keeping
  the action route in the same cost class as a pointer route, never an
  a11y re-materialization.

### 1. Why this is the milestone

The a11y moat (§16 I1) is the thesis's crown jewel, and its weakest link
is exactly this: M3/M9 made the tree *readable* incrementally, M6 made
widgets *nameable*, but the *operable* leg still stopped at one action —
and the platform silently munged everything else into it, which is the
same docs-drift-with-a-latent-wiring-bug class M8 repaired for input and
M9 for output. ADR-0015 named this tier "unblocked rather than blocked";
unblocked seams are the roadmap. The alternative candidates (a hit-test
spatial index, the M3-era LRU atlas eviction debt) are performance
repairs inside one pass — they move a number, not a contract.

### 2. The seams that did not move

- **The corpus grows by the coordinated rule, nothing else moves.** One
  bench file, one criterion id, one regression contract — 47 → 48 cases;
  the M5/M6/M7/M8/M9 pin suites move together (the M10 suite asserts all
  five).
- **Pass 10's dispatch shape is untouched.** Consumption still merges
  `PAINT | A11Y` on the target and nothing else; refusal queues no masks
  (test-enforced: an unconsumed action costs an idle frame).
- **The DAG is untouched.** `ScrollUnit` is mirrored in core, so neither
  core nor the widgets learn the word "accesskit"; the platform keeps the
  only adapter dependency (ADR-9).

### 3. What this does not do

- No announcements or live regions (the AccessKit `Live` property is a
  sink-vocabulary change and deserves its own milestone with its own
  delta semantics), no `SetValue`/text-selection actions (the UTF-16
  selection model is still an open seam), no new roles (a `Slider` is the
  natural `Increment`/`Decrement` consumer and arrives with the slider).
- No focus-loss visual edge on the keyboard path: a button held-pressed
  by Space that loses focus keeps its pressed fill until the next
  Space/Enter cycle — the same trade GTK makes; a `set_focused` repaint
  is the recorded follow-up seam.
- No per-action capability advertisement (AccessKit `supports_*`
  actions): refusal is honest and observable today; advertising is a
  widget-canon sweep once more actions exist.

## Consequences

- The workspace stays at one milestone per minor cut: 0.8.0 (the pending
  release) now carries M9 + M10.
- The corpus is 48 cases; the M5–M10 suites pin the number together.
- The baselines are re-published from the collecting host; the
  `arena_masks/10000` entry was refreshed in the same pass after the
  same-host checker flagged a ~30% drift that reproduces identically at
  the M9 tag's tree (measured by stashing the working tree) — host drift,
  not a code regression, and exactly what the same-host gates exist to
  surface and re-arm.
- The CI perf-contracts bench invocation pins `--benches`, so the
  criterion-only flags never reach a libtest harness (a bench-less lib
  target rejects `--warm-up-time` and fails the job before any bench
  runs).
- A screen-reader user can now scroll the list canon from the AT side; a
  keyboard user can drive every button; and no assistive technology can
  be silently lied to about what its actions did.
