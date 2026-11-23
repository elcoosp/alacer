//! The default element registry: the M0 widget set, registered into
//! [`alacer_view::WidgetRegistry`] (ADR-9: the view layer's registry
//! machinery stays in the foundation; the concrete widget set registers
//! itself here, where the widget types live).

use alacer_core::style::Style;
use alacer_view::{ElementProps, WidgetRegistry};

/// Creates the registry pre-loaded with the element set: the M0 canon
/// (`label`, `button`, `column`, `row`, `spacer`, `panel`, `list`) plus the
/// M6 canon (`checkbox`, `text_input`, `table`, `table_row`, `table_cell`,
/// `column_header`) plus the M32 value tier (`slider`, `progress`) plus the
/// M82 paint seam (`canvas`) plus the M124 radio tier (`radio`,
/// `radio_group`) plus the M125 link tier (`link`) plus the M126 heading
/// tier (`heading`) plus the M127 tab tier (`tab`, `tab_list`) plus the
/// M128 spin-button tier (`spin_button`) plus the M129 toolbar tier
/// (`toolbar`) plus the M130 switch tier (`switch`) plus the M131
/// tab-panel tier (`tab_panel`) plus the M133 meter tier (`meter`)
/// plus the M134 dialog tier (`dialog`) plus the M135 tree tier (`tree`,
/// `tree_item`) plus the M136 tooltip tier (`tooltip`) plus the M137 menu
/// tier (`menu_bar`, `menu`, `menu_item`) plus the M138 image tier
/// (`image`) plus the M143 landmark tier (`banner`, `navigation`,
/// `main`, `content_info`) and its M144 growth (`complementary`,
/// `form`, `region`, `search`).
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
    registry.register("meter", meter_create, meter_patch);
    registry.register("canvas", canvas_create, canvas_patch);
    registry.register("radio", radio_create, radio_patch);
    registry.register("radio_group", radio_group_create, radio_group_patch);
    registry.register("link", link_create, link_patch);
    registry.register("heading", heading_create, heading_patch);
    registry.register("tab", tab_create, tab_patch);
    registry.register("tab_list", tab_list_create, tab_list_patch);
    registry.register("spin_button", spin_button_create, spin_button_patch);
    registry.register("toolbar", toolbar_create, toolbar_patch);
    registry.register("switch", switch_create, switch_patch);
    registry.register("tab_panel", tab_panel_create, tab_panel_patch);
    registry.register("dialog", dialog_create, dialog_patch);
    registry.register("tree", tree_create, tree_patch);
    registry.register("tree_item", tree_item_create, tree_item_patch);
    registry.register("tooltip", tooltip_create, tooltip_patch);
    registry.register("menu_bar", menu_bar_create, menu_bar_patch);
    registry.register("menu", menu_create, menu_patch);
    registry.register("menu_item", menu_item_create, menu_item_patch);
    registry.register("image", image_create, image_patch);
    registry.register("banner", banner_create, landmark_patch);
    registry.register("navigation", navigation_create, landmark_patch);
    registry.register("main", main_create, landmark_patch);
    registry.register("content_info", content_info_create, landmark_patch);
    // The landmark tier's remaining words (M144, ADR-0148): the growth
    // the constructor words ride — four doors beside the four, the one
    // shared patch door unchanged.
    registry.register("complementary", complementary_create, landmark_patch);
    registry.register("form", form_create, landmark_patch);
    registry.register("region", region_create, landmark_patch);
    registry.register("search", search_create, landmark_patch);
    registry
}

/// The canvas element (`canvas { name: …, on_paint: Painter::new(…) }` —
/// M82): the paint seam's DSL door.
fn canvas_create(props: &ElementProps<'_>) -> crate::Canvas {
    let mut canvas = crate::Canvas::new(props.text("name").unwrap_or_default());
    if let Some(width) = props.number("width")
        && let Some(height) = props.number("height")
    {
        canvas = canvas.size(width, height);
    }
    canvas.painter_from(props.painter("on_paint"))
}

