# ADR-0079: M73 scope — the runtime theme seam

- **Status:** Accepted (2026-11-15)
- **Implements:** the paint tier's own seam — the palette every widget
  borrowed for its un-overridden paint roles was ten compile-time
  constants on `alacer_widgets::theme`: "make it dark" meant rebuilding
  every widget, and no door answered the ask at all → the palette grows
  one installed implementation the paint pass resolves, the way the
  clipboard is a seam (ADR-0020) and text measurement is a seam
  (ADR-0009), never a second paint vocabulary or a private repaint
  channel
- **Companion:** ADR-0009 (the seam discipline this copy rides),
  ADR-0063 (the route-drop trail's shape the record copies),
  ADR-0011 (the coordinated rule the corpus grows by), ADR-0010 (the
  additive wire rule), ADR-0001 (A1 — the one invalidation currency the
  broadcast rides), architecture §7.2 pass 8, §13.3, §18 (the M73
  roadmap row)

## Decision

- **The palette is one [`ThemeSeam`](crates/alacer-core/src/theme.rs)
  the pipeline owns.** [`Palette`](crates/alacer-core/src/theme.rs)
  carries the nine paint roles as plain `Copy` data (surface, panel,
  accent, accent_pressed, selection, text, text_muted, border,
  on_accent), the trait answers the roles and a tag — total, per frame,
  `Send + Sync` like `MeasureFn` — and
  [`CanonPalette`](crates/alacer-core/src/theme.rs) is the provided
  implementation: the exact constants 0.16 shipped, installable by
  name, the values the blessed snapshot corpus was blessed with. A
  default build (nothing installed) resolves `Palette::CANON`, so a
  retheme never silently re-paints an untouched app.
- **The wire-up rides the paint pass, once per frame.**
  [`PaintCtx`](crates/alacer-core/src/scene.rs) carries the resolved
  palette; the paint pass resolves the installed seam once per pass —
  one dynamic call per frame, never per node — and every widget paints
  the roles its app did not override from the copy it holds. An
  app-set color — a builder prop, a `set_*` write — stays the override
  it was: the swap re-resolves the roles the widgets *borrowed*, never
  the props the app *set*.
- **The broadcast is the currency's own rule.**
  [`PassPipeline::set_theme`](crates/alacer-core/src/passes.rs)
  broadcasts `Change::PAINT` on every node — the mask's own documented
  semantics ("a color change repaints this node only", for every node)
  — never a private repaint channel (A1). A swap that lands where the
  palette already is answers `false` and queues nothing: no broadcast,
  no record, no facts (the idle-frame property, A1's rule at the theme
  tier — the value tier's write rule, priced where the write resolves).
- **The swap answers on the causal log.** One
  [`ThemeRecord`](crates/alacer-core/src/inspect.rs) per swap that
  landed — the palette's tag, never its colors (the payload rule's
  shape: the log names the palette, `PaintCtx` carries it) — the
  route-drop trail's shape: the note site is the door itself (public
  like `note_frame_refusal`, the bench corpus's pricing door), the
  queue is bounded (256 facts per thread, oldest-first eviction), the
  tick's start drain streams one frame after the door answered (the
  theme drain's arm after the view-patch drain: a frame's trail reads
  routes → tiers → the palette those routes painted into), the disabled
  path costs one branch per swap (the A7 idle contract). The wire grows
  additively at version 1 (`theme` — the `palette` tag, ADR-0010); the
  embedded client renders the palette's row beside the other trails.
- **Font metrics stay the canon constants.** The seam is
  paint-color-only: `FONT_SIZE`/`LINE_HEIGHT` feed measure (layout),
  whose contract has no palette hand — runtime metrics are a
  pre-registered re-open rule below, never a half-wired role this cut
  pretends to re-resolve.
- **The demo drives the seam from the consumer side.**
  `examples/theme_swap.rs` swaps palettes through the app's own door —
  the canon, a midnight palette, a high-contrast palette — the *trail*
  label mirroring what the log streamed, the status line reading the
  palette's tag back.
- **The corpus grows 73 → 76 by the coordinated rule.** The note
  site's own cost joins the corpus (`theme/note_disabled` — the A7
  contract's one disabled branch per note; `theme/note_enabled_bounded`
  — the enabled steady state's bounded push, the eviction priced) and
  the door's own idle cost beside it (`theme/door_no_op_swap` — the
  equality check the A1 rule prices) — bench, gate, and baseline in the
  same cut; the m-suite pins re-agree at 76.

## Re-open rules (pre-registered)

- a consumer needs runtime font metrics → the seam grows the metrics
  beside the roles (`measure_spec` grows the palette hand the layout
  pass resolves — the paint pass's shape, at the layout tier), priced
  by a real reader — never a half-wired role.
- a consumer prices *which nodes* the swap repainted → the record grows
  the broadcast's node count beside the tag (the lengths rule — the
  ids never ride), priced by a real reader.
- a consumer needs the OS's dark-mode fact → the platform grows the
  setting seam the palette reads (the adapter the platform owns,
  ADR-0009), never a poll loop the widget vocabulary guesses at.
- a consumer prices the panel element's absent `color` prop as a role →
  the registry grows the prop-absent arm behind a real reader (the
  registry default borrows the canon surface — pixel-identical today,
  priced when a consumer asks).
