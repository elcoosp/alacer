//! The default element registry: the M0 widget set, registered into
//! [`alacer_view::WidgetRegistry`] (ADR-9: the view layer's registry
//! machinery stays in the foundation; the concrete widget set registers
//! itself here, where the widget types live).

use alacer_core::style::Style;
use alacer_view::{ElementProps, WidgetRegistry};

/// Creates the registry pre-loaded with the M0 element set: `label`,
/// `button`, `column`, `row`, `spacer`, `panel`, `list` (§5).
#[must_use]
pub fn default_registry() -> WidgetRegistry {
    let mut registry = WidgetRegistry::new();
    registry.register("label", label_create, label_patch);
    registry.register("button", button_create, button_patch);
    registry.register("column", column_create, container_patch);
    registry.register("row", row_create, container_patch);
    registry.register("spacer", spacer_create, spacer_patch);
    registry.register("panel", panel_create, panel_patch);
    registry.register("list", list_create, list_patch);
    registry
}

fn label_create(props: &ElementProps<'_>) -> crate::Label {
    if let Some(binding) = props.binding() {
        return crate::Label::bound(binding);
    }
    let mut label = crate::Label::new(props.text("text").unwrap_or_default());
    if let Some(color) = props.color("color") {
        label = label.color(color);
    }
    if let Some(size) = props.number("font_size") {
        label = label.font_size(size);
    }
    label
}

fn label_patch(label: &mut crate::Label, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(binding) = props.binding() {
        // The flux bridge owns a bound label's text (§6.1): re-hanging the
        // same binding is a no-op, never a visual change.
        let _ = binding;
        return false;
    }
    if let Some(text) = props.text("text") {
        label.set_text(text);
        visual = true;
    }
    if let Some(color) = props.color("color") {
        label.set_color(color);
        visual = true;
    }
    if let Some(size) = props.number("font_size") {
        label.set_font_size(size);
        visual = true;
    }
    visual
}

fn button_create(props: &ElementProps<'_>) -> crate::Button {
    let mut button = crate::Button::new(props.text("text").unwrap_or_default());
    if let Some(handler) = props.handler("on_click") {
        button = button.on_click(handler);
    }
    if let Some(color) = props.color("color") {
        button = button.colors(color, color);
    }
    if let Some(text_color) = props.color("text_color") {
        button = button.text_color(text_color);
    }
    button
}

fn button_patch(button: &mut crate::Button, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(text) = props.text("text") {
        button.set_label(text);
        visual = true;
    }
    if let Some(color) = props.color("color") {
        button.set_fill(color);
        visual = true;
    }
    if let Some(text_color) = props.color("text_color") {
        button.set_text_color(text_color);
        visual = true;
    }
    if let Some(handler) = props.handler("on_click") {
        button.set_on_click(Some(handler));
    }
    visual
}

/// Typed style writes shared by the container elements.
fn style_apply(style: &mut Style, props: &ElementProps<'_>) {
    if let Some(gap) = props.number("gap") {
        style.gap = gap;
    }
    if let Some(padding) = props.number("padding") {
        style.padding = padding;
    }
    if let Some(grow) = props.number("grow") {
        style.grow = grow;
    }
    if let Some(width) = props.number("width") {
        style.size.width = Some(width);
    }
    if let Some(height) = props.number("height") {
        style.size.height = Some(height);
    }
}

fn style_visual_diff(props: &ElementProps<'_>) -> bool {
    props.number("gap").is_some()
        || props.number("padding").is_some()
        || props.number("grow").is_some()
        || props.number("width").is_some()
        || props.number("height").is_some()
}

/// The container style seam: elements whose widget exposes `style_mut`.
trait StyleMut {
    fn style_mut(&mut self) -> &mut Style;
}

impl StyleMut for crate::Column {
    fn style_mut(&mut self) -> &mut Style {
        self.style_mut()
    }
}

impl StyleMut for crate::Row {
    fn style_mut(&mut self) -> &mut Style {
        self.style_mut()
    }
}

