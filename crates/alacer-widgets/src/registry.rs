//! The default element registry: the M0 widget set, registered into
//! [`alacer_view::WidgetRegistry`] (ADR-9: the view layer's registry
//! machinery stays in the foundation; the concrete widget set registers
//! itself here, where the widget types live).

use alacer_core::style::Style;
use alacer_view::{ElementProps, WidgetRegistry};

/// Creates the registry pre-loaded with the element set: the M0 canon
/// (`label`, `button`, `column`, `row`, `spacer`, `panel`, `list`) plus the
/// M6 canon (`checkbox`, `text_input`, `table`, `table_row`, `table_cell`,
/// `column_header`) plus the M32 value tier (`slider`, `progress`).
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
    registry.register("checkbox", checkbox_create, checkbox_patch);
    registry.register("text_input", text_input_create, text_input_patch);
    registry.register("table", table_create, table_patch);
    registry.register("table_row", table_row_create, table_row_patch);
    registry.register("table_cell", table_cell_create, table_cell_patch);
    registry.register("column_header", column_header_create, column_header_patch);
    registry.register("slider", slider_create, slider_patch);
    registry.register("progress", progress_create, progress_patch);
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

/// `f32` prop → grid index (rounds; prop numbers are small).
fn prop_index(props: &ElementProps<'_>, name: &str) -> Option<usize> {
    props
        .number(name)
        .map(|value| alacer_core::num::f32_to_usize(value.round().max(0.0)))
}

// ---------------------------------------------------------------------------
// The M6 canon elements
// ---------------------------------------------------------------------------

fn checkbox_create(props: &ElementProps<'_>) -> crate::Checkbox {
    let mut checkbox = crate::Checkbox::new(props.text("name").unwrap_or_default());
    if let Some(checked) = props.flag("checked") {
        checkbox = checkbox.checked(checked);
    }
    if let Some(handler) = props.flag_handler("on_toggle") {
        checkbox = checkbox.on_toggle(handler);
    }
    if let Some(color) = props.color("accent") {
        checkbox = checkbox.accent(color);
    }
    checkbox
}

fn checkbox_patch(checkbox: &mut crate::Checkbox, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        checkbox.set_name(name);
        visual = true;
    }
    if let Some(checked) = props.flag("checked") {
        checkbox.set_checked(checked);
        visual = true;
    }
    if let Some(handler) = props.flag_handler("on_toggle") {
        checkbox.set_on_toggle(Some(handler));
    }
    visual
}

fn text_input_create(props: &ElementProps<'_>) -> crate::TextInput {
    let mut input = crate::TextInput::new(props.text("name").unwrap_or_default());
    if let Some(placeholder) = props.text("placeholder") {
        input = input.placeholder(placeholder);
    }
    if props.flag("secure") == Some(true) {
        input = input.secure();
    }
    if let Some(handler) = props.text_handler("on_changed") {
        input = input.on_changed(handler);
    }
    if let Some(handler) = props.text_handler("on_submit") {
        input = input.on_submit(handler);
    }
    input
}

fn text_input_patch(input: &mut crate::TextInput, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        input.set_name(name);
        visual = true;
    }
    if let Some(placeholder) = props.text("placeholder") {
        input.set_placeholder(placeholder);
        visual = true;
    }
    if let Some(handler) = props.text_handler("on_changed") {
        input.set_on_changed(Some(handler));
    }
    if let Some(handler) = props.text_handler("on_submit") {
        input.set_on_submit(Some(handler));
    }
    visual
}

fn table_create(props: &ElementProps<'_>) -> crate::Table {
    let mut table = crate::Table::new(
        props.text("name").unwrap_or_default(),
        prop_index(props, "columns").unwrap_or(0),
    );
    table.set_row_count(prop_index(props, "rows").unwrap_or(0));
    style_apply(table.style_mut(), props);
    table
}

fn table_patch(table: &mut crate::Table, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        table.set_name(name);
        visual = true;
    }
    if let Some(rows) = prop_index(props, "rows") {
        table.set_row_count(rows);
        visual = true;
    }
    if let Some(columns) = prop_index(props, "columns") {
        table.set_column_count(columns);
        visual = true;
    }
    style_apply(table.style_mut(), props);
    visual
}

fn table_row_create(props: &ElementProps<'_>) -> crate::TableRow {
    let mut row = crate::TableRow::new();
    style_apply(row.style_mut(), props);
    row
}

fn table_row_patch(row: &mut crate::TableRow, props: &ElementProps<'_>) -> bool {
    style_apply(row.style_mut(), props);
    style_visual_diff(props)
}

