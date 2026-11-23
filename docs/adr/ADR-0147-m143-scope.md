# ADR-0147: M143 scope — the landmark tier: the page-structure words the AT walks a document by

- **Status:** Accepted (2026-11-23)
- **Implements:** the vocabulary hole the image tier (ADR-0142) left
  beside the page: through M142 the snapshot carried the widget
  vocabulary — buttons, fields, strips, panes, pictures — and the app's
  *page furniture* had no words. An app wiring a header, a nav strip, a
  content region and a footer reached for bare panels wearing no
  container word (the region the AT cannot name is a region it cannot
  offer on demand, the ADR-5 rule the named roles carry) — or for a
  [`Toolbar`] wearing the wrong word (a toolbar *offers tools*, the M129
  strip the controls live in; a landmark *names the document's own
  structure*). The AT's landmarks list — the document-wide travel a
  screen reader offers over a page's banner, navigation, main region and
  footer — read nothing the tree said. The tier ships the four
  page-structure words: `Role::Banner`, `Role::Navigation`,
  `Role::Main` and `Role::ContentInfo`
  (crates/alacer-core/src/widget.rs), said by the tree, mapped to
  AccessKit's own roles one branch beside the image's
  (crates/alacer-a11y/src/lib.rs).
- **Companion:** ADR-0142 (the image tier — the named-container shape's
  newest member and the map's one-branch precedent), ADR-0128 (the
  member-and-container discipline the words ride), ADR-0133 (the
  toolbar — the strip the landmark refuses to wear), ADR-0099 (the
  required-name set the roles join), ADR-0031 (the focus walk the
  furniture never becomes a stop of), architecture §11.1 (the tier
  table), §17 (the register), §18 (the roadmap row)

## Decision

- **One shape, four facts.** The tier is one widget — `Landmark`
  (crates/alacer-widgets/src/landmark.rs) — with four constructor words
  (`Landmark::banner`, `Landmark::navigation`, `Landmark::main`,
  `Landmark::content_info`), the role the constructor's own
  declaration. Four roles are not four widgets: a banner, a nav strip, a
  main region and a footer are the same layout — a named vertical run —
  wearing four different facts, and the type says so (the M129 toolbar's
  one-strip shape, at the page tier). The registry doors parse four
  elements (`banner`, `navigation`, `main`, `content_info`) sharing one
  patch door — the word the one write that reports the move (the idle
  -frame property, at the landmark tier).
- **The furniture is the children's layout, never a second channel.**
  The children ride the arena exactly a pane's do (ADR-0135's
  named-container rule), each keeping the role and label it already
  carries — the banner's label is still a label, the nav's links are
  still links. A landmark is not a control: it holds no focus and
  answers no activation (the M126/M129 container rule, at the landmark
  tier) — the controls the app mounts inside it hold the focus, and the
  Tab walk never lands on the furniture.
- **The words join the required-name set, never the focusable set.** An
  unnamed landmark reads as a bare box — the set the AT cannot name is a
  set it cannot walk (ADR-5), so `Role::requires_name` grows the four;
  the furniture never takes the walk's stop, so `focusable` stays the
  default refusal. The CI walker's diagnostics name the words
  (`banner`, `navigation`, `main`, `content_info` — `Role::name()`'s own
  list).
- **The words price the words and nothing else.** The tier grows no
  snapshot route: the named-container shape the M124/M127/M129/M131/
  M134/M135/M136 containers rode needs no new a11y machinery — the sink
  maps the four roles one branch beside the image's, and the properties
  the tiers already published (the name, the description) carry the
  rest. The word-write price rides the widgets crate's own bench
  (`landmark/name_write_256` against `landmark/idle_write_256`),
  *outside* the published corpus — the coordinated rule grows the
  baseline at the next published cut, never at a widget bench's (the
  m136 pin's own shape, the m143 suite's pin documenting it again).

## Consequences

- **The landmarks list is the tree's.** A screen reader's
  document-wide travel now reads a page built on the vocabulary — the
  banner first, the way through, the content, the colophon last — the
  facts AccessKit's own roles spell, carried by the same tree the rest
  of the snapshot rides. The m143 exit criteria pin the words, the
  not-a-control contract, and the DSL doors against the real pipeline.
- **The vocabulary's growth is constructor words, not machinery.** The
  landmark tier's remaining ARIA facts (`complementary`, `form`,
  `region`, `search`) join as constructors and registry doors beside the
  four — the named-container shape unchanged, the map growing one branch
  per fact, never a second widget per fact.
- **Kill-switch:** a consumer needs landmarks to bound the focus walk
  (the modal trap's shape at the page tier) → grow the walk beside the
  M25 scope's own contract (`focus_move_in`'s rule), priced by the
  harness — never a second boundary; a consumer needs landmark-aware
  reading order (the AT's landmarks list reaching the wire) → grow the
  snapshot beside the inspector's vocabulary, priced by a real reader —
  never a second tree (see `docs/adr/ADR-0147-m143-scope.md`).
