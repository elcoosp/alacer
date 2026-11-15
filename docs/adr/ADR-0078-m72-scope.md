# ADR-0078: M72 scope — the view's patch joins the causal log

- **Status:** Accepted (2026-11-15)
- **Implements:** the view tier's own kill rule — a consumer cannot
  answer "what did my re-render change — and how much?" from the log
  the causal trail streams: the view reconciliation, the tier every
  re-render rides, answered a `PatchReport` the imperative caller read
  and dropped (the mount's build streamed nothing at all), no record
  on the log → the view grows its own named fact beside the tiers
  that already price themselves, never a munge onto the door that
  answers
- **Companion:** ADR-0063 (the route-drop trail's shape the record
  copies), ADR-0077 (the tick's start drain the stream rides, the
  frame tier's own trail this one joins), ADR-0011 (the coordinated
  rule the corpus grows by), ADR-0010 (the additive wire rule),
  architecture §6.1, §6.2, §13.3, §18 (the M72 roadmap row)

## Decision

- **The view tier grows its own record.**
  [`ViewPatchRecord`](crates/alacer-core/src/inspect.rs) carries the
  reconciler's four counts beside each other — `created` (subtree
  mounts), `removed`, `patched` (a visual prop applied), `moved` (LIS
  non-members) — the [`PatchReport`](crates/alacer-view/src/mount.rs)'s
  own accounting as data. The record is named by its lengths (the
  payload rule's shape: a fact, not a message): the props a patch
  applied never ride, the text they carry can.
- **The note sites are the exact places the patch resolves.** The
  reconciler's mount notes the build (the tree named by its element
  count — the lengths rule, `created` the opening fact) and the update
  notes the diff (the four counts verbatim, gated on the diff moving
  something — the idle-frame property). The record rides *beside* the
  door's own `PatchReport` — `update` still answers the report it
  always answered — never instead of it.
- **One fact per patch that moved something, never per frame.** The
  frames that re-render nothing stream no patch fact, and the no-op
  re-render — a diff that moved nothing — streams none either (the
  idle-frame property, the m32 canon's own pre-registered rule: a
  write that lands where the value already is queues no masks, and no
  facts; the flood rule at the view tier, shipped at this cut).
- **The drain rides the tick's start.** The view-patch trail is the
  route-drop trail's shape (ADR-0063): a fact queues where the patch
  resolves, the dispatching pipeline drains it one frame later into
  [`InspectEvent::ViewPatch`](crates/alacer-core/src/inspect.rs). The
  queue is bounded (256 facts per thread, oldest-first eviction — the
  newest patches are the ones a consumer can still act on), the
  disabled path costs one branch per note (the A7 idle contract), the
  drain empties the queue whether an observer listens or not.
- **The wire grows additively at version 1.** The `view_patch` event
  is a new kind (`created` / `removed` / `patched` / `moved`) — the
  additive rule ADR-0010 names; clients that ignore unknown kinds are
  unaffected; the embedded client renders the counts' row beside the
  other trails.
- **The demo drives the seam from the consumer side.**
  `examples/patch_trail.rs` re-renders a keyed list through the
  consumer's own door — the add's `created`, the no-op's honest
  silence, the remove's `removed` — reads the `PatchReport` the door
  still answers, and streams the record on the same log — one fact
  per patch that moved something, the *trail* label mirroring what
  the log streamed.
- **The corpus grows 71 → 73 by the coordinated rule.** The note
  site's own cost joins the corpus (`view_patch/note_disabled` — the
  A7 contract's one disabled branch per note;
  `view_patch/note_enabled_bounded` — the enabled steady state's
  bounded push, the eviction priced), and the M71 cut's promised
  `frame_refusal` baselines publish in the same run (bench, gate, and
  baseline in the same cut — the m5 bijection re-agrees at 73, the
  corpus pin the stale snapshot left behind).

## Re-open rules (pre-registered)

- a consumer needs the patch's driver named → the record grows the
  rider beside the counts (`ViewPatchRider`: the hot-reload swap's
  regenerate beside the app's own re-render — ADR-7's patch, the M30
  attribution rule's shape), priced by a real reader — never a
  munged default-on attribution.
- a consumer prices the patch's cost in time → the record grows the
  duration behind a real reader's ask, never a per-prop breakdown
  (the payload rule's shape: the counts are the fact).
- the changed-something gate shipped at this cut (the m32 canon's
  idle-frame property priced it before any consumer had to); a
  consumer needing the no-op re-render as a fact → the site grows the
  zeros behind that reader's ask, never a default-on echo of every
  inert re-render.
