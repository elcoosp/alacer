//! The randomized-scene fuzz rig (§9.3, Q8; §13.4's fuzz targets).
//!
//! The renderer's own unit tests and the deterministic
//! `partial_repaint_equivalence` suite prove the Q8 partial-repaint
//! equivalence contract for *hand-picked* scenes. This module is the
//! net-new capability that proves it **universally**: a
//! [`SceneGenerator`] producing random, valid [`SceneSnapshot`]s — quads
//! with varying sizes, colors, rounded corners, borders and clips;
//! glyphs served by a deterministic [`FuzzGlyphs`] source; premultiplied
//! raster images with random source windows, destinations and sampling —
//! plus random [`Damage`] regions guaranteed (by construction) to
//! intersect at least one primitive's extent.
//!
//! Randomness is a seeded LCG ([`Lcg`], the same construction the
//! incremental-layout tests in `alacer-layout` use) so every fuzz run is
//! reproducible from its seed — a failure is a regression test, not a
//! flake. [`FuzzGlyphs`] is *keyed*-deterministic: a glyph query returns
//! the same bitmap every time it is asked, which the partial path's
//! redraw depends on (the scissored pass re-queries only the primitives
//! reaching into the damaged region; their stamps must be identical to
//! the full pass's).
//!
//! The generation bounds are not arbitrary: [`FuzzGlyphs`] keeps every
//! bitmap inside the generous glyph box `scene::primitive_extent` wraps
//! around a glyph's baseline origin (±0.6 em horizontally, 1.1 em above,
//! 0.6 em below) — the documented contract that lets the pipeline
//! compute damage without font internals. That makes every generated
//! scene a *valid* scene per the pipeline's own damage math, so a
//! zero-diff equivalence failure is a renderer bug, not a generator bug.

use alacer_core::color::Color;
use alacer_core::damage::{Damage, DamageBuilder};
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::num::{f32_to_i32, usize_to_f32, usize_to_u8, usize_to_u32};
use alacer_core::scene::{
    Glyph, GlyphBitmap, GlyphSource, Image, ImageSampling, Primitive, Quad, SceneSnapshot,
    primitive_extent,
};
use std::sync::Arc;

/// A small, seeded linear congruential generator (the numerical-recipes
/// constants, as used by the `alacer-layout` host tests): the fuzz rig's
/// randomness source, reproducible from one `u64` seed.
///
/// Drawes are the high 31 bits of the advanced state — the low bits of
/// this modulus are weak, and a fuzz generator must not correlate its
/// rect coordinates.
#[derive(Debug, Clone)]
pub struct Lcg(u64);

impl Lcg {
    /// A generator seeded with `seed`.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// The next raw draw (high 31 bits of the advanced state).
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    /// The next uniform value in `[0, 1)` (24 mantissa bits — the honest
    /// float granularity).
    #[must_use]
    pub fn next_unit_f32(&mut self) -> f32 {
        let draw = usize::try_from(self.next_u64()).unwrap_or(0) % (1 << 24);
        usize_to_f32(draw) / usize_to_f32(1 << 24)
    }

    /// The next uniform value in `[lo, hi)`; `lo <= hi` is the caller's
    /// contract.
    #[must_use]
    pub fn next_f32_in(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next_unit_f32() * (hi - lo)
    }

    /// The next uniform value in `[lo, hi]` (both ends reachable).
    #[must_use]
    pub fn next_usize_in(&mut self, lo: usize, hi: usize) -> usize {
        let span = hi.saturating_sub(lo) + 1;
        lo + usize::try_from(self.next_u64()).unwrap_or(0) % span
    }

    /// The next byte.
    #[must_use]
    pub fn next_u8(&mut self) -> u8 {
        usize_to_u8(usize::try_from(self.next_u64()).unwrap_or(0) % 256)
    }

    /// Whether a `probability`-likely event fires.
    #[must_use]
    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_unit_f32() < probability
    }
}

