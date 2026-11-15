# ADR-0082: M76 scope — the flip names its repaint count

- **Status:** Accepted (2026-11-15)
- **Implements:** ADR-0080's second pre-registered re-open rule — "a
  consumer prices *which nodes* the flip repainted → the record grows
  the broadcast's node count beside the fact (the lengths rule — the
  ids never ride), priced by a real reader": through M75 a flip's
  [`SettingRecord`](crates/alacer-core/src/inspect.rs) named the fact
  alone, and a consumer asking "how much did the OS flip repaint?"
  had to re-derive it — the record grows the broadcast's node count
  beside the fact, the log answering "what does the vocabulary consult
  now, when did the OS move it, and how much did the flip repaint?"
  from one row
- **Companion:** ADR-0080 (the setting seam the record rides — the
  pre-registered re-open rule this cut cashes), ADR-0081 (the
  composition door whose swaps stream beside the counted flips),
  ADR-0010 (the additive wire rule — the field grows at version 1),
  ADR-0011 (the coordinated rule the corpus grows by), architecture
  §7.2 pass 8, §18 (the M76 roadmap row)

## Decision

- **The record grows the count beside the fact — the lengths rule.**
  [`SettingRecord`](crates/alacer-core/src/inspect.rs) carries
  `repainted: usize` beside `dark`: how many nodes the flip's
  `Change::PAINT` merge touched, computed where the broadcast resolves
  ([`PassPipeline::set_settings`](crates/alacer-core/src/passes.rs)'s
  own pre-order walk — the walk's length *is* the flip's reach, never
  re-counted at the reader), never the ids (the payload rule's shape:
  a count is not an identity). The count is the fact of the flip's own
  moment: a tree that grows later never rewrites a queued record. The
  no-op flip queues no record — there is no count to name, the
  idle-frame property unchanged (A1's rule at the setting tier).
- **The wire grows additively at version 1.** The `setting` event
  carries `repainted` beside `dark` (ADR-0010's additive field rule —
  clients that ignore unknown fields are unaffected), the embedded
  client rendering the count in the row's detail line ("the flip
  repainted N nodes"), the protocol document pinning the field's
  stable meaning.
- **The composition rides the grown record unchanged.** ADR-0081's
  composed swaps stream their `Theme` facts beside the flip's counted
  `Setting` fact — the theme record stays the tag alone, the setting
  record naming the same reach the flip broadcast (one flip, one
  count, however many palettes the composition resolved).
- **The demo names the reach.**
  [`examples/dark_mode.rs`](crates/alacer/examples/dark_mode.rs)'s
  exit read-back prints the count beside the fact — the consumer-side
  shape the m76 suite pins.
- **The corpus grows 81 → 82 by the coordinated rule.** The counted
  flip read end to end joins the corpus
  (`settings/door_flip_counted` — the fact that lands with its
  broadcast count, the tick's start drain streaming it into the trail
  reader, the reader naming the count — the real reader the lengths
  rule priced) — bench, gate, and baseline in the same cut; the
  m-suite pins re-agree at 82.

## Re-open rules (pre-registered)

- a consumer prices the flip's *per-node* cost → the pass-overhead
  corpus prices the broadcast merge directly (the existing
  `arena_masks` cases), the record's count never becoming a
  per-node ledger — the lengths rule holds.
- a consumer needs the count on other tiers' records → each tier's
  record grows its own lengths beside its fact by its own coordinated
  cut (the view-patch record's four counts the M72 precedent), never
  a shared side-channel.
- a consumer prices the count's own maintenance → the count is the
  broadcast walk's length collected where the walk already runs: zero
  added work on the door path beyond the record field, priced by
  `settings/door_flip_counted` beside the door's own cases.