fn canvas_patch(canvas: &mut crate::Canvas, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        canvas.set_name(name);
        visual = true;
    }
    if let Some(width) = props.number("width")
        && let Some(height) = props.number("height")
    {
        canvas.set_size(width, height);
        visual = true;
    }
    // The painter's identity is the repaint contract (the widget's own
    // rule): a rebuilt closure reports visual, a hoisted painter
    // re-hangs quietly — the idle-frame property, at the canvas seam.
    // An absent `on_paint` leaves the hung painter alone (the patcher
    // applies only the props the view carries).
    if let Some(painter) = props.painter("on_paint") {
        visual |= canvas.set_painter(Some(painter));
    }
    visual
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
    if let Some(description) = props.text("description") {
        button = button.description(description);
    }
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
    // The description word (M95, ADR-0099): the tree's truth the pixels
    // never carry — the patch tier queues `A11Y` from the prop class, the
    // write itself reporting whether it moved (the idle-frame property).
    if let Some(description) = props.text("description") {
        button.set_description(Some(description.to_string()));
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

impl StyleMut for crate::RadioGroup {
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
    if let Some(content_height) = props.number("content_height") {
        list.handle().set_content_height(content_height);
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
    // The content's own height (the scroll seam's contract): the handle's
    // write queues the scroll tier's masks and names its clamps (ADR-0057),
    // so the patch ride carries the list's whole scroll story.
    if let Some(content_height) = props.number("content_height") {
        list.handle().set_content_height(content_height);
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

/// The checkbox's tri-state `state` word (M94, ADR-0098): `"on"` /
/// `"off"` / `"mixed"` — the vocabulary the tree announces, unknown words
/// refused by absence (the patcher applies only the props it understands,
/// the canvas painter's own rule).
fn toggle_state_word(word: &str) -> Option<alacer_core::widget::ToggleState> {
    match word {
        "on" => Some(alacer_core::widget::ToggleState::On),
        "off" => Some(alacer_core::widget::ToggleState::Off),
        "mixed" => Some(alacer_core::widget::ToggleState::Mixed),
        _ => None,
    }
}

fn checkbox_create(props: &ElementProps<'_>) -> crate::Checkbox {
    let mut checkbox = crate::Checkbox::new(props.text("name").unwrap_or_default());
    if let Some(word) = props.text("state")
        && let Some(state) = toggle_state_word(word)
    {
        checkbox = checkbox.state(state);
    } else if let Some(checked) = props.flag("checked") {
        checkbox = checkbox.checked(checked);
    }
    if let Some(handler) = props.flag_handler("on_toggle") {
        checkbox = checkbox.on_toggle(handler);
    }
    if let Some(description) = props.text("description") {
        checkbox = checkbox.description(description);
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
    if let Some(word) = props.text("state")
        && let Some(state) = toggle_state_word(word)
    {
        checkbox.set_state(state);
        visual = true;
    } else if let Some(checked) = props.flag("checked") {
        checkbox.set_checked(checked);
        visual = true;
    }
    // The description word (M95, ADR-0099): the tree's truth the pixels
    // never carry — the patch tier queues `A11Y` from the prop class, the
    // write itself reporting whether it moved (the idle-frame property).
    if let Some(description) = props.text("description") {
        checkbox.set_description(Some(description.to_string()));
    }
    if let Some(handler) = props.flag_handler("on_toggle") {
        checkbox.set_on_toggle(Some(handler));
    }
    visual
}

/// The radio element (`radio { name: …, checked: …, on_select: … }` —
/// M124): the radio tier's DSL door — the selection, the place word,
/// and the handler the exclusivity wiring rides.
fn radio_create(props: &ElementProps<'_>) -> crate::Radio {
    let mut radio = crate::Radio::new(props.text("name").unwrap_or_default());
    if let Some(checked) = props.flag("checked") {
        radio = radio.checked(checked);
    }
    if let Some(position) = prop_index(props, "position") {
        radio = radio.position_in_set(position);
    }
    if let Some(handler) = props.flag_handler("on_select") {
        radio = radio.on_select(handler);
    }
    if let Some(description) = props.text("description") {
        radio = radio.description(description);
    }
    if let Some(color) = props.color("accent") {
        radio = radio.accent(color);
    }
    radio
}

fn radio_patch(radio: &mut crate::Radio, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        radio.set_name(name);
        visual = true;
    }
    if let Some(checked) = props.flag("checked") {
        radio.set_checked(checked);
        visual = true;
    }
    // The place word (M119, ADR-0123): the tree's truth the pixels
    // never carry — the write itself reporting whether it moved (the
    // idle-frame property, at the set tier).
    if let Some(position) = prop_index(props, "position") {
        visual |= radio.set_position_in_set(Some(position));
    }
    // The description word (M95, ADR-0099): the patch tier queues
    // `A11Y` from the prop class, the write reporting the move.
    if let Some(description) = props.text("description") {
        radio.set_description(Some(description.to_string()));
    }
    if let Some(handler) = props.flag_handler("on_select") {
        radio.set_on_select(Some(handler));
    }
    visual
}

/// The radio group element (`radio_group { name: …, size: … }` — M124):
/// the named container the set reads as, the span word its carrier.
fn radio_group_create(props: &ElementProps<'_>) -> crate::RadioGroup {
    let mut group = crate::RadioGroup::new(props.text("name").unwrap_or_default());
    if let Some(size) = prop_index(props, "size") {
        group = group.size_of_set(size);
    }
    style_apply(group.style_mut(), props);
    group
}

fn radio_group_patch(group: &mut crate::RadioGroup, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        group.set_name(name);
        visual = true;
    }
    if let Some(size) = prop_index(props, "size") {
        visual |= group.set_size_of_set(Some(size));
    }
    style_apply(group.style_mut(), props);
    style_visual_diff(props) || visual
}

/// The link element (`link { name: …, on_open: … }` — M125): the link
/// tier's DSL door — the open act the app's navigation rides.
fn link_create(props: &ElementProps<'_>) -> crate::Link {
    let mut link = crate::Link::new(props.text("name").unwrap_or_default());
    if let Some(handler) = props.handler("on_open") {
        link = link.on_open(handler);
    }
    if let Some(description) = props.text("description") {
        link = link.description(description);
    }
    if let Some(color) = props.color("color") {
        link = link.text_color(color);
    }
    link
}

fn link_patch(link: &mut crate::Link, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        link.set_name(name);
        visual = true;
    }
    // The description word (M95, ADR-0099): the patch tier queues `A11Y`
    // from the prop class, the write reporting the move.
    if let Some(description) = props.text("description") {
        link.set_description(Some(description.to_string()));
    }
    if let Some(handler) = props.handler("on_open") {
        link.set_on_open(Some(handler));
    }
    visual
}

fn text_input_create(props: &ElementProps<'_>) -> crate::TextInput {
    let mut input = crate::TextInput::new(props.text("name").unwrap_or_default());
    if let Some(placeholder) = props.text("placeholder") {
        input = input.placeholder(placeholder);
    }
    if let Some(description) = props.text("description") {
        input = input.description(description);
    }
    if props.flag("secure") == Some(true) {
        input = input.secure();
    }
    // The purpose word (M132, ADR-0136): the search declaration rides
    // the same door the M100 secure word rides.
    if props.flag("search") == Some(true) {
        input = input.search_field(true);
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
        // The write that moved is the only mask the tier costs (the M21
        // idle-frame property); the view's a11y-*also* class adds the
        // re-announcement beside the paint (M96, ADR-0100).
        visual |= input.set_placeholder(placeholder);
    }
    // The description word (M95, ADR-0099): the tree's truth the pixels
    // never carry — the patch tier queues `A11Y` from the prop class, the
    // write itself reporting whether it moved (the idle-frame property).
    if let Some(description) = props.text("description") {
        input.set_description(Some(description.to_string()));
    }
    // The purpose word (M132, ADR-0136): the declaration moved live — the
    // write reporting the move, the patch tier queueing `A11Y` only from
    // it (the M21 idle-frame property, at the purpose tier).
    if let Some(search) = props.flag("search") {
        visual |= input.set_search_field(search);
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
    if let Some(color) = props.color("header_fill") {
        table = table.header_fill(color);
    }
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
    // The header fill rides `Option` both ways: a present prop sets the
    // override, an absent one restores the theme's panel role (ADR-0079) —
    // the write reporting whether the tier moved (the idle-frame property).
    if table.set_header_fill(props.color("header_fill")) {
        visual = true;
    }
    style_apply(table.style_mut(), props);
    visual
}

fn table_row_create(props: &ElementProps<'_>) -> crate::TableRow {
    let mut row = crate::TableRow::new();
    if let Some(selected) = props.flag("selected") {
        row = row.selected(selected);
    }
    style_apply(row.style_mut(), props);
    row
}

fn table_row_patch(row: &mut crate::TableRow, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(selected) = props.flag("selected")
        && row.set_selected(Some(selected))
    {
        visual = true;
    }
    style_apply(row.style_mut(), props);
    style_visual_diff(props) || visual
}

fn table_cell_create(props: &ElementProps<'_>) -> crate::TableCell {
    let cell = crate::TableCell::new(
        props.text("text").unwrap_or_default(),
        prop_index(props, "row").unwrap_or(0),
        prop_index(props, "column").unwrap_or(0),
    );
    if props.flag("strong").unwrap_or(false) {
        return cell.strong(true);
    }
    cell
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
    // The emphasis tier rides the flag both ways: `false` (or an absent
    // prop — the flag's default) is the honest baseline, the write
    // reporting whether the tier moved (the idle-frame property).
    if cell.set_strong(props.flag("strong").unwrap_or(false)) {
        visual = true;
    }
    visual
}

fn column_header_create(props: &ElementProps<'_>) -> crate::ColumnHeader {
    let mut header = crate::ColumnHeader::new(
        props.text("text").unwrap_or_default(),
        prop_index(props, "column").unwrap_or(0),
    );
    if let Some(direction) = props.text("sort").and_then(sort_word) {
        header = header.sort(direction);
    }
    header
}

fn column_header_patch(header: &mut crate::ColumnHeader, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(text) = props.text("text") {
        header.set_text(text);
        visual = true;
    }
    if let Some(word) = props.text("sort")
        && header.set_sort(sort_word(word))
    {
        visual = true;
    }
    visual
}

/// The sort word's direction — the M112 vocabulary at the DSL tier:
/// `"ascending"`, `"descending"`, and `"other"` name the order
/// AccessKit's `SortDirection` property carries (ARIA's `aria-sort`);
/// `"none"` (and any unknown word) is the honest absence — the header
/// declares no sort, the way a view states `selected: false` instead of
/// omitting the word so the patch can clear it.
fn sort_word(word: &str) -> Option<alacer_core::SortDirection> {
    match word {
        "ascending" => Some(alacer_core::SortDirection::Ascending),
        "descending" => Some(alacer_core::SortDirection::Descending),
        "other" => Some(alacer_core::SortDirection::Other),
        _ => None,
    }
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

/// The meter element (`meter { name: …, min: …, max: …, value: … }` —
/// M133): the gauge tier's DSL door — the scalar measurement the AT
/// reads as a gauge, the scale the app declares, the reading riding
/// the value tier's own words.
fn meter_create(props: &ElementProps<'_>) -> crate::Meter {
    let name = props.text("name").unwrap_or_default();
    let min = props.number("min").unwrap_or(0.0);
    let max = props.number("max").unwrap_or(100.0);
    // The two doors the value tier's report half ships (the M21
    // progress rule, at the meter tier): the reactive bridge (a
    // `binding` prop — the flux bridge owns the reading, §6.1) or the
    // static value. The binding wins, the bound label's rule.
    if let Some(binding) = props.number_binding() {
        return crate::Meter::bound(name, min, max, binding);
    }
    let mut meter = crate::Meter::new(name, min, max);
    if let Some(value) = props.number("value") {
        meter = meter.value(value);
    }
    if let Some(fill) = props.color("fill") {
        meter = meter.fill(fill);
    }
    meter
}

fn meter_patch(meter: &mut crate::Meter, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        meter.set_name(name);
        visual = true;
    }
    if let Some(binding) = props.number_binding() {
        // The bridge owns a bound gauge's reading: re-hanging the same
        // binding is a no-op, never a visual change (the bound label's
        // rule, at the value tier).
        let _ = binding;
        return visual;
    }
    if let Some(value) = props.number("value") {
        // The widget's own answer is the idle-frame property: a write
        // that lands where the reading already is reports `false` — no
        // masks, an idle frame (the M21 progress rule, at the meter
        // tier).
        visual |= meter.set_value(value);
    }
    visual
}

/// The heading element (`heading { name: …, level: … }` — M126): the
/// heading tier's DSL door — the section title, the depth the outline
/// walks, the words the tree carries.
fn heading_create(props: &ElementProps<'_>) -> crate::Heading {
    let mut heading = crate::Heading::new(props.text("name").unwrap_or_default());
    if let Some(level) = prop_index(props, "level") {
        heading = heading.level(level);
    }
    if let Some(language) = props.text("language") {
        heading = heading.language(language);
    }
    if let Some(color) = props.color("color") {
        heading = heading.color(color);
    }
    if let Some(size) = props.number("font_size") {
        heading = heading.font_size(size);
    }
    heading
}

fn heading_patch(heading: &mut crate::Heading, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        heading.set_name(name);
        visual = true;
    }
    // The depth word (M113, ADR-0117): the write reporting the move —
    // the idle write queues nothing (the M21 rule, at the heading
    // tier).
    if let Some(level) = prop_index(props, "level") {
        let _ = heading.set_level(Some(level));
    }
    // The language word (M123, ADR-0127): the write reporting the
    // move, the honest absence the withdrawal restores.
    if let Some(language) = props.text("language") {
        let _ = heading.set_language(Some(language.to_string()));
    }
    if let Some(color) = props.color("color") {
        heading.set_color(color);
        visual = true;
    }
    if let Some(size) = props.number("font_size") {
        heading.set_font_size(size);
        visual = true;
    }
    visual
}

/// The tab element (`tab { name: …, selected: …, on_select: … }` —
/// M127): the tab tier's DSL door — the page word, the selection, the
/// select handler the exclusivity rides.
fn tab_create(props: &ElementProps<'_>) -> crate::Tab {
    let mut tab = crate::Tab::new(props.text("name").unwrap_or_default());
    if let Some(selected) = props.flag("selected") {
        tab = tab.selected(selected);
    }
    if let Some(handler) = props.flag_handler("on_select") {
        tab = tab.on_select(handler);
    }
    if let Some(description) = props.text("description") {
        tab = tab.description(description);
    }
    if let Some(color) = props.color("accent") {
        tab = tab.accent(color);
    }
    tab
}

fn tab_patch(tab: &mut crate::Tab, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        tab.set_name(name);
        visual = true;
    }
    // The description word (M95, ADR-0099): the patch tier queues
    // `A11Y` from the prop class, the write reporting the move.
    if let Some(description) = props.text("description") {
        tab.set_description(Some(description.to_string()));
    }
    if let Some(handler) = props.flag_handler("on_select") {
        tab.set_on_select(Some(handler));
    }
    // The programmatic selection door: the patcher applies the prop
    // the view carries, the user's doors never deselecting (the
    // radio's own rule at the page tier).
    if let Some(selected) = props.flag("selected") {
        tab.set_selected(selected);
    }
    visual
}

/// The tab list element (`tab_list { name: … }` — M127): the strip the
/// pages read as, the pages' horizontal layout — never a second
/// channel to them.
fn tab_list_create(props: &ElementProps<'_>) -> crate::TabList {
    let mut strip = crate::TabList::new(props.text("name").unwrap_or_default());
    style_apply(strip.style_mut(), props);
    strip
}

fn tab_list_patch(strip: &mut crate::TabList, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        strip.set_name(name);
        visual = true;
    }
    style_apply(strip.style_mut(), props);
    style_visual_diff(props) || visual
}