/// The glyph source the fuzz scenes draw with: deterministic coverage
/// bitmaps derived from the `(glyph_id, size)` query key, kept inside the
/// metric box `scene::primitive_extent` documents (±0.6 em horizontally,
/// 1.1 em above the baseline, 0.6 em below) — the stamp never leaves the
/// box, so the pipeline's damage math holds for every glyph the rig
/// emits.
///
/// Every seventh glyph id is an honest empty outline (`None`): the
/// renderer draws nothing for it, exactly as it would for a space.
#[derive(Debug, Default)]
pub struct FuzzGlyphs;

impl FuzzGlyphs {
    /// A glyph source serving every fuzz scene's glyphs.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl GlyphSource for FuzzGlyphs {
    fn glyph_bitmap(&mut self, _font_id: u64, glyph_id: u32, size: f32) -> Option<GlyphBitmap> {
        if glyph_id.is_multiple_of(7) {
            return None;
        }
        let size = size.max(10.0);
        let mut rng = Lcg::new(
            u64::from(glyph_id)
                .wrapping_mul(0x9E_37_79_B9_7F_4A_7C_15)
                .wrapping_add(u64::from(size.to_bits())),
        );
        // Offsets and extents inside the documented glyph box (see the
        // module header), with slack for the baseline origin's integer
        // rounding: sizes stay ≥ 10 px, so the 0.05–0.1 em margins cover
        // the rounding error.
        let left = rng.next_f32_in(-0.5 * size, 0.3 * size);
        let top = rng.next_f32_in(-0.9 * size, -0.05 * size);
        let width = rng.next_usize_in(1, ceil_usize(1.45 * size - left));
        let height = rng.next_usize_in(1, ceil_usize(0.55 * size - top));
        let coverage: Vec<u8> = (0..width * height)
            .map(|_| if rng.chance(0.3) { 0 } else { rng.next_u8() })
            .collect();
        Some(GlyphBitmap {
            left: f32_to_i32(left),
            top: f32_to_i32(top),
            width: usize_to_u32(width),
            height: usize_to_u32(height),
            coverage: Arc::from(coverage),
        })
    }
}

/// Rounds a positive float up to a `usize` of at least 1 (the fuzz's
/// bitmap-dimension floors).
fn ceil_usize(value: f32) -> usize {
    usize::try_from(f32_to_i32(value.ceil()))
        .unwrap_or(1)
        .max(1)
}

/// A color with the rig's alpha discipline: fully opaque, or one of two
/// semi-transparent tiers — semi-transparent fills exercise the blending
/// a scissored redraw must reproduce exactly.
fn random_color(rng: &mut Lcg) -> Color {
    let alpha = match rng.next_usize_in(0, 2) {
        0 => 255_u8,
        1 => 168,
        _ => 96,
    };
    Color::new(rng.next_u8(), rng.next_u8(), rng.next_u8(), alpha)
}

/// Generates random, valid scenes and damage for the fuzz suites (§9.3,
/// §13.4): the equivalence fuzz's input side.
///
/// "Valid" is a contract, not a hope: every generated primitive lies in
/// or near `viewport`, every glyph stamp stays inside the documented
/// metric box, every image's bytes are honest premultiplied RGBA8 whose
/// channels never exceed their alpha, and every damage rect intersects
/// at least one primitive's extent — so the equivalence suites prove the
/// *renderer*, never a generator artifact.
#[derive(Debug)]
pub struct SceneGenerator {
    viewport: Size,
    rng: Lcg,
}

impl SceneGenerator {
    /// A generator for `viewport`-sized scenes, seeded with `seed`.
    #[must_use]
    pub const fn new(seed: u64, viewport: Size) -> Self {
        Self {
            viewport,
            rng: Lcg::new(seed),
        }
    }

