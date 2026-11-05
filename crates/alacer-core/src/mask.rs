//! The one currency of change (axiom A1).
//!
//! Every cross-cutting consumer — layout, paint, accessibility, devtools — reads
//! these per-node masks. A feature that cannot express itself as a mask is
//! rejected or redesigned.

use bitflags::bitflags;

bitflags! {
    /// Per-node invalidation flags: the single currency every pass consumes.
    ///
    /// Propagation to ancestors is normative — see
    /// [`Change::propagated_to_ancestors`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Change: u8 {
        /// Tree shape changed: children added, removed or reordered.
        const STRUCTURE = 0b0000_0001;
        /// Geometry may change: sizes or positions must be recomputed.
        const LAYOUT = 0b0000_0010;
        /// Visual output may change: the node's scene ops must be re-emitted.
        const PAINT = 0b0000_0100;
        /// Accessible information may change: the AccessKit update must be rebuilt.
        const A11Y = 0b0000_1000;
        /// Focus state changed.
        const FOCUS = 0b0001_0000;
        /// Every flag at once — structural churn.
        const ALL = Self::STRUCTURE.bits() | Self::LAYOUT.bits() | Self::PAINT.bits() | Self::A11Y.bits() | Self::FOCUS.bits();
    }
}

impl Change {
    /// The masks an *ancestor* must absorb when this node carries `self`.
    ///
    /// Normative propagation table (architecture §5.2, A1):
    ///
    /// | Node carries  | Ancestor absorbs                | Rationale |
    /// |---------------|---------------------------------|-----------|
    /// | `STRUCTURE`   | `STRUCTURE \| LAYOUT \| PAINT \| A11Y` | ancestors re-resolve focusables, container geometry, damage and the a11y tree |
    /// | `LAYOUT`      | `LAYOUT \| PAINT \| A11Y`       | a child resize reshapes ancestor layout, damage and a11y bounds |
    /// | `PAINT`       | `PAINT`                         | a color change repaints this node only |
    /// | `A11Y`        | `A11Y`                          | a label change touches this node's a11y entry only |
    /// | `FOCUS`       | `FOCUS`                         | focus is a side-channel, never masks (§7.2 pass 10) |
    #[must_use]
    pub fn propagated_to_ancestors(self) -> Self {
        let mut out = Self::empty();
        if self.contains(Self::STRUCTURE) {
            out |= Self::STRUCTURE
                .union(Self::LAYOUT)
                .union(Self::PAINT)
                .union(Self::A11Y);
        }
        if self.contains(Self::LAYOUT) {
            out |= Self::LAYOUT.union(Self::PAINT).union(Self::A11Y);
        }
        if self.contains(Self::PAINT) {
            out |= Self::PAINT;
        }
        if self.contains(Self::A11Y) {
            out |= Self::A11Y;
        }
        if self.contains(Self::FOCUS) {
            out |= Self::FOCUS;
        }
        out
    }

    /// `true` when a layout recomputation is required.
    #[must_use]
    pub fn forces_layout(self) -> bool {
        self.intersects(Self::LAYOUT.union(Self::STRUCTURE))
    }

    /// `true` when the node's scene ops must be re-emitted.
    #[must_use]
    pub fn forces_paint(self) -> bool {
        self.intersects(Self::PAINT.union(Self::STRUCTURE))
    }

    /// `true` when the AccessKit update must be rebuilt for this node.
    #[must_use]
    pub fn forces_a11y(self) -> bool {
        self.intersects(Self::A11Y.union(Self::STRUCTURE))
    }
}

#[cfg(test)]
mod tests {
    use super::Change;

    #[test]
    fn structure_propagates_everything() {
        let propagated = Change::STRUCTURE.propagated_to_ancestors();
        assert!(propagated.contains(Change::STRUCTURE));
        assert!(propagated.contains(Change::LAYOUT));
        assert!(propagated.contains(Change::PAINT));
        assert!(propagated.contains(Change::A11Y));
    }

    #[test]
    fn layout_propagates_paint_and_a11y() {
        let propagated = Change::LAYOUT.propagated_to_ancestors();
        assert!(propagated.contains(Change::LAYOUT));
        assert!(propagated.contains(Change::PAINT));
        assert!(propagated.contains(Change::A11Y));
        assert!(!propagated.contains(Change::STRUCTURE));
    }

    #[test]
    fn paint_does_not_reach_ancestors_beyond_itself() {
        let propagated = Change::PAINT.propagated_to_ancestors();
        assert_eq!(propagated, Change::PAINT);
    }

    #[test]
    fn a11y_stays_local() {
        assert_eq!(Change::A11Y.propagated_to_ancestors(), Change::A11Y);
    }

    #[test]
    fn focus_is_a_side_channel() {
        assert_eq!(Change::FOCUS.propagated_to_ancestors(), Change::FOCUS);
        assert!(!Change::FOCUS.forces_layout());
        assert!(!Change::FOCUS.forces_paint());
    }

    #[test]
    fn combined_masks_accumulate_ancestor_consequences() {
        let propagated = Change::LAYOUT.union(Change::A11Y).propagated_to_ancestors();
        assert!(propagated.contains(Change::LAYOUT));
        assert!(propagated.contains(Change::PAINT));
        assert!(propagated.contains(Change::A11Y));
    }

    #[test]
    fn predicates_partition_the_flags() {
        assert!(Change::LAYOUT.forces_layout());
        assert!(Change::STRUCTURE.forces_layout());
        assert!(Change::PAINT.forces_paint());
        assert!(Change::A11Y.forces_a11y());
        assert!(Change::STRUCTURE.forces_a11y());
    }
}