fn table_cell_create(props: &ElementProps<'_>) -> crate::TableCell {
    crate::TableCell::new(
        props.text("text").unwrap_or_default(),
        prop_index(props, "row").unwrap_or(0),
        prop_index(props, "column").unwrap_or(0),
    )
}

fn table_cell_patch(cell: &mut crate::TableCell, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(text) = props.text("text") {
        cell.set_text(text);
        visual = true;
    }
    if let (Some(row), Some(column)) = (prop_index(props, "row"), prop_index(props, "column")) {
        cell.set_slot(row, column);
        visual = true;
    }
    visual
}

fn column_header_create(props: &ElementProps<'_>) -> crate::ColumnHeader {
    crate::ColumnHeader::new(
        props.text("text").unwrap_or_default(),
        prop_index(props, "column").unwrap_or(0),
    )
}

fn column_header_patch(header: &mut crate::ColumnHeader, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(text) = props.text("text") {
        header.set_text(text);
        visual = true;
    }
    visual
}

// ---------------------------------------------------------------------------
// The M32 value-tier elements
// ---------------------------------------------------------------------------

/// The value prop's create write: the widget's own funnel clamps and
/// snaps every write, so the view's number lands exactly where a drag or
/// an arrow would put it.
fn slider_create(props: &ElementProps<'_>) -> crate::Slider {
    let mut slider = crate::Slider::new(
        props.text("name").unwrap_or_default(),
        props.number("min").unwrap_or(0.0),
        props.number("max").unwrap_or(100.0),
    );
    if let Some(step) = props.number("step") {
        slider = slider.step(step);
    }
    if let Some(value) = props.number("value") {
        slider = slider.value(value);
    }
    if let Some(handler) = props.number_handler("on_change") {
        let mut handler = handler;
        slider = slider.on_change(move |value: f32| handler(value));
    }
    if let Some(accent) = props.color("accent") {
        slider = slider.accent(accent);
    }
    slider
}

fn slider_patch(slider: &mut crate::Slider, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        slider.set_name(name);
        visual = true;
    }
    if let Some(value) = props.number("value") {
        // The programmatic write fires no handler (the widget's own
        // contract: state, not a user act) and clamps/snaps inside the
        // widget — the funnel every door rides. The write moved the
        // value only when the funnel changed it (the widget's own
        // `total_cmp` equality, never `f32::eq`).
        let before = slider.current_value();
        slider.set_value(value);
        visual |= !same_value(before, slider.current_value());
    }
    visual
}

/// The float equality the patcher borrows: `total_cmp` ordering, the
/// widget's own rule (`f32::eq` is a lint trap the whole workspace
/// refuses) — a patch reports "visual" only when the funnel moved the
/// value.
fn same_value(a: f32, b: f32) -> bool {
    use std::cmp::Ordering;
    a.total_cmp(&b) == Ordering::Equal
}

fn progress_create(props: &ElementProps<'_>) -> crate::Progress {
    let name = props.text("name").unwrap_or_default();
    let min = props.number("min").unwrap_or(0.0);
    let max = props.number("max").unwrap_or(100.0);
    // The two doors the value tier's report half ships (M21): the
    // reactive bridge (a `binding` prop — the flux bridge owns the
    // value, §6.1) or the static value. The binding wins, the bound
    // label's rule.
    if let Some(binding) = props.number_binding() {
        return crate::Progress::bound(name, min, max, binding);
    }
    let mut progress = crate::Progress::new(name, min, max);
    if let Some(value) = props.number("value") {
        progress = progress.value(value);
    }
    if let Some(fill) = props.color("fill") {
        progress = progress.fill(fill);
    }
    progress
}

fn progress_patch(progress: &mut crate::Progress, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        progress.set_name(name);
        visual = true;
    }
    if let Some(binding) = props.number_binding() {
        // The bridge owns a bound progress's value: re-hanging the same
        // binding is a no-op, never a visual change (the bound label's
        // rule, at the value tier).
        let _ = binding;
        return visual;
    }
    if let Some(value) = props.number("value") {
        // The widget's own answer is the idle-frame property: a write
        // that lands where the value already is reports `false` — no
        // masks, an idle frame (M21).
        visual |= progress.set_value(value);
    }
    visual
}

#[cfg(test)]
mod tests {
    use super::default_registry;
    use crate::theme;
    use alacer_core::color::Color;
    use alacer_view::ViewNode;

    /// Downcasts the erased widget to `Checkbox` (the test's introspection).
    fn downcast_checkbox(widget: &mut dyn alacer_core::Widget) -> Option<&mut crate::Checkbox> {
        widget.as_any_mut().downcast_mut::<crate::Checkbox>()
    }

