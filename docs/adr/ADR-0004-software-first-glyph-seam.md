# ADR-0004: One scene journal; glyph coverage bitmaps are the raster seam

- **Status:** Accepted (2026-11-05)
- **Extends:** the architecture's ADR-2 and A8

## Decision

`GlyphBitmap` (coverage + placement) and the `GlyphSource` trait live in
`alacer-core`. `alacer-text` owns the single rasterizer (swash scaled
outlines through zeno's mask renderer); the wgpu backend uploads the same
bitmaps to its R8 atlas, the software backend blends them directly. There
is no second rasterization path.

## Consequences

- `alacer-render` stays free of swash.
- The atlas eviction policy for M0 is **full clear on overflow** —
  documented, instrumented (`atlas_clears`); LRU arrives with M3 per §9.4.
- The M0 GPU pipeline draws quads as rounded-rect SDFs with border rings;
  instanced, one draw call, 64-byte instance stride.
