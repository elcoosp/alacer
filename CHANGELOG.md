# Changelog

All notable changes to **Alacer** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
generated from Conventional Commits by [git-cliff](https://git-cliff.org).

## [Unreleased]

### Added

- **M42 — the clipboard read names its facts (ADR-0048)**: ADR-0047's
  third re-open rule cashed — a consumer needs the read side's facts
  the paste's `None` collapsed (the empty clipboard, the non-text
  payload, the unopenable clipboard) so the seam's read door grows
  beside the write door: one *provided* `ClipboardSeam::read_reported`
  answering `ClipboardReadFact` (the M37 provided-method discipline —
  the provided default maps the `Option` door and names its absence
  honestly, a real engine overrides and names its species; the
  `read_text` door stays the contract of record byte-for-byte, and a
  refusal through either inserts nothing). The species are the
  engine's own answers: `unnamed` (the honest absence), `unsupported`
  (no clipboard subsystem at all, or one that cannot open), `empty`
  (nothing the OS counts as content), `non_text` (content, but not
  text — an image), `os_error` (the unclassifiable) — the arboard
  engine splitting the empty and the non-text payload by the OS's own
  second answer (the image question asked only when the text read
  refused; each species the OS's answer to a question the engine
  asked). The round trip's record grows the additive `read` field
  beside the paste's verdict (`Some(species)` on a read-refused paste,
  `None` on a delivery, on the field's own refusal — a payload that
  sanitized to nothing, the read delivered — and on every write-side
  round trip: a copy never asked a read question), the M39
  `empty_clipboard` collapse split at last — one verdict, two facts,
  the log answering "did the paste refuse for the read, or for the
  field". The wire grows the same field at version 1 (clients that
  ignore unknown fields unaffected), the recorder's ring carrying the
  record verbatim, and the embedded client rendering the species
  beside the paste's refusal. The memory engine names its own known
  emptiness, so headless CI's paste refusals carry the species too.
  The trail's shape is untouched (the same 256-bounded queue, the same
  tick's start drain, one branch per chord while nobody listens), a
  refused read inserts nothing and queues no masks, and
  `examples/clipboard_read.rs` is the consumer demonstration.