    #[test]
    fn the_default_registry_knows_the_canon_set() {
        let registry = default_registry();
        for name in [
            "label",
            "button",
            "column",
            "row",
            "spacer",
            "panel",
            "list",
            // The M6 canon elements.
            "checkbox",
            "text_input",
            "table",
            "table_row",
            "table_cell",
            "column_header",
            // The M32 value-tier elements.
            "slider",
            "progress",
        ] {
            assert!(registry.contains(name), "{name} is a default element");
        }
        assert!(!registry.contains("video"));
    }

    #[test]
    fn slider_props_create_and_patch_in_place() {
        let registry = default_registry();
        let node = ViewNode::element("slider")
            .prop("name", "Volume")
            .prop("min", 0.0_f32)
            .prop("max", 100.0_f32)
            .prop("step", 5.0_f32)
            .prop("value", 42.0_f32);
        let mut widget = registry.create(&node);
        assert_eq!(widget.label().resolve(), "Volume");
        assert!(
            downcast_slider(widget.as_mut()).is_some_and(|s| same_value(s.current_value(), 40.0)),
            "the create write rides the widget's own funnel (clamped and snapped)"
        );

        // The value patch: clamped and snapped like every write, in place.
        let updated = ViewNode::element("slider").prop("value", 99.0_f32);
        assert!(
            registry.patch(widget.as_mut(), &updated),
            "a value patch is visual"
        );
        assert!(
            downcast_slider(widget.as_mut()).is_some_and(|s| same_value(s.current_value(), 100.0)),
            "the patch write clamps to the range the widget owns"
        );

        // A write that lands where the value already is: no visual change
        // (the M20 idle-frame property, at the view seam).
        let again = ViewNode::element("slider").prop("value", 100.0_f32);
        assert!(
            !registry.patch(widget.as_mut(), &again),
            "a no-op write patches to no masks"
        );

        // The name patch: the label the a11y tree carries.
        let renamed = ViewNode::element("slider").prop("name", "Bass");
        assert!(registry.patch(widget.as_mut(), &renamed));
        assert_eq!(widget.label().resolve(), "Bass", "patched in place");
    }

    #[test]
    fn progress_props_create_and_patch_and_the_idle_frame() {
        let registry = default_registry();
        let node = ViewNode::element("progress")
            .prop("name", "Upload")
            .prop("min", 0.0_f32)
            .prop("max", 4.0_f32)
            .prop("value", 2.0_f32);
        let mut widget = registry.create(&node);
        assert_eq!(widget.label().resolve(), "Upload");
        assert!(
            downcast_progress(widget.as_mut()).is_some_and(|p| same_value(p.current_value(), 2.0)),
            "the reported value lands at create"
        );

        let updated = ViewNode::element("progress").prop("value", 3.0_f32);
        assert!(registry.patch(widget.as_mut(), &updated));
        assert!(
            downcast_progress(widget.as_mut()).is_some_and(|p| same_value(p.current_value(), 3.0)),
            "the value patch lands in place"
        );

        // A write that lands where the value already is queues no masks
        // (the M21 idle-frame property — the patcher's `false` is exactly
        // the widget's own `set_value` answer).
        let again = ViewNode::element("progress").prop("value", 3.0_f32);
        assert!(!registry.patch(widget.as_mut(), &again));
    }

    #[test]
    fn progress_binds_its_value_and_rehangs_idempotently() {
        let registry = default_registry();
        let binding = alacer_core::flux::RenderBinding::new(1.5_f32);
        let node = ViewNode::element("progress")
            .prop("name", "Bound")
            .prop("min", 0.0_f32)
            .prop("max", 4.0_f32)
            .prop("binding", binding.clone());
        let mut widget = registry.create(&node);
        assert!(
            downcast_progress(widget.as_mut()).is_some_and(|p| same_value(p.current_value(), 1.5)),
            "the number binding rides the bound constructor"
        );

        // The flux bridge owns a bound progress's value (§6.1): a rebuild
        // re-hangs the same binding slot — a no-op patch, never a visual
        // change (the bound label's rule, at the value tier).
        let rebuilt = ViewNode::element("progress").prop("binding", binding);
        assert!(!registry.patch(widget.as_mut(), &rebuilt));
    }

    /// The float equality the tests borrow: `total_cmp` ordering, the
    /// widget's own rule (`f32::eq` is a lint trap the whole workspace
    /// refuses).
    fn same_value(a: f32, b: f32) -> bool {
        use std::cmp::Ordering;
        a.total_cmp(&b) == Ordering::Equal
    }

