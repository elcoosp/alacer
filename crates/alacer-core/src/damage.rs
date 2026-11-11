//! The damage model: partial paint with an escape hatch (§9.3).
//!
//! The union of the frame's changed paint rects is merged into **at most
//! three** [`Damage`] region boxes using **least-growth overflow merging**;
//! geometry changes contribute *both* the old and the new rect. When the
//! budget overflows — or the regions would cover so much of the viewport
//! that clearing is cheaper — the frame falls back to a full repaint.
//! Correctness first: partial paint is an optimization with a defined
//! fallback, never a risk.
//!
//! The algorithm is ported clean-room from *documented behavior* (the
//! Slint partial renderer's ≤3-rect budget and least-growth merge; its code
//! is GPL/commercial, so only the ALGO travels — `audit/LICENSES.md`
//! classifies it IDEA/ALGO-only).

use crate::geom::{Rect, Size};

/// The region budget: at most three boxes per frame (§9.3).
pub const DAMAGE_REGION_BUDGET: usize = 3;

/// When the merged regions cover at least this share of the viewport, the
/// full-frame path is cheaper than scissored draws and is chosen instead.
const FULL_FRAME_COVER_RATIO: f32 = 0.75;

/// One frame's paint damage: up to [`DAMAGE_REGION_BUDGET`] region boxes,
/// or the full-frame fallback.
#[derive(Debug, Clone, PartialEq)]
pub struct Damage {
    regions: [Option<Rect>; DAMAGE_REGION_BUDGET],
    full: bool,
}

impl Default for Damage {
    fn default() -> Self {
        Self::NONE
    }
}

impl Damage {
    /// No paint damage at all — the present can be skipped entirely.
    pub const NONE: Self = Self {
        regions: [None; DAMAGE_REGION_BUDGET],
        full: false,
    };

    /// The full-frame fallback: every pixel is repainted.
    pub const FULL: Self = Self {
        regions: [None; DAMAGE_REGION_BUDGET],
        full: true,
    };

    /// Whether the frame must be repainted in full (the escape hatch).
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.full
    }

    /// Whether nothing needs repainting (the present can be skipped).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.full && self.regions.iter().all(Option::is_none)
    }

    /// The merged region boxes, clamped to `viewport`, in merge order.
    #[must_use]
    pub fn regions(&self, viewport: Size) -> Vec<Rect> {
        if self.full {
            return vec![Rect::new(crate::geom::Point::zero(), viewport)];
        }
        self.regions
            .iter()
            .filter_map(|r| r.and_then(|rect| clamp_to_viewport(rect, viewport)))
            .collect()
    }
}

/// Clamps `rect` to `viewport`, dropping empty results.
fn clamp_to_viewport(rect: Rect, viewport: Size) -> Option<Rect> {
    let vp = Rect::new(crate::geom::Point::zero(), viewport);
    rect.intersection(&vp)
}

/// Accumulates changed paint rects and merges them into at most
/// [`DAMAGE_REGION_BUDGET`] boxes with least-growth merging (§9.3).
#[derive(Debug, Default)]
pub struct DamageBuilder {
    regions: [Option<Rect>; DAMAGE_REGION_BUDGET],
}