/// The spin button element (`spin_button { name: …, min: …, max: … }` —
/// M128): the bounded number field the AT walks by arrows — the value
/// tier's discrete write half, the DSL door the widget set shares.
fn spin_button_create(props: &ElementProps<'_>) -> crate::SpinButton {
    let mut field = crate::SpinButton::new(
        props.text("name").unwrap_or_default(),
        props.number("min").unwrap_or(0.0),
        props.number("max").unwrap_or(100.0),
    );
    if let Some(step) = props.number("step") {
        field = field.step(step);
    }
    if let Some(value) = props.number("value") {
        field = field.value(value);
    }
    if let Some(handler) = props.number_handler("on_change") {
        let mut handler = handler;
        field = field.on_change(move |value: f32| handler(value));
    }
    if let Some(description) = props.text("description") {
        field = field.description(description);
    }
    if let Some(color) = props.color("accent") {
        field = field.accent(color);
    }
    field
}

fn spin_button_patch(field: &mut crate::SpinButton, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        field.set_name(name);
        visual = true;
    }
    // The description word (M95, ADR-0099): the patch tier queues
    // `A11Y` from the prop class, the write reporting the move.
    if let Some(description) = props.text("description") {
        field.set_description(Some(description.to_string()));
    }
    if let Some(value) = props.number("value") {
        // The programmatic write fires no handler (the widget's own
        // contract: state, not a user act) and clamps/snaps inside the
        // widget — the funnel every door rides. The write moved the
        // value only when the funnel changed it (the widget's own
        // `total_cmp` equality, never `f32::eq`).
        let before = field.current_value();
        field.set_value(value);
        visual |= !same_value(before, field.current_value());
    }
    visual
}

