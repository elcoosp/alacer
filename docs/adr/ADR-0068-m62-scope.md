# ADR-0068: M62 scope — the typing door's refusals join the causal log

- **Status:** Accepted (2026-11-14)
- **Implements:** ADR-0064's second kill rule — "a consumer needs the
  *typing door's refusals* — the closed field's silence, the
  availability tier's wall — as data on the log → the record grows an
  outcome beside the landed shape (the value trail's own refusal
  vocabulary, ADR-0046), never a munged delivered edit" — the reader
  prices it now: the M58 record named every edit that *landed*, but the
  wall was pure silence (the M58 pin's own words: "no edit lands, no
  fact queues"), and a typed run beside a wall was indistinguishable
  from an honest gap — "what happened to my edit" was answered by
  absence, priced only by what the log did *not* say
- **Companion:** ADR-0064 (the key-edit record the outcome rides
  beside — the typing door's own record since M58), ADR-0066 (the
  chords' refusals riding behind their own actions — the third kill
  rule's own words: "ADR-0064's second kill rule's own words"), the
  value tier's refusal vocabulary (ADR-0046, ADR-0027's
  `unavailable` tag — the species named at the AT tier first),
  ADR-0028 (the availability tier whose wall this prices — the M22
  sentinel discipline that bounds the window), ADR-0049 (the funnel
  stamp the wall's facts ride — the M43 rule at the typing tier),
  ADR-0010 (the additive wire rule), architecture §13.3, §18 (the M62
  roadmap row)

## Decision

- **The record names what the door answered.** The key-edit record
  ([`KeyEditRecord`](crates/alacer-core/src/inspect.rs)) grows one
  additive `outcome` field: [`KeyEditOutcome::Landed`] — every M58
  record, the delivered shape the record already prices — or
  [`KeyEditOutcome::Refused`] with [`KeyEditRefusal::Unavailable`] as
  its species: the availability tier's wall (M22, ADR-0028) named in
  the value tier's own word (the `unavailable` tag the AT tier's
  closed door streams, ADR-0027 — one refusal vocabulary across the
  tiers, never a second name for the same wall). A refusal prices the
  *ask*: the action and driver the open door would have answered with
  (`insert`/`backspace`/`delete`/the word chords under `keyboard`,
  `paste`/`cut` under `clipboard`, the commit under `ime` — the M60
  attribution rule holding at the wall), the shape priced at nothing
  (`replaced: 0`, `inserted: 0`), the selection the untouched one.
  Never a munged delivered edit: a refusal behind a landed shape would
  double the ask the wall never answered, and a no-op edit on an open
  door stays silent (the M20 rule's own word — an answered request is
  not a refusal).
- **The ask is named by the door's own match.** The wall prices only
  what an open door would have answered with a fact: the widget
  classifies the routed key with the classify half of `edit_key`'s
  match ([`refused_action`](crates/alacer-widgets/src/text_input.rs) —
  the arms that edit the value), so a character, a unit or word
  delete, and the chords name themselves while the keys an open door
  answers without editing (the caret's walk, the submit's Enter, the
  journal's and clipboard's copy chords, an unknown chord) ride no
  fact — the wall prices edit asks, never every routed keystroke. A
  release names no ask (the open door acts on presses only, §10.3).
- **The window is the availability tier's own.** The M22 sentinel
  discipline evicts focus the frame a control becomes unavailable —
  the eviction is the frame's pass, not the door — so the wall's facts
  ride the frame the control is held down in: a keystroke routed in
  that frame still reaches the held field's door, and the wall answers
  it. The asks the boundary carries to *no* field after the eviction
  are the route's own silence (the key route's fallbacks; the IME
  route's `no_focused_field` drop, M59/ADR-0065) — never a munged
  typing-door fact: the door heard no ask, the log prices no ask.
  The m58 pin re-agrees with the boundary named ("the closed door's
  silence is the availability tier's own answer" — the *evicted
  frame's* silence, the wall's own window priced by the m62 suite).
- **The supersede stays the delivered run's fold.** The flood rule's
  supersede grows the outcome guard: a coalesced run folds only into a
  `Landed` tip — a refusal at the trail's tip supersedes nothing even
  when the control, driver, and caret arithmetic would fold — the
  wall's fact and the run's record tellable apart on one log, never a
  refusal merged into the delivery it follows. The refusal itself
  never rides the fold (`coalesced: false` always): one fact per ask.
- **The wire and the client speak it at version 1.** The `key_edit`
  event grows two additive fields (the protocol's own versioning rule,
  ADR-0010 — `PROTOCOL_VERSION` stays 1, clients that ignore unknown
  fields unaffected): `outcome` (`landed`/`refused` — the snake_case
  tags the value tier's vocabulary pins) beside `refusal`
  (`unavailable` on a refusal, `null` on a landing). The embedded
  client renders the echo — "the keyboard refused — the wall held
  (unavailable)" — and `examples/edit_refusals.rs` is the consumer
  demonstration (tracing at `alacer::edit_refusals`): the open door's
  run, the wall naming its ask, the chords refusing behind their own
  actions, the navigation keys riding none, the IME's commit refused
  with its own driver, the reopened door landing again.
- **No new corpus case: the corpus stands at 65.** The wall rides the
  dispatch route the keys already take, outside the frame's hot loops
  (the routed ask is user-rate, the note one branch per event while
  nobody listens — the A7 idle contract, the coordinated rule's
  no-growth arm, the M57/M59/M60/M61 precedent). No new bench case:
  the record rides the route the editing bench already prices
  (`editing/route_100_key_edits_1k` — the delivered route now carries
  the outcome beside every fact, the supersede guard one comparison
  on the run's fold; the wall itself is off the delivered hot path, a
  held field's route). The living documents re-agree at the cut (the
  README, the register, the protocol document's `key_edit`
  vocabulary, the catalog, the CHANGELOG).

## Kill criteria (re-open rules)

- A consumer prices the *composition's hold* — the keys the IME's
  canvas made inert (the preedit session's own wall, the field a live
  session closes to keys) — on the log → the vocabulary grows the
  hold species behind a real reader's ask, never a default-on echo of
  every inert keystroke.
- A consumer prices the *payloadless chord* — the paste whose read
  refused before the door (the empty clipboard's honest absence, the
  m60 pin) — as a typing-door fact beside the round trip → the wall
  grows the read's species beside the door's (the M42 split at the
  typing tier), never a double-priced round trip.
- A consumer prices the *answered-but-moved-nothing ask* — the
  backspace at the value's start, the delete at its end (the open
  door's no-op, today silent) — on the log → the outcome grows the
  answered species (the M20 rule's own word), never a munged refusal.