impl DamageBuilder {
    /// An empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one changed rect (empty rects are ignored; on geometry change
    /// callers add the *old* and the *new* rect as two adds).
    ///
    /// # Panics
    ///
    /// Never for well-formed input: the `expect` guards an invariant the
    /// function just established (the builder is full), not caller state.
    pub fn add(&mut self, rect: Rect) {
        if rect.size.width <= 0.0 || rect.size.height <= 0.0 {
            return;
        }
        if self.regions.iter().all(Option::is_none) {
            self.regions[0] = Some(rect);
            return;
        }
        // Full builder: merge into the pair with the least area growth.
        if let Some(slot) = self.regions.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(rect);
            return;
        }
        let mut best_index = 0_usize;
        let mut best_cost = f32::INFINITY;
        let mut best_merged = rect;
        for (index, slot) in self.regions.iter().enumerate() {
            let existing = slot.expect("checked full above");
            let merged = existing.union(&rect);
            let cost = merged.area() - existing.area();
            if cost < best_cost {
                best_cost = cost;
                best_index = index;
                best_merged = merged;
            }
        }
        self.regions[best_index] = Some(best_merged);
    }

    /// Adds every rect of `other`'s regions (damage accumulation helpers).
    pub fn extend(&mut self, rects: impl IntoIterator<Item = Rect>) {
        for rect in rects {
            self.add(rect);
        }
    }

    /// Finishes the merge: clamps to the viewport, applies the cover-ratio
    /// escape hatch, and returns the frame's [`Damage`].
    #[must_use]
    pub fn finish(&self, viewport: Size) -> Damage {
        if viewport.width <= 0.0 || viewport.height <= 0.0 {
            return Damage::FULL;
        }
        let mut regions = [None; DAMAGE_REGION_BUDGET];
        for (slot, rect) in regions.iter_mut().zip(self.regions.iter()) {
            *slot = rect.and_then(|r| clamp_to_viewport(r, viewport));
        }
        let live: Vec<Rect> = regions.iter().filter_map(|r| *r).collect();
        if live.is_empty() {
            return Damage::NONE;
        }
        let viewport_area = viewport.width * viewport.height;
        let covered: f32 = live.iter().map(Rect::area).sum();
        if viewport_area > 0.0 && covered / viewport_area >= FULL_FRAME_COVER_RATIO {
            return Damage::FULL;
        }
        Damage {
            regions,
            full: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DAMAGE_REGION_BUDGET, Damage, DamageBuilder};
    use crate::geom::{Point, Rect, Size};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(x, y), Size::new(w, h))
    }

    #[test]
    fn empty_builder_yields_no_damage() {
        let damage = DamageBuilder::new().finish(Size::new(100.0, 100.0));
        assert!(damage.is_empty());
        assert!(!damage.is_full());
        assert_eq!(damage.regions(Size::new(100.0, 100.0)), []);
    }

    #[test]
    fn empty_rects_are_ignored() {
        let mut builder = DamageBuilder::new();
        builder.add(rect(5.0, 5.0, 0.0, 10.0));
        builder.add(rect(5.0, 5.0, 10.0, 0.0));
        assert!(builder.finish(Size::new(100.0, 100.0)).is_empty());
    }

    #[test]
    fn one_rect_stays_one_region() {
        let mut builder = DamageBuilder::new();
        builder.add(rect(10.0, 10.0, 20.0, 30.0));
        let damage = builder.finish(Size::new(100.0, 100.0));
        assert_eq!(
            damage.regions(Size::new(100.0, 100.0)),
            vec![rect(10.0, 10.0, 20.0, 30.0)]
        );
    }

    #[test]
    fn disjoint_rects_fill_the_budget_before_merging() {
        let mut builder = DamageBuilder::new();
        for i in 0..DAMAGE_REGION_BUDGET {
            builder.add(rect(
                crate::num::usize_to_f32(i) * 30.0 + 1.0,
                0.0,
                10.0,
                10.0,
            ));
        }
        let damage = builder.finish(Size::new(1000.0, 1000.0));
        assert_eq!(damage.regions(Size::new(1000.0, 1000.0)).len(), 3);

        // A fourth rect must merge, not overflow the array.
        builder.add(rect(700.0, 700.0, 10.0, 10.0));
        let damage = builder.finish(Size::new(1000.0, 1000.0));
        let regions = damage.regions(Size::new(1000.0, 1000.0));
        assert_eq!(regions.len(), 3, "budget holds under merging");
        assert!(
            regions
                .iter()
                .any(|r| r.contains(rect(700.0, 700.0, 10.0, 10.0).origin)),
            "the fourth rect merged into some region"
        );
    }

    #[test]
    fn merging_picks_the_least_growth_pair() {
        // Two far-apart regions; a third rect close to region A must grow
        // A (cheapest) rather than B.
        let mut builder = DamageBuilder::new();
        builder.add(rect(0.0, 0.0, 10.0, 10.0)); // A
        builder.add(rect(500.0, 500.0, 10.0, 10.0)); // B
        builder.add(rect(2.0, 2.0, 10.0, 10.0)); // nearest A
        let regions = builder
            .finish(Size::new(1000.0, 1000.0))
            .regions(Size::new(1000.0, 1000.0));
        let a = regions
            .iter()
            .find(|r| r.contains(Point::new(1.0, 1.0)))
            .expect("A survives");
        assert!(
            a.max_x() <= 15.0 && a.max_y() <= 15.0,
            "A absorbed the near rect, got {a:?}"
        );
        let b = regions
            .iter()
            .find(|r| r.contains(Point::new(501.0, 501.0)))
            .expect("B untouched");
        assert_eq!(*b, rect(500.0, 500.0, 10.0, 10.0), "B stayed minimal");
    }

    #[test]
    fn old_and_new_rects_union_on_geometry_change() {
        // A widget moved from (0,0) to (50,50): both rects are damage.
        let mut builder = DamageBuilder::new();
        builder.add(rect(0.0, 0.0, 10.0, 10.0));
        builder.add(rect(50.0, 50.0, 10.0, 10.0));
        let damage = builder.finish(Size::new(1000.0, 1000.0));
        assert_eq!(
            damage.regions(Size::new(1000.0, 1000.0)),
            vec![rect(0.0, 0.0, 10.0, 10.0), rect(50.0, 50.0, 10.0, 10.0)],
            "both the old and the new rect survive the merge"
        );
    }

    #[test]
    fn regions_clamp_to_the_viewport() {
        let mut builder = DamageBuilder::new();
        builder.add(rect(90.0, 90.0, 40.0, 40.0)); // 30px out of a 100×100 vp
        let damage = builder.finish(Size::new(100.0, 100.0));
        assert_eq!(
            damage.regions(Size::new(100.0, 100.0)),
            vec![rect(90.0, 90.0, 10.0, 10.0)]
        );
    }

    #[test]
    fn near_full_coverage_falls_back_to_full_frame() {
        let mut builder = DamageBuilder::new();
        builder.add(rect(0.0, 0.0, 100.0, 80.0));
        let damage = builder.finish(Size::new(100.0, 100.0));
        assert!(damage.is_full(), "80% coverage beats the 75% hatch");
        assert_eq!(
            damage.regions(Size::new(100.0, 100.0)),
            vec![rect(0.0, 0.0, 100.0, 100.0)]
        );
    }

    #[test]
    fn scattered_small_damage_stays_partial() {
        let mut builder = DamageBuilder::new();
        for i in 0..DAMAGE_REGION_BUDGET {
            builder.add(rect(crate::num::usize_to_f32(i) * 40.0, 0.0, 5.0, 5.0));
        }
        let damage = builder.finish(Size::new(1000.0, 1000.0));
        assert!(!damage.is_full());
        assert!(!damage.is_empty());
    }

    #[test]
    fn full_and_none_are_the_canonical_extremes() {
        assert!(Damage::FULL.is_full());
        assert!(!Damage::FULL.is_empty());
        assert!(Damage::NONE.is_empty());
        assert!(!Damage::NONE.is_full());
    }
}