/// The toolbar element (`toolbar { name: … }` — M129): the named strip
/// the tools read as — the tools' horizontal layout, never a second
/// channel to them.
fn toolbar_create(props: &ElementProps<'_>) -> crate::Toolbar {
    let mut strip = crate::Toolbar::new(props.text("name").unwrap_or_default());
    style_apply(strip.style_mut(), props);
    strip
}

fn toolbar_patch(strip: &mut crate::Toolbar, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        strip.set_name(name);
        visual = true;
    }
    style_apply(strip.style_mut(), props);
    style_visual_diff(props) || visual
}

/// The switch element (`switch { name: …, on: …, on_toggle: … }` —
/// M130): the instant on/off control's DSL door — the binary pair the
/// tree announces, the handler the app's wiring rides.
fn switch_create(props: &ElementProps<'_>) -> crate::Switch {
    let mut switch = crate::Switch::new(props.text("name").unwrap_or_default());
    if let Some(on) = props.flag("on") {
        switch = switch.on(on);
    }
    if let Some(handler) = props.flag_handler("on_toggle") {
        switch = switch.on_toggle(handler);
    }
    if let Some(description) = props.text("description") {
        switch = switch.description(description);
    }
    if let Some(color) = props.color("accent") {
        switch = switch.accent(color);
    }
    style_apply(switch.style_mut(), props);
    switch
}

fn switch_patch(switch: &mut crate::Switch, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        switch.set_name(name);
        visual = true;
    }
    // The on/off word (M130, ADR-0134): the programmatic write reporting
    // whether it moved — the patch tier queues masks only from the move,
    // the idle write the patch vocabulary's honest floor.
    if let Some(on) = props.flag("on") {
        visual |= switch.set_on(on);
    }
    // The description word (M95, ADR-0099): the tree's truth the pixels
    // never carry — the patch tier queues `A11Y` from the prop class, the
    // write itself reporting whether it moved (the idle-frame property).
    if let Some(description) = props.text("description") {
        switch.set_description(Some(description.to_string()));
    }
    if let Some(handler) = props.flag_handler("on_toggle") {
        switch.set_on_toggle(Some(handler));
    }
    style_apply(switch.style_mut(), props);
    style_visual_diff(props) || visual
}

/// The tab panel element (`tab_panel { name: … }` — M131): the named
/// pane the selected page's content lives in — the content's vertical
/// layout, never a second channel to it.
fn tab_panel_create(props: &ElementProps<'_>) -> crate::TabPanel {
    let mut pane = crate::TabPanel::new(props.text("name").unwrap_or_default());
    if let Some(description) = props.text("description") {
        pane = pane.description(description);
    }
    style_apply(pane.style_mut(), props);
    pane
}

fn tab_panel_patch(pane: &mut crate::TabPanel, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        pane.set_name(name);
        visual = true;
    }
    // The description word (M95, ADR-0099): the patch tier queues
    // `A11Y` from the prop class, the write reporting the move.
    if let Some(description) = props.text("description") {
        pane.set_description(Some(description.to_string()));
    }
    style_apply(pane.style_mut(), props);
    style_visual_diff(props) || visual
}

/// The dialog element (`dialog { name: … }` — M134): the dialog tier's
/// DSL door — the window the modal vocabulary speaks as "this is a
/// dialog", the content's vertical layout, never a second channel to
/// it.
fn dialog_create(props: &ElementProps<'_>) -> crate::Dialog {
    let mut window = crate::Dialog::new(props.text("name").unwrap_or_default());
    if let Some(description) = props.text("description") {
        window = window.description(description);
    }
    style_apply(window.style_mut(), props);
    window
}

