# ADR-0020: M14 scope — the clipboard joins the editing contract

- **Status:** Accepted (2026-11-09)
- **Implements:** the §10.3 editing contract's clipboard tier — the
  selection that is scalar-addressable since M1, keyboard-extended and
  AT-managed since M11, pointer-placed since M12, word-addressable since
  M13 — becomes **transferable**: copy writes it to the OS clipboard,
  cut moves it, paste inserts the clipboard's text at the caret
- **Companion:** ADR-0019 (the word tier this rides), ADR-0018 (the
  pointer tier), ADR-0017 (the selection model every writer shares),
  architecture §10.3, §12 (the adopted platform services), §18 (the M14
  roadmap row)

## Decision

M13's "why this is the milestone" recorded the exhausted seam list's
remaining vocabulary; the editing contract's own series — M11 selection,
M12 pointer, M13 word — leaves exactly one tier a text field's user
still cannot reach: the clipboard. ADR-0017 §3 deferred it with the
shape already fixed: "Copy/Cut/Paste are platform-seam actions — arboard
is not a workspace dependency, and a clipboard milestone owns the
permission/escape story with it." M14 is that arrival, in the same
"join the contract" shape as M8/M9/M10/M11/M12/M13 — one contract
widens, every consumer rides the same currency:

- **The clipboard seam carries the OS round trip.** `ClipboardSeam` joins
  the assembly seams beside the text seam (`alacer-core/clipboard.rs`):
  `read_text() -> Option<String>` and `write_text(&str) -> bool`,
  installed once per thread at assembly time, the same thread-local
  indirection, for the same reason — widgets live two layers below the
  platform and must never carry an OS dependency. There is **no provided
  implementation**, deliberately: the OS clipboard is a platform fact —
  the honest answer needs the platform's own ownership protocols, exactly
  what the adopted cores carry. A widget-internal buffer would silently
  break the round trip every other application shares, exactly the
  documented guess the honesty rules forbid. The workspace's engine
  implementation rides arboard (the audit's adopted clipboard core,
  architecture §12/§20) in `alacer-platform` — ADR-9 keeps the adapter
  dependencies in the platform crate — opened lazily, so a headless or
  sandboxed session answers reads with `None` and refuses writes instead
  of panicking.
- **The permission/escape story is the seam's contract, not a
  afterthought.** `read_text` returns `None` for an empty clipboard, a
  non-text payload, and a clipboard the process cannot open — a paste
  refuses on `None` and queues no masks. `write_text` reports whether
  the platform accepted the write; a refused write is never munged into
  success, so a **cut whose write was refused does not delete** — the
  selection survives a clipboard that failed it. Refused chords are
  consumed by the field (they are the field's chords — GTK consumes the
  accelerator) but queue no masks: the refusal costs an idle frame, the
  honesty the action tier already pins (ADR-0016).
- **Core fixes the paste rule.** The clipboard carries arbitrary
  platform text — multi-line, CRLF, orphan CR — and the field is
  single-line. `paste_text(raw) -> String` lives beside the seam: every
  line break (CR, LF, or a CRLF pair, counted once) pastes as exactly
  one space, the convention single-line entries across the platform
  stack share. Pure text arithmetic, one definition, no widget dialect —
  the `word_hop` discipline applied to paste.
- **The editor consumes it, the M11 semantics riding along.** Ctrl/Cmd+C
  writes the selection's exact UTF-16 span (resolved through the same
  byte range the highlight paints) and edits nothing; Ctrl/Cmd+X writes
  it and deletes the range (the M11 edit rule); Ctrl/Cmd+V inserts the
  sanitized clipboard text at the caret, consuming an active selection
  first — an edit like any other, riding the same `PAINT | A11Y`
  currency. Clipboard chords are keys like any other: inert while a
  preedit composes, and a degenerate selection copies nothing (a copy
  never invents content).
- **The a11y tree rides unchanged.** A paste is the same value+selection
  delta every writer shares — keyboard, pointer, word chords, the AT
  value tier, and now the clipboard — and the AT reads it through the
  M9/M11 machinery untouched.

### 1. Why this is the milestone

