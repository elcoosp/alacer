# ADR-0064: M58 scope — the typing door's edits join the causal log

- **Status:** Accepted (2026-11-14)
- **Implements:** ADR-0062's first kill rule — "a consumer prices the
  keyboard's and IME's edits → the edit stream grows its own
  per-change record beside the journal's, never a per-keystroke
  flood" — the reader prices them now: a trail reader cannot answer
  "what did the typist change in this field" from the log alone. The
  journal's `EditRecord` streams the depths an edit left; the AT's
  and app's doors stream their own `TextWriteRecord`s; the typing
  door's own changes — the character that landed, the unit the
  backspace consumed, the word the chord took, the composition the
  IME committed — were traced and gone
- **Companion:** ADR-0021 (the journal's coalescing rule the flood
  rule rides), ADR-0019 (the word chords' own vocabulary the species
  carry), ADR-0011 (the secret-safety rule the lengths follow), ADR-
  0049 (the funnel stamp the records carry), ADR-0062 (the ADR whose
  kill rule this cashes), ADR-0010 (the additive wire rule),
  architecture §13.3, §18 (the M58 roadmap row)

## Decision

- **The typing door names its own edits.** The field's committed
  value changes rode one funnel (`crates/alacer-widgets/src/
  text_input.rs`, `edit`) and the journal recorded every one — but
  the trail-side facts split by door: the AT's asks and the app's
  `set_value` stream `TextWriteRecord`s, and the typing door (the
  keyboard's keys, the IME's commits) streamed nothing but the
  journal's depths. The vocabulary (`crates/alacer-core/src/
  inspect.rs`, the trail in `crates/alacer-core/src/key_edit.rs`):
  one `KeyEditRecord` per edit — the action the field's own editing
  vocabulary names (`insert`, `backspace`, `delete`,
  `word_delete_back`, `word_delete_forward` — the word chords are
  never munged onto the unit deletes they mirror, the M13 tier's own
  words), the driver who wrote it (`keyboard`, `ime` — the M30
  attribution rule at the typing tier), the change's shape named by
  lengths (`replaced`/`inserted`, UTF-16 units — never the payload:
  a field's value can carry a secret, the clipboard trail's rule,
  ADR-0045), the landing selection beside it, and the journal's own
  fold. A typed character is not a `SetValue`: the record is the
  typing door's, never a munge onto the AT's action tier.
- **The flood rule is the journal's own fold.** `EditHistory::record`
  answers whether the edit folded into its tip (the pure-append
  rule, ADR-0021 — the same answer its `EditRecord` already carried),
  and the trail supersedes a coalesced run's record in place: the
  same control, the same driver, a pure insertion continuing exactly
  at the landing the record carries. N keystrokes in one pure-append
  run land as one record whose shape is the run's; an edit the
  journal kept as its own step (the run's first keystroke, a
  deletion, an IME commit beside a typed run) pushes its own record —
  the trail never merges what the journal kept apart, and the
  drivers' records never merge even when the journal's tip folded
  the shape.
- **The doors keep their trails.** The clipboard chords' edits ride
  the clipboard trail and the journal's (the round trip is the
  clipboard record's fact, the depths the journal's); the AT's asks
  and the app's `set_value` ride the text-write trail (one ask, one
  fact — ADR-0062's raw-write split unchanged); `undo`/`redo` ride
  the journal's. The typing door's notes name the typing door only —
  one fact per trail, never a double-priced edit. A no-op door
  (backspace at an empty field's start, a chord with no word to take)
  answers no fact — the journal recorded nothing, the trail queues
  none.
- **The facts queue where the edit resolves and stream at the tick's
  start.** The trail is the text-write trail's shape (ADR-0061 — the
  value trail's rhythm): thread-local, bounded (`KEY_EDIT_TRAIL_CAP`
  = 256, oldest-first eviction), enabled by `PassPipeline::
  set_observer` beside the other trails, silent while nobody listens
  (the A7 idle contract — one branch per edit; the notes ride the
  routes' existing guard reads). The drain runs after the route-drop
  drain (a key edit explains the field the typist changed; it
  streams beside the doors' facts and the boundary's), into
  `InspectEvent::KeyEdit`.
- **The IME route holds the funnel's stamp.** The key route has held
  the `ActionTargetGuard` since M45; the IME route held none, so a
  committed composition's fact would ride no control. Since M58 the
  IME route holds the same guard for the length of one event
  (`crates/alacer-core/src/passes.rs`, `dispatch_ime` — the key
  route's own discipline): the stamp fills only the identity the
  widget cannot know, the app's direct mutate seam rides its honest
  absence, and a dropped guard leaves no stale identity.
- **The wire and the client speak it at version 1.** The wire's
  `key_edit` event carries the record flat (the additive rule
  ADR-0010 names): the action beside the driver, the shape's lengths,
  the landing selection, the fold, the stamp — a new event kind,
  additive by the protocol's own versioning rule, `PROTOCOL_VERSION`
  stays 1, clients that ignore unknown kinds unaffected. The embedded
  client renders the echo — "the keyboard insert (0 replaced by 3) —
  the selection at 3..3 — the journal folded the run" — and
  `examples/key_edits.rs` is the consumer demonstration (tracing at
  `alacer::key_edits`): the typed run named as one fact, the
  backspace's span, the word chord's species, the IME's commit named
  by its driver, and the app door's write beside them streaming
  nothing (the honest absence), the doors tellable apart on one log.
- **One new corpus case: the corpus stands at 65.** The noted route
  joins the editing bench (`route_100_key_edits_1k` — the observed
  typing door, the trail's push and supersede paths under load, the
  regression gate in `benchmarks/contracts.json` and the baseline in
  `benchmarks/baselines/baselines.json`, the m-suite pins re-agreed
  at 65). The living documents re-agree at the cut (the README, the
  register, the protocol document's `key_edit` vocabulary, the
  catalog, the CHANGELOG).

## Kill criteria (re-open rules)

- A consumer prices the *field's composing state* — the preedit's
  own trail, the composition's inline shape — on the log → the IME
  tier grows its own record beside the commit's (the preedit is not
  an edit; it lands nowhere), priced by a real reader, never a
  default-on echo of every composition frame.
- A consumer needs the *typing door's refusals* — the closed field's
  silence, the availability tier's wall — as data on the log → the
  record grows an outcome beside the landed shape (the value trail's
  own refusal vocabulary, ADR-0046), never a munged delivered edit.
- A consumer prices the *clipboard chords' field changes* — the cut's
  span, the paste's shape — as typing-door facts beside the round
  trip → the record's action vocabulary grows the chords behind the
  clipboard trail's own re-open rule (ADR-0045's words), never a
  double-priced edit by default.