    /// One random scene: 8–36 primitives in scene order, mixing quads
    /// (sharp and rounded, bordered and bare, clipped and free), glyphs,
    /// and raster images.
    #[must_use]
    pub fn generate_scene(&mut self) -> SceneSnapshot {
        let count = self.rng.next_usize_in(8, 36);
        let mut primitives = Vec::with_capacity(count);
        for _ in 0..count {
            match self.rng.next_usize_in(0, 99) {
                0..=54 => primitives.push(Primitive::Quad(self.random_quad())),
                55..=79 => primitives.push(Primitive::Glyph(self.random_glyph())),
                _ => primitives.push(Primitive::Image(self.random_image())),
            }
        }
        SceneSnapshot { primitives }
    }

    /// Random damage over `scene`: one to three rects, each guaranteed by
    /// construction to intersect some primitive's extent *inside the
    /// viewport* (the fuzz's "the damage reaches into painted pixels"
    /// discipline — regions are anchored on the visible part of the
    /// extent, so a region can never collapse to nothing when the
    /// builder's finish clamps it), merged through the real
    /// [`DamageBuilder`] — so the cover-ratio escape hatch fires exactly
    /// as it does in production when the random regions grow past 75% of
    /// the viewport.
    ///
    /// # Panics
    ///
    /// Never: the generator's scenes always carry in-viewport primitives
    /// (glyphs and images are mounted inside `viewport`), so the
    /// visible-extent pool is never empty for a generated scene.
    #[must_use]
    pub fn generate_damage(&mut self, scene: &SceneSnapshot) -> Damage {
        let mut builder = DamageBuilder::new();
        if scene.primitives.is_empty() {
            return Damage::NONE;
        }
        let viewport_rect = Rect::new(Point::zero(), self.viewport);
        let visible: Vec<usize> = scene
            .primitives
            .iter()
            .enumerate()
            .filter(|(_, primitive)| {
                primitive_extent(primitive)
                    .intersection(&viewport_rect)
                    .is_some()
            })
            .map(|(index, _)| index)
            .collect();
        let Some(last) = visible.len().checked_sub(1) else {
            return Damage::NONE;
        };
        let regions = self.rng.next_usize_in(1, 3);
        for _ in 0..regions {
            let index = visible[self.rng.next_usize_in(0, last)];
            let extent = primitive_extent(&scene.primitives[index]);
            let visible_extent = extent
                .intersection(&viewport_rect)
                .expect("the pool filtered for visible extents");
            builder.add(self.rect_overlapping(visible_extent));
        }
        builder.finish(self.viewport)
    }

    /// A quad with randomized geometry, fill, rounding, border, and clip:
    /// origins bleed past the viewport edges, rounded corners and
    /// centered borders stress the path raster, and a clip (when drawn)
    /// intersects the quad so the combined mask has real work.
    fn random_quad(&mut self) -> Quad {
        let origin = Point::new(
            self.rng.next_f32_in(-24.0, self.viewport.width - 16.0),
            self.rng.next_f32_in(-24.0, self.viewport.height - 16.0),
        );
        let size = Size::new(
            self.rng.next_f32_in(4.0, 96.0),
            self.rng.next_f32_in(4.0, 96.0),
        );
        let radius = if self.rng.chance(0.4) {
            size.width.min(size.height) * 0.45 * self.rng.next_unit_f32()
        } else {
            0.0
        };
        let (border_width, border_color) = if self.rng.chance(0.3) {
            (self.rng.next_f32_in(1.0, 4.0), random_color(&mut self.rng))
        } else {
            (0.0, Color::TRANSPARENT)
        };
        let clip = if self.rng.chance(0.25) {
            Some(self.clip_over(Rect::new(origin, size)))
        } else {
            None
        };
        Quad {
            rect: Rect::new(origin, size),
            color: random_color(&mut self.rng),
            corner_radius: radius,
            border_color,
            border_width,
            clip,
        }
    }

    /// A glyph inside the viewport: sizes stay ≥ 10 px so
    /// [`FuzzGlyphs`]'s metric-box bounds hold their rounding slack.
    fn random_glyph(&mut self) -> Glyph {
        Glyph {
            position: Point::new(
                self.rng.next_f32_in(8.0, self.viewport.width - 8.0),
                self.rng.next_f32_in(16.0, self.viewport.height - 4.0),
            ),
            size: self.rng.next_f32_in(10.0, 26.0),
            color: random_color(&mut self.rng),
            font_id: u64::from(u32::try_from(self.rng.next_usize_in(1, 2)).unwrap_or(1)),
            glyph_id: u32::try_from(self.rng.next_usize_in(0, 63)).unwrap_or(0),
            clip: None,
        }
    }