- **M43 — the value record names its control (ADR-0049)**: ADR-0046's
  first re-open rule cashed — a consumer needs the *who* (the
  control's identity on the record) so the record grows beside the
  action's routing, never at the note site: one additive
  `ValueRecord::target` carrying the AccessKit-stable id hash the
  dispatch funnel routed, the stamp side living where the knowledge
  lives (`value::ActionTargetGuard`, held by
  `PassPipeline::dispatch_a11y_action` for the length of one widget
  call — the facts queued beneath the action carrying the id, a
  panicking widget leaving no stale identity, the widget never
  guessing its own identity). A multi-control form's trail is
  attributable at last: two sliders on one page, the facts tellable
  apart, the refusals (the parse's, the read-only tier's) naming the
  control that answered them beside the deliveries naming theirs. The
  wire grows the same field at version 1 (the clipboard `engine`
  field's rule — clients that ignore unknown fields unaffected), the
  recorder's ring carrying the record verbatim, and the embedded
  client rendering the control beside the record. The widget's own
  doors stay off the value log — the keyboard's step and the drag
  stream no value fact (the input tier's records carry those rides
  until a consumer prices them, the second kill rule's open arm), the
  attribution exclusive to the AT tier. The writes themselves are
  untouched (the funnel's clamp-and-snap answering byte-for-byte, the
  A7 idle contract one branch per write while nobody listens), and
  `examples/value_targets.rs` is the consumer demonstration.
- **M41 — the clipboard engine names its facts (ADR-0047)**: ADR-0045's
  own re-open rule cashed — a consumer needs the clipboard's
  *platform-side* facts the widget's round trips cannot name, so the
  seam's engine contract grows beside `ClipboardSeam`: one *provided*
  `write_reported` door answering `ClipboardWriteFact` (the M37
  provided-method discipline — the provided default maps the `bool`
  door and names its absence honestly, a real engine overrides and
  names its species; the `write_text` door stays the contract of record
  byte-for-byte, and a refusal through either deletes nothing). The
  species are the engine's own answers: `unnamed` (the honest absence),
  `unsupported` (no clipboard subsystem at all), `occupied` (another
  party held the clipboard), `conversion` (the payload the format
  cannot carry), `os_error` (the platform answered, the engine never
  classifies past its answer) — the arboard engine mapping its own
  error kinds onto the species in the windowed assembly. The round
  trip's record grows the additive `engine` field beside the
  `write_refused` verdict (`Some(species)` on a refused write, `None`
  on a delivery and on every other refusal — a paste reads, a
  no-selection copy never asked the engine), the wire growing the same
  field at version 1 (clients that ignore unknown fields unaffected),
  the recorder's ring carrying the record verbatim, and the embedded
  client rendering the species beside the refusal — the log answering
  "which clipboard failed it", not only "it failed". The trail's shape
  is untouched (the same 256-bounded queue, the same tick's start
  drain, one branch per chord while nobody listens).
- **M40 — the value tier rides the causal log (ADR-0046)**: the ride
  ADR-0010's additive rule and the M31/M33/M36/M37/M39 rhythm
  pre-register cashed. The value tier's AT actions (ADR-0026, the
  `Slider`; ADR-0027, the read-only `Progress`) answered the widget's
  own `bool`, whose `false` collapsed four different facts — the
  control the availability tier holds down, the payload that parsed to
  nothing, the payload the funnel cannot carry, the control that never
  had a write door — into one silence; a status line, an inspector
  trail, or a test could not answer "did the write land — where, or
  why not" without re-implementing the observation at the action site.
  Now every value-tier write the AT asks streams one
  `InspectEvent::Value` — the action beside the outcome and the value
  the control holds, the refusals named (`unavailable`,
  `invalid_payload`, `non_finite_value`, `no_write_door`), the refused
  writes streaming too. The two parse facts the `bool` collapsed are
  named apart at last ("loud" is `invalid_payload`, "inf" is
  `non_finite_value` — the m20 suite's two refusals), a no-op delivery
  answers like the landed write it is (the M20 idle-frame property,
  the log naming it), and a question outside the value tier (a press,
  a scroll, a text edit) is not a value fact. The fact is noted where
  the write resolves (`value::note_write`, the new trail module — the
  journal's own rule), and the value is the tree's own vocabulary the
  AT already reads, never a secret (the clipboard's count-not-payload
  rule was a secret's rule; the value tier has no payload to hide).
  The trail is the clipboard trail's shape (thread-local, 256-bounded,
  oldest-first eviction, the A7 gate — one branch per action while
  nobody listens), the tick's start drain running after the edit drain
  (a value write explains itself; it deletes nothing); the wire grows
  one `value` kind additively at version 1 (the clipboard event's flat
  rule — `docs/inspector-protocol.md` documents it), the recorder's
  ring holds 512 beside the other seams' rings, and the embedded client
  renders the echo in the causal log pane.
- **M39 — the clipboard rides the causal log (ADR-0045)**: the ride
  ADR-0010's additive rule and the M31/M33/M36/M37 rhythm
  pre-register cashed. The editing contract's clipboard chords
  (ADR-0020) answered the field's own `bool`, whose `false` collapsed
  three different facts — the selection that was not there, the
  clipboard that carried nothing, the platform that refused the write
  — into one silence; a status line, an inspector trail, or a test
  could not answer "did the text move — or why not" without
  re-implementing the observation at the chord site. Now every
  round trip the editing chords answer streams one
  `InspectEvent::Clipboard` — the action beside the outcome and the
  payload's UTF-8 scalar count, the refusals named (`no_selection`,
  `empty_clipboard`, `write_refused`), the refused round trips
  streaming too. The fact is noted where the round trip resolves
  (`clipboard::note_round_trip`, the seam module owning the trail —
  the journal's own rule), and the funnels were restructured so one
  round trip streams one fact: a cut writes its own record and never
  rides the copy funnel, the trail reading clipboard → edit (the
  causal order the round trip ran — what moved out, then the deletion
  it drove). The count names the fact; the payload never rides — a
  clipboard can carry a secret, and the log is a devtools surface,
  not a shadow clipboard. The trail is the edit trail's shape
  (thread-local, 256-bounded, oldest-first eviction, the A7 gate —
  one branch per chord while nobody listens), the tick's start drain
  running ahead of the edit drain; the wire grows one `clipboard`
  kind additively at version 1 (the announce event's flat rule —
  `docs/inspector-protocol.md` documents it), the recorder's ring
  holds 512 beside the other seams' rings, and the embedded client
  renders the echo in the causal log pane.
  `examples/clipboard_trail.rs` is the consumer demonstration: the
  beats run the real chords before the window opens (the honest
  `no_selection`, the counted copy, the cut that precedes the
  journal's step, the paste that lands what the cut moved, the locked
  door whose refused write never deletes) and the *trail* label
  mirrors what the log streamed; live chords play the same round
  trips in the window. The corpus stands at 63 (the no-growth arm —
  the trail is dev-mode and bounded, no new hot loop), and the
  living documents re-agree at M39 (the README's status block, count,
  quickstart, and table; the catalog; the register; the roadmap; the
  m-suite pins moving M38 → M39 and 45 → 46 across the chain; the
  protocol document's `clipboard` section).
- **M38 — the primary chord (ADR-0044)**: the platform's primary
  modifier as chord vocabulary — the re-open rule ADR-0043
  pre-registered (ADR-0042's before it) cashed. A chord's shape is
  *physical*, so an app's most common intent — "the platform's
  primary accelerator" (`Ctrl+S` on Linux/Windows, `Cmd+S` on macOS)
  — could not be named by the exact `Modifiers` shape: the app forked
  its table per platform, or reached for a stringly-typed chord the
  funnel cannot compare (the rule ADR-0041 refused). Now one
  `Chord::primary()` builder resolves at build time the way
  `Modifiers::primary` reads — Command on Apple platforms, Control
  elsewhere — no new field, no logical flag, no match-time platform
  context: the chord's shape *is* the event's physical shape, the
  exact-shape match (the M35 invariant) untouched and the funnel's
  compare the compare it always was. The resolution is the chord's
  identity: off Apple platforms a `primary()` chord *is* a
  `control()` chord — one binding per chord makes a rebind move in
  place, never shadow — and on Apple platforms the two are different
  chords, free for different actions. The aliasing never widens: a
  `Cmd+S` event off Apple platforms is a different shape, the consult
  falls through honestly, and the record carries the chord the funnel
  *asked* about. The `Chord::control` doc-drift is fixed, not the
  flag: the doc claimed "Control (Linux/Windows) or Command (macOS)" —
  a claim the physical shape never kept — and named a `primary()`
  builder that did not exist; it now pins the physical `control` flag
  and points at `Chord::primary`. The wire is untouched (the `key`
  event's `modifiers` field prints the resolved physical shape — the
  truth the wire always carried), and `docs/inspector-protocol.md`
  says so.
  `examples/primary_chords.rs` is the consumer demonstration: the
  real `dispatch_key` beats with the observer riding — the honest
  uninstalled window, the fired record carrying its name under the
  platform's own flag, the other platform's flag falling through
  without aliasing, the physical shape binding over the primary one
  *in place* — with tracing spans at `alacer::primary_chords`. The
  corpus stands at 63: the resolution happens at bind time (a `cfg!`
  branch, no match-time cost — the coordinated rule's no-growth arm).
  The living documents re-agree at M38 and 0.17.0 (the README's
  status block, count, quickstart, and table; the catalog; the
  register; the roadmap; the m-suite pins moving M37 → M38 and 44 →
  45 across the chain; the protocol document's `modifiers` sentence).

- **M37 — the binding names itself in the record (ADR-0043)**: the
  consult's prose name rides — the re-open rule ADR-0042 pre-registered
  on its own kill-criteria list cashed. The M36 record carried the
  chord beside the species (the chord the fired binding's identity,
  one binding per chord), but a trail reader that wanted the
  shortcut's *prose* name — a settings UI reading the log, an
  inspector row a human labels — had to keep its own chord → name
  table beside the log. Now the seam grows a *provided*
  `KeymapSeam::fired_name` beside `dispatch`: the provided `Keymap`
  table answers the binding its `dispatch` would run (the same match,
  the same chord scan), a custom seam names its own answer, and one
  that overrides nothing keeps the default `None` — the M35/M36
  custom seams compiling and streaming unnamed, the record honest
  about the absence. The answer is the record field's own
  `Option<String>` (a wrapper seam delegates through its guard, where
  a borrowed `&str` could not escape), one `consult_name` pure read
  answers it beside `consult` — running no action, the consult's
  dispatch staying the only door (never a second consult channel) —
  and `PassPipeline::dispatch_key` captures it *before* the consult's
  dispatch (the name the consult resolves to, never what a mid-flight
  rebind leaves behind) and only under an observer (the A7 idle
  contract: the unobserved funnel byte-for-byte). `KeyRecord` grows
  `name` beside chord and outcome — `Some` only on a fired consult, `None`
  on a miss or an uninstalled window or a nameless seam — trading its
  M36 `Copy` for `Clone` (the name is a heap `String`; the chord keeps
  the by-value ride). The wire grows the `key` event's `name` field
  additively at version 1 (the rider field's rule, ADR-0010), the
  embedded client renders the name beside the outcome, and
  `docs/inspector-protocol.md` documents the field.
  `examples/shortcut_names.rs` is the consumer demonstration: the real
  `dispatch_key` beats with the observer riding — the provided table's
  binding naming itself, the miss that names nothing, the custom
  seam's own answer, the nameless seam's honest absence, the rebind
  moving the name with the chord — with tracing spans at
  `alacer::shortcut_names`. The corpus stands at 63: the name leg
  costs the unobserved funnel nothing and rides the published keymap
  route's consult (the coordinated rule's no-growth arm, the
  M31/M33/M36 rule). The living documents re-agree at M37 and 0.17.0
  (the README's status block, count, inspector bullet, quickstart,
  and table; the catalog; the register; the roadmap; the §13.3
  questions table; the m-suite pins moving M36 → M37 and 43 → 44
  across the chain; the protocol document's `key` section).

- **M36 — the keymap rides the causal log (ADR-0042)**: the consult's
  answer as data — the re-open rule ADR-0041 pre-registered on its own
  kill-criteria list cashed. The M35 funnel consult answered a bare
  `bool` whose `false` collapsed three different facts; now one
  `KeymapConsult` (`Uninstalled` / `FellThrough` / `Fired`, `fired()`
  the verdict accessor, `name()` the wire's tag) and one `consult`
  door answer the read as data with `dispatch_installed` staying the
  fired verdict (the M35 contract byte-for-byte), and
  `PassPipeline::dispatch_key` streams one `InspectEvent::Key`
  (`KeyRecord`: the chord beside the species) per consult it runs —
  the moment it resolves, before the fallbacks' focus records, so a
  key event's trail reads keymap → focus → input → frame. The chord is
  the fired binding's identity (one binding per chord), the focused
  widget's precedence is visible as an absent record, and `Key`/
  `Chord` gain the additive `Copy` the by-value record rides. The
  trail is dev-mode and bounded: one branch per consult when no
  observer is installed (the A7 idle contract,
  `PassPipeline::set_observer` enables it), the recorder growing the
  key ring beside its rings (`CausalRecorder::keys()`, 512). The wire
  grows the `key` event additively at version 1 (the rider field's
  rule, ADR-0010), the embedded client renders the echo beside the
  focus and edit trails, and `docs/inspector-protocol.md` documents
  the event.
  `examples/keymap_trail.rs` is the consumer demonstration: the real
  `dispatch_key` beats with the observer riding — the uninstalled
  window's honest answer, the fired chord naming itself, the miss
  that falls through, the rebind moving the record's chord with it —
  with tracing spans at `alacer::keymap_trail`. The corpus stands at
  63: the record rides the published keymap route's consult and
  streams only under an observer (the coordinated rule's no-growth
  arm, the M31/M33 rule). The living documents re-agree at M36 and
  0.17.0 (the README's status block, count, inspector bullet,
  quickstart, and table; the catalog; the register; the roadmap; the
  §13.3 questions table; the m-suite pins moving M35 → M36 and 42 →
  43 across the chain; the protocol document's `key` section).

- **M31 — the journal rides the causal log (ADR-0037)**: every
  `EditHistory` action streams one `InspectEvent::Edit` — the hook the
  journal's own docs pre-registered ("the inspector may *someday*
  surface the depth") cashed. The record carries the action
  (`EditAction`: `record`/`undo`/`redo`/`reset`, `name()` the wire's
  tag), the `coalesced` flag (the pure-append fold visible as a depth
  that never moves), the `refused` flag (an exhausted undo or an empty
  redo branch — the refusal streams too, the depths naming the journal
  it did not move), and the depths the action left. The journal queues
  the fact on the editing thread (the clipboard-seam shape, ADR-0020 —
  handlers cannot reach the pipeline, axiom A1) and the dispatching
  pipeline drains it at the tick's start (the announcement drain's
  shape, ADR-0024) — one frame after the chord. The trail is dev-mode
  and bounded: one branch per action when no observer is installed
  (the A7 idle contract; `PassPipeline::set_observer` enables it), a
  256-fact queue with the journal's own oldest-first eviction (the
  §14.3 rule applied to the echo). The wire grows the `edit` event
  additively at version 1 (the rider field's rule, ADR-0010), the
  recorder grows the bounded edit ring beside its rings
  (`CausalRecorder::edits()`, 512), and the embedded client renders
  the echo beside the focus trail.
  `examples/edit_trail.rs` is the consumer demonstration: the M15
  journal driven with the observer riding — the coalescing, the
  applied chords, the app-write reset, the exhausted journal's
  refusal — with tracing spans at `alacer::edit_trail`. The corpus
  stands at 62: the echo rides no hot loop the corpus does not
  already price (the coordinated rule's no-growth arm).

- **M32 — the value tier joins the view registry (ADR-0038)**: the
  re-open rule the M6 canon's registry work left open cashed — the
  `slider` and `progress` elements join `default_registry`, so the
  value tier's widgets are `rsx!` vocabulary like the M6 canon before
  them. Create reads the value props (`name`/`min`/`max`/`step`/
  `value`/`on_change`/`accent`, `fill`, `binding`); patch takes what
  has an in-place writer, and the patch answers are the widgets' own:
  the slider's programmatic write clamps and snaps inside the widget,
  fires no handler, and reports visual only when the funnel moved the
  value (the widget's own `total_cmp` equality — never `f32::eq`);
  the progress's `set_value` bool is the M21 idle-frame property — a
  write that lands where the value already is queues no masks. The
  prop vocabulary grows the two types the tier needs:
  `PropValue::NumberHandler` carries the funnel's value (`on_change` —
  the flag/text handlers' rule at a value, the patcher's
  handler-churn exemption covering it), and `PropValue::NumberBinding`
  rides the f32 `RenderBinding` (`binding: my_f32_binding` — the value
  typed by what it carries; a bound progress's patch re-hangs the same
  slot idempotently, the bound label's rule at the value tier). The
  bridge still owns the value: the assembly-time `bind_value` the
  `counter_view` rule names — the DSL expresses the shape, the flux
  bridge expresses the reactivity, neither grows a second channel
  (axiom A1). `examples/value_view.rs` is the consumer demonstration:
  the slider's funnel routing to one signal, the bridge fanning the
  write out to the bar, the label, and the tree's answer. The corpus
  stands at 62: the elements ride the published view routes (the
  coordinated rule's no-growth arm).

- **M33 — the announcement rides the causal log (ADR-0039)**: every
  speech act the pipeline answers streams one `InspectEvent::Announce`
  — the hook the M18 contract's own refusals pre-registered ("why did
  the live region speak — or refuse") cashed. The record carries the
  politeness the act asked for and the outcome it got
  (`AnnounceOutcome::name` the wire's tag: `delivered`, `superseded` —
  the last-write-wins rule visible as data, the batch draining whole —
  or `refused` with the named species `AnnounceRefusal::{EmptyText,
  NoRoot}`; the refusals stream too, the M28 honesty at the speech-act
  tier). The facts stream where the answer resolves: the direct door's
  answers at the door (`PassPipeline::announce`), the push-side empty
  refusal on the thread's trail (the clipboard-seam shape, ADR-0020),
  the drain's batch answers at the tick's start (the edit trail's
  shape, ADR-0037). The trail is dev-mode and bounded: one branch per
  act when no observer is installed (the A7 idle contract;
  `PassPipeline::set_observer` enables it beside the edit trail's), a
  256-fact queue with the §14.3 oldest-first eviction. The wire grows
  the `announce` event additively at version 1 (the rider field's
  rule, ADR-0010), the recorder grows the bounded announce ring beside
  its rings (`CausalRecorder::announces()`, 512), and the embedded
  client renders the echo beside the focus and edit trails.
  `examples/announce_trail.rs` is the consumer demonstration: the M18
  seam driven with the observer riding — the delivered act, the empty
  refusal, the storm's superseded facts beside the survivor — with
  tracing spans at `alacer::announce_trail`. The corpus stands at 62:
  the speech acts ride the published announcement route (the
  coordinated rule's no-growth arm; the route re-measured at this cut:
  18.62 ms median against the published 18.81 ms, -1.0%).

- **M34 — the backend features are true (ADR-0040)**: the capability
  the feature names pre-registered (`backend-wgpu` /
  `backend-softbuffer` forward to optional dependencies, promising
  single-backend builds the tree never delivered — the platform's
  backend modules, enum, and construction paths compiled
  unconditionally, so `--no-default-features` failed with 151 errors
  and every single-backend combination failed at `cargo check`)
  cashed. Each present path compiles with its feature: the modules,
  re-exports, `Backend` variants, and per-backend builders gate with
  the feature that owns them — all four combinations compile
  warning-free under `-D warnings` (both backends — the default, its
  behavior unchanged — each alone, and none). A preference whose
  backend the build compiled out refuses honestly at startup:
  `PlatformError::BackendUnavailable { requested }` (the preference's
  own name, `BackendPreference::name` the single source) — the display
  naming the gate that answers it, the chain ending at the build fact
  (the M16 boundary's leaf rule at the build tier; never a panic,
  never a munged fallback). CI gains the features job exercising all
  four combinations: the no-backend build compiling and running its
  refusal tests, each backend alone, and the facade's software-only
  and pipeline-only builds.

- **M35 — the keymap seam: the app's shortcuts ride the dispatch
  contract (ADR-0041)**: the keyboard's window-level half ships — the
  actions an application names (save, find, export) answer wherever
  the focus sits, through one [`KeymapSeam`] the assembly site
  installs (the clipboard seam's thread-local discipline, ADR-0020:
  `Chord` names one logical `Key` plus the exact `Modifiers` shape it
  answers — `Ctrl+S` never answers `Ctrl+Shift+S`, that is a
  different chord free for another action; `Keymap` is the provided
  table, one binding per chord, `bind` replacing in place so the
  settings story is a move and never a shadow, `unbind` returning the
  chord to the framework's fallbacks, `names` the introspection a
  settings read-back walks; the install trio `install` /
  `is_installed` / `dispatch_installed`). `PassPipeline::dispatch_key`
  consults the seam when the focused widget did not answer and before
  the framework's own fallbacks — the precedence made three-tier and
  pinned by the m35 exit suite: the focused widget's vocabulary (a
  focused input's Space types, never fires), the app's shortcuts (the
  explicit beats the implicit: a bound plain `Tab` holds focus while
  the unbound `Shift+Tab` — a different chord — still walks
  backward), and the framework's Tab cycling (M24) and directional
  ring (M29) standing exactly where the app stayed silent. The
  consult needs no focused widget at all: window-level vocabulary
  answers wherever the focus sits, and a shortcut is focus-neutral —
  the fired action queues no masks of its own (the M21 idle-frame
  property at the app seam). The press-only match carries the GTK
  activation rule to the window tier (a release is never a second
  firing, an auto-repeat one run per physical press), the fired
  binding names itself on the `alacer::keymap` target (the house
  tracing idiom), and the seam is optional so the read is total — no
  panic door, the clipboard's rule inverted for an optional seam; a
  window with no shortcuts answers byte-for-byte today's behavior
  (the M11 rule at the dispatch seam, the A7 idle contract
  unchanged). `examples/shortcut_map.rs` is the consumer
  demonstration (tracing at `alacer::shortcut_map`). The corpus
  grows 62 → 63 by the coordinated rule: the keymap's mixed route
  (`keymap/dispatch_100_mixed_1k` — one thousand events alternating
  a miss and a mid-table hit against one hundred bindings, through
  the funnel's own read) joins the published routes with its gate
  and its baseline in the same cut (the m5–m30 chain's pins moving
  together).

- **M30 — the trail names the rider (ADR-0036)**: every `FocusRecord`
  the causal log streams grows a `rider` field beside its `movement` —
  a `FocusRider` naming who rode the walk: `Keyboard` (the keyboard's
  Tab/Shift+Tab fallback, the M24 seam), `Ring` (the four directional
  keys' fallback, the M29 seam — a D-pad button, a TV remote's ring),
  or `Consumer` (the app's own `focus_move`/`focus_move_in` door
  calls) — ADR-0035's pre-registered re-open rule ("a consumer needs
  the rides distinguished in the causal log → grow `FocusRecord`
  beside its `movement` (a rider field), priced by a real reader —
  never a second trail") cashed. One funnel, three riders, one trail:
  the attribution is declared where the route is known — the public
  doors stream `Consumer`, the dispatch fallbacks stream through one
  private rider-aware door (`Keyboard` for Tab, `Ring` for the
  arrows), the walk behind the doors untouched (the same candidate
  list, the same availability filter, the same wrap, the same
  immediate resolution, the same no-mask idle refusals — the M11 rule
  stays). The wire grows the `rider` field (`keyboard` / `ring` /
  `consumer`, `FocusRider::name` the single source) as the additive
  kind the protocol's versioning rule admits at version 1 —
  `PROTOCOL_VERSION` stays 1 — and the embedded client renders the
  rider beside the movement (M31). `examples/focus_riders.rs` is the
  consumer demonstration: the M26 window walked by all three riders,
  every beat logging the record's rider beside the answer it names,
  with tracing spans at `alacer::focus_riders`. The corpus stands at
  62: the riders ride the published routes (the coordinated rule's
  no-growth arm).

- **M29 — the directional keys ride the walk (ADR-0035)**: a plain
  directional key the focused widget did not answer — a D-pad button,
  a TV remote's ring — maps onto its `FocusMove` at
  `PassPipeline::dispatch_key`'s fallback (the seam Tab and Shift+Tab
  ride since M24) and lands the geometry's answer through the real
  dispatch path — ADR-0032's pre-registered re-open rule cashed. The
  platform's key map already carries the four arrows
  (`alacer-platform/src/keys.rs`), so the seam is one `match` — no
  second dispatch channel. The widget's answer always takes precedence
  (a focused input's caret, a slider's step — the m20/m11/m13
  contracts stand), a modifier held means the key belongs to whatever
  vocabulary answers it (the chords are the widgets'; the window's
  walk is never a chord's rider), and a refused direction falls
  through honestly (`EventResponse::Ignored`, the refused Tab's
  shape) with the M28 trail carrying the arrows' rides like every
  ride — one funnel, now three riders: Tab, the code, the ring.
  `examples/dpad_navigation.rs` is the consumer demonstration: the
  M26 window walked by arrow presses through the real dispatch, with
  tracing spans at `alacer::dpad_navigation`. The corpus stands at 62:
  the arrows ride the published routes (no new criterion case).

- **M28 — the walk joins the causal log (ADR-0034)**: every focus-move
  door ride streams one `InspectEvent::Focus` the moment the door
  resolves, carrying a `FocusRecord` (the movement the caller asked
  for, the scope that bounded the walk — `None` for the unscoped
  funnel — and the door's own `FocusMoveOutcome` verbatim) —
  ADR-0033's pre-registered re-open rule ("a consumer needs to observe
  the walk's refusals as *streamed events*") cashed. One funnel, two
  doors, one trail: the keyboard's Tab fallback rides `focus_move`, so
  its rides stream the same records the consumer's code streams, and
  the scoped door's dead-scope refusal streams before the walk runs.
  The walk behind the doors is untouched (the same candidate list, the
  same availability filter, the same wrap, the same immediate
  resolution, the same no-mask idle refusals — the M11 rule stays);
  without an observer the doors pay one branch per call (the A7 idle
  contract unchanged). The recorder grows the bounded focus ring
  beside its frames/edges/writes/inputs (`FOCUS_RING_CAP`, oldest
  dropped first — the §14.3 discipline; `CausalRecorder::focus()` is
  the read door), and the wire grows `WireEvent::Focus` with the
  documented flat shape (`movement`, `scope`, `moved`, `from`, `to`,
  `wrapped`, `refusal`, `node`, `direction`) — a new event kind,
  additive by the protocol's own versioning rule, `PROTOCOL_VERSION`
  stays 1. `examples/focus_trail.rs` is the consumer demonstration:
  the M26 window walked one beat per `FocusMove` with the observer
  riding, every beat logging the streamed record beside the door's
  own answer, with tracing spans at `alacer::focus_trail`. The
  observed route (`input/route_100_observed_moves_1k`) joins the
  published corpus beside the four focus routes, the corpus pins
  moving 61 → 62 across the m5–m27 chain in the same cut.

- **M27 — the walk's answer as data (ADR-0033)**: the two focus doors
  (`PassPipeline::focus_move`, `PassPipeline::focus_move_in`) grow their
  answer from the M24-era boolean into the `FocusMoveOutcome` vocabulary —
  ADR-0031's pre-registered re-open rule ("a door needs to observe the
  walk's refusals as data") cashed. A `Moved(FocusLanding)` carries the
  endpoints (`from` — `None` for the entry — and `to`) and the wrap (the
  tree-order move crossed a list end; the entry and the directional moves
  never carry it); a `Refused(FocusRefusal)` names its species — the
  empty list, the dead scope, the geometry-less focus, the empty
  direction (with its node and direction), the single-candidate stall —
  instead of five refusals collapsed into one silent `false`. The walk
  behind the doors is untouched: the same candidate list, the same
  availability filter, the same wrap, the same immediate resolution, the
  same no-mask idle refusals (the M11 rule stays; the answer stops
  making the caller guess), the per-species help staying plain text on
  the `alacer::focus` target (ADR-0029's rule), the keyboard's Tab
  fallback routing on the `moved()` verdict accessor unchanged.
  `examples/focus_outcome.rs` is the consumer demonstration: the M26
  window walked one beat per `FocusMove`, every beat logging what the
  walk answered as data, with tracing spans at `alacer::focus_outcome`.
  The outcome route (`input/route_100_outcome_moves_1k`) joins the
  published corpus beside the three focus routes, the corpus pins moving
  60 → 61 across the m5–m26 chain in the same cut. The repair debt the
  sweep priced rides the cut: the M23-era required `SceneJournal::stats`
  never landed on three `NullJournal` impls (the `pass_overhead` bench
  and two test harnesses — the M23 cut's own benches and tests could not
  compile at the shipped HEAD), the three focus-route baselines the M26
  cut promised at its measurement run and never published (the m5
  bijection red at HEAD, `baselines.json` stalled at 57 entries against
  60 pins) land this cut, the `focus_trap`/`focus_traversal` examples'
  unboxed multi-line-closure buttons and unused `Widget` imports go (the
  examples did not compile at HEAD), and the dead `Slider::set_on_change`
  and the dead `AccessKitSink::is_initialized` (its only caller was its
  own unit test — the M25 species) go with them.

- **M26 — the directional focus walk (ADR-0032)**: the walk reads the
  window, not only the tree. ADR-0030's pre-registered re-open rule
  (spatial/D-pad navigation) ships as four `FocusMove` directions
  (`Left`/`Right`/`Up`/`Down`) riding the same two doors
  (`PassPipeline::focus_move`, `PassPipeline::focus_move_in`) and the
  same one walk (`cycle_focus_in(scope, movement)` — the boolean the
  M25 walk took becomes the movement it aims). The tree-order moves
  keep the pre-order list and their wrap; the directional moves read
  the bounds pass 6 wrote (`directional_neighbor` — a candidate
  qualifies fully beyond the focused node's edge in the direction, the
  nearest edge distance wins, cross-axis center misalignment breaks
  ties, then tree order), the M22 availability filter runs before the
  geometry answers, the scope bounds the directional walk exactly as it
  bounds the tree-order one, and space never wraps — the window's edge
  is the walk's end, an edge refusal queues no masks (an idle frame),
  and the refusals carry their help as plain diagnostic text on the
  `alacer::focus` target (ADR-0029's rule). The entry contract is
  repaired to what ADR-0031 has documented since M25: from no focus —
  or from a focus outside the filtered candidate list — the walk takes
  the list's first candidate, both directions (the M25-era arithmetic
  landed the second forward and the last backward).
  `examples/focus_direction.rs` is the consumer demonstration: a
  two-by-two grid where the geometry's answer and the tree's next stop
  disagree, the corner's honest refusal while `Next` still wraps, and
  the modal's form walked directionally inside its scope, with tracing
  spans at `alacer::focus_direction`. The micro-debt the sweep priced
  rides the cut: the selection's dead `is_degenerate` and the color's
  dead `to_premultiplied_rgba8` (its only caller was its own unit test
  — the M25 species), and the corpus debt the M24/M25 coordinated rule
  left rides the cut: the promised focus routes
  (`input/route_100_focus_moves_1k`, `input/route_100_scoped_moves_1k`)
  join the published corpus beside the directional route
  (`input/route_100_directional_moves_1k`) at this cut's measurement
  run, the corpus pins moving 57 → 60 across the m5–m25 chain in the
  same commit.

- **M25 — the focus scope (ADR-0031)**: the focus walk bounds to a
  subtree. ADR-0030's pre-registered re-open rule ships as one
  `PassPipeline::focus_move_in(scope, movement)` door beside the
  unscoped funnel — the modal's focus trap priced by the real UI it
  names. The scope is the subtree's root node (the cheapest honest
  predicate specialization: no closures in the public API, no second
  walk), the walk generalizes behind both doors (`cycle_focus_in` — the
  same pre-order candidate list pass 2 recomputes, the M22 cycle-time
  `enabled` filter, the wrap at both ends, the immediate resolution),
  the scope bounds the walk never the candidate list (`focusables()`
  and `focus()` keep answering the whole tree, the unscoped door stays
  the keyboard's funnel, and a root-scoped walk lands exactly where
  `focus_move` lands), and the refusals are honest and named — a dead
  scope, an empty trap, or a trap the availability tier holds down
  answers `false` with no masks (an idle frame, never a
  first-candidate reset), their help plain diagnostic text on the
  `alacer::focus` target (ADR-0029's rule). The landings and refusals
  ride the `alacer::focus` tracing target, and
  `examples/focus_trap.rs` is the consumer demonstration: a background
  control and a modal's form walked forward, backward, and into the
  refusal inside the trap, with the window's Tab proving the unscoped
  door live. The micro-debt the sweep priced rides the cut: the layout
  mapping's dead `fixed_size_of` and the color's dead `to_f32_rgba`
  (documented for a GPU-uniform consumer that never landed), and the
  living documents re-agree at M25 and 0.16.0 — the README badge and
  status block riding the pending-cut convention, the migration guides'
  family line among them, and the m7 release-state pin re-joining the
  chain after missing the M23/M24 sweeps.

- **M24 — the focus walk (ADR-0030)**: the focus walk joins the
  dispatch contract. Tab's private cycle becomes public vocabulary: one
  `FocusMove` direction (`Next`/`Prev`) and one pipeline door —
  `PassPipeline::focus_move` — that the keyboard's Tab/Shift+Tab
  fallback and the consumer's code both ride: one funnel, two doors,
  the walk's order never duplicated at the call site. The walk follows
  the pre-order
  candidate list pass 2 recomputes, skips controls the availability
  tier holds down (the M22 cycle-time `enabled` check), wraps at both
  ends, resolves immediately (key routing sees the new focus this
  event), and refuses honestly when no candidate exists — no masks, an
  idle frame, never a first-candidate reset. The landings and refusals
  ride the `alacer::focus` tracing target, and
  `examples/focus_traversal.rs` is the consumer demonstration: a
  four-control form walked forward, backward, and into the refusal,
  with the window's Tab proving the funnel live. The micro-debt the
  sweep priced rides the cut: the flux runtime's dead `effect_count`
  introspection and the harness's dead `spec_of`/`with_text_context`/
  `text_size` probes, and the living documents re-agree at M24 — the
  m1 suite's header among them, which had said "36 blessed snapshots"
  over a corpus its own pin counts at 37.

- **M23 — the frame's ledger (ADR-0029)**: the scene journal becomes
  legible to its consumers. Every seam since M16 paired with a consumer
  demonstration, but the M2 headline — the journal whose economics every
  other seam spends — stayed undemonstrated, and its introspection
  accessors were dead code precisely because no consumer existed. One
  required `SceneJournal::stats()` seam ships, carrying the
  `JournalStats` vocabulary (commits, drops, live layers) with
  `PassPipeline::journal_stats()` as the pipeline's one read door; the
  consumer demonstration ships: `examples/frame_ledger.rs` reads the
  ledger at the two beats that matter — the mount frame (every layer
  commits once) and one ticker write batch (exactly one layer
  re-commits, the static layer's clean pixels replaying untouched) —
  with the interactive *Tick* press riding the same one-batch route and
  the ledger on the `alacer::frame_ledger` tracing target. The m1
  corpus's broken trio is repaired at the source: the `list_scrolled`
  scenarios never fed `set_content_height`, so the scroll clamp folded
  every offset to zero and three distinct scroll positions blessed one
  byte-identical image — the shared helper now derives the content
  height from the laid-out children, `list_scrolled_120` grows to twelve
  panels so its 120 px target is inside the clamp's real range, the
  m23 exit suite pins the delta headlessly and the three images
  pairwise-distinct, and the next bless run re-blesses them distinct
  (CONTRIBUTING's diff-is-the-review rule). The micro-debt the sweep
  priced rides the cut: the journal's dead inherent accessors
  (`commits`, `drops`, `layer_count`, the slice-view `layer` duplicate)
  replaced by the seam that gives their data a caller, the todo canon's
  dead `footer_node` getter with its write-only field, the `rsx!`
  body-cardinality diagnostic that now names its cardinality with
  miette-class `help:` text (plain diagnostic text — the
  no-new-dependency rule outranks adopting the crate), and the
  inspector's accept loop reporting the session errors its silent
  `let _ =` swallowed (through the adopted `tracing` core — the one
  dependency edge the cut adds, and it stays inside the resolved graph).
  The living documents re-agree at 0.14.0: the README's badge, status
  block, quickstart, perf bullet, and example table; the catalog; the
  architecture register (28 → 29 with ADR-0029); the roadmap; the
  migration guides' family line; the widgets manifest's shipped
  vocabulary; the facade's shipped-surface rustdoc; and the m20/m21/m22
  pins move with them in the same commit
- **M22 — the unavailable control (ADR-0028)**: the availability tier
  joins the widget contract. The ledger M21 emptied had no value-tier
  candidate left, and the audit-before-the-cut pass priced the property
  every real form needs beside it: no widget could *report that it must
  refuse* — a spent button looked, read, focused, and answered exactly
  like a live one, the AT announced a control the tree claimed was
  actionable, and the press died silently. One `Widget::enabled` source
  of truth ships: the default `focusable()` derivation consults it (an
  unavailable control is never a Tab stop and never a focus candidate),
  pass 3's sentinel discipline evicts focus from a control that becomes
  unavailable, and every door refuses honestly — the pointer is a wall
  that never clicks through (the HTML `disabled` semantics), the
  keyboard and the IME are answered with silence, and every AT action is
  refused, never munged (the M11 rule). The flag rides pass 9's
  snapshot — `TreeNode.enabled`, no second channel — and the a11y sink
  maps it faithfully onto AccessKit's `disabled` property, so the AT
  reads the refusal before it happens. Two doors land the same
  `PAINT | A11Y` currency: the fixed door (`.enabled(bool)` at
  construction, `set_enabled` in place) answers the move it made — a
  write that lands where the flag already is queues nothing — and the
  button's binding door (`enabled_binding`) tracks a `RenderBinding<bool>`
  the flux bridge drives (the `Progress::bound` rule, at the
  availability tier). The look tells the truth the tree carries: the
  muted paint the theme constants borrow, with the shipped corpus's
  enabled pixels untouched. The consumer demonstration ships:
  `examples/disabled.rs` drives the bound door from the consumer side
  (the *Enable sending* checkbox's one write batch) and walks the mutate
  seam at startup, with `tracing` spans at the new `alacer::disabled`
  target. The coordinated corpus completes: the refused press's criterion
  case `disabled/route_100_refused_presses_1k` is measured (100
  samples), published with its baseline entry, and gated same-host —
  corpus 56 → 57, every M-suite pin moved together — and the m1 corpus
  grows 36 → 37 with the availability window. The micro-debt the sweep
  priced rides the cut: the software renderer's dead `glyphs_rendered`
  (the M21 gate's own blind spot — the same-named accessor the wgpu cut
  removed still shipped here), the dead `PassPipeline::clear_observer`
  and `PassPipeline::timings_enabled`, and the `ViewRoot::root`
  duplicate whose doc claimed a caller that does not exist; the `rsx!`
  fall-through diagnostic names what stood where an element was
  expected, and the unterminated-`{` text error carries the escape
  hatch's `help:`. The living documents re-agree: the README's count,
  bullet, table, and status block, the catalog, the architecture
  register (28 → 29), the roadmap, and this changelog moved with it
- **M21 — the reported value (ADR-0027)**: the value tier grows its read
  half. The ledger M20 emptied had one unclaimed completion: the AT can
  *write* a value, but an app had no widget-shaped way to *report* one.
  The progress indicator ships as `Progress`
  (`crates/alacer-widgets/src/progress.rs`) over a new
  `Role::ProgressIndicator` in core — requires-name (an unnamed report
  reads as a bare number, ADR-5), not focusable (a report holds no
  focus), and carrying the app's bounded scalar through the M3
  `value()` contract, with the a11y sink mapping the role faithfully
  (`AccessKitRole::ProgressIndicator`). Two app doors land the same
  `PAINT | A11Y` currency: the binding bridge (one write batch, the
  caller's masks — the reactive-text seam, at a value) and the mutate
  seam (the pass-4 queue's lease, the write in place, the write's own
  masks on the `WidgetMut` guard), whose `set_value` answers the move it
  made so a write that lands where the value already is queues nothing —
  the idle-frame property, at the app seam. The AT tier is read-only by
  construction: every action is refused honestly, never munged (the M11
  rule). The consumer demonstration ships: `examples/progress.rs` drives
  both doors with `tracing` spans at the new `alacer::progress` target —
  the mutate seam's first consumer demonstration. The coordinated corpus
  completes: the value-write route's criterion case
  `progress/route_100_value_writes_1k` is measured (100 samples),
  published with its baseline entry, and gated same-host — corpus
  55 → 56, every M-suite pin moved together — and the m1 corpus grows
  35 → 36 with the reported value's window. The micro-debt the audit
  priced rides the cut: the dead `PassPipeline::has_observer` and the
  wgpu backend's dead stats surface (`glyphs_rendered`,
  `instance_frames_skipped`, `atlas_clears`) go. The living documents
  re-agree: the migration guides' 0.12.0 self-description names the
  shipped family again (the M16 pattern's remainder), and the README,
  the catalog, the architecture register (27 → 28), the roadmap, and
  this changelog moved with it
- **M20 — the valued widget (ADR-0026)**: the slider joins the widget
  vocabulary and the value tier becomes a widget's contract. The
  ledger's last feature candidate ships as `Slider`
  (`crates/alacer-widgets/src/slider.rs`) over a new `Role::Slider` in
  core — requires-name (an unnamed value control reads as a bare
  number, ADR-5), focusable, and carrying its bounded value through the
  M3 `value()` contract, with the a11y sink mapping the role faithfully
  (`AccessKitRole::Slider`). One funnel, four doors: a drag writes the
  value from the pointer's x against the node's resolved frame (the
  capture route ADR-0025 shipped keeps the thumb with the hand), the
  arrow keys step by the step, Home/End jump to the ends, and the AT's
  tier — `SetValue` (parsed, then snapped like every write),
  `Increment`, `Decrement` — answers through the same action route the
  editor and the scroll area hold. Every write clamps to the range and
  snaps to the step inside the widget; a write that lands where the
  value already is queues no masks (the idle-frame property, at the
  widget tier); a non-numeric or non-finite request is refused
  honestly — never munged into a step, the M11 rule. The consumer
  demonstration ships: `examples/slider.rs` (one slider, one bound
  status line mirroring the value the tree carries), pinned by the
  widget's unit tests and the new m20 exit suite. The coordinated
  corpus completes M19's promise: the capture route's criterion case
  `input/route_100_captured_moves_1k` is measured (100 samples),
  published with its baseline entry, and gated same-host — corpus
  54 → 55, every M-suite pin moved together — and the m1 corpus grows
  33 → 35 with the promised drag-capture scenario and the slider's own
  window. The living documents re-agree: the README's badge and status
  block finally quote the 0.13.0 family the manifest bumped (the M16
  pattern's remainder), the vocabulary list names the widgets it
  ships, and the catalog, migration guides, architecture register
  (26 → 27 with ADR-0026), and roadmap moved with it
- **M19 — the captured drag (ADR-0025)**: the pointer capture joins
  the dispatch contract. Pass 1 hit-tested every event independently,
  so a drag that wandered off its target lost its route mid-gesture —
  the M12 drag-select clamped at the field's edge, and a
  press-inside/release-outside button stayed pressed with no release
  ever arriving. A press now implicitly holds its deepest accepting
  hit (single-pointer — the event vocabulary carries no pointer id,
  ADR-0014's shape), the gesture's `Move`/`Up`/`Leave` stream routes
  through the captured node's ancestor chain with the same
  capture→bubble phases, the `Up`/`Leave` and the target's death
  release the capture honestly, the wheel stays hit-tested (it scrolls
  the hover, not the gesture), and a frame with no capture in flight
  is the per-event hit test exactly as before — one `Option` check on
  the hot path. `PassPipeline::captured` introspects the holder and
  the capture route traces at the new `alacer::capture` target (the
  per-seam idiom). The consumer demonstration ships:
  `examples/drag_capture.rs` (press the panel, drag past its edge, the
  log keeps counting; wave without pressing and only the moves inside
  the frame arrive), pinned by the pipeline's unit tests and the new
  m19 exit suite. The surface work rides the cut: the dead
  `EventCoalescer::pending` goes (zero call sites workspace-wide, the
  M18 pattern), and the `rsx!` single-root diagnostic names the token
  it found with its help text instead of discarding it. The living
  documents re-agree: the README's criterion count finally quotes the
  54-case baseline, the architecture's §10.1 routing spec states the
  capture it ships (the ADR register grows to 26 with ADR-0025), and
  the examples catalog carries the demonstration. The capture route's
  criterion case and the example's blessed-corpus scenario join at the
  measurement run by the coordinated rule (ADR-0011; CONTRIBUTING's
  publishing rule)
- **M18 — the announced delta (ADR-0024)**: the a11y announcement seam
  joins the invalidation currency. The app's voice — state changes that
  own no widget ("Saved", "3 items left", "Connection lost") — ships as
  a reserved live-region node in pass 9's snapshot: the same snapshot,
  the same `A11yDelta`, the same `Change::A11Y` currency as every other
  a11y fact (no second channel). The region is present from the first
  frame (the root's last child, named empty), so announcing is one
  node-only delta — no `STRUCTURE` churn, no ghost removals; the
  politeness rides the node's `aria-live` priority and the sink emits
  the ARIA pairing (`status` speaks politely, `alert` interrupts, the
  `accesskit::Live` property a screen reader watches). Two doors, one
  destination: `PassPipeline::announce` for assembly sites and tests,
  and the handler-reachable `alacer_core::announce::announce` queue
  (the clipboard-seam shape) drained at the next tick's start — the
  input queue's shape, one frame later, last write wins, a refused act
  (empty text, no root) honest with no masks. The consumer
  demonstration ships: `examples/announcements.rs` (polite, assertive,
  and the honest refusal, one `tracing` span per route at the
  `alacer::announce` target), pinned by the m1 corpus's new
  `announcements` scenario (the corpus grows 32 → 33 scenarios, 34 →
  35 PNGs), and the announcement route joins the bench corpus by the
  coordinated rule — `announce/route_100_announcements_1k`, 53 → 54
  cases, the baseline published with its same-host regression contract
  and every M-suite pin moved together. The error-chain discipline
  completes: `PlatformError::Softbuffer` and `BackendError::
  SurfaceCreation` carry their structured causes
  (`softbuffer::SoftBufferError`, `wgpu::CreateSurfaceError`) instead
  of flattened strings — no public error type in the workspace
  discards a cause into prose now. The dead convenience APIs go:
  `PassPipeline::has_queued_input`, `run_hot_window_with_channel`, and
  the `HotView`/`ViewRoot` `registry()` accessors had zero call sites.
  The living documents re-agree: the architecture's ADR count (now 25
  with the register's new row), the migration guides' snapshot counts
  (33 styled scenarios / 35 PNGs), the journal's materialization-shape
  rustdoc (the bench consumes `SceneBatches`, the backends walk the
  snapshot), the core pass-order rustdoc (the ten-pass contract), and
  the README's `counter_view` attribution (the example reconciles in
  place; it carries no keys)
- **M17 — the demonstrated queue (ADR-0023)**: the continuation of the
  consumable-surface work. The M8 coalescer — the one core vocabulary
  with a published budget and no runnable demonstration — gets its
  consumer demonstration: `examples/input_storm.rs` drives the
  frame-batched input queue from the consumer side, a custom
  `StormPanel` probe (a `Widget` with its required role and label)
  records every routed pointer event, *Read the log* mirrors the log
  into the window through the one-batch route, and the observability
  rides the workspace's idiom (`tracing` spans at the `alacer::input`
  target, the routed count as a field); the deterministic fifty-move
  storm is pinned by the m1 corpus's new `input_storm` scenario (the
  corpus grows 31 → 32 scenarios, 33 → 34 PNGs; the bench corpus stays
  53, the storm's hot path is already contracted). The living documents
  re-agree with the checked-in data: the README now quotes the
  published baseline's ratios — 14.06× for the W1 incremental-vs-full
  gate and 40.7× for the input-storm batching budget, which the
  M13–M15 corpus re-publishes had moved while the prose kept the
  M9-era 14.76× and 40.9× — the migration guides map onto Alacer 0.12.0
  instead of 0.5, and the facade's rustdoc describes the shipped
  surface instead of the M0 dependency spine. The error-chain
  discipline reaches the two seams M16 did not: `Recording::save` and
  `Recording::load` hand their `serde_json` cause to the
  `io::Error` chain (a malformed `.alc` — a bug report, by ADR-10's
  definition — now walks to the real parse error; the schema-mismatch
  leaf keeps its honest both-schemas message), and `LayoutError` — the
  public `LayoutEngine` contract's never-constructed error type —
  carries the failing node as data (`node: Option<NodeId>`) instead of
  the doc's promise that widget ids live in prose
- **M16 — the consumable contract (ADR-0022)**: the consumer surface the
  editing series (M11–M15) grew faster than it could be consumed. The
  0.12.0 cut completes: the twelve internal dependency pins in
  `[workspace.dependencies]` and the lockfile ride
  `[workspace.package]`'s version (the version-bump commit had left them
  at 0.11.0, and `cargo` refused to resolve the half-bumped workspace),
  and the README's badge follows the workspace. The error boundary
  carries its chain: `PlatformError` and `BackendError` implement
  `Error::source` (the `EventLoopError`, `OsError`, `RequestDeviceError`,
  and `wgpu::Error` inside them stay reachable instead of being
  formatted away), and the no-adapter message names the escape hatch it
  knows (`ALACER_SOFTWARE=1` for the softbuffer path). The widgets
  crate's manifest description and module rustdoc describe the shipped
  vocabulary (label, button, text input, checkbox, column, row, spacer,
  list, panel, table, and the M6 canons) instead of the M0 set. The M15
  journal gets its consumer demonstration: `examples/undo_redo.rs`
  drives `EditHistory` directly — the pure-append coalescing rule, the
  selection each step returns, the app-write reset, the honest refusal,
  `undo_depth`/`can_redo` live in the window, and one `tracing` span per
  route at the `alacer::editing` target — and a blessed snapshot pins
  the demonstration (the m1 corpus grows 32 → 33 PNGs, the bench corpus
  stays 53; `tracing` joins the facade as a dev-dependency, so nothing
  published moves)
- **M15 — the undo stack joins the editing contract (ADR-0021)**: the
  value — scalar-addressable since M1, selection-managed since M11,
  pointer-edited since M12, word-edited since M13, clipboard-edited
  since M14 — is reversible. The `EditHistory` journal joins the editing
  rules beside `paste_text` and `word_hop` (`alacer-core/history.rs`):
  pure UTF-16 text arithmetic, no widget dialect — each step carries the
  replaced span, the removed and inserted text, and the `(anchor, caret)`
  selection on both sides, so an undo restores not just the text but the
  range the user was extending when they made the edit. The coalescing
  rule is the pure-append rule (a typed run undoes as one step; a
  deletion, a word chord, a cut, a paste, an IME commit are their own),
  a whole-value replacement — the view's reconciliation, the AT's
  `SetValue` — resets the journal (the app's write is never undone), and
  the journal is bounded at 100 steps with the oldest evicted FIFO (a
  cache with an eviction policy and tests that prove it, §14.3). The
  editor routes every committed edit through one journaling funnel and
  answers Ctrl/Cmd+Z, Shift+Ctrl/Cmd+Z, and the Ctrl/Cmd+Y alternate
  with the M11 selection rules: an exhausted or reset journal is
  consumed with no masks — an idle frame — and the chords are inert
  while a preedit composes, the a11y delta riding unchanged
- **M15 — the harness and the corpus**: the corpus grows 52 → 53 by the
  coordinated rule — the new `editing/route_100_undo_redo_1k` criterion
  bench (100 undo/redo chords routed through the pipeline on a
  1,000-node arena, measured on the shipped `TextInput`) is published
  with a same-host regression gate, and the M5–M14 pin suites move
  together with the new M15 exit suite
- **M14 — the clipboard joins the editing contract (ADR-0020)**: the
  selection — scalar-addressable since M1, keyboard-extended and
  AT-managed since M11, pointer-placed since M12, word-addressable since
  M13 — is transferable. The clipboard seam joins the assembly seams
  (`ClipboardSeam`: `read_text`/`write_text`, installed once per thread
  like the text seam) with no provided implementation on principle: the
  OS clipboard is a platform fact, and a widget-internal buffer would
  silently break the round trip every other application shares — the
  engine rides arboard (the adopted core the architecture already
  ratified), owned by `alacer-platform` per the adapter-dependency rule,
  opened lazily so a headless session answers reads with `None` and
  refuses writes instead of panicking. The permission/escape story is
  the seam's contract: a refused write is never munged into success, so
  a cut whose write was refused does not delete, and a refused chord —
  a copy without a selection, a paste from an empty clipboard — is
  consumed but costs an idle frame. Core fixes the single-line paste
  rule (`paste_text`: every line break — CR, LF, or a CRLF pair counted
  once — pastes as exactly one space; one definition, no widget
  dialect). The editor consumes it: Ctrl/Cmd+C copies the selection's
  exact UTF-16 span and edits nothing, Ctrl/Cmd+X moves it out, Ctrl/
  Cmd+V inserts the sanitized text consuming an active selection first —
  an edit like any other, riding the same `PAINT | A11Y` currency, inert
  while a preedit composes — and the a11y tree rides unchanged
- **M14 — the harness and the corpus**: `TestHarness::set_clipboard` /
  `clipboard_text` drive the seam the tests' in-memory store answers —
  a headless CI never races the operator's actual OS clipboard; the
  corpus grows 51 → 52 by the coordinated rule — the new
  `editing/route_100_clipboard_ops_1k` criterion bench (100
  select-copy-paste round trips routed through the pipeline on a
  1,000-node arena, measured on the shipped `TextInput`) is published
  with a same-host regression gate, and the M5–M13 pin suites move
  together with the new M14 exit suite
- **M13 — the word joins the editing contract (ADR-0019)**: the caret —
  scalar-addressable since M1, keyboard-extended since M11, pointer-placed
  since M12 — is word-addressable. The text seam grows the required
  `word_segments` — the run's UAX #29 word segmentation in the §10.3
  UTF-16 contract unit, with no provided implementation on principle: a
  whitespace-and-ASCII approximation would silently mis-hop `don't` and
  every multi-script run, so the contract requires the engine's tables
  (ICU's `WordSegmenter`, the same segmenter parley's line breaking
  adopts, dictionary-backed for Thai and friends). Core fixes the hop
  rule (`word_hop`) — a hop lands on the far edge of the word-like
  segment that spans the caret, or on the near edge of the next one;
  separators are never landings, so `", "` takes one hop; the edges
  clamp. The editor consumes it: Alt/Ctrl/Meta+Left/Right hop word-wise
  (Shift extends from the anchor, a plain press collapses a live
  selection first — the GTK rules the arrows already follow),
  Ctrl/Alt+Backspace and Delete delete exactly the span the caret would
  hop (a selection is consumed whole first, the M11 edit rule), and a
  double-click selects the word under the hit — anchor at its start,
  caret at its end — collapsing on a separator rather than wandering to
  the nearest word. The event contract carries the gesture fact the
  widget cannot time: `PointerEvent::Down` grows `click_count`, computed
  by the platform's click chain (400 ms, 4 px — GTK's defaults), and the
  a11y tree rides unchanged — a word selection is the same anchor+caret
  vocabulary every writer shares
- **M13 — the harness and the corpus**: `TestHarness::double_click`
  drives the gesture the way the platform's click chain reports it; the
  corpus grows 50 → 51 by the coordinated rule — the new
  `editing/route_100_word_moves_1k` criterion bench (100
  direction-alternating word hops routed through the pipeline on a
  1,000-node arena, measured on the shipped `TextInput`) is published
  with a same-host regression gate, and the M5–M12 pin suites move
  together with the new M13 exit suite
- **M12 — the pointer joins the editing contract (ADR-0018)**: the
  caret — until now keyboard- and AT-addressable only — is
  pointer-addressable. A click inside the field places the caret at the
  hit point (edges clamp, ties break to the cluster's start), a
  press-drag extends the selection from the M11 anchor — the drag is the
  anchor state's second writer, keyboard and pointer alike — and a
  release ends the gesture while the range survives as a state; an
  unpressed move is hover and never moves the caret; the pre-M12
  press-collapses-at-the-stale-caret fallback is deleted. The seam that
  makes it honest is two-sided: the pass-1 router hands every pointer
  handler the node's resolved layout frame (`EventCtx::bounds()`, fed
  from the arena before the capture walk), and the text seam grows the
  bounds-aware offset mapping (`TextSeam::hit_test`) — the caret and the
  highlight's own prefix widths, inverted, so a click maps onto what is
  drawn; during a composition the committed text is what offsets map to
  (the preedit is overlay spliced at the caret, not a click target)
- **M12 — the harness and the corpus**: `TestHarness::pointer_press` /
  `pointer_move` / `pointer_release` / `drag` drive the gesture the way
  a user's hand and the recordings deliver it; the corpus grows 49 → 50
  by the coordinated rule — the new `editing/route_100_caret_clicks_1k`
  criterion bench (100 queued caret placements routed through the
  pipeline on a 1,000-node arena, measured on the shipped `TextInput`)
  is published with a same-host regression gate, and the
  M5/M6/M7/M8/M9/M10/M11 pin suites move together with the new M12 exit
  suite
- **M11 — the UTF-16 selection model joins the editing and a11y contracts (ADR-0017)**:
  the text input grows the anchor+caret selection in the §10.3 contract
  unit — Shift+arrows/Home/End extend from the anchor, Ctrl/Cmd+A selects
  all, a plain arrow collapses to the near edge first (GTK semantics), and
  every edit consumes the selection: typing replaces it, Backspace/Delete
  clear the whole range, a click collapses it, and an IME commit is an
  insertion like any other; the highlight paints from the prefix widths
  the text seam already measures; the focused input exposes the range as
  tree vocabulary (`Widget::selection()` → `TreeNode.selection` →
  AccessKit `TextSelection`), dropping the report while a preedit composes;
  and the AT value tier joins the faithful mapping — `SetValue`,
  `ReplaceSelectedText`, and `SetTextSelection` carry their payload or are
  dropped at the boundary, a refused value action costs an idle frame (a
  button refuses the value tier, an input refuses the press tier), and the
  pre-M11 bug that typed an `a` on Ctrl+A is deleted
- **M11 — the harness and the corpus**: `TestHarness::selection(role,
  name)` reads the range exactly as the AT sees it (normalized UTF-16
  offsets from the a11y snapshot); the corpus grows 48 → 49 by the
  coordinated rule — the new `a11y_action/route_100_setvalue_1k` criterion
  bench (100 payload-carrying `SetValue` actions routed through pass 10 on
  a 1,000-node arena, worst-case target) is published with a same-host
  regression gate, and the M5/M6/M7/M8/M9/M10 pin suites move together with
  the new M11 exit suite
- **M10 — the a11y action tier joins the dispatch contract (ADR-0016)**:
  the platform maps every AccessKit `ActionRequest` faithfully instead of
  forcing it to `DefaultPress` — `Click`, `Increment`, `Decrement`, and
  `ScrollUp`/`ScrollDown` (with the request's `ScrollUnit`) route through
  pass 10, and an action a widget does not implement is refused with no
  masks and no effects, never munged into a press; the `List` answers the
  scroll/step vocabulary translation-only (an item is the 40 px line, a
  page is the viewport, `Increment`/`Decrement` step by a page, clamped
  like the wheel path), and a focused `Button` is keyboard-operable —
  Space/Enter fire one click per press-and-release (GTK release
  semantics, auto-repeat absorbed)
- **M10 — the harness and the corpus**: `TestHarness::act(role, name,
  action)` drives the full action tier through the a11y query API and
  returns the consumption verdict (refusals are assertable); the corpus
  grows 47 → 48 by the coordinated rule — the new
  `a11y_action/route_100_actions_1k` criterion bench (100 mixed actions
  routed through pass 10 on a 1,000-node arena) is published with a
  same-host regression gate, and the M5/M6/M7/M8/M9 pin suites move
  together with the new M10 exit suite
- **M10 — housekeeping**: the CI perf-contracts bench invocation pins
  `--benches` so criterion-only flags never reach a libtest harness; the
  `arena_masks/10000` baseline entry was re-published after the same-host
  checker flagged ~30% host drift that reproduces identically at the M9
  tree (measured by stashing the working tree)
- **M9 — the windowed a11y path joins the incremental contract (ADR-0015)**:
  the pipeline's own `AccessKitSink` becomes the only `TreeUpdate`
  materialization path — the real window drains the sink's dirty-only
  deltas at every frame boundary and answers an adapter-activation request
  (`InitialTreeRequested`) with the sink's own full-tree build, so the
  pre-M9 per-frame full rebuild (and its dropped M6 vocabulary: values,
  toggle state, grid slots) is gone; toggle state and table geometry now
  reach the OS adapter, and the sink's produced-update buffer is ring-capped
  instead of growing without bound in windowed runs
- **M9 — the a11y contract as a number**: the criterion pair
  `a11y/full_rebuild_1k` (the pre-M9 shape) vs
  `a11y/incremental_1k_dirty_10pct` (the M9 shape), published with two
  regression contracts and the `a11y_incremental_speedup` budget (full ÷
  incremental ≥ 2.0×); the corpus grows 45 → 47 by the coordinated rule,
  with the M5/M6/M7/M8 pin suites moving together and the new M9 suite
  asserting all four
- **M9 — the exit suite** (`alacer-testing/tests/m9_exit_criteria.rs`):
  the first frame drains the init tree and quiet frames drain nothing; a
  toggle reaches the window as a dirty-only update carrying its `Toggled`
  state; text values ride the delta; removals surface through the sink's
  removal record and the resolved tree shrinks; the activation response
  equals a fresh sink's init build byte for byte; the ratio is a published
  budget
- **M8 — frame-batched input (ADR-0014)**: the event coalescer moves into
  `alacer-core` (`alacer_core::input::EventCoalescer`) and becomes a field
  of the `PassPipeline` — the platform *queues* pointer events and never
  drains; `tick` drains the coalesced batch at its start, so a storm of
  fifty moves costs one coalesced routing, one event pass, one frame (the
  same one-frame currency signal writes already obey, ADR-2/ADR-6)
- **M8 — the batching contract as a number**: the criterion pair
  `input/storm_50_moves_queued` (7.32 µs — 50 events, one tick) vs
  `input/storm_50_moves_direct` (299.21 µs — the pre-M8 shape), published
  with two regression contracts and the `input_storm_batching` budget
  (direct ÷ queued ≥ 2.0×, measured 40.9×); the corpus grows 43 → 45 by
  the coordinated rule, with the M5/M6/M7 pin suites moving together and
  the new M8 suite asserting all three
- **M8 — the exit suite** (`alacer-testing/tests/m8_exit_criteria.rs`): a
  fifty-move storm routes exactly one coalesced event (probe widget, event
  counter, and frame ledger agree); a ten-tick wheel burst lands as one
  accumulated delta; a direction flip splits the run; horizontal wheel
  deltas reach widgets; the list scrolls through the queue with zero
  measure calls; the platform source pins push-don't-drain
- **M8 — the cast seam completes**: `f64_to_f32`, `f32_to_usize`, and the
  `usize_to_u8`/`usize_to_u16` rounding helpers join `alacer_core::num`,
  the workspace's single documented rounding/casting point
- **M8 — the harness can storm too**: `TestHarness::queue_pointer` exposes
  the frame-batched input queue to headless suites

### Changed

- **The workspace lint regime now actually applies.** The
  `[workspace.lints]` table (missing_docs deny, unsafe_code forbid, clippy
  pedantic) was never inherited — no member crate opted in — so the
  "pedantic denied workspace-wide, zero allow-list" claim was inert. Every
  member crate now carries `[lints] workspace = true`, pedantic fires
  (`warn` + the CI `-D warnings`), and the ~90 latent pedantic violations
  the activation surfaced are fixed or carried by reasoned `#[expect]`s —
  still zero `#[allow]` anywhere
- The platform's wheel mapping carries both axes (the pre-M8 mapping
  dropped `x`); `MouseScrollDelta` → pixel vector is a pure, unit-tested
  mapping (`wheel_delta`)
- The platform's per-callback drain block and its redundant double
  `take_pending_masks` are deleted — input routing decisions live in the
  pipeline, the present decision in the damage ladder
- The baselines are re-published on the same host class (fingerprint
  `ee37c3efc5995352`, dirty tree): the W1 budget re-measures at 14.76×;
  the committed cross-framework comparison is untouched (ADR-0013's rig of
  record)
- `docs/architecture.md` §18 gains the M6/M7 roadmap rows (the README
  claimed them; the table had drifted) and the M8 row; §10.2 notes the
  delivered wiring
- **M9 — the a11y seam drops the split-brain**: `alacer-platform` deletes
  its snapshot-rebuilding `TreeUpdateBuilder` and the `tree_dirty` flag;
  `alacer-core` gains `A11ySink::as_any_mut` (the `Widget::as_any_mut`
  pattern — the trait stays adapter-free) and `PassPipeline::a11y_sink_mut`;
  `alacer-a11y` factors one `build_update` behind `drain_updates` and the
  pure `full_tree_update`; the producer-less `PlatformEvent::TreeChanged`
  variant becomes the embedder's flush nudge it was named for
- The baselines are re-published for M9 (corpus 47, one collecting host):
  the `a11y_incremental_speedup` ratio joins the two existing ratio budgets
  as the host-independent gates
- `docs/architecture.md` gains the M9 roadmap row (ADR-0015) and the
  ADR-0012…0015 register rows; §4.2's phantom `alacer-hot`/`alacer-devtools`
  rows become the real `alacer-macro`/`alacer-inspect` and the facade row
  names the hot-reload runtime it actually hosts

### Fixed

- **`ALACER_SOFTWARE=1` is the platform's own**: the env the README's
  prerequisites, quickstart, and tip promised honored itself at startup —
  `AlacerApp::new` runs the caller's preference through the pure
  `BackendPreference::software_env` parser (the env is passed, never read,
  so tests drive every shape without mutating process state), a `Wgpu`
  preference the env overrides presents through softbuffer, a
  `Softbuffer` preference is already the env's answer, and a value other
  than `1` is refused honestly — a `tracing::warn!` on
  `alacer::platform` naming the value it saw and the one it accepts,
  never a guess (the M11 rule at the platform seam). Until now only the
  examples' per-example `choose_backend` helpers read the env, so a
  consumer app had to re-implement the switch the docs described as the
  platform's.

## [0.17.0] — 2026-11-10



### Added


- The walk's answer becomes data


## [0.16.0] — 2026-11-10



### Added


- The focus walk bounds to a subtree

- The focus trap drives the consumer's window

- The directional walk reads the window

- The directional walk drives the consumer's window


### Changed


- The dead fixed-size spec goes

- The dead color conversion goes

- The dead selection predicate goes

- The dead color conversion goes


### Documentation


- The living documents re-agree at M25 and 0.16.0

- The living documents re-agree at M26 and 0.16.0


### Fixed


- The m7 release-state pin re-joins the chain


### Performance


- The focus routes join the published corpus


### Tests


- The m25 exit suite pins the focus scope

- The m26 exit suite pins the directional walk

- The m-suite pins move with the 60-case corpus


## [0.15.0] — 2026-11-10



### Added


- The focus walk joins the dispatch contract

- The focus walk drives the consumer's window


### Documentation


- The living documents re-agree at M24


### Miscellaneous


- The dead effect counter goes

- The harness's dead probes go


### Tests


- The m24 exit suite pins the focus walk


## [0.14.0] — 2026-11-10



### Added


- The availability tier joins the widget contract

- The sink maps the disabled flag faithfully

- The actionable controls answer the availability tier

- The availability tier drives the consumer's window

- The journal reports its own ledger

- The frame's ledger drives the consumer's window


### Changed


- The root duplicate goes

- The dead glyph counter goes


### Documentation


- The living documents re-agree at M22

- The living documents re-agree at M23 and 0.14.0


### Fixed


- The fall-through diagnostic names the token

- The body-cardinality diagnostic names its cardinality

- The accept loop reports the session errors it swallowed


### Miscellaneous


- The availability case joins the blessed corpus

- The manifest describes the shipped vocabulary

- The handoff manifest rides the gitignore


### Performance


- The refused route joins the published corpus


### Tests


- The m22 exit suite pins the availability tier

- The m-suite pins move with the 57-case corpus

- The m23 exit suite pins the ledger and the scroll trio


## [0.7.0] — 2026-11-08

### Added

- **M7 — the reproducible benchmark rig (ADR-0013)**:
  `scripts/collect_rig.py` writes `benchmarks/rig.json` — the host identity
  (the same sha256-truncated fingerprint basis the baseline uses, so a rig
  and a baseline cross-reference), the toolchain, the measurement protocol
  (5 process runs per case, median-of-medians, criterion sample size and
  window), the backend mode (headless CPU; vsync labeled
  not-applicable-headless rather than omitted), and the pinned dependency
  versions of every measured framework, read from the lockfiles
- **M7 — the competitor harnesses** (`benchmarks/competitors/`): two
  standalone cargo projects — `egui_workloads` (egui 0.37, the named
  immediate-mode DX baseline, driven headlessly through `Context::run_ui` +
  `Context::tessellate`) and `masonry_workloads` (Masonry 0.4, the audit's
  retained pass-contract reference, driven through the public `RenderRoot`
  API with the accessibility tree enabled) — each implementing the §14.2
  beat workloads W1/W2/W3 idiomatically, with selfcheck tests pinning what
  a bench frame claims (the full 10k walk, the re-wrapping paragraphs, the
  1k paint ops) and a README carrying the no-strawman provenance table
  (the framework APIs used, and the virtualization answers — egui's
  `show_rows`, Masonry's `Portal` — deliberately not taken, documented)
- **M7 — the comparison runner** (`scripts/run_comparison.py`): builds the
  competitor projects, runs each case five times, reads the Alacer side
  from the published baseline (same-host by fingerprint, or re-measured
  with `--remeasure-alacer`), and publishes
  `benchmarks/comparisons/results.json` — schema 1, one labeled entry per
  (workload, framework) with median, run medians, scope note, and pinned
  version, plus the same-host beat verdict; `--validate` checks the
  committed artifacts cheaply (the mode the exit suite and CI call)
- **M7 — the first published comparison**: collected on the declared rig
  (fingerprint `ee37c3efc5995352`, same host as the published baseline):
  W1 — incremental relayout of the 10k-node churn frame — measures
  **1.10 ms against egui's 20.7 ms full immediate frame (18.8×) and
  Masonry's 4.6 ms full pass pipeline (4.2×): the beat holds**; W2 and W3
  are published as measured, including the rows where the competitors win
  (Masonry's 0.62 ms re-wrap pipeline beats Alacer's 1.00 ms incremental
  text relayout; both competitors beat Alacer's 23.5 ms W3 frame, whose row
  includes the damaged software raster the competitors' rows exclude —
  the scope labels say so); every row conservative against the thesis per
  ADR-0013 §3
- **M7 — the publication surface**: `benchmarks/contracts.json` grows the
  `comparison_corpus_published` test-enforced gate (enforced by the M7 exit
  suite); the perf dashboard grows the cross-framework comparison section —
  per-workload medians with scope labels, the beat badges, and the advisory
  note when the collecting rig differs from the baseline's
- **M7 — the exit suite** (`alacer-testing/tests/m7_exit_criteria.rs`): the
  rig is declared and referenced by the results; the competitor corpus
  covers {egui, masonry} × {W1, W2, W3} as standalone, pinned,
  no-strawman projects; the runner validates the committed artifacts; the
  beat is computable and same-host with every measurement scoped; the
  contract gate and the dashboard publish the comparison; the M5/M6 corpus
  pins do not move (43 workspace cases — M7 adds competitor data outside
  the corpus, never inside it); the ADR names the harness pieces
- ADR-0013: the M7 scope decision record

### Changed

- The workspace ships as 0.7.0
- The README status reflects M0–M7 and carries the measured beat in the
  features list; the perf-features bullet says 43 criterion cases (it had
  drifted back to 38)
- The `regressions` corpus stays at 43 gates: competitor absolute medians
  are deliberately not regression-gated (ADR-0011's same-host rule —
  absolute gates on silicon the project does not own are benchmark
  theatre)

## [0.6.0] — 2026-11-07

### Added

- **M6 — the widget canon completes the v1 definition of done
  (ADR-0012)**: the `Checkbox` (role `CheckBox`, click / Space / AccessKit
  default action all toggle, the checked state travels the a11y tree as the
  neutral `ToggleState` and lands as AccessKit's `Toggled` property) and the
  `Table` / `TableRow` / `TableCell` / `ColumnHeader` family (roles mapped
  onto AccessKit's `Table` / `Row` / `Cell` / `ColumnHeader`; the neutral
  `TableSlot` carries the grid geometry, so the AccessKit update publishes
  row/column counts and zero-based cell positions — a *navigable* grid, not
  just a pile of roles); `Key::Space` joins the key vocabulary (winit's
  `NamedKey::Space` maps at the platform seam) and a consumed a11y action
  now invalidates through the one currency (`PAINT | A11Y` on the target —
  A1)
- **M6 — the canon apps as shared builders** (`alacer-widgets::canon`):
  `todo_app` (the §5 shape — input submits on Enter via the new
  `TextInput::on_submit`, rows toggle and delete through the a11y tree, the
  footer binding counts what is left) and `data_table_app` (toolbar +
  "Add row" seam + positioned cells that renumber on removal); one
  implementation, two consumers — the `todo` and `data_table` examples
  assemble them behind a window, the M6 exit suite drives them headlessly
  through the a11y queries; `TextInput::on_submit` is the Enter-to-commit
  seam the canon needs
- **M6 — the view vocabulary grows the typed handler props**:
  `PropValue::TextHandler` (`on_submit`, `on_changed` — the event carries
  the committed text) and `PropValue::FlagHandler` (`on_toggle` — the event
  carries the widget's new state), with `ElementProps::text_handler` /
  `flag_handler` accessors; the default registry registers the canon
  elements (`checkbox`, `text_input`, `table`, `table_row`, `table_cell`,
  `column_header`)
- **M6 — the shared workloads W2/W3/W5 as measured, gated benches**:
  W2 text-dense relayout (`alacer-layout/benches/w2_text.rs` — 1k
  paragraphs under ±5% width jitter, incremental + full denominator), W3
  animation loop (`alacer-testing/benches/w3_animation.rs` — 1k nodes,
  translation advance + 10% paint churn through the *real* pipeline and the
  damaged software raster), W5 signal-graph churn
  (`alacer-core/benches/w5_signal_graph.rs` — 10k signals, 1k memos,
  batched writes → one rerun → mask drain); the corpus grows 38 → 43 (the
  M5 and M6 suites pin the same number), every new case carries its
  contracts.json regression gate, the baseline was republished by
  `scripts/collect_baseline.py`, and the dashboard regenerated
- **M6 — the integer wrap rounding, adopted as ALGO** (§8.3): the layout
  host floors text wrap widths to whole device pixels before they reach the
  measurer or the cache key — sub-pixel container jitter cannot flip a line
  break; the no-re-wrap-oscillation gate is test-enforced
  (`alacer-layout/tests/w2_gate.rs`: integrality, sub-pixel stability, and
  cache-hit-on-revisit) and published on the dashboard as
  `w2_no_rewrap_oscillation`
- **M6 — record/replay** (`alacer-testing::recording`, §13.4): the `.alc`
  format (versioned JSON; semantic steps — `press` by role + name, typed
  text, keys, wheel, click — so recordings survive layout drift), the
  `Session` recorder (record + execute in one pass), `Recording::replay` /
  `replay_with` (the per-step hook is where apps with explicit re-render
  seams plug in), and the committed canonical recording
  (`crates/alacer-testing/recordings/todo.alc`, re-blessed with
  `ALACER_RECORD=1 cargo test -p alacer-testing`) replayed green by the
  exit suite — bug reports that are also regression tests
- **M6 — the exit suite** (`alacer-testing/tests/m6_exit_criteria.rs`):
  the checkbox toggles through every input path with the state reaching the
  a11y snapshot; the data table is reachable, named, and positioned; the
  canon apps work end-to-end; the committed recording replays green; the
  workload corpus is published and contracted at 43; the W5 scheduling
  contract holds at graph scale (one write batch ⇒ one effect rerun, memos
  never glitch); the gallery exercises the canon widgets; the M5 pin agrees
- ADR-0012: the M6 scope decision record

### Changed

- The workspace ships as 0.6.0
- The M6 canon builders (`todo_app`, `data_table_app`) take a `WakeUp`
  trigger (the patch-channel fire) so windowed apps re-render with the same
  one-frame contract the exit suite drives headlessly; headless callers
  pass `None` and call `rerender` explicitly
- The M5 corpus pin moves 38 → 43 and the "five benching crates" language
  becomes six (the harness crate now benches W3)
- The perf-contract corpus on the dashboard: 43 baselines, 6 test-enforced
  gates (the W2 no-oscillation gate and the W5 one-rerun gate join the four
  M5 gates)
- The gallery exercises the canon widgets (checkbox + data table tour
  sections)
- The examples catalog lists the canon examples with their contracts

## [0.5.0] — 2026-11-06

### Added

- **M5 — criterion baselines published as contracts (ADR-0011)**: all 38
  criterion benchmark cases across the five benching crates distilled into
  `benchmarks/baselines/baselines.json` by `scripts/collect_baseline.py` —
  median/mean/MAD/sample count per case plus the environment that produced
  them (git commit, dirty flag, toolchain, host fingerprint); the
  architecture's falsifiable gates published as measured **budget
  contracts** (W1 ≥ 2× incremental relayout, measured 14.01×) and one
  **same-host regression gate** per case (fresh median ≤ published × 1.25
  on a matching host fingerprint; foreign hosts skip by policy,
  `--strict-host` for pinned hardware); the uncontracted-benchmark rule
  fails any bench case without a published gate
- **M5 — the perf-contract checker** (`scripts/check_perf_contracts.py`):
  enforces the budgets everywhere (ratios are host-independent), the
  same-host regressions on matching fingerprints, and the contracts ↔
  baseline bijection; exit codes for CI
- **M5 — the perf dashboard** (`scripts/perf_dashboard.py` →
  `benchmarks/dashboard/index.html`): one self-contained HTML file — the
  measured environment, budget contracts with ratios and PASS/FAIL, the
  full baseline table with delta bars, and the four test-enforced gates
  (pass overhead < 15%, TreeUpdate p99 < 0.5 ms, the patch frame budget,
  zero-measure scrolling) each naming its enforcement site; CI
  regenerates and uploads it on main
- **M5 — the examples gallery**: `crates/alacer/examples/gallery.rs` — the
  whole core widget vocabulary (labels, buttons, text input, panels,
  scrolling list, rows, spacers, the shared theme) in one window through
  the canonical write seam (signals in, masks out, one batch one frame);
  `docs/examples.md` catalogs every example with its contract and the
  bless workflow
- **M5 — the egui/Iced migration guides** (`docs/migration/from-egui.md`,
  `from-iced.md`): the two audited reference frameworks mapped
  section-for-section (mental model, app shell, widget mapping,
  reactivity, layout, input and events, accessibility, testing, interop
  limits) with real API on both sides and the version-scope honesty note
- ADR-0011: the M5 scope decision record
- The M5 exit suite (`alacer-testing/tests/m5_exit_criteria.rs`): the
  baseline schema + environment header, the contracts ↔ baseline bijection
  at the pinned corpus size (38), the budgets holding on the published
  data (including the architecture's own 2× W1 value), dashboard
  coverage, catalog coverage, the gallery exercising the widget
  vocabulary through `Flux::batch_of` + `RenderBinding`, and both guides
  carrying the required section set

### Changed

- The workspace ships as 0.5.0
- CI gains the `perf-contracts` job: benches run at reduced sampling, the
  budget contracts are enforced against the published baseline, and the
  dashboard is regenerated and uploaded as an artifact
- `alacer-testing` gains a `serde_json` dev-dependency (the M5 exit suite
  reads the published contracts and baseline)

## [0.4.0] — 2026-11-06

### Added

- **M3 — the a11y contract**: the a11y pass produces `A11yDelta`s (updated /
  removed / focus / initializes); AccessKit `TreeUpdate`s stay dirty-node-only,
  removals surface exactly once, focus travels in the update, editables expose
  `value()`; the §18 gate — incremental TreeUpdate p99 < 0.5 ms @ 1k nodes —
  is test-enforced
- **M3 — partial paint**: the damage model (`alacer-core::damage`, ≤3 regions,
  least-growth merging, NONE/partial/FULL ladder) computed by the paint pass
  from old journal extents + new primitive extents, carried in
  `FrameReport::damage`; software backend repaints regions onto a persistent
  framebuffer, wgpu backend `LoadOp::Load`s an offscreen frame with scissor
  rects; the equivalence fuzz pins partial == full pixel-exactly (Q8)
- **M3 — the IME contract**: UTF-16-native key/IME event vocabulary, the
  `utf16` arithmetic seam (mid-surrogate offsets floor, never slice),
  focus-routed `dispatch_key`/`dispatch_ime` with Tab cycling, per-frame
  `ImeRequest` (purpose + caret anchor), the winit byte→UTF-16 platform seam,
  and the `TextInput` reference widget (caret editing, inline preedit, commit,
  secure mode, focus ring, a11y value)
- `TextInput` widget + the `text_input` example
- ADR-0009: the M3 scope decision record
- **M4 — the hot-reload contract (§13.2, ADR-7)**: `alacer-view::HotView`,
  the view-generation runtime — a patch swaps the generation, the
  reconciler re-renders scoped (patch-in-place, keyed LIS, exact
  differences), and state survives by construction; the facade's
  `hot-reload` feature links `subsecond`, wraps generations in
  `subsecond::call` (the jump-table detour point) and registers the patch
  handler; the platform's `PatchChannel` wakes the winit loop from any
  thread and answers one patch event with exactly one scoped re-render
  (`PlatformEvent::Patch` + `AlacerApp::set_patch_hook`)
- **M4 — the CI hot-patch suite**: `alacer-testing/tests/m4_exit_criteria.rs`
  pins the framework's half of the contract headlessly — a patched
  generation renders its new values with the old state (signal, binding,
  node identity), a patch costs exactly one layout+paint cycle and leaves
  the tree idle (A7), ten-generation churn with interleaved mutations stays
  consistent, untouched subtrees stay out of the patch frame, and the
  unobserved path stays silent (capture off, zero recording); the facade's
  `hot_reload_subsecond` tests pin the subsecond passthrough headless, and
  the `hot_reload` example is the sub-second demo (`dx serve --hotpatch`
  upgrades it to real binary patching, no code change)
- **M4 — the Causality inspector (§13.3)**: the `alacer-core::inspect` seam
  (`InspectEvent`: edge activations, provenance writes, routed inputs,
  frame summaries with the pass ledger + the dirty-node mask heatmap) and
  the `alacer-inspect` crate — `CausalRecorder` over bounded ring buffers
  (every ring's eviction is test-proven, §14.3), a dependency-light RFC
  6455 WebSocket server (`sha1` + `base64`, no async runtime), the
  documented WS protocol (`docs/inspector-protocol.md`, version 1), and the
  embedded single-file HTML client; the five §13.3 questions — why did
  this render, who wrote this signal, who subscribes, why is this frame
  slow, what did this mask touch — answered from recorded truth;
  `Flux::write` / `FluxWriter` is the opt-in write-provenance path;
  `Flux::set_causal_capture` gates activation capture; the protocol suite
  (`alacer-inspect/tests/protocol.rs`) pins the handshake (the RFC 6455
  reference vector), the greeting shape, streaming order, and the controls;
  the `inspector_demo` example is the live demo
- ADR-0010: the M4 scope decision record

### Changed

- `A11ySink::apply` takes a shared `Arc<TreeSnapshot>` (built once per a11y
  run — no per-frame deep clone) and an `&A11yDelta`
- `SceneJournal` gains `layer(node)` — the damage pass reads the layer it is
  about to replace
- Pass 3 notifies focus endpoints (`Widget::set_focused`) and merges their
  PAINT masks same-tick; the tick drains all masks pass 3 merged
- `Widget` gains `on_key_event`, `on_ime_event`, `set_focused`, `ime_purpose`,
  `value` (all defaulted)
- Buttons take keyboard focus on press (click-to-focus, §10.3)
- The workspace is 12 crates (ADR-9's target): `alacer-inspect` joins as
  the devtools layer (`alacer-core` only; `scripts/check_dag.py` enforces
  the order; `alacer-testing` may consume it for the M4 suite)
- `PassPipeline` gains `set_observer` / `clear_observer` — the devtools
  sink; `None` in every default path (the A7 idle contract and the M1
  overhead gate are unchanged and re-verified)
- The workspace ships as 0.4.0 with the full crates.io metadata set on
  every manifest

## [0.3.0] — 2026-11-05



### Added


- Add the Change mask invalidation currency

- Add the single cast seam

- Add shared euclid-backed geometry types

- Add the RGBA color type

- Add id-path prefix hashing

- Add the pointer event vocabulary

- Add typed layout styles and the measure seam

- Add the Widget trait with required role and label

- Add the retained widget arena

- Add the flux bridge over reactive_graph

- Add the scene primitive vocabulary and PaintCtx

- Add the pass contract and PassPipeline

- Wire the core crate public surface

- Add keyed reconciliation

- Add the Alacer-to-Taffy style conversion

- Add the LRU measure cache

- Add torin-style dirty-reason semantics

- Add the persistent Taffy LayoutHost

- Wire the layout crate public surface

- Add the frame-windowed layout cache

- Add the font registry

- Add the zeno glyph rasterizer

- Add the parley facade and thread-local context

- Add the node-layered scene journal

- Add the tiny-skia software rasterizer

- Wire the render crate public surface

- Add the AccessKit sink with incremental TreeUpdates

- Add the Panel decoration

- Add Column, Row and Spacer containers

- Add the Label widget

- Add the Button widget

- Add the scrolling List

- Wire the widget family, theme and BoxedWidget

- Add the event coalescer

- Add the softbuffer present path

- Add the wgpu renderer

- Add the AlacerApp winit driver

- Add the headless TestHarness

- Add the alacer facade crate

- Add u32_to_f32, u64_to_f32 and u128_to_u64 helpers

- Add the EventPhase capture/bubble enum

- Add Widget::focusable and the as_any_mut escape hatch

- Add the FluxRwSignal type alias

- Add the text seam (widgets shape text without alacer-text)

- Add reorder_children, is_live, and drain reporting

- Implement the ten-pass contract with the overhead gate

- Wire text_seam and re-export the M1 pass vocabulary

- Add the ThreadLocalTextSeam adapter

- Add the ViewNode tree and PropValue vocabulary

- Add the WidgetRegistry element table

- Add the ViewRoot in-place reconciler

- Wire the view tree, registry and reconciler modules

- Add the rsx! view DSL proc-macro

- Add in-place setters, as_any_mut and the text seam

- Add style_mut, as_any_mut and Spacer::set_size

- Add in-place setters, as_any_mut and the text seam

- Add style_mut and as_any_mut

- Add set_height, set_color and as_any_mut

- Register the default M0 element set

- Expose the default element registry

- Install the text seam at AlacerApp assembly

- Add the a11y query builder, press, and the text seam

- Expose rsx! and mount_view through the facade

- The root candidate, the bounded publish, and the sync diff

- The cached scene replay


### Benchmarks


- Add arena build/mask/walk benchmarks

- Add flux batch-coalescing and drain benchmarks

- Add reconciliation benchmarks

- Add the incremental vs full relayout benchmark

- Add cold/cached text layout benchmarks

- Add journal commit and replay benchmarks

- Add the §7.2 pass-overhead churn benchmark


### Build


- Scaffold the Alacer workspace

- Pin the initial dependency graph

- Cargo-deny license gate (no AGPL, no unknown licenses)

- Declare the alacer-core crate

- Declare the alacer-view crate

- Declare the alacer-layout crate

- Declare the alacer-text crate

- Declare the alacer-render crate

- Declare the alacer-a11y crate

- Declare the alacer-widgets crate

- Declare the alacer-platform crate

- Declare the alacer-testing crate

- Declare the alacer facade crate

- Register the pass_overhead benchmark target

- Declare the alacer-macro proc-macro crate

- Tighten to core + view (the M1 DAG close-out)

- Add alacer-macro and alacer-view deps

- Add alacer-macro to the facade s dependencies


### Continuous Integration


- Add the crate layering DAG checker

- Add the fmt/clippy/test/doc/deny/dag pipeline

- Add the git-cliff release automation

- Extend the layering for alacer-macro and the M1 tightening

- Ship the counter_view example binary


### Documentation


- Add the counter example

- Add the scrolling list example

- Add the git-cliff configuration

- Add the initial CHANGELOG

- Add the CONTRIBUTING guide

- Add the system architecture document

- Record the M0 scope decision

- Record the workspace discipline decision

- Record the scheduling contract decision

- Record the persistent-Taffy decision

- Record the software-first glyph-seam decision

- Record the testing productization decision

- Record the release automation decision

- Add the counter_view rsx! example

- Record the M1 scope decision

- Add the 0.2.0 release notes

- Update README for the M1 milestone

- Record the M2 scope decision

- The M2 milestone record and the 187-test count


### Fixed


- Treat dead arena slots as not-live during taffy sync

- Repair the broken snapshot corpus — glyph blend, y-convention, stretch, compose


### Miscellaneous


- Ignore build artifacts and editor/OS debris

- Pin the 1.99.1 toolchain with clippy and rustfmt

- Set rustfmt max_width to 100

- Add the Apache-2.0 license text

- Add the MIT license text

- Regenerate the lockfile for the 0.2.0 workspace

- Bless broken pngs

- Readme

- Logo & deny


### Tests


- Add the counter snapshot golden

- Add the scrolled-list snapshot golden

- Add the M0 exit-criteria integration tests

- Implement as_any_mut on the arena bench Node

- Implement as_any_mut on the incremental test Probe

- Implement as_any_mut on the incremental bench TestBox

- Add the M1 exit criteria and 30-scenario corpus

- Add the rsx! end-to-end integration suite

<!-- generated by git-cliff -->
