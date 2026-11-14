# ADR-0067: M61 scope — the field's composing state joins the causal log

- **Status:** Accepted (2026-11-14)
- **Implements:** ADR-0064's first kill rule — "a consumer prices the
  *field's composing state* — the preedit's own trail, the
  composition's inline shape — on the log → the IME tier grows its
  own record beside the commit's (the preedit is not an edit; it
  lands nowhere), priced by a real reader, never a default-on echo of
  every composition frame" — the reader prices it now: a committed
  composition streamed its `KeyEditRecord` (the M58 record, the IME
  driver named), but the composing state itself — what the field
  showed while the IME owned the canvas, how large the composition
  grew, where the platform's cursor sat inside it — was traced and
  gone: "what is the field showing now" was unanswerable from the log
  alone
- **Companion:** ADR-0064 (the key-edit record the composing tier
  rides beside — the commit's own landing, the funnel stamp the IME
  route holds since M58), ADR-0065 (the IME route's drops the
  composing tier complements — the boundary's silence, not the
  field's), ADR-0011 (the secret-safety rule the shape-only record
  follows at the composition tier), ADR-0061 (the text-write trail's
  shape the preedit trail rides — the tick's start drain, the
  thread-local bounded queue), ADR-0010 (the additive wire rule),
  architecture §13.3, §18 (the M61 roadmap row)

## Decision

- **The composing tier names its own state.** The field's composing
  state rode the widget's IME door
  (`crates/alacer-widgets/src/text_input.rs`, the `Preedit` arm) —
  spliced inline at the caret, never committed, never journaled — and
  streamed nothing. The vocabulary: one
  [`PreeditRecord`](crates/alacer-core/src/inspect.rs) per composing
  fact — the phase (`composing`: the live session's inline shape;
  `cleared`: the composing state ended with no landing, the honest
  `0`), the composition's shape named by its UTF-16 length — never
  the payload (a field's value can carry a secret, ADR-0011's rule at
  the composition tier; the secret-entry field suppresses the preedit
  echo at the platform, the trail keeps the same discipline), the
  platform's preedit-local cursor beside it (a position inside the
  preedit, never a field offset — the composition lands nowhere), and
  the funnel's stamp (the M43 rule at the composing tier — the IME
  route has held the guard since M58, the identity never guessed at
  the note site).
- **The flood rule is the session's own supersede.** A live
  composition is one record: each composing arrival supersedes the
  session's record in place (the same control, the composing phase),
  the shape growing to the composition's — N composition frames while
  nobody drained land as one record whose shape is the session's,
  never a default-on echo of every composition frame (the kill rule's
  own words). A cleared arrival pushes its own fact; a composing fact
  beside a cleared one is a new session's own record — the supersede
  never crosses the phase boundary or the stamp's identity. One live
  session, one record; one fact per drain window.
- **The preedit is not an edit, and the commit is not a preedit.** The
  composing tier's record never munges onto the edit tier's actions
  (a composition is not an `Insert`) and the commit's landing keeps
  its own key-edit fact (M58's record, the IME driver named) — one
  fact per trail, the trails tellable apart on one log: the
  composition's shape rode the preedit trail, the landing rode the
  key-edit trail, a cleared state with no commit beside it reads as
  the cancelled session. A clear with no live session behind it (the
  spurious empty preedit) answers no fact — the field had no
  composing state to end.
- **The trail is the key-edit trail's shape.** A new trail module
  (`crates/alacer-core/src/preedit.rs` — the tier that resolves the
  composing state owns the stream; a second stream for one route
  would be two logs for one door): thread-local, bounded
  (`PREEDIT_TRAIL_CAP` = 256, oldest-first eviction), enabled by
  `PassPipeline::set_observer` beside the other trails, silent while
  nobody listens (the A7 idle contract — one branch per preedit
  event). The drain runs after the key-edit drain (a composition's
  shape explains the field the IME is composing into; it streams
  beside the commit's own key-edit fact), into
  `InspectEvent::Preedit`.
- **The wire and the client speak it at version 1.** The wire's
  `preedit` event carries the record flat (the additive rule ADR-0010
  names — a new event kind, additive by the protocol's own
  versioning rule, `PROTOCOL_VERSION` stays 1, clients that ignore
  unknown kinds unaffected): the phase beside the shape's length, the
  preedit-local cursor, the stamp — the payload itself never rides
  the wire. The embedded client renders the echo — "the composition
  shows (shape 3) — the cursor at 2" — and `examples/preedit_trail.rs`
  is the consumer demonstration (tracing at `alacer::preedit_trail`):
  the session named as one growing record, the cleared state naming
  its end, the commit landing beside the session, the spurious clear
  riding the honest absence.
- **No new corpus case: the corpus stands at 65.** The note rides the
  dispatch route the IME events already take, outside the frame's hot
  loops (the composing frames are user-rate, the note one branch per
  event while nobody listens — the A7 idle contract, the coordinated
  rule's no-growth arm, the M57/M59/M60 precedent). The living
  documents re-agree at the cut (the README, the register, the
  protocol document's `preedit` vocabulary, the catalog, the
  CHANGELOG).

## Kill criteria (re-open rules)

- A consumer prices the *composition's payload* — the composing text
  itself, not its shape — on the log → the record grows it behind an
  explicit opt-in seam (the clipboard trail's own re-open rule,
  ADR-0045's words), never a default-on wire field.
- A consumer prices the *session's boundaries* — the opened/closed
  edges of a composition apart from its growing shape — → the record
  grows the edges behind a real reader's ask, never a speculative
  state machine.
- A consumer prices the *secret-entry echo's absence* — the suppressed
  preedit the platform hides — as its own fact → the composing tier
  grows the named absence beside the value tier's, never a guessed
  echo.