fn dialog_patch(window: &mut crate::Dialog, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        window.set_name(name);
        visual = true;
    }
    // The description word (M95, ADR-0099): the patch tier queues
    // `A11Y` from the prop class, the write reporting the move.
    if let Some(description) = props.text("description") {
        window.set_description(Some(description.to_string()));
    }
    style_apply(window.style_mut(), props);
    style_visual_diff(props) || visual
}

/// The tree element (`tree { name: … }` — M135): the tree tier's
/// container DSL door — the collection the AT walks by disclosure and
/// depth, the members' vertical layout, never a second channel to
/// them.
fn tree_create(props: &ElementProps<'_>) -> crate::Tree {
    let mut collection = crate::Tree::new(props.text("name").unwrap_or_default());
    if let Some(description) = props.text("description") {
        collection = collection.description(description);
    }
    if let Some(multiselectable) = props.flag("multiselectable") {
        collection = collection.multiselectable(multiselectable);
    }
    if let Some(busy) = props.flag("busy") {
        collection = collection.busy(busy);
    }
    if let Some(size_of_set) = prop_index(props, "size_of_set") {
        collection = collection.size_of_set(size_of_set);
    }
    style_apply(collection.style_mut(), props);
    collection
}

fn tree_patch(collection: &mut crate::Tree, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        collection.set_name(name);
        visual = true;
    }
    // The description word (M95, ADR-0099): the patch tier queues
    // `A11Y` from the prop class, the write reporting the move.
    if let Some(description) = props.text("description") {
        collection.set_description(Some(description.to_string()));
    }
    if let Some(multiselectable) = props.flag("multiselectable") {
        collection.set_multiselectable(multiselectable);
    }
    if let Some(busy) = props.flag("busy") {
        collection.set_busy(busy);
    }
    if let Some(size_of_set) = prop_index(props, "size_of_set") {
        collection.set_size_of_set(Some(size_of_set));
    }
    style_apply(collection.style_mut(), props);
    style_visual_diff(props) || visual
}

/// The tree-item element (`tree_item { name: … }` — M135): the tree
/// tier's member DSL door — the branch or leaf word, the collection
/// words riding the tiers the tree already published.
fn tree_item_create(props: &ElementProps<'_>) -> crate::TreeItem {
    let mut member = crate::TreeItem::new(props.text("name").unwrap_or_default());
    if let Some(selected) = props.flag("selected") {
        member = member.selected(selected);
    }
    if let Some(expanded) = props.flag("expanded") {
        member = member.expanded(expanded);
    }
    if let Some(level) = prop_index(props, "level") {
        member = member.level(level);
    }
    if let Some(position_in_set) = prop_index(props, "position_in_set") {
        member = member.position_in_set(position_in_set);
    }
    style_apply(member.style_mut(), props);
    member
}

fn tree_item_patch(member: &mut crate::TreeItem, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        member.set_name(name);
        visual = true;
    }
    if let Some(selected) = props.flag("selected") {
        member.set_selected(Some(selected));
    }
    if let Some(expanded) = props.flag("expanded") {
        member.set_expanded(Some(expanded));
    }
    if let Some(level) = prop_index(props, "level") {
        member.set_level(Some(level));
    }
    if let Some(position_in_set) = prop_index(props, "position_in_set") {
        member.set_position_in_set(Some(position_in_set));
    }
    style_apply(member.style_mut(), props);
    style_visual_diff(props) || visual
}

/// The tooltip element (`tooltip { name: … }` — M136): the named help
/// bubble the pointer pair summons — the help's vertical layout, never
/// a second channel to it.
fn tooltip_create(props: &ElementProps<'_>) -> crate::Tooltip {
    let mut bubble = crate::Tooltip::new(props.text("name").unwrap_or_default());
    style_apply(bubble.style_mut(), props);
    bubble
}

fn tooltip_patch(bubble: &mut crate::Tooltip, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        bubble.set_name(name);
        visual = true;
    }
    style_apply(bubble.style_mut(), props);
    style_visual_diff(props) || visual
}

/// The menu-bar element (`menu_bar { name: … }` — M137): the menu
/// tier's strip door, the menus riding its arena slots.
fn menu_bar_create(props: &ElementProps<'_>) -> crate::MenuBar {
    let mut strip = crate::MenuBar::new(props.text("name").unwrap_or_default());
    style_apply(strip.style_mut(), props);
    strip
}

fn menu_bar_patch(strip: &mut crate::MenuBar, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        strip.set_name(name);
        visual = true;
    }
    style_apply(strip.style_mut(), props);
    style_visual_diff(props) || visual
}

/// The menu element (`menu { name: … }` — M137): the menu tier's
/// summoned-list door, the entries riding its arena slots (presence the
/// app's wiring — the view mounts the list exactly while the menu is up).
fn menu_create(props: &ElementProps<'_>) -> crate::Menu {
    let mut list = crate::Menu::new(props.text("name").unwrap_or_default());
    style_apply(list.style_mut(), props);
    list
}

fn menu_patch(list: &mut crate::Menu, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        list.set_name(name);
        visual = true;
    }
    style_apply(list.style_mut(), props);
    style_visual_diff(props) || visual
}

/// The menu-item element (`menu_item { text: …, on_click: … }` — M137):
/// the menu tier's entry door, the activation funnel the button
/// element's own.
fn menu_item_create(props: &ElementProps<'_>) -> crate::MenuItem {
    let mut entry = crate::MenuItem::new(props.text("text").unwrap_or_default());
    if let Some(handler) = props.handler("on_click") {
        entry = entry.on_click(handler);
    }
    style_apply(entry.style_mut(), props);
    entry
}

fn menu_item_patch(entry: &mut crate::MenuItem, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(text) = props.text("text") {
        entry.set_label(text);
        visual = true;
    }
    if let Some(handler) = props.handler("on_click") {
        entry.set_on_click(Some(handler));
    }
    style_apply(entry.style_mut(), props);
    style_visual_diff(props) || visual
}

/// The sampling prop's word (`sampling: "nearest" | "bilinear"`): the
/// raster tier's sampling enum spelled the checkbox `state` word's way —
/// unknown words refused by absence (the patcher applies only the props
/// it understands).
fn sampling_word(word: &str) -> Option<alacer_core::scene::ImageSampling> {
    match word {
        "nearest" => Some(alacer_core::scene::ImageSampling::Nearest),
        "bilinear" => Some(alacer_core::scene::ImageSampling::Bilinear),
        _ => None,
    }
}