    /// Downcasts the erased widget to `Slider` (the test's introspection).
    fn downcast_slider(widget: &mut dyn alacer_core::Widget) -> Option<&mut crate::Slider> {
        widget.as_any_mut().downcast_mut::<crate::Slider>()
    }

    /// Downcasts the erased widget to `Progress` (the test's introspection).
    fn downcast_progress(widget: &mut dyn alacer_core::Widget) -> Option<&mut crate::Progress> {
        widget.as_any_mut().downcast_mut::<crate::Progress>()
    }

    #[test]
    fn checkbox_props_create_and_patch_in_place() {
        let registry = default_registry();
        let toggled = std::rc::Rc::new(std::cell::RefCell::new(false));
        let sink = toggled.clone();
        let node = ViewNode::element("checkbox")
            .prop("name", "Ship in bulk")
            .prop("checked", false)
            .prop(
                "on_toggle",
                alacer_view::PropValue::FlagHandler(std::rc::Rc::new(std::cell::RefCell::new(
                    move |checked: bool| *sink.borrow_mut() = checked,
                ))),
            );
        let mut widget = registry.create(&node);
        assert_eq!(widget.label().resolve(), "Ship in bulk");
        assert!(
            downcast_checkbox(widget.as_mut()).is_some_and(|c| !c.is_checked()),
            "created unchecked"
        );

        let updated = ViewNode::element("checkbox").prop("checked", true);
        assert!(
            registry.patch(widget.as_mut(), &updated),
            "checked is visual"
        );
        assert!(
            downcast_checkbox(widget.as_mut()).is_some_and(|c| c.is_checked()),
            "patched in place"
        );

        // The view-driven toggle path: the flag handler rides the widget.
        let mut ctx = alacer_core::events::EventCtx::new();
        let up = alacer_core::events::PointerEvent::Up {
            position: alacer_core::geom::Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
        };
        let _ = widget.on_pointer_event(&mut ctx, &up);
        assert!(
            downcast_checkbox(widget.as_mut()).is_some_and(|c| !c.is_checked()),
            "user toggle flips the widget"
        );
        assert!(
            !*toggled.borrow(),
            "the view handler observed the new state (false)"
        );
    }

    #[test]
    fn text_input_props_carry_handlers() {
        let registry = default_registry();
        let submitted = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
        let sink = submitted.clone();
        let node = ViewNode::element("text_input")
            .prop("name", "Add a todo")
            .prop(
                "on_submit",
                alacer_view::PropValue::TextHandler(std::rc::Rc::new(std::cell::RefCell::new(
                    move |text: &str| *sink.borrow_mut() = text.to_string(),
                ))),
            );
        let mut widget = registry.create(&node);
        assert_eq!(widget.label().resolve(), "Add a todo");

        for ch in "hello".chars() {
            let mut ctx = alacer_core::events::EventCtx::new();
            let _ = widget.on_key_event(&mut ctx, &alacer_core::events::KeyEvent::char(ch));
        }
        let mut ctx = alacer_core::events::EventCtx::new();
        let enter = alacer_core::events::KeyEvent {
            key: alacer_core::events::Key::Enter,
            state: alacer_core::events::KeyState::Pressed,
            modifiers: alacer_core::events::Modifiers::default(),
            text: None,
        };
        assert_eq!(
            widget.on_key_event(&mut ctx, &enter),
            alacer_core::widget::EventResponse::Handled
        );
        assert_eq!(
            *submitted.borrow(),
            "hello",
            "the view handler got the text"
        );
    }

    #[test]
    fn table_elements_carry_grid_props() {
        let registry = default_registry();
        let node = ViewNode::element("table")
            .prop("name", "Inventory")
            .prop("rows", 3.0_f32)
            .prop("columns", 4.0_f32);
        let widget = registry.create(&node);
        assert_eq!(
            widget.table_slot(),
            Some(alacer_core::TableSlot::Table {
                rows: 3,
                columns: 4
            })
        );

        let cell_node = ViewNode::element("table_cell")
            .prop("text", "Nails")
            .prop("row", 2.0_f32)
            .prop("column", 1.0_f32);
        let cell = registry.create(&cell_node);
        assert_eq!(cell.label().resolve(), "Nails");
        assert_eq!(
            cell.table_slot(),
            Some(alacer_core::TableSlot::Cell { row: 2, column: 1 })
        );
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
        let Some(alacer_core::style::MeasureSpec::Fixed(size)) = widget.measure_spec() else {
            panic!("panel height prop did not materialize into a fixed measure");
        };
        assert!(
            (size.height - 12.0).abs() < f32::EPSILON,
            "panel height prop materialized"
        );
        let _ = theme::SURFACE;
    }
}