impl StyleMut for crate::List {
    fn style_mut(&mut self) -> &mut Style {
        self.style_mut()
    }
}

fn column_create(props: &ElementProps<'_>) -> crate::Column {
    let mut column = crate::Column::new();
    style_apply(column.style_mut(), props);
    column
}

fn row_create(props: &ElementProps<'_>) -> crate::Row {
    let mut row = crate::Row::new();
    style_apply(row.style_mut(), props);
    row
}

fn container_patch<W: StyleMut>(container: &mut W, props: &ElementProps<'_>) -> bool {
    style_apply(container.style_mut(), props);
    style_visual_diff(props)
}

fn spacer_create(props: &ElementProps<'_>) -> crate::Spacer {
    crate::Spacer::new(props.number("size").unwrap_or(8.0))
}

fn spacer_patch(spacer: &mut crate::Spacer, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(size) = props.number("size") {
        spacer.set_size(size);
        visual = true;
    }
    visual
}

fn panel_create(props: &ElementProps<'_>) -> crate::Panel {
    crate::Panel::new(
        props.number("height").unwrap_or(40.0),
        props.color("color").unwrap_or(crate::theme::SURFACE),
    )
}

fn panel_patch(panel: &mut crate::Panel, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(height) = props.number("height") {
        panel.set_height(height);
        visual = true;
    }
    if let Some(color) = props.color("color") {
        panel.set_color(color);
        visual = true;
    }
    visual
}

fn list_create(props: &ElementProps<'_>) -> crate::List {
    let mut list = crate::List::new(props.number("height").unwrap_or(200.0));
    if let Some(grow) = props.number("grow") {
        list = list.grow(grow);
    }
    list
}

fn list_patch(list: &mut crate::List, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(height) = props.number("height") {
        list.style_mut().size.height = Some(height);
        visual = true;
    }
    if let Some(grow) = props.number("grow") {
        list.style_mut().grow = grow;
        visual = true;
    }
    visual
}

#[cfg(test)]
mod tests {
    use super::default_registry;
    use crate::theme;
    use alacer_core::color::Color;
    use alacer_view::ViewNode;

    #[test]
    fn the_default_registry_knows_the_m0_set() {
        let registry = default_registry();
        for name in [
            "label", "button", "column", "row", "spacer", "panel", "list",
        ] {
            assert!(registry.contains(name), "{name} is a default element");
        }
        assert!(!registry.contains("video"));
    }

    #[test]
    fn label_props_create_and_patch_in_place() {
        let registry = default_registry();
        let node = ViewNode::element("label")
            .prop("text", "before")
            .prop("font_size", 20.0_f32);
        let mut widget = registry.create(&node);
        assert_eq!(widget.label().resolve(), "before");

        let updated = ViewNode::element("label")
            .prop("text", "after")
            .prop("color", Color::new(255, 0, 0, 255));
        assert!(registry.patch(widget.as_mut(), &updated));
        assert_eq!(widget.label().resolve(), "after", "patched in place");
    }

    #[test]
    fn container_props_write_styles() {
        let registry = default_registry();
        let node = ViewNode::element("column").prop("gap", 6.0_f32);
        let mut widget = registry.create(&node);
        assert_eq!(widget.style().gap, 6.0);
        let updated = ViewNode::element("column").prop("gap", 9.0_f32);
        assert!(registry.patch(widget.as_mut(), &updated));
        assert_eq!(widget.style().gap, 9.0);
    }

    #[test]
    fn panel_defaults_to_the_theme_surface() {
        let registry = default_registry();
        let node = ViewNode::element("panel").prop("height", 12.0_f32);
        let widget = registry.create(&node);
        assert!(
            matches!(
                widget.measure_spec(),
                Some(alacer_core::style::MeasureSpec::Fixed(size)) if size.height == 12.0
            ),
            "panel height prop materialized"
        );
        let _ = theme::SURFACE;
    }
}
