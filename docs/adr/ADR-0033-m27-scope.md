# ADR-0033: M27 scope — the walk's answer becomes data: the focus outcome vocabulary, the focus routes' baseline debt published, the repair debt the M23–M26 cuts left

- **Status:** Accepted (2026-11-10)
- **Implements:** the re-open rule ADR-0031 pre-registered (and re-stated
  by ADR-0032) — "a door needs to observe the walk's refusals as data →
  grow the answer into a result vocabulary priced by that consumer" —
  plus the repair debt the sweep priced: the M23-era `SceneJournal::stats`
  requirement three impls never received (the M23 cut's own benches and
  tests could not compile at the shipped HEAD), the focus-route baselines
  the M26 cut promised at its "measurement run" and never published (the
  m5 bijection red at HEAD), the examples that did not compile at HEAD,
  and the dead public API the M25 species prices
- **Companion:** ADR-0030 (the walk the doors share), ADR-0031 (the
  scope; the re-open rule this ADR cashes), ADR-0032 (the directions;
  the refusal help-text discipline the outcome names), ADR-0028 (the
  availability tier the empty-list species answers), ADR-0029 (the
  plain-help-text rule the species carry), ADR-0011 (the coordinated
  corpus rule the routes land under), architecture §10.3 (the keyboard
  contract the walk belongs to), §18 (the M27 roadmap row)

## Decision

The two focus doors have answered `bool` since M24. The shape was honest
for the routing call site (the keyboard's Tab fallback takes the key when
the walk moved) and silent for everyone else: a caller staring at `false`
could not tell the empty arena from the held-down trap from the edge's
"nothing lies here" from the single-candidate stall — five refusals
collapsed into one answer, each carrying its help as tracing text the
caller cannot route on. ADR-0031 named the rule when the scope shipped
("a door needs to observe the walk's refusals as data") and ADR-0032
re-stated it when the directions shipped. M27 cashes the rule:

- **The doors' answer grows into `FocusMoveOutcome`.** `focus_move` and
  `focus_move_in` return `FocusMoveOutcome` instead of `bool`. `Moved`
  carries a `FocusLanding` — `from` (the node that held focus, `None`
  for the entry), `to` (the node the walk rests on, the same one
  `focus()` answers), `wrapped` (the tree-order move crossed a list end;
  the entry landing never carries it and the directional landings never
  do — space has no wrap, ADR-0032). `Refused` carries a `FocusRefusal`
  naming its species: `NoCandidates` (an empty arena, every control
  unavailable, a trap the tier holds down), `DeadScope { scope }` (the
  M25 refusal, now named), `NoGeometry { node }` (a directional move
  before the first layout wrote the bounds), `NothingInDirection {
  node, direction }` (the M26 edge, naming its node and its direction),
  `AlreadyFocused { node }` (the single-candidate stall — the next stop
  is itself, named rather than munged). The walk behind the doors is
  untouched: the same pre-order candidate list, the same cycle-time
  `enabled` filter (M22, ADR-0028), the same wrap, the same immediate
  resolution, the same `FOCUS | PAINT` currency on the two endpoints,
  the same no-mask idle refusals — the M11 rule stays; the answer stops
  making the caller guess.
- **The verdict accessor keeps the routing call sites one line.**
  `FocusMoveOutcome::moved()` is the boolean's successor for the call
  sites that only route on the verdict — the keyboard's Tab fallback
  (`dispatch_key`) reads `.moved()` and the funnel rides unchanged. The
  data accessors (`landing()`, `refusal()`) carry the full answer for
  the consumers the rule priced: a status line that names the wall
  instead of mirroring a silent frame, a test that pins the walk by its
  answer instead of its shadow on `focus()`, and the inspector trail a
  future cut grows beside `InspectEvent` (the re-open rule below).
- **The per-species help stays plain text on the `alacer::focus`
  target.** The outcome names the species; the tracing keeps the
  `help:` sentences (ADR-0029's rule — the miette-class sentence, never
  the miette crate). The refusal help the M25/M26 cuts shipped is
  unchanged; the door-level refusal line stays generic exactly as the
  M24 cut wrote it, the walk's species traces staying the specific ones.
- **The demonstration reads the answer as data.**
  `examples/focus_outcome.rs` walks the M26 window — the two-by-two grid
  and the modal's form — one beat per `FocusMove`, and every beat logs
  what the walk answered: `Moved` landings name their endpoints and the
  wrap, the refused beats name their species (the corner's empty
  direction, the held-down trap's empty list, the dead scope, the
  single-candidate stall), the read doors keeping their whole-tree
  answers. Tracing at `alacer::focus_outcome`, the workspace's per-seam
  idiom. The m27 exit suite pins the outcomes headlessly on the real
  grid and the real trap through the real layout host: the landing's
  endpoints and wrap, the edge refusal naming node and direction, the
  held-down trap's empty list, the dead scope naming itself, the stall
  naming its node, and the tree-vocabulary read-back rule the M24 suite
  pinned for the boolean.
