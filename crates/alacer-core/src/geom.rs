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

#[cfg(test)]
mod tests {
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
}