/// The image element (`image { name: …, description: …, pixels: …,
/// sampling: … }` — M138): the image tier's DSL door, the word the
/// pixels' app owns — the raster word riding the same door the tier
/// already had.
fn image_create(props: &ElementProps<'_>) -> crate::Image {
    let mut picture = crate::Image::new(props.text("name").unwrap_or_default());
    if let Some(description) = props.text("description") {
        picture = picture.description(description);
    }
    if let Some(pixels) = props.pixels("pixels") {
        picture = picture.pixels(pixels.bytes, pixels.width, pixels.height);
    }
    if let Some(word) = props.text("sampling")
        && let Some(sampling) = sampling_word(word)
    {
        picture = picture.sampling(sampling);
    }
    style_apply(picture.style_mut(), props);
    picture
}

fn image_patch(picture: &mut crate::Image, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        picture.set_name(name);
        visual = true;
    }
    // The description word (M95, ADR-0099): the tree's truth the pixels
    // never carry — the patch tier queues `A11Y` from the prop class, the
    // write itself reporting whether it moved (the idle-frame property).
    if let Some(description) = props.text("description") {
        picture.set_description(Some(description.to_string()));
    }
    // The raster word (the raster tier): the buffer's identity is the
    // repaint contract (the painter's own rule) — a rebuilt buffer reports
    // visual, a write that lands where the bytes already are queues
    // nothing (the idle-frame property, at the raster tier).
    if let Some(pixels) = props.pixels("pixels") {
        visual |= picture.set_pixels(pixels.bytes, pixels.width, pixels.height);
    }
    if let Some(word) = props.text("sampling")
        && let Some(sampling) = sampling_word(word)
    {
        visual |= picture.set_sampling(sampling);
    }
    style_apply(picture.style_mut(), props);
    style_visual_diff(props) || visual
}

/// The landmark element doors (M143): the page-structure words the AT's
/// landmarks list walks — `banner { name: … }`, `navigation { name: … }`,
/// `main { name: … }`, `content_info { name: … }` — one shape, four
/// facts, the word the constructor's own declaration.
fn banner_create(props: &ElementProps<'_>) -> crate::Landmark {
    let mut region = crate::Landmark::banner(props.text("name").unwrap_or_default());
    style_apply(region.style_mut(), props);
    region
}

fn navigation_create(props: &ElementProps<'_>) -> crate::Landmark {
    let mut region = crate::Landmark::navigation(props.text("name").unwrap_or_default());
    style_apply(region.style_mut(), props);
    region
}

fn main_create(props: &ElementProps<'_>) -> crate::Landmark {
    let mut region = crate::Landmark::main(props.text("name").unwrap_or_default());
    style_apply(region.style_mut(), props);
    region
}

fn content_info_create(props: &ElementProps<'_>) -> crate::Landmark {
    let mut region = crate::Landmark::content_info(props.text("name").unwrap_or_default());
    style_apply(region.style_mut(), props);
    region
}

/// The landmark tier's remaining element doors (M144, ADR-0148): the
/// ARIA facts ADR-0147 pre-registered — `complementary { name: … }`,
/// `form { name: … }`, `region { name: … }`, `search { name: … }` —
/// one shape beside the four, the word the constructor's own
/// declaration.
fn complementary_create(props: &ElementProps<'_>) -> crate::Landmark {
    let mut region = crate::Landmark::complementary(props.text("name").unwrap_or_default());
    style_apply(region.style_mut(), props);
    region
}

fn form_create(props: &ElementProps<'_>) -> crate::Landmark {
    let mut region = crate::Landmark::form(props.text("name").unwrap_or_default());
    style_apply(region.style_mut(), props);
    region
}

fn region_create(props: &ElementProps<'_>) -> crate::Landmark {
    let mut region = crate::Landmark::region(props.text("name").unwrap_or_default());
    style_apply(region.style_mut(), props);
    region
}

fn search_create(props: &ElementProps<'_>) -> crate::Landmark {
    let mut region = crate::Landmark::search(props.text("name").unwrap_or_default());
    style_apply(region.style_mut(), props);
    region
}

/// The landmarks' shared patch door: the word is the one write the
/// furniture takes (the write that lands where the word already is
/// queues nothing — the idle-frame property, at the landmark tier).
fn landmark_patch(region: &mut crate::Landmark, props: &ElementProps<'_>) -> bool {
    let mut visual = false;
    if let Some(name) = props.text("name") {
        region.set_name(name);
        visual = true;
    }
    style_apply(region.style_mut(), props);
    style_visual_diff(props) || visual
}

#[cfg(test)]
mod tests {
    use super::default_registry;
    use crate::theme;
    use alacer_core::color::Color;
    use alacer_core::widget::Widget as _;
    use alacer_view::ViewNode;

    /// Downcasts the erased widget to `Checkbox` (the test's introspection).
    fn downcast_checkbox(widget: &mut dyn alacer_core::Widget) -> Option<&mut crate::Checkbox> {
        widget.as_any_mut().downcast_mut::<crate::Checkbox>()
    }

    /// Downcasts the erased widget to `Radio` (the test's introspection).
    fn downcast_radio(widget: &mut dyn alacer_core::Widget) -> Option<&mut crate::Radio> {
        widget.as_any_mut().downcast_mut::<crate::Radio>()
    }

    /// Downcasts the erased widget to `SpinButton` (the test's
    /// introspection).
    fn downcast_spin_button(
        widget: &mut dyn alacer_core::Widget,
    ) -> Option<&mut crate::SpinButton> {
        widget.as_any_mut().downcast_mut::<crate::SpinButton>()
    }

    /// Downcasts the erased widget to `Switch` (the test's introspection).
    fn downcast_switch(widget: &mut dyn alacer_core::Widget) -> Option<&mut crate::Switch> {
        widget.as_any_mut().downcast_mut::<crate::Switch>()
    }

