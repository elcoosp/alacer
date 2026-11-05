# ADR-0008: M2 scope — the root candidate, the bounded publish, and the replay cache

- **Status:** Accepted (2026-11-05)
- **Supersedes:** the roadmap's M2 bullet (architecture §18) for the M1 codebase
- **Companion:** ADR-0003 (persistent Taffy), ADR-0007 (M1 scope)

## Decision

M2 turns the invalidation currency into *bounded* work. Four pieces:

1. **The root candidate (§8.2 item 3, torin's `measure()` semantics).** On a
   non-structural layout frame, the engine computes the **closest common
   ancestor** of the dirty set and recomputes only that subtree, feeding
   taffy the candidate's *cached size* as its definite available space. If
   the candidate's size changes (content grew into an auto-sized ancestor),
   the candidate walks **up** one level and recomputes — torin's
   dependants walk, expressed as an upward loop bounded by tree depth. The
   chain stops at the fill root, where the viewport is the constraint.
   Structural frames and the first frame compute from the fill root as
   before. Verified empirically over taffy 0.15: a mid-tree compute leaves
   ancestors' cached layouts untouched, recomputes the subtree, and **zeros
   the compute root's location** — the publisher must use the candidate's
   *cached* parent-relative location, never its post-compute one.

2. **The bounded publish.** `publish_bounds` walks only the recomputed
   region, seeded from the candidate's parent's absolute bounds (valid:
   the parent is outside the region, so it did not move) plus the cached
   location. Full publish stays for structural frames. Bounds outside the
   region stay valid by construction — that is what makes them skippable.

3. **Structural sync pays for diffs, not for the tree.** `sync_tree` diffs
   each container's current child order against its cached one and calls
   taffy's `set_children` only on real changes — the old unconditional
   re-set dirtied the entire persistent tree every frame, silently
   defeating taffy's caches. Liveness is one pre-order walk (was: a
   per-node ancestor walk, O(n·depth)).

4. **The replay path is cached (§9.1).** `FrameReport.scene` becomes an
   `Arc<SceneSnapshot>`: the paint pass materializes once, idle and
   replay-only frames share the allocation (§14.3's warm path). The paint
   order is cached between structural changes (pre-order is a pure
   function of tree shape), and the materialized snapshot is rebuilt only
   when a layer commits.

**The exit gate is a test:** the W1 workload (10k-node build, 10%/frame
style churn, §14.2) as `benches/w1_churn.rs` plus an executable gate —
median incremental relayout must be ≥ 2× faster than a full relayout of
the same final tree (the §18 kill criterion; 5× is the M2 target, 2× the
design floor).

## Consequences

- Bounds coherence becomes region-local: the engine guarantees
  *incremental relayout ≡ full relayout* through the §8.4 oracle fuzz,
  now mirroring the churn onto the pristine tree so the comparison is of
  identical final trees.
- The a11y pass still walks the whole tree per run (M3's contract — its
  incremental update path lands with the a11y work); the W1 gate measures
  the layout engine, not the frame.
- `FrameReport.scene` is `Arc<SceneSnapshot>`: consumers get a shared
  read-only scene; the harness and renderers keep their `&`-based APIs.
- The candidate loop is bounded: at most tree-depth recomputes per frame,
  each strictly smaller than the last full compute.