    /// A premultiplied raster: random bitmap, honest premultiplied bytes
    /// (channels never exceed alpha), a source window (full or partial),
    /// a destination that blits 1:1 or scales, both sampling modes, and
    /// an occasional clip. Destinations stay inside the viewport, so the
    /// scaled footprint never leaves the frame.
    fn random_image(&mut self) -> Image {
        let width = self.rng.next_usize_in(2, 20);
        let height = self.rng.next_usize_in(2, 20);
        let mut bytes = Vec::with_capacity(width * height * 4);
        for _ in 0..width * height {
            let alpha = self.rng.next_u8();
            // Premultiplied discipline: every channel at or under its own
            // alpha (a malformed pixel is refused by absence — the raster
            // tier's honest-refusal contract the fuzz must not trip).
            let ink = |rng: &mut Lcg| usize_to_u8(rng.next_usize_in(0, usize::from(alpha)));
            let (r, g, b) = if alpha == 0 {
                (0, 0, 0)
            } else {
                (ink(&mut self.rng), ink(&mut self.rng), ink(&mut self.rng))
            };
            bytes.extend([r, g, b, alpha]);
        }
        let bitmap_w = usize_to_f32(width);
        let bitmap_h = usize_to_f32(height);
        let src = if self.rng.chance(0.6) {
            Rect::new(Point::zero(), Size::new(bitmap_w, bitmap_h))
        } else {
            let x0 = self.rng.next_usize_in(0, width - 2);
            let y0 = self.rng.next_usize_in(0, height - 2);
            Rect::new(
                Point::new(usize_to_f32(x0), usize_to_f32(y0)),
                Size::new(
                    usize_to_f32(self.rng.next_usize_in(1, width - 1 - x0)),
                    usize_to_f32(self.rng.next_usize_in(1, height - 1 - y0)),
                ),
            )
        };
        let scale_x = self.one_to_one_or_scale();
        let scale_y = self.one_to_one_or_scale();
        let dst = Rect::new(
            Point::new(
                self.rng
                    .next_f32_in(0.0, self.viewport.width - src.size.width * scale_x),
                self.rng
                    .next_f32_in(0.0, self.viewport.height - src.size.height * scale_y),
            ),
            Size::new(src.size.width * scale_x, src.size.height * scale_y),
        );
        Image {
            bytes: Arc::from(bytes),
            width: usize_to_u32(width),
            height: usize_to_u32(height),
            src,
            dst,
            clip: if self.rng.chance(0.25) {
                Some(self.clip_over(dst))
            } else {
                None
            },
            sampling: if self.rng.chance(0.5) {
                ImageSampling::Nearest
            } else {
                ImageSampling::Bilinear
            },
        }
    }

    /// Either the 1:1 blit or a scale in `[1, 2.5]` (both image tiers).
    fn one_to_one_or_scale(&mut self) -> f32 {
        if self.rng.chance(0.5) {
            1.0
        } else {
            self.rng.next_f32_in(1.0, 2.5)
        }
    }

    /// A random rect guaranteed to intersect `area`: the origin jitters
    /// at most half the area's extent while the size keeps at least 30%
    /// of it, so both axes always overlap (the fuzz's by-construction
    /// discipline; the suites assert it independently).
    fn rect_overlapping(&mut self, area: Rect) -> Rect {
        let origin = Point::new(
            area.origin.x + self.rng.next_f32_in(-0.25, 0.5) * area.size.width,
            area.origin.y + self.rng.next_f32_in(-0.25, 0.5) * area.size.height,
        );
        let size = Size::new(
            self.rng.next_f32_in(0.3, 1.0) * area.size.width.max(2.0),
            self.rng.next_f32_in(0.3, 1.0) * area.size.height.max(2.0),
        );
        Rect::new(origin, size)
    }