- **The repair debt the sweep priced rides the cut.** The M23 cut made
  `SceneJournal::stats()` a required trait method and updated its impls
  — except three: the `pass_overhead` bench's `NullJournal` and the two
  test harnesses' `NullJournal`s (`alacer-view/src/mount.rs`,
  `alacer-layout/src/host.rs`). At the shipped HEAD the workspace's own
  benches and tests could not compile (`cargo check -p alacer-layout
  --all-targets` fails; `cargo test --workspace` fails) — the M23 exit
  suite itself could not have run. The three impls land the honest zero
  ledger (`JournalStats::default()` — "a journal that does not account
  answers the zero value", the trait's own words). The M26 cut's
  changelog claimed the three focus routes "join the published corpus
  beside the directional route at this cut's measurement run" — the
  measurement run never happened: the pins moved 57 → 60 while
  `baselines.json` stalled at 57 entries and the m5 bijection
  (`every_published_baseline_has_a_regression_contract_and_vice_versa`)
  went red at HEAD. This cut publishes the baselines — `cargo bench`,
  `scripts/collect_baseline.py` — and grows the corpus 60 → 61 with
  this ADR's route (`input/route_100_outcome_moves_1k`), the gates and
  the pins moving in the same cut. The `focus_trap` and `focus_traversal`
  examples did not compile at HEAD (two multi-line-closure buttons each
  missing `.boxed()`, an unused `Widget` import each) — the catalog's
  "every example builds in CI" claim false at HEAD; the buttons box and
  the imports go. The dead `Slider::set_on_change` goes — zero call
  sites anywhere (the view registry patches handlers under a different
  name), the M25 species. The dead `AccessKitSink::is_initialized` goes
  — its only caller was its own unit test, whose next assertion
  (`update.tree.is_some()`) already proves the fact; the `initialized`
  field stays (it gates the full-tree response at `apply`).
- **The living documents re-agree at M27, pending the cut.** The
  README's badge and status block ride the pending-cut convention (the
  successor minor under `[Unreleased]`), the status count (M0 through
  M27), the ADR count (thirty-four), the features bullet, the
  quickstart, the perf bullet (61 cases), and the example table; the
  catalog's row; the register (33 → 34 with this ADR); the §10.3 focus
  paragraph; the roadmap row; the m-suite pins, which move across the
  m5–m26 chain in the same cut — the corpus pins to 61, the document
  pins to M27 and thirty-four, and the m24/m25/m26 signature pins to
  the outcome form; and the CHANGELOG stages the milestone.

## Consequences

- The doors' return type moves — the pre-1.0 API motion the status
  block declares. The call sites that routed on the boolean read
  `.moved()`; the call sites that mirrored the landing through
  `focus()` can read the landing directly. The `bool` past survives in
  the accessor, not in the signature.
- The hot path carries no new work: the outcome is two words of return
  payload (a `Copy` enum over `NodeId`s and a flag), the walk's branches
  unchanged, the route bench (`input/route_100_outcome_moves_1k`)
  pricing the data path against the boolean's on the same 1k arena.
- The perf corpus grows 60 → 61 at this cut's measurement run by the
  coordinated rule — measured, published, and gated in the same cut
  (CONTRIBUTING's publishing rule). The 57-entry baseline stall the M26
  cut left is repaired in the same run.
- The m1 corpus does not grow: the demonstration shows no new visual
  arrangement (the outcome demo walks the M26 window's own paint), and
  the m1 suite's 37-scenario pin stands.
- No new dependency edges: the vocabulary is core's own, the spans ride
  the `tracing` core every member already resolves, and the help-text
  discipline stays plain diagnostic text (ADR-0029's rule).

## Kill criteria (re-open rules)

- A door needs a shape the outcome cannot express (a walk that reports
  candidate distances, a multi-step replay, a history of the walk's
  stops) → grow the vocabulary beside `FocusLanding`, priced by a real
  consumer — never a second walk or a client-side reimplementation of
  the candidate list.
- A consumer needs to observe the walk's refusals as *streamed events*
  (the inspector's causal log: "why did focus change / why didn't it")
  → grow the observation seam beside `InspectEvent`, priced by that
  consumer; until one exists, the outcome-as-return is the honest shape
  (ADR-0031's rule stands).
- A platform key needs to ride a direction (a D-pad button, a TV
  remote's ring) → map the key onto `FocusMove` at the platform's
  key-fallback seam, priced by a real backend — never a second dispatch
  channel (ADR-0032's rule stands).
