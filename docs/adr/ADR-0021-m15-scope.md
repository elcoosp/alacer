# ADR-0021: M15 scope — the undo stack joins the editing contract

- **Status:** Accepted (2026-11-09)
- **Implements:** the §10.3 editing contract's history tier — the value
  that is scalar-addressable since M1, keyboard-extended and AT-managed
  since M11, pointer-placed since M12, word-addressable since M13, and
  transferable since M14 — becomes **reversible**: undo replays the
  inverse of the newest committed edit, redo replays the undone one, and
  every step carries the selection it consumed
- **Companion:** ADR-0020 (the clipboard tier this rides), ADR-0019 (the
  word tier), ADR-0017 (the selection model every step restores),
  architecture §10.3, §14.3 (the eviction-policy rule), §18 (the M15
  roadmap row)

## Decision

M14's "what this does not do" recorded the exhausted seam list's next
vocabulary in the code's own words: "an editing-undo stack is its own
milestone's vocabulary." M15 is that arrival, in the same "join the
contract" shape as M8–M14 — one contract widens, every consumer rides the
same currency:

- **The history vocabulary lives in core, one definition.**
  `EditHistory` and its `EditEntry` join the editing rules beside
  `paste_text` and `word_hop` (`alacer-core/history.rs`): pure text
  arithmetic over the contract's UTF-16 units ([`crate::utf16`]), no
  widget dialect, no platform indirection — the history *is* the field's
  state, not a platform fact, so unlike the clipboard seam there is no
  thread-local indirection to install. Each entry carries the replaced
  span, the removed and the inserted text, and both selections — the
  `(anchor, caret)` pair the edit consumed and the one it left — so an
  undo restores not just the text but the range the user was extending
  when they made the edit (the M11 vocabulary, ADR-0017).
- **The pure-append coalescing rule is the whole coalescing story.**
  Consecutive insertions that continue exactly at the previous
  insertion's end, with the selection chained between them, coalesce
  into one undo step — typing a word a character at a time undoes as one
  step, and an IME commit (one insertion through the same funnel) undoes
  as one step. Every other edit — a deletion, a word chord, a cut, a
  paste, a caret discontinuity — is its own step. The simplification is
  recorded, not hidden: GTK also breaks a typed run at word boundaries
  and on a timer; both refinements need vocabulary this milestone does
  not grow (§3).
- **The reset rule is the app's boundary.** A whole-value replacement —
  the view's reconciliation, the AT's `SetValue` — resets the journal:
  the app wrote the value, not the user, and undo must never fight the
  app's own write. This is the same reset a programmatic set has in
  every platform editor, and it keeps the AT value tier honest — a
  refused-then-replaced value cannot be resurrected behind the app's
  back.
- **The journal is bounded, with the eviction policy test-pinned.** 100
  steps by default (the `DEFAULT_CAPACITY`), the oldest evicted FIFO
  when the bound is crossed — a cache with an eviction policy and tests
  that prove it (§14.3, contributing rule 5), never an unbounded vector
  riding a long editing session.
- **The editor consumes it through one funnel.** Every committed value
  change — typing, backspace, forward delete, the word deletions, cut,
  paste, an IME commit — now rides a single `edit(from, to, insertion)`
  funnel that journals the inverse as it mutates, so no edit path grows
  its own caret arithmetic or its own dialect of the history. Ctrl/Cmd+Z
  undoes, Shift+Ctrl/Cmd+Z and the Ctrl/Cmd+Y alternate (the Win32/GTK
  binding) redo, with the M11 semantics riding along: a restored step
  brings back the selection it consumed, an exhausted journal refuses —
  the chord is consumed (the field's own chord, GTK consumes the
  accelerator) but queues no masks, an idle frame, the discipline the
  action and clipboard tiers already pin — and the chords are inert
  while a preedit composes, the M13 rule every command tier follows.
- **The a11y tree rides unchanged.** An undo is the same value+selection
  delta every writer shares — keyboard, pointer, word chords, the
  clipboard, and now the history — and the AT reads it through the
  M9/M11 machinery untouched.

### 1. Why this is the milestone

