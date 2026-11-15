# ADR-0080: M74 scope — the OS setting seam

- **Status:** Accepted (2026-11-15)
- **Implements:** the paint tier's setting half — the OS's dark-mode
  fact existed nowhere in the pipeline through M73: a widget that
  wanted it guessed or polled, and "follow the OS" meant rebuilding
  the palette by hand — the fact installs as one
  [`SettingSeam`](crates/alacer-core/src/settings.rs)
  (`PassPipeline::set_settings`), the way the palette installs as one
  `ThemeSeam` (ADR-0079) and the clipboard is a seam (ADR-0020), never
  a poll loop the widget vocabulary guesses at
- **Companion:** ADR-0079 (the theme seam this copy rides — the
  pre-registered re-open rule this cut cashes), ADR-0009 (the adapter
  the platform owns), ADR-0001 (A1 — the one invalidation currency the
  broadcast rides), ADR-0011 (the coordinated rule the corpus grows
  by), ADR-0010 (the additive wire rule), architecture §7.2 pass 8,
  §18 (the M74 roadmap row)

## Decision

- **The fact is one [`SettingSeam`](crates/alacer-core/src/settings.rs)
  the pipeline owns.** The trait answers the OS's dark-mode fact —
  total, per frame, `Send + Sync` like `ThemeSeam` — and nothing else:
  a setting that cannot answer the fact is a setting that cannot
  consult it. [`CanonSettings`](crates/alacer-core/src/settings.rs) is
  the provided implementation: light, always — the fact the blessed
  snapshot corpus was blessed with. A default build (nothing
  installed) resolves light, so an OS flip never silently re-paints an
  untouched app.
- **The adapter is the platform's (ADR-0009).**
  [`WinitSettings`](crates/alacer-platform/src/lib.rs) maps the
  window's reported `Theme` onto the fact — the opening `theme()` and
  every `ThemeChanged` event push the fact through the door; a
  platform that reports no fact installs nothing (the honest absence —
  the canon light resolves), never a guess. The app installs its own
  OS palette when it wants palette-following; the adapter's job is the
  fact alone.
- **The broadcast is the currency's own rule.**
  [`PassPipeline::set_settings`](crates/alacer-core/src/passes.rs)
  broadcasts `Change::PAINT` on every node — the mask's own documented
  semantics ("a color change repaints this node only", for every node)
  — never a private repaint channel (A1). A flip that lands where the
  fact already is answers `false` and queues nothing: no broadcast, no
  record, no facts — the canon light installed where nothing is
  installed *is* that no-op (the default build already resolves
  light). The fact never munges the palette: the roles the widgets
  borrowed re-resolve from the *installed palette* exactly as they
  were — an app that wants palette-following swaps its palette through
  the theme door it already owns.
- **The fact resolves beside the palette, once per pass.**
  [`PaintCtx`](crates/alacer-core/src/scene.rs) carries the resolved
  `dark` fact — one dynamic call per frame beside the palette's own,
  never per node — and the vocabulary consults it for the decisions no
  palette role answers (an inset shadow's direction, a thumbnail's
  border weight). The setting is color-blind by construction: it
  carries the fact, never colors — the palette remains the one paint
  vocabulary (ADR-0079's shape).
- **The flip answers on the causal log.** One
  [`SettingRecord`](crates/alacer-core/src/inspect.rs) per flip that
  landed — the bool *is* the fact (the payload rule at the setting
  tier has nothing to strip) — the theme trail's shape: the note site
  is the door itself (public like `note_theme`, the bench corpus's
  pricing door), the queue is bounded (256 facts per thread,
  oldest-first eviction), the tick's start drain streams one frame
  after the door answered — the drain running just ahead of the theme
  trail's, so a frame's trail reads the fact the palette resolved
  beside → the palette that fact painted into — the disabled path
  costs one branch per flip (the A7 idle contract). The wire grows
  additively at version 1 (`setting` — the `dark` fact, ADR-0010); the
  embedded client renders the setting's row beside the theme's.
- **The demo drives the seam from the consumer side.**
  `examples/dark_mode.rs` walks the OS's beats through the adapter's
  door — the canon light builds, dark lands and repaints every node,
  the second dark ask is the honest no-op, light lands beside them — a
  fact probe painting from `ctx.dark()` directly, the *trail* label
  mirroring what the log streamed.
- **The corpus grows 76 → 79 by the coordinated rule.** The note
  site's own cost joins the corpus (`settings/note_disabled` — the A7
  contract's one disabled branch per note; `settings/note_enabled_bounded`
  — the enabled steady state's bounded push, the eviction priced) and
  the door's own idle cost beside it (`settings/door_no_op_flip` — the
  equality check the A1 rule prices) — bench, gate, and baseline in
  the same cut; the m-suite pins re-agree at 79.

## Re-open rules (pre-registered)

- a consumer needs more OS facts than dark mode (reduced motion, high
  contrast, accent color) → the trait grows the fact beside `dark_mode`
  (the same one dynamic call per pass, priced by a real reader) — never
  a second seam per fact.
- a consumer prices *which nodes* the flip repainted → the record grows
  the broadcast's node count beside the fact (the lengths rule — the
  ids never ride), priced by a real reader.
- a consumer needs the palette to *follow* the OS fact → the pipeline
  grows the composition door (the installed palette re-swapped on the
  flip's own record), priced by a real reader — never a palette that
  reaches into the platform's adapter.
- a consumer prices the settings seam's per-pass resolution on the
  overhead ledger → the resolution joins the pass-overhead corpus, the
  same coordinated rule this cut rode.
