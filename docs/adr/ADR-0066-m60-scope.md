# ADR-0066: M60 scope — the clipboard chords' field changes join the key-edit vocabulary

- **Status:** Accepted (2026-11-14)
- **Implements:** ADR-0064's third kill rule — "a consumer prices the
  *clipboard chords' field changes* — the cut's span, the paste's
  shape — as typing-door facts beside the round trip → the record's
  action vocabulary grows the chords behind the clipboard trail's own
  re-open rule (ADR-0045's words), never a double-priced edit by
  default" — the reader prices them now: a trail reader could see the
  round trip's counts (the clipboard record's `chars`) and the
  journal's depths (the `EditRecord`s), but the field's own change —
  the span the paste consumed, the payload it carried, the span the
  cut moved out — had no typing-door fact: "what did the paste change
  in this field" was answerable only by re-pricing the round trip's
  counts against the journal's, on a different trail, a frame apart
- **Companion:** ADR-0064 (the key-edit record the chords join — the
  M58 vocabulary, the flood rule the chords never ride), ADR-0020
  (the clipboard contract the chords ride — the refuse-don't-munge
  rule the cut keeps), ADR-0045 (the clipboard trail the round trip
  keeps — the payload-never-rides rule the lengths follow), ADR-0010
  (the additive wire rule), architecture §13.3, §18 (the M60 roadmap
  row)

## Decision

- **The chords name their field changes.** The paste's insertion and
  the cut's deletion rode the field's one edit funnel
  (`crates/alacer-widgets/src/text_input.rs`, `paste`/`cut` calling
  `edit`/`insert`) and the journal recorded every one — but the
  typing door streamed none (the M58 door discipline kept the chords
  off it). The vocabulary (`crates/alacer-core/src/inspect.rs`):
  [`KeyEditAction`] grows `paste` and `cut` — the chords' own
  species, never munged onto the insert and delete they mirror (the
  M13 tier's own words; a cut that streamed a delete fact and a cut
  fact would double the edit, the round trip's own rule, ADR-0020) —
  and [`KeyEditDriver`] grows `clipboard` (the M30 attribution rule
  at the typing tier: the chord rode the clipboard's door, not the
  keyboard's). The note sites stand where the chord resolves: the
  paste notes after its insert lands, the cut after its deletion —
  the round trip's fact first, the causal order the chord ran
  (clipboard → edit).
- **One fact per trail, never a double-priced edit.** The round trip
  keeps the clipboard trail's fact (ADR-0045); the journal keeps the
  depths; the typing door names the field's change — three tiers,
  three facts, each trail pricing its own tier's story, no trail
  pricing another's. The refused paste (an empty store's refused
  read, a payload that sanitized to nothing) and the copy (a chord
  that edits nothing) answer no key-edit fact — the journal recorded
  nothing, the trail queues none (the M58 no-op rule, unchanged).
- **The chords never ride the flood rule.** The trail's supersede is
  the typed run's rule (the same control, the same driver, a pure
  insertion continuing exactly at the landing — `insert` actions
  only): a chord's record is never superseded into a typed run's
  record (the driver guard holds them apart even when the journal's
  tip folded the shape), two chords never supersede each other — the
  chord is a discrete act, one fact per chord. Zero trail-mechanics
  changes: the existing guard already holds (`key_edit.rs`), and the
  pins prove it (`crates/alacer-core/src/key_edit.rs` unit tests,
  `crates/alacer-testing/tests/m60_exit_criteria.rs`).
- **The wire speaks it at version 1.** No new kind: `paste`/`cut` are
  new *values* of the documented `action` field and `clipboard` a new
  value of the documented `driver` field (the additive rule ADR-0010
  names — the M59 precedent, `PROTOCOL_VERSION` stays 1, clients
  that render the strings render the echo unchanged). The embedded
  client's prose names the chord's door ("the clipboard's chord
  landed (0 replaced by 4)"), and `examples/paste_edits.rs` is the
  consumer demonstration (tracing at `alacer::paste_edits`): the
  paste named by its driver and shape, the paste over a selection
  pricing both spans, the cut's span, the refused paste riding the
  honest absence, the copy editing nothing, the app door's write
  beside them streaming none — the doors tellable apart on one log.
- **No new corpus case: the corpus stands at 65.** The note rides the
  dispatch route the chords already take, outside the frame's hot
  loops (one branch per chord while nobody listens — the A7 idle
  contract, the coordinated rule's no-growth arm, the M57/M59
  precedent), and the route itself is already benched
  (`editing/route_100_clipboard_ops_1k` drives select-all/copy/paste;
  the noted path rides inside the published case, its regression
  gate unchanged). The living documents re-agree at the cut (the
  README, the register, the protocol document's `key_edit`
  vocabulary, the catalog, the CHANGELOG, and the M58 suite's pin —
  the clipboard-chord absence the kill rule re-opened re-agrees to
  the priced chord).

## Kill criteria (re-open rules)

- A consumer prices the *paste's payload source* — which clipboard
  the chord read, the engine's species beside the field's change — on
  the log → the record grows the engine fact behind the clipboard
  trail's own re-open rule (ADR-0047's words), never a default-on
  wire field.
- A consumer prices the *cut's payload destination* — the span that
  moved out as typing-door fact *and* the round trip that received it
  as one correlated pair — → the wire grows the correlation id behind
  a real reader's ask, never a speculative join.
- A consumer prices the *chords' refusals* — the closed field's wall,
  the availability tier's — as typing-door facts → the record grows
  an outcome beside the landed shape (ADR-0064's second kill rule's
  own words), never a munged delivered edit.