The recorded unblocked seams after M14 were announcements/live regions,
the pointer-capture seam, `supports_*` advertisement, and a `Slider` —
and the one the code itself calls its next move: ADR-0020 §3 deferred
undo/redo with its shape already named ("an editing-undo stack is its
own milestone's vocabulary"), and the editing series (M11→M14) built
exactly the machinery it needs — the anchor+caret selection every step
restores, the single edit funnel's predecessors, the refuse-don't-munge
discipline its refusals follow — while leaving every field unable to
take back an edit, the one text-field affordance every platform user
reaches for without thinking. It crosses the contract at every tier at
once: the rule layer (the journal is pure core arithmetic, no widget
dialect), the editing contract (an undo is an edit like any other,
riding the same `PAINT | A11Y` currency), and the state story (the
bounded journal is a cache with a policy, §14.3). The alternative
candidates are their own milestones' shape: announcements need the
live-region vocabulary AccessKit's `SelectionChanged` carries, the
pointer-capture seam needs a gesture state machine behind §10.1's
recorded follow-up, and a `Slider` needs the numeric value tier
ADR-0017 §3 already scoped — none is unblocked by a deferral the
previous ADR wrote down.

### 2. The seams that did not move

- **The corpus grows by the coordinated rule, nothing else moves.** One
  bench case (`editing/route_100_undo_redo_1k` — one hundred undo/redo
  chords routed through the pipeline on a 1,000-node arena, measured on
  the shipped `TextInput`, not a probe), one regression contract —
  52 → 53 cases; the M5–M14 pin suites move together.
- **Pass 1's shape is untouched.** The history chords are keyboard
  edits: focus-routed, no arena walk, no new event shape. The M8 queue
  semantics, the capture/bubble order, and the click chain are exactly
  as they were; the journal rides in the widget's own state, so the
  arena, the masks, and the pass contract are unchanged.
- **The DAG is untouched in shape.** `EditHistory` and its rules live in
  `alacer-core`; the editor's consumers in `alacer-widgets`; nothing
  mounts below the widgets, so no dependency edge moves — the platform
  layer has no engine to install (there is no OS fact in an undo stack)
  and the facade's assembly is unchanged.

### 3. What this does not do

- No timed or word-boundary coalescing: GTK breaks a typed run at a
  UAX #29 boundary and on an idle timer; the pure-append rule is the
  whole story here, and both refinements are recorded as the debt this
  milestone does not carry (the seam for the first — the segmentation —
  already exists on the text seam).
- No per-step redo of a *programmatic* write, no multi-field undo, no
  grouped transactions (a todo canon's "add item" is several edits in
  several fields — grouping them is a vocabulary of its own, post-0.1).
- No `Ctrl/Cmd+Y`-family alternates beyond the one binding, no
  `Alt+Backspace` undo alias (the word chords own that chord since
  ADR-0019), no secondary bindings: the command set is Ctrl/Cmd+Z and
  Shift+Ctrl/Cmd+Z, with Ctrl/Cmd+Y as the recorded alternate.
- No announcements (`SelectionChanged` remains the live-region
  milestone's vocabulary, ADR-0016 §3 unchanged); no inspector
  integration — the journal exposes `undo_depth` and the can-undo/can-
  redo reads, the Causality surface rides a later seam.
- No redo history surviving a reset: the reset rule clears both stacks,
  deliberately — a fork the app cannot see is a fork that lies.

## Consequences

- The workspace stays at one milestone per minor cut: the next release
  (0.12.0) carries M15.
- The corpus is 53 cases; the M5–M14 suites pin the number together and
  the M15 suite asserts all ten pins plus the published baseline/
  contract pair.
- The baselines are re-published from the collecting host (the ADR-0016
  procedure): the new case's median rides the same snapshot, and the
  same-host regression gates re-arm against it.
- `EditHistory` is new public API at 0.11.x: nothing existing implemented
  it, so nothing breaks — every edit funnel consumer journals in this
  milestone, and the `can_undo`/`can_redo`/`undo_depth` reads are the
  introspection the tests (and someday the inspector) drive.
- A keyboard user can finally take back an edit — and an AT user rides
  the same value tier — so the editing contract is complete in the way
  every platform text field is: the history stayed one piece of the one
  invalidation currency no matter which input modality wrote the edit it
  reverses.
