//! Typed style conversion: Alacer's core `Style` → Taffy's `Style`.
//!
//! One conversion, one direction, no leakage of Taffy types above this crate.

use alacer_core::geom::Size;
use alacer_core::style::{
    CrossAxisAlignment, Display, FlexDirection, MainAxisAlignment, MeasureSpec, Style,
};
use taffy::style_helpers::length;
use taffy::{
    AlignContent, AlignItems, AvailableSpace, Dimension, Display as TaffyDisplay,
    FlexDirection as TaffyFlexDirection, JustifyContent, LengthPercentageAuto, Rect,
    Style as TaffyStyle,
};

/// Converts an Alacer style to its Taffy equivalent.
#[must_use]
pub fn to_taffy(style: &Style) -> TaffyStyle {
    TaffyStyle {
        display: match style.display {
            Display::Flex => TaffyDisplay::Flex,
            Display::None => TaffyDisplay::None,
        },
        flex_direction: match style.flex_direction {
            FlexDirection::Row => TaffyFlexDirection::Row,
            FlexDirection::Column => TaffyFlexDirection::Column,
        },
        flex_grow: style.grow,
        flex_shrink: style.shrink,
        size: map_size(style.size),
        min_size: map_min_max(style.min_size),
        max_size: map_min_max(style.max_size),
        padding: Rect {
            left: length(style.padding),
            right: length(style.padding),
            top: length(style.padding),
            bottom: length(style.padding),
        },
        gap: taffy::Size {
            width: length(style.gap),
            height: length(0.0),
        },
        justify_content: match style.justify {
            MainAxisAlignment::Start => JustifyContent::FLEX_START,
            MainAxisAlignment::Center => JustifyContent::CENTER,
            MainAxisAlignment::End => JustifyContent::FLEX_END,
        },
        align_items: match style.align {
            CrossAxisAlignment::Start => AlignItems::FLEX_START,
            CrossAxisAlignment::Center => AlignItems::CENTER,
            CrossAxisAlignment::End => AlignItems::FLEX_END,
            CrossAxisAlignment::Stretch => AlignItems::STRETCH,
        },
        align_content: AlignContent::FLEX_START,
        ..TaffyStyle::default()
    }
}

fn map_size(s: alacer_core::style::Size2) -> taffy::Size<Dimension> {
    taffy::Size {
        width: s.width.map_or(Dimension::auto(), Dimension::length),
        height: s.height.map_or(Dimension::auto(), Dimension::length),
    }
}

fn map_min_max(s: alacer_core::style::Size2) -> taffy::Size<LengthPercentageAuto> {
    taffy::Size {
        width: s
            .width
            .map_or(LengthPercentageAuto::auto(), LengthPercentageAuto::length),
        height: s
            .height
            .map_or(LengthPercentageAuto::auto(), LengthPercentageAuto::length),
    }
}

/// Resolves a fixed measure spec into the style's fixed size, if applicable.
#[must_use]
pub fn fixed_size_of(spec: &MeasureSpec) -> Option<Size> {
    match spec {
        MeasureSpec::Fixed(size) => Some(*size),
        MeasureSpec::Text(_) => None,
    }
}

/// The available width as an `Option<f32>`, the measure seam's vocabulary.
#[must_use]
pub fn definite_width(space: AvailableSpace) -> Option<f32> {
    match space {
        AvailableSpace::Definite(w) => Some(w),
        AvailableSpace::MaxContent | AvailableSpace::MinContent => None,
    }
}

#[cfg(test)]
mod tests {
    use super::to_taffy;
    use alacer_core::style::{CrossAxisAlignment, Display, FlexDirection, Style};

    #[test]
    fn column_styles_map_to_taffy() {
        let alacer = Style {
            flex_direction: FlexDirection::Column,
            grow: 1.0,
            padding: 8.0,
            gap: 4.0,
            align: CrossAxisAlignment::Center,
            ..Style::default()
        };
        let t = to_taffy(&alacer);
        assert_eq!(t.flex_direction, taffy::FlexDirection::Column);
        assert_eq!(t.flex_grow, 1.0);
        assert_eq!(t.padding.left, taffy::LengthPercentage::length(8.0));
        assert_eq!(t.gap.width, taffy::LengthPercentage::length(4.0));
        assert_eq!(t.align_items, taffy::AlignItems::CENTER);
    }

    #[test]
    fn display_none_maps() {
        let t = to_taffy(&Style {
            display: Display::None,
            ..Style::default()
        });
        assert_eq!(t.display, taffy::Display::None);
    }

    #[test]
    fn fixed_sizes_map_to_lengths() {
        let s = Style {
            size: alacer_core::style::Size2::uniform(40.0),
            ..Style::default()
        };
        let t = to_taffy(&s);
        assert_eq!(t.size.width, taffy::Dimension::length(40.0));
        assert_eq!(t.size.height, taffy::Dimension::length(40.0));
    }
}