    #[test]
    fn switch_props_create_and_patch_in_place() {
        let registry = default_registry();
        let node = ViewNode::element("switch")
            .prop("name", "Wi-Fi")
            .prop("on", true)
            .prop("description", "Turns the radio off and on");
        let mut widget = registry.create(&node);
        assert_eq!(widget.label().resolve(), "Wi-Fi");
        assert_eq!(widget.role(), alacer_core::widget::Role::Switch);
        assert!(
            downcast_switch(widget.as_mut()).is_some_and(|switch| switch.is_on()),
            "the create write lands the mounted-on state"
        );

        // The on/off patch: the programmatic write in place, reporting
        // the move it lands (the M21 idle-frame property, at the patch
        // tier).
        let updated = ViewNode::element("switch").prop("on", false);
        assert!(
            registry.patch(widget.as_mut(), &updated),
            "the moved write lands — the patch reports it"
        );
        assert!(
            downcast_switch(widget.as_mut()).is_some_and(|switch| !switch.is_on()),
            "the write landed off"
        );

        // The idle write: the patch that lands where the on/off already
        // is reports nothing to repaint — no masks, an idle frame.
        let idle = ViewNode::element("switch").prop("on", false);
        assert!(
            !registry.patch(widget.as_mut(), &idle),
            "the write landed where the flag already is — nothing to repaint"
        );

        // The handler word rides the patch tier: the app's wiring swaps
        // live (the radio's own rule, at the switch tier); the write
        // itself moves no pixels — the handler is not the paint.
        let rewired = ViewNode::element("switch").prop(
            "on_toggle",
            alacer_view::PropValue::FlagHandler(std::rc::Rc::new(std::cell::RefCell::new(
                move |_on: bool| {},
            ))),
        );
        let _ = registry.patch(widget.as_mut(), &rewired);
        let retoggled = ViewNode::element("switch").prop("on", true);
        assert!(
            registry.patch(widget.as_mut(), &retoggled),
            "the moved write still lands after the rewire"
        );
    }