The recorded unblocked seams after M13 were clipboard, announcements/
live regions, the pointer-capture seam, `supports_*` advertisement, and
a `Slider`. The clipboard is the one the code itself calls its next
move: ADR-0017 §3 reserved it as "the clipboard milestone" when the
selection model landed, ADR-0019 §1 listed it first among the
alternatives that were "their own milestones' vocabulary", and the
editing series (M11→M12→M13) built exactly the machinery it needs — the
anchor+caret selection, the hit-tested caret, the word-aware edits —
while leaving every field unable to hand a selection to another
application, the one text-field affordance every platform user expects
without thinking. It crosses the contract at every tier at once: the
seam layer (a clipboard is a platform fact the widget must never own),
the editing contract (a paste is an insertion like typing; a cut is a
deletion like Backspace), and the release story (arboard joins the
adopted cores the architecture already ratified). The alternative
candidates are their own milestones' shape: announcements need the
live-region vocabulary AccessKit's `SelectionChanged` carries, the
pointer-capture seam needs a gesture state machine behind §10.1's
recorded follow-up, and a `Slider` needs the numeric value tier
ADR-0017 §3 already scoped — none is unblocked by a seam the code names
as its next move.

### 2. The seams that did not move

- **The corpus grows by the coordinated rule, nothing else moves.** One
  bench case (`editing/route_100_clipboard_ops_1k` — one hundred
  select-copy-paste round trips routed through the pipeline on a
  1,000-node arena, measured on the shipped `TextInput`, not a probe),
  one regression contract — 51 → 52 cases; the M5–M13 pin suites move
  together.
- **Pass 1's shape is untouched.** The clipboard chords are keyboard
  edits: focus-routed, no arena walk, no new event shape — the
  `Modifiers` the platform already tracks are the only input-side fact.
  The M8 queue semantics, the capture/bubble order, and the click chain
  are exactly as they were.
- **The DAG is untouched in shape.** `ClipboardSeam` and `paste_text`
  live in `alacer-core`; the editor's consumers in `alacer-widgets`; the
  engine implementation in `alacer-platform`, which grows the arboard
  dependency edge the architecture's adopted-core table already listed
  (§12/§20) — the platform keeps its role as the only adapter-dependent
  crate (ADR-9). The facade's `pipeline_default` installs the same
  engine so headless pipelines answer the chords with the same escape
  semantics.

### 3. What this does not do

- No paste history, no secondary selections (X11's PRIMARY), no rich
  text or image payloads: `read_text`/`write_text` is the text tier
  only — the data-transfer formats stay §10.3's post-0.1 note.
- No multi-line field: the single-line paste rule is the milestone's
  whole story here; a multi-line editor grows a different rule beside
  the same seam when that widget exists.
- No `Shift+Insert`/`Ctrl+Insert`/`Shift+Delete` secondary bindings, no
  macOS line-navigation special cases: the command set is Ctrl/Cmd+
  C/X/V (Meta is the command modifier); the GTK secondary set and the
  ADR-0019 §3 macOS debt are recorded here, not hidden.
- No announcements (`SelectionChanged` remains the live-region
  milestone's vocabulary, ADR-0016 §3 unchanged); no pointer capture
  (§10.1's recorded follow-up); no word-wise drag extension (ADR-0019
  §3's simplification stands).
- No undo/redo: the clipboard tier writes history into the OS, not into
  the field — an editing-undo stack is its own milestone's vocabulary.

## Consequences

- The workspace stays at one milestone per minor cut: the next release
  (0.11.0) carries M14.
- The corpus is 52 cases; the M5–M13 suites pin the number together and
  the M14 suite asserts all nine pins plus the published baseline/
  contract pair.
- The baselines are re-published from the collecting host (the ADR-0016
  procedure): the new case's median rides the same snapshot, and the
  same-host regression gates re-arm against it.
- `ClipboardSeam` is a new seam at 0.10.x: nothing existing implements
  it, so nothing breaks — every assembly site (window, default
  pipeline, harness) installs an implementation in this milestone.
- A keyboard user can finally move a selection out of the field and
  bring text in; an AT user rides the same value tier; and the editing
  contract is complete in the way every platform text field is — the
  clipboard stayed one piece of the one invalidation currency no matter
  which input modality wrote it.