    /// A random clip that intersects `area` (the same construction as
    /// [`Self::rect_overlapping`]) — a clip that never touched its
    /// primitive would not exercise the combined mask at all.
    fn clip_over(&mut self, area: Rect) -> Rect {
        self.rect_overlapping(area)
    }
}

/// The viewport the fuzz suites render into (small, deterministic,
/// headless).
pub const FUZZ_VIEWPORT: Size = Size::new(160.0, 120.0);

#[cfg(test)]
mod tests {
    use super::{FUZZ_VIEWPORT, FuzzGlyphs, Lcg, SceneGenerator};
    use alacer_core::num::{f32_to_i32, i32_to_f32, u32_to_f32};
    use alacer_core::scene::{GlyphSource, Primitive, primitive_extent};

    #[test]
    fn lcg_is_deterministic_per_seed() {
        let mut a = Lcg::new(42);
        let mut b = Lcg::new(42);
        for _ in 0..64 {
            assert_eq!(a.next_u64(), b.next_u64(), "same seed, same draws");
        }
        assert_ne!(
            Lcg::new(42).next_u64(),
            Lcg::new(43).next_u64(),
            "different seeds diverge"
        );
        let mut d = Lcg::new(42);
        let _ = d.next_u64();
        assert_ne!(Lcg::new(42).next_u64(), d.next_u64(), "the stream advances");
    }

    #[test]
    fn fuzz_glyphs_are_keyed_deterministic() {
        let mut first = FuzzGlyphs::new();
        let mut second = FuzzGlyphs::new();
        for glyph_id in 0..64_u32 {
            let a = first.glyph_bitmap(1, glyph_id, 14.0);
            let b = second.glyph_bitmap(1, glyph_id, 14.0);
            assert_eq!(a, b, "glyph {glyph_id} must be identical per query");
            if glyph_id.is_multiple_of(7) {
                assert!(a.is_none(), "every seventh glyph is an empty outline");
            }
        }
    }

    #[test]
    fn generated_scenes_are_valid_and_damage_intersects() {
        for seed in 0..64_u64 {
            let mut generator = SceneGenerator::new(seed, FUZZ_VIEWPORT);
            let scene = generator.generate_scene();
            assert!(
                (8..=36).contains(&scene.primitives.len()),
                "seed {seed}: scene size within the generator's contract"
            );
            let damage = generator.generate_damage(&scene);
            let regions = damage.regions(FUZZ_VIEWPORT);
            assert!(
                !regions.is_empty() || damage.is_full(),
                "seed {seed}: the generated damage repaints something"
            );
            for region in &regions {
                assert!(
                    scene
                        .primitives
                        .iter()
                        .any(|primitive| primitive_extent(primitive).intersects(region)),
                    "seed {seed}: every region reaches into some primitive's extent"
                );
            }
            // The glyph stamps must sit inside the documented metric box:
            // the fuzz's own validity discipline, checked at the source.
            for primitive in &scene.primitives {
                let Primitive::Glyph(glyph) = primitive else {
                    continue;
                };
                let mut glyphs = FuzzGlyphs::new();
                let Some(bitmap) = glyphs.glyph_bitmap(glyph.font_id, glyph.glyph_id, glyph.size)
                else {
                    continue;
                };
                let stamp = alacer_core::geom::Rect::new(
                    alacer_core::geom::Point::new(
                        i32_to_f32(f32_to_i32(glyph.position.x)) + i32_to_f32(bitmap.left),
                        i32_to_f32(f32_to_i32(glyph.position.y)) + i32_to_f32(bitmap.top),
                    ),
                    alacer_core::geom::Size::new(
                        u32_to_f32(bitmap.width),
                        u32_to_f32(bitmap.height),
                    ),
                );
                assert!(
                    primitive_extent(primitive).contains_rect(&stamp),
                    "seed {seed}: the glyph stamp stays inside the metric box"
                );
            }
        }
    }
}
