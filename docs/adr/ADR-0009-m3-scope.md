# ADR-0009: M3 scope — the a11y contract, partial paint, and the IME contract

- **Status:** Accepted (2026-11-06)
- **Implements:** the roadmap's M3 bullet (architecture §18): "A11y contract +
  a11y-driven queries + partial paint on GPU path + IME contract"
- **Companion:** ADR-0005 (a11y), ADR-0008 (M2 scope — the replay cache this
  builds on)

## Decision

M3 makes the two remaining consumers of the change mask *incremental* and
*bounded*, and gives keyboard input its contract.

### 1. The a11y contract: `A11yDelta` (§11.1)

Pass 9's product is now an explicit **delta**, not a bare dirty set:

- `updated` — the id-path hashes whose a11y entry changed this frame (dirty
  nodes only; a clean subtree costs nothing — §11.1 item 4);
- `removed` — ids present last a11y frame, gone now (dead widgets surface
  exactly once; the pipeline diffs the previous id set against the current
  one, O(n) per structural frame only);
- `focus` — the focused node's id-path hash; a focus move runs the pass even
  with no other a11y work (platform focus lives in the update, not a node);
- `initializes` — first frame or structural rebuild: adapters treat the
  update as the full forest.

The `A11ySink` trait receives the snapshot **shared** (`Arc<TreeSnapshot>`):
the pass builds it once, the sink keeps it for queries — no per-frame deep
clone of the tree, so the budget measures TreeUpdate production, not
bookkeeping. Editable widgets expose their content: `Widget::value()` flows
into the neutral `TreeNode.value` and out as AccessKit's `set_value`.

**The exit gate is a test:** `alacer-testing/tests/m3_exit_criteria.rs`
measures incremental `TreeUpdate` production at 1k nodes; **p99 < 0.5 ms**
(the §18 M3 gate; measured at ~a few µs — the dirty-node-only discipline is
the whole point, and the test also asserts incremental cost is a small
fraction of the one-time init build).

### 2. Partial paint: damage with an escape hatch (§9.3, Q4/Q8)

`alacer-core::damage` ports the documented ≤3-region budget with least-growth
merging, clean-room (Slint's code is GPL; only the ALGO travels). The paint
pass computes every repainted node's **old journal extent** (the layer it is
about to replace) and **new primitive extent** (the layer it just built) —
geometry changes damage both rects — and merges them into at most three
regions clamped to the viewport. The ladder:

- nothing painted → `Damage::NONE` (the present is skipped entirely);
- structural/first frames → `Damage::FULL` (the journal was cleared);
- otherwise → the merged regions, **falling back to FULL** when the regions
  would cover ≥ 75% of the viewport or the budget overflows catastrophically.

Correctness details the implementation pins:

- An **empty old layer erases nothing** — containers have empty layers by
  construction; guessing from bounds would damage the whole viewport on every
  descendant change (measured, then fixed exactly this way).
- Primitive extents are **conservative**: quads inflate by half the border
  stroke + AA margin; glyphs span a generous em-box because bitmap metrics
  belong to the font and the pipeline must not read them.
- `Damage` rides `FrameReport::damage` to both backends. The **software**
  path keeps a persistent framebuffer: each region is cleared and the
  primitives intersecting it are redrawn scissored to the region. The **GPU**
  path renders into an offscreen frame texture, `LoadOp::Load`s it, scissor
  rects each region with a synthetic background quad as the erase step, and
  blits to the swapchain (`wgpu::util::TextureBlitter`) — identical damage
  semantics, per Q8.
- **The equivalence contract is a fuzz** (`m3_partial_frames_equal_full_frames_through_the_real_pipeline`
  in the exit suite + the renderer-level LCG fuzz): over a real interaction
  script — clicks, typing, CJK IME composition, Tab, surrogate-pair commits —
  the accumulated partial framebuffer must be **pixel-equal** to a fresh full
  render after every step. Drift is a bug, not a tolerance.

### 3. The IME contract: UTF-16 end to end (§10.3)

- The event vocabulary (`Key`, `KeyEvent`, `Modifiers`, `ImeEvent`) and the
  UTF-16 arithmetic (`alacer_core::utf16`) are core-owned; every caret offset
  and preedit cursor counts **UTF-16 code units**. Offsets that land inside a
  surrogate pair **floor left** — never sliced, never out of range.
- Routing is **focus-scoped**: `dispatch_key`/`dispatch_ime` reach the
  focused widget only. `Tab`/`Shift+Tab` cycle focus when the widget ignores
  them (pre-order candidates, wrapping). A widget asking for `Change::FOCUS`
  through `EventCtx` is the click-to-focus seam.
- Pass 3 notifies both focus endpoints (`Widget::set_focused`) and merges
  their PAINT masks same-tick (focus rings do not lag a frame); the tick
  drains all masks pass 3 merged — the currency coalesces per frame.
- The **request half**: after layout, the pipeline derives an `ImeRequest`
  (`node`, `ImePurpose`, caret region) from the focused widget's
  `ime_purpose()`. The platform applies the delta frame over frame:
  `set_ime_allowed` on purpose changes, `set_ime_cursor_area` on caret moves.
- The **platform seam** converts winit's byte-wise preedit cursor into the
  contract's UTF-16 units (`alacer_platform::keys::ime_cursor_utf16`) — the
  fcitx5/ibus boundary. The conversion floors mid-rune byte offsets, so a
  buggy backend cannot slice a scalar value.
- `TextInput` is the reference widget: caret editing (chars, backspace,
  delete, arrows walking scalar boundaries, Home/End), inline preedit with
  the IME's own cursor, commit-at-caret, cancel on empty preedit or blur,
  keys inert while composing, `ImePurpose::Text`/`Secure`, value exposed to
  the a11y tree.

The "fcitx5/ibus CI matrix green" criterion is honored at the level this
repository can enforce headlessly: the UTF-16 contract, the byte→UTF-16
conversion, and the full preedit→commit/cancel sequences are
pipeline-exercised in the M3 suite; the platform adapter glue is
compile-verified (A4's software-first discipline — no display server in CI).

### 4. Everything else that moved

- `SceneJournal` gains `layer(node)` — the damage pass reads the layer it is
  about to replace.
- `remove_subtree` semantics unchanged; a11y removals ride the id-set diff.
- winit keyboard/IME events route through `alacer-platform::keys`;
  `ModifiersChanged` feeds a tracked modifier snapshot (winit does not
  attach modifiers to key events).
- The softbuffer present path keeps a persistent framebuffer sized to the
  window; resize invalidates it.

## Consequences

- The a11y cost is now provably bounded by the dirty set, not the tree — the
  §18 kill criterion ("a11y cost breaks frame budget → lazy-subtree a11y
  updates only") has not fired: p99 at 1k nodes is µs-scale.
- Partial paint is opt-out by construction only in its fallbacks; every
  partial frame is verified pixel-equal to the full path it replaces.
- The IME contract has one unit of measure everywhere; the platform seam is
  the only place bytes exist.
- Next (M4): subsecond hot reload + the Causality inspector; the a11y and
  damage machinery here is what the inspector will visualize.
