//! Shared geometry types, euclid-backed, in logical pixels.
//!
//! Layout computes in `f32` end to end; there is exactly one rounding point at
//! compose, per axis, in device pixels (architecture §8.3). All types are tagged
//! with the [`Px`] unit so that logical and physical values cannot mix silently.

/// The pixel unit tag. M0 assumes a 1:1 logical-to-physical scale; the tag keeps
/// the door open for a `ScaleFactor` conversion without changing call sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Px;

/// A 2D point in pixels.
pub type Point = euclid::Point2D<f32, Px>;

/// A 2D size in pixels.
pub type Size = euclid::Size2D<f32, Px>;

/// A 2D displacement vector in pixels.
pub type Vector = euclid::Vector2D<f32, Px>;

/// An axis-aligned rectangle in pixels.
pub type Rect = euclid::Rect<f32, Px>;

/// The region's whole-pixel form: integer bounds expanded outward (the
/// floor of the left/top edge, the ceil of the right/bottom), clamped to
/// the frame.
///
/// The partial-paint contract's one line: the clear, the scissor, and the
/// primitive-intersection test must draw the same line. Both backends
/// snap through this function — the software path's tiny-skia non-AA mask
/// fill and rect fill disagree at sub-pixel edges (the Q8 equivalence
/// fuzz caught boundary pixels repainting over stale blends), and the
/// wgpu path's `scissor_rect` must cut at the same pixels the software
/// scissor mask does. Outward is the conservative direction: a pixel the
/// damage nominally touches (its area intersects the rect) is repainted,
/// never skipped.
#[must_use]
pub fn snap_region_to_pixels(region: &Rect, width: u32, height: u32) -> Option<Rect> {
    let max_x = f32::from(u16::try_from(width).unwrap_or(u16::MAX));
    let max_y = f32::from(u16::try_from(height).unwrap_or(u16::MAX));
    let left = region.origin.x.floor().max(0.0);
    let top = region.origin.y.floor().max(0.0);
    let right = region.max_x().ceil().min(max_x);
    let bottom = region.max_y().ceil().min(max_y);
    if right <= left || bottom <= top {
        return None;
    }
    Some(Rect::new(
        Point::new(left, top),
        Size::new(right - left, bottom - top),
    ))
}

#[cfg(test)]
mod tests {
    use super::snap_region_to_pixels;
    use super::{Point, Rect, Size};

    #[test]
    fn rect_helpers_round_trip() {
        let rect = Rect::new(Point::new(1.0, 2.0), Size::new(10.0, 20.0));
        assert_eq!(rect.origin, Point::new(1.0, 2.0));
        assert_eq!(rect.size, Size::new(10.0, 20.0));
        assert_eq!(rect.max_x(), 11.0);
        assert_eq!(rect.max_y(), 22.0);
        assert!(rect.contains(Point::new(5.0, 5.0)));
        assert!(!rect.contains(Point::new(12.0, 5.0)));
    }

    #[test]
    fn snap_expands_outward_and_clamps_to_the_frame() {
        let region = Rect::new(Point::new(10.5, 10.5), Size::new(5.3, 4.3));
        let snapped = snap_region_to_pixels(&region, 160, 120).expect("region in frame");
        assert_eq!(snapped.origin, Point::new(10.0, 10.0), "left/top floor");
        assert_eq!(snapped.max_x(), 16.0, "right ceil — outward");
        assert_eq!(snapped.max_y(), 15.0, "bottom ceil — outward");
        // A region bleeding past the frame clamps to the pixel bounds.
        let bleeding = Rect::new(Point::new(-3.2, -1.8), Size::new(200.0, 200.0));
        let snapped = snap_region_to_pixels(&bleeding, 160, 120).expect("region intersects frame");
        assert_eq!(snapped.origin, Point::new(0.0, 0.0));
        assert_eq!((snapped.max_x(), snapped.max_y()), (160.0, 120.0));
        // A region wholly outside the frame snaps to nothing.
        let outside = Rect::new(Point::new(200.0, 200.0), Size::new(10.0, 10.0));
        assert!(snap_region_to_pixels(&outside, 160, 120).is_none());
    }
}