    #[test]
    fn spin_button_props_create_and_patch_in_place() {
        let registry = default_registry();
        let node = ViewNode::element("spin_button")
            .prop("name", "Quantity")
            .prop("min", 1.0_f32)
            .prop("max", 12.0_f32)
            .prop("step", 2.0_f32)
            .prop("value", 5.0_f32);
        let mut widget = registry.create(&node);
        assert_eq!(widget.label().resolve(), "Quantity");
        assert!(
            downcast_spin_button(widget.as_mut())
                .is_some_and(|f| same_value(f.current_value(), 5.0)),
            "the create write rides the widget's own funnel (clamped and snapped)"
        );
        assert_eq!(widget.role(), alacer_core::widget::Role::SpinButton);

        // The value patch: clamped and snapped like every write, in place.
        let updated = ViewNode::element("spin_button").prop("value", 99.0_f32);
        assert!(
            registry.patch(widget.as_mut(), &updated),
            "a value patch is visual"
        );
        assert!(
            downcast_spin_button(widget.as_mut())
                .is_some_and(|f| same_value(f.current_value(), 12.0)),
            "the patch write clamps to the range the widget owns"
        );

        // A write that lands where the value already is: no visual change
        // (the M20 idle-frame property, at the view seam).
        let again = ViewNode::element("spin_button").prop("value", 12.0_f32);
        assert!(
            !registry.patch(widget.as_mut(), &again),
            "a no-op write patches to no masks"
        );

        // The name patch: the label the a11y tree carries.
        let renamed = ViewNode::element("spin_button").prop("name", "Copies");
        assert!(registry.patch(widget.as_mut(), &renamed));
        assert_eq!(widget.label().resolve(), "Copies", "patched in place");
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
            // The M124 radio tier.
            "radio",
            "radio_group",
            // The M125 link tier.
            "link",
            // The M126 heading tier.
            "heading",
            // The M127 tab tier.
            "tab",
            "tab_list",
            // The M128 spin-button tier.
            "spin_button",
            // The M129 toolbar tier.
            "toolbar",
            // The M130 switch tier.
            "switch",
            // The M131 tab-panel tier.
            "tab_panel",
            // The M133 meter tier.
            "meter",
            // The M134 dialog tier.
            "dialog",
            // The M135 tree tier.
            "tree",
            "tree_item",
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

    #[test]
    fn meter_props_create_and_patch_and_the_idle_frame() {
        let registry = default_registry();
        let node = ViewNode::element("meter")
            .prop("name", "Disk usage")
            .prop("min", 0.0_f32)
            .prop("max", 100.0_f32)
            .prop("value", 62.0_f32);
        let mut widget = registry.create(&node);
        assert_eq!(widget.label().resolve(), "Disk usage");
        assert_eq!(widget.role(), alacer_core::widget::Role::Meter);
        assert!(
            downcast_meter(widget.as_mut()).is_some_and(|m| same_value(m.current_value(), 62.0)),
            "the reported reading lands at create"
        );

        let updated = ViewNode::element("meter").prop("value", 48.0_f32);
        assert!(registry.patch(widget.as_mut(), &updated));
        assert!(
            downcast_meter(widget.as_mut()).is_some_and(|m| same_value(m.current_value(), 48.0)),
            "the reading patch lands in place"
        );

        // A write that lands where the reading already is queues no
        // masks (the M21 idle-frame property at the meter tier — the
        // patcher's `false` is exactly the widget's own `set_value`
        // answer).
        let again = ViewNode::element("meter").prop("value", 48.0_f32);
        assert!(!registry.patch(widget.as_mut(), &again));

        // The name's patch is the visual truth it always was.
        let renamed = ViewNode::element("meter").prop("name", "Disk free");
        assert!(registry.patch(widget.as_mut(), &renamed));
        assert_eq!(widget.label().resolve(), "Disk free", "patched in place");
    }

    #[test]
    fn meter_binds_its_reading_and_rehangs_idempotently() {
        let registry = default_registry();
        let binding = alacer_core::flux::RenderBinding::new(87.0_f32);
        let node = ViewNode::element("meter")
            .prop("name", "Battery")
            .prop("min", 0.0_f32)
            .prop("max", 100.0_f32)
            .prop("binding", binding.clone());
        let mut widget = registry.create(&node);
        assert!(
            downcast_meter(widget.as_mut()).is_some_and(|m| same_value(m.current_value(), 87.0)),
            "the number binding rides the bound constructor"
        );

        // The flux bridge owns a bound gauge's reading (§6.1): a rebuild
        // re-hangs the same binding slot — a no-op patch, never a visual
        // change (the bound label's rule, at the value tier).
        let rebuilt = ViewNode::element("meter").prop("binding", binding);
        assert!(!registry.patch(widget.as_mut(), &rebuilt));
    }

    #[test]
    fn dialog_props_create_and_patch_in_place() {
        let registry = default_registry();
        let node = ViewNode::element("dialog")
            .prop("name", "Confirm publish")
            .prop("description", "The file goes live");
        let mut widget = registry.create(&node);
        assert_eq!(widget.label().resolve(), "Confirm publish");
        assert_eq!(widget.role(), alacer_core::widget::Role::Dialog);
        assert_eq!(
            widget.description(),
            Some("The file goes live"),
            "the help text lands at create"
        );

        // The name's patch is the visual truth it always was (the
        // container's word is the app's declaration).
        let renamed = ViewNode::element("dialog").prop("name", "Confirm deploy");
        assert!(registry.patch(widget.as_mut(), &renamed));
        assert_eq!(
            widget.label().resolve(),
            "Confirm deploy",
            "patched in place"
        );

        // The description write reporting the move: the idle write —
        // the text landing where it already is — queues nothing (the
        // M21 idle-frame property, at the description tier).
        let again = ViewNode::element("dialog").prop("description", "The file goes live");
        assert!(!registry.patch(widget.as_mut(), &again));
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

    fn downcast_meter(widget: &mut dyn alacer_core::Widget) -> Option<&mut crate::Meter> {
        widget.as_any_mut().downcast_mut::<crate::Meter>()
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
    fn radio_props_create_and_patch_in_place() {
        let registry = default_registry();
        let selected = std::rc::Rc::new(std::cell::RefCell::new(false));
        let sink = selected.clone();
        let node = ViewNode::element("radio")
            .prop("name", "Express")
            .prop("checked", false)
            .prop("position", 2.0_f32)
            .prop(
                "on_select",
                alacer_view::PropValue::FlagHandler(std::rc::Rc::new(std::cell::RefCell::new(
                    move |picked: bool| *sink.borrow_mut() = picked,
                ))),
            );
        let mut widget = registry.create(&node);
        assert_eq!(widget.label().resolve(), "Express");
        assert_eq!(
            alacer_core::widget::Widget::position_in_set(widget.as_ref()),
            Some(2),
            "the place word rides the constructor"
        );
        assert!(
            downcast_radio(widget.as_mut()).is_some_and(|r| !r.is_checked()),
            "created unchecked"
        );

        let updated = ViewNode::element("radio").prop("checked", true);
        assert!(
            registry.patch(widget.as_mut(), &updated),
            "checked is visual"
        );
        assert!(
            downcast_radio(widget.as_mut()).is_some_and(|r| r.is_checked()),
            "patched in place"
        );

        // The view-driven select path: the flag handler rides the widget,
        // and the redundant ask is the idle frame (no visual, no fire).
        let mut ctx = alacer_core::events::EventCtx::new();
        let up = alacer_core::events::PointerEvent::Up {
            position: alacer_core::geom::Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
        };
        let _ = widget.on_pointer_event(&mut ctx, &up);
        assert!(
            downcast_radio(widget.as_mut()).is_some_and(|r| r.is_checked()),
            "the user's door never deselects — the redundant ask moved nothing"
        );
        assert!(
            !*selected.borrow(),
            "the redundant ask fired no handler (already selected)"
        );

        // The place patch: the word moved is the only mask the tier costs.
        let moved = ViewNode::element("radio").prop("position", 3.0_f32);
        assert!(registry.patch(widget.as_mut(), &moved), "the place moved");
        let idle = ViewNode::element("radio").prop("position", 3.0_f32);
        assert!(
            !registry.patch(widget.as_mut(), &idle),
            "the write that lands where the place already is patches to no masks"
        );
    }

    #[test]
    fn radio_group_props_carry_the_span_word() {
        let registry = default_registry();
        let node = ViewNode::element("radio_group")
            .prop("name", "Shipping")
            .prop("size", 3.0_f32)
            .prop("gap", 6.0_f32);
        let mut widget = registry.create(&node);
        assert_eq!(widget.label().resolve(), "Shipping");
        assert_eq!(
            alacer_core::widget::Widget::size_of_set(widget.as_ref()),
            Some(3)
        );
        assert_eq!(widget.style().gap, 6.0);

        // The span patch: moved answers visual, the idle write does not.
        let grown = ViewNode::element("radio_group").prop("size", 4.0_f32);
        assert!(registry.patch(widget.as_mut(), &grown));
        let again = ViewNode::element("radio_group").prop("size", 4.0_f32);
        assert!(!registry.patch(widget.as_mut(), &again));
    }

    #[test]
    fn link_props_create_and_patch_in_place() {
        let registry = default_registry();
        let opened = std::rc::Rc::new(std::cell::RefCell::new(false));
        let sink = opened.clone();
        let node = ViewNode::element("link")
            .prop("name", "Read the architecture")
            .prop("description", "Opens the docs site")
            .prop(
                "on_open",
                alacer_view::PropValue::Handler(std::rc::Rc::new(std::cell::RefCell::new(
                    move || *sink.borrow_mut() = true,
                ))),
            );
        let mut widget = registry.create(&node);
        assert_eq!(widget.label().resolve(), "Read the architecture");

        // The view-driven open path: the handler rides the widget.
        let mut ctx = alacer_core::events::EventCtx::new();
        let up = alacer_core::events::PointerEvent::Up {
            position: alacer_core::geom::Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
        };
        let _ = widget.on_pointer_event(&mut ctx, &up);
        assert!(
            *opened.borrow(),
            "the release opened — the handler observed the activation"
        );

        // The name patch: the label the a11y tree carries.
        let renamed = ViewNode::element("link").prop("name", "API reference");
        assert!(registry.patch(widget.as_mut(), &renamed));
        assert_eq!(
            widget.label().resolve(),
            "API reference",
            "patched in place"
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
        // The absent `color` prop resolves the canon surface (ADR-0079's
        // pin): the registry default is the palette's own surface role,
        // byte-identical to what 0.16 painted. The comparison builds the
        // same panel the registry created — the `height` prop's 12.0, the
        // color absent — so the only difference under test is the color.
        assert_eq!(
            crate::Panel::new(12.0, crate::theme::SURFACE).style(),
            widget.style(),
            "the absent color prop's default is the canon surface"
        );
        assert_eq!(
            alacer_core::theme::Palette::CANON.surface,
            theme::SURFACE,
            "the canon palette pins the surface the registry borrows"
        );
    }
}
