//! The data-table canon (M6): `Table` / `TableRow` / `TableCell` /
//! `ColumnHeader` as retained composition.
//!
//! A `Table` is a column container carrying its grid geometry — rows ×
//! columns, kept in step by the row-count API as the app mounts and removes
//! rows. `ColumnHeader` and `TableCell` are named text cells that report
//! their zero-based grid positions through [`Widget::table_slot`], so the
//! AccessKit update carries a *navigable* grid (row/column counts and
//! indexes), not just a pile of roles.

use alacer_core::announce::Politeness;
use alacer_core::color::Color;
use alacer_core::geom::Point;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{LabelSource, Role, TableSlot, Widget};

/// A data table: headers + rows, geometry carried by the widget (M6).
///
/// Children are mounted into the arena by the app (the retained model); the
/// widget's `table_slot` reports the *declared* geometry, kept in sync via
/// [`Table::set_row_count`] as rows come and go.
pub struct Table {
    name: String,
    columns: usize,
    rows: usize,
    style: Style,
    /// The app's header-fill override; `None` borrows the theme's panel
    /// role at paint time (ADR-0079).
    header_fill: Option<Color>,
    /// The multiselectable tier's word (M109, ADR-0113): whether the
    /// collection allows more than one pick — `false` the honest
    /// absence.
    multiselectable: bool,
    /// The busy tier's word (M117, ADR-0121): whether the collection's
    /// subtree is being updated — `false` the honest absence.
    busy: bool,
    /// The set tier's container word (M119, ADR-0123): the collection's
    /// declared item count (AccessKit's own placement: the size rides
    /// the container, where ARIA's `aria-setsize` is set on each item)
    /// — `None` the honest absence, the span the table declared not.
    size_of_set: Option<usize>,
    /// The live tier's word (M120, ADR-0124): the table's declared live
    /// region — `None` the honest absence, the container that declared
    /// no region.
    live: Option<Politeness>,
}

impl Table {
    /// A table named `name` with `columns` header columns.
    #[must_use]
    pub fn new(name: impl Into<String>, columns: usize) -> Self {
        Self {
            name: name.into(),
            columns,
            rows: 0,
            style: Style {
                gap: 2.0,
                ..Style::column()
            },
            header_fill: None,
            multiselectable: false,
            busy: false,
            size_of_set: None,
            live: None,
        }
    }

    /// Mounts the table multi-pick (M109, ADR-0113): the collection's
    /// capacity said by the tree from the first frame — AccessKit's
    /// `Multiselectable` property (ARIA's `aria-multiselectable`) names
    /// the capacity to the AT beside the member picks the selected tier
    /// (ADR-0112) already carried. The word prices nothing but the
    /// word: the table's doors answer exactly as they always did — the
    /// word names capacity, it walls nothing. The app's live door rides
    /// [`Table::set_multiselectable`].
    #[must_use]
    pub fn multiselectable(mut self, multiselectable: bool) -> Self {
        self.multiselectable = multiselectable;
        self
    }

    /// The multiselectable door (M109, ADR-0113): the app's capacity
    /// moved — the word the tree says follows it on the re-announcement
    /// the caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). Report whether the
    /// word moved — the caller requests `A11Y` only when this answers
    /// `true`, so a write that lands where the word already is queues
    /// nothing (the M21 idle-frame property, at the multiselectable
    /// tier).
    pub fn set_multiselectable(&mut self, multiselectable: bool) -> bool {
        if self.multiselectable == multiselectable {
            return false;
        }
        self.multiselectable = multiselectable;
        true
    }

    /// Mounts the table's updating word (M117, ADR-0121): the
    /// subtree's fact said by the tree from the first frame —
    /// AccessKit's `Busy` property (ARIA's `aria-busy`) names the
    /// region the AT holds its speech on while the stream is still
    /// writing it, the declared counterpart of the announcement
    /// machinery the framework already owns (§11.6, ADR-0024). The
    /// word prices nothing but the word: the table's doors answer
    /// exactly as they always did — the word names the updating fact,
    /// it walls nothing. The app's live door rides
    /// [`Table::set_busy`].
    #[must_use]
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    /// The busy door (M117, ADR-0121): the app's updating fact moved —
    /// the word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). Report whether the
    /// word moved — the caller requests `A11Y` only when this answers
    /// `true`, so a write that lands where the word already is queues
    /// nothing (the M21 idle-frame property, at the busy tier).
    pub fn set_busy(&mut self, busy: bool) -> bool {
        if self.busy == busy {
            return false;
        }
        self.busy = busy;
        true
    }

    /// Mounts the table's span (M119, ADR-0123): the set tier's
    /// container word said by the tree from the first frame —
    /// AccessKit's `SizeOfSet` property (ARIA's `aria-setsize`) names
    /// the collection's item count to the AT beside the member places
    /// the tier's `position_in_set` word carries. The word prices
    /// nothing but the word: the table's doors answer exactly as they
    /// always did — the word names the span, it walls nothing. The
    /// app's live door rides [`Table::set_size_of_set`].
    #[must_use]
    pub fn size_of_set(mut self, size_of_set: usize) -> Self {
        self.size_of_set = Some(size_of_set);
        self
    }

    /// The size door (M119, ADR-0123): the app's collection moved —
    /// the word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). `None` clears the
    /// word — the honest absence restored, the span the table declared
    /// not. Report whether the word moved — the caller requests `A11Y`
    /// only when this answers `true`, so a write that lands where the
    /// word already is queues nothing (the M21 idle-frame property, at
    /// the set tier).
    pub fn set_size_of_set(&mut self, size_of_set: Option<usize>) -> bool {
        if self.size_of_set == size_of_set {
            return false;
        }
        self.size_of_set = size_of_set;
        true
    }

    /// Mounts the table's declared live region (M120, ADR-0124): the
    /// region whose changes the AT speaks, said by the tree from the
    /// first frame — AccessKit's `Live` property (ARIA's `aria-live`)
    /// names the region to the AT, the politeness ([`Politeness`]) the
    /// announcement seam's own vocabulary carries, the declared
    /// counterpart of the pipeline's reserved region (§11.6, ADR-0024
    /// — the imperative half the framework owns, the declarative half
    /// the app says). The word prices nothing but the word: the
    /// table's doors answer exactly as they always did — the word
    /// names the region the AT watches, it walls nothing. The app's
    /// live door rides [`Table::set_live`].
    #[must_use]
    pub fn live(mut self, live: Politeness) -> Self {
        self.live = Some(live);
        self
    }

    /// The live door (M120, ADR-0124): the app's declared region
    /// moved — the word the tree says follows it on the re-announcement
    /// the caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). `None` clears the
    /// word — the honest absence restored, the container that declared
    /// no region. Report whether the word moved — the caller requests
    /// `A11Y` only when this answers `true`, so a write that lands
    /// where the word already is queues nothing (the M21 idle-frame
    /// property, at the live tier).
    pub fn set_live(&mut self, live: Option<Politeness>) -> bool {
        if self.live == live {
            return false;
        }
        self.live = live;
        true
    }

    /// The live grid geometry for the a11y attributes (rows × columns).
    pub fn set_row_count(&mut self, rows: usize) {
        self.rows = rows;
    }

    /// The column count (headers; view reconciliation).
    pub fn set_column_count(&mut self, columns: usize) {
        self.columns = columns;
    }

    /// Renames the table in place (view reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Current row count (test introspection).
    #[must_use]
    pub const fn row_count(&self) -> usize {
        self.rows
    }

    /// Current column count (test introspection).
    #[must_use]
    pub const fn column_count(&self) -> usize {
        self.columns
    }

    /// Header row fill. Without it the header borrows the theme's
    /// panel role at paint time (ADR-0079).
    #[must_use]
    pub fn header_fill(mut self, color: Color) -> Self {
        self.header_fill = Some(color);
        self
    }

    /// Sets the header fill in place (view reconciliation) — `None`
    /// restores the theme's panel role at paint time (ADR-0079).
    pub fn set_header_fill(&mut self, color: Option<Color>) -> bool {
        if self.header_fill == color {
            return false;
        }
        self.header_fill = color;
        true
    }

    /// Layout styles.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Shared-borrow layout style access for view reconciliation (§6.1).
    #[must_use]
    pub const fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl Widget for Table {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Table
    }
    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }
    fn style(&self) -> Style {
        self.style.clone()
    }
    fn table_slot(&self) -> Option<TableSlot> {
        Some(TableSlot::Table {
            rows: self.rows,
            columns: self.columns,
        })
    }
    fn multiselectable(&self) -> bool {
        self.multiselectable
    }

    fn busy(&self) -> bool {
        self.busy
    }
    fn size_of_set(&self) -> Option<usize> {
        self.size_of_set
    }
    fn live(&self) -> Option<Politeness> {
        self.live
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        // The table's own chrome is none; headers carry their fill. Painting
        // a hairline table background keeps rows visually grouped.
        let palette = ctx.palette();
        crate::paint_background(
            ctx,
            (ctx.size().width, ctx.size().height),
            self.header_fill.unwrap_or(palette.panel),
            4.0,
            (palette.border, 1.0),
        );
    }
}

/// One data-table row: a horizontal container of cells.
pub struct TableRow {
    style: Style,
    /// The selected tier's word (M108, ADR-0112): the row's declared
    /// pick — `None` the honest absence, the row of no selection
    /// collection.
    selected: Option<bool>,
    /// The expanded tier's word (M110, ADR-0114): the row's declared
    /// disclosure — `None` the honest absence, the row that opens and
    /// closes not.
    expanded: Option<bool>,
    /// The level tier's word (M113, ADR-0117): the row's declared
    /// hierarchical depth, zero-based (AccessKit's own base, beside
    /// ARIA's one-based `aria-level`) — `None` the honest absence, the
    /// flat row.
    level: Option<usize>,
    /// The set tier's member word (M119, ADR-0123): the row's declared
    /// place in the collection, zero-based (AccessKit's own base,
    /// beside ARIA's one-based `aria-posinset`) — `None` the honest
    /// absence, the unplaced row.
    position_in_set: Option<usize>,
}

impl TableRow {
    /// An empty row; cells are mounted by the app.
    #[must_use]
    pub fn new() -> Self {
        Self {
            style: Style {
                gap: 8.0,
                ..Style::row()
            },
            selected: None,
            expanded: None,
            level: None,
            position_in_set: None,
        }
    }

    /// Mounts the row picked (or explicitly unpicked) (M108, ADR-0112):
    /// the collection's word said by the tree from the first frame —
    /// AccessKit's `Selected` property (ARIA's `aria-selected`) names
    /// the pick to the AT beside the position the M6 slot already
    /// carried. The word prices nothing but the word: the row's doors
    /// answer exactly as they always did — the word names state, it
    /// walls nothing. The app's live door rides
    /// [`TableRow::set_selected`].
    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// The selected door (M108, ADR-0112): the app's selection moved —
    /// the word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). Report whether the
    /// word moved — the caller requests `A11Y` only when this answers
    /// `true`, so a write that lands where the word already is queues
    /// nothing (the M21 idle-frame property, at the selected tier).
    /// `None` clears the word — the honest absence restored, the row of
    /// no selection collection.
    pub fn set_selected(&mut self, selected: Option<bool>) -> bool {
        if self.selected == selected {
            return false;
        }
        self.selected = selected;
        true
    }

    /// Mounts the row opened (or explicitly collapsed) (M110, ADR-0114):
    /// the collection member's disclosure said by the tree from the
    /// first frame — AccessKit's `Expanded` property (ARIA's
    /// `aria-expanded`) names the disclosure to the AT beside the picks
    /// the selected tier already carried. The word prices nothing but
    /// the word: the row's doors answer exactly as they always did —
    /// the word names state, it walls nothing, the open and close
    /// routes the widget's own. The app's live door rides
    /// [`TableRow::set_expanded`].
    #[must_use]
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    /// The expanded door (M110, ADR-0114): the app's disclosure moved —
    /// the word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). Report whether the
    /// word moved — the caller requests `A11Y` only when this answers
    /// `true`, so a write that lands where the word already is queues
    /// nothing (the M21 idle-frame property, at the expanded tier).
    /// `None` clears the word — the honest absence restored, the row
    /// that opens and closes not.
    pub fn set_expanded(&mut self, expanded: Option<bool>) -> bool {
        if self.expanded == expanded {
            return false;
        }
        self.expanded = expanded;
        true
    }

    /// Mounts the row's depth (M113, ADR-0117): the nested member's
    /// place in the collection said by the tree from the first frame —
    /// AccessKit's `Level` property (zero-based, beside ARIA's
    /// one-based `aria-level`) names the depth to the AT beside the
    /// position the M6 slot already carried. The word prices nothing
    /// but the word: the row's doors answer exactly as they always did
    /// — the word names the depth, it walls nothing. The app's live
    /// door rides [`TableRow::set_level`].
    #[must_use]
    pub fn level(mut self, level: usize) -> Self {
        self.level = Some(level);
        self
    }

    /// The level door (M113, ADR-0117): the app's depth moved — the
    /// word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). `None` clears the
    /// word — the honest absence restored, the flat row. Report
    /// whether the word moved — the caller requests `A11Y` only when
    /// this answers `true`, so a write that lands where the word
    /// already is queues nothing (the M21 idle-frame property, at the
    /// level tier).
    pub fn set_level(&mut self, level: Option<usize>) -> bool {
        if self.level == level {
            return false;
        }
        self.level = level;
        true
    }

    /// Mounts the row's place in its collection (M119, ADR-0123): the
    /// set tier's member word said by the tree from the first frame —
    /// AccessKit's `PositionInSet` property (ARIA's `aria-posinset`)
    /// names the place to the AT beside the depth the level tier
    /// already carried. The word prices nothing but the word: the row's
    /// doors answer exactly as they always did — the word names the
    /// place, it walls nothing. The app's live door rides
    /// [`TableRow::set_position_in_set`].
    #[must_use]
    pub fn position_in_set(mut self, position_in_set: usize) -> Self {
        self.position_in_set = Some(position_in_set);
        self
    }

    /// The position door (M119, ADR-0123): the app's collection moved —
    /// the word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). `None` clears the
    /// word — the honest absence restored, the unplaced row. Report
    /// whether the word moved — the caller requests `A11Y` only when
    /// this answers `true`, so a write that lands where the word
    /// already is queues nothing (the M21 idle-frame property, at the
    /// set tier).
    pub fn set_position_in_set(&mut self, position_in_set: Option<usize>) -> bool {
        if self.position_in_set == position_in_set {
            return false;
        }
        self.position_in_set = position_in_set;
        true
    }

    /// Layout styles.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Shared-borrow layout style access for view reconciliation (§6.1).
    #[must_use]
    pub const fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl Default for TableRow {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TableRow {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::TableRow
    }
    fn label(&self) -> LabelSource {
        // Rows read through their cells (AccessKit rows are nameless
        // containers; the cells carry the content).
        LabelSource::FromChildren
    }
    fn style(&self) -> Style {
        self.style.clone()
    }
    fn selected(&self) -> Option<bool> {
        self.selected
    }
    fn expanded(&self) -> Option<bool> {
        self.expanded
    }
    fn level(&self) -> Option<usize> {
        self.level
    }
    fn position_in_set(&self) -> Option<usize> {
        self.position_in_set
    }
}

/// A text cell: a table's leaf content with its grid position (M6).
pub struct TableCell {
    text: String,
    slot: (usize, usize),
    /// The app's text-color override; `None` borrows the theme's text
    /// role at paint time (ADR-0079).
    color: Option<Color>,
    font_size: f32,
    style: Style,
    strong: bool,
    /// The selected tier's word (M108, ADR-0112): the cell's declared
    /// pick — `None` the honest absence, the cell of no selection
    /// collection.
    selected: Option<bool>,
}

impl TableCell {
    /// A cell at zero-based `(row, column)` displaying `text`.
    #[must_use]
    pub fn new(text: impl Into<String>, row: usize, column: usize) -> Self {
        Self {
            text: text.into(),
            slot: (row, column),
            color: None,
            font_size: crate::theme::FONT_SIZE,
            style: Style {
                padding: 6.0,
                ..Style::default()
            },
            strong: false,
            selected: None,
        }
    }

    /// Mounts the cell picked (or explicitly unpicked) (M108, ADR-0112):
    /// the collection's word said by the tree from the first frame —
    /// AccessKit's `Selected` property (ARIA's `aria-selected`) names
    /// the pick to the AT beside the position the M6 slot already
    /// carried. The word prices nothing but the word: the cell's doors
    /// answer exactly as they always did — the word names state, it
    /// walls nothing. The app's live door rides
    /// [`TableCell::set_selected`].
    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// The selected door (M108, ADR-0112): the app's selection moved —
    /// the word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). Report whether the
    /// word moved — the caller requests `A11Y` only when this answers
    /// `true`, so a write that lands where the word already is queues
    /// nothing (the M21 idle-frame property, at the selected tier).
    /// `None` clears the word — the honest absence restored, the cell
    /// of no selection collection.
    pub fn set_selected(&mut self, selected: Option<bool>) -> bool {
        if self.selected == selected {
            return false;
        }
        self.selected = selected;
        true
    }

    /// Emphasis styling (header-like data cells).
    #[must_use]
    pub fn strong(mut self, strong: bool) -> Self {
        self.strong = strong;
        self
    }

    /// Sets the emphasis tier in place (view reconciliation) — the write
    /// reporting whether the tier moved (the idle-frame property).
    pub fn set_strong(&mut self, strong: bool) -> bool {
        if self.strong == strong {
            return false;
        }
        self.strong = strong;
        true
    }

    /// Text color. Without it the cell borrows the theme's text role
    /// at paint time (ADR-0079).
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Font size.
    #[must_use]
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Replaces the cell text in place (view reconciliation).
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Updates the grid position in place (rows shifting renumber cells).
    pub fn set_slot(&mut self, row: usize, column: usize) {
        self.slot = (row, column);
    }

    /// Current grid position (test introspection).
    #[must_use]
    pub const fn slot(&self) -> (usize, usize) {
        self.slot
    }

    fn intrinsic(&self) -> alacer_core::geom::Size {
        alacer_core::text_seam::with_seam(|seam| {
            let text = seam.measure(
                &TextSpec {
                    text: self.text.clone(),
                    font_id: 0,
                    font_size: self.font_size,
                },
                None,
            );
            alacer_core::geom::Size::new(text.width + 12.0, text.height.max(18.0) + 12.0)
        })
    }
}

impl Widget for TableCell {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::TableCell
    }
    fn label(&self) -> LabelSource {
        LabelSource::text(self.text.clone())
    }
    fn style(&self) -> Style {
        self.style.clone()
    }
    fn table_slot(&self) -> Option<TableSlot> {
        let (row, column) = self.slot;
        Some(TableSlot::Cell { row, column })
    }
    fn selected(&self) -> Option<bool> {
        self.selected
    }
    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(self.intrinsic()))
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        // The borrowed role resolves per paint pass (ADR-0079).
        let color = self.color.unwrap_or(ctx.palette().text);
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.text, self.font_size, None);
            let text_origin = Point::new(6.0, (ctx.size().height - layout.height).max(0.0) * 0.5);
            for glyph in &layout.glyphs {
                ctx.push_glyph(alacer_core::scene::Glyph {
                    position: Point::new(text_origin.x + glyph.x, text_origin.y + glyph.y),
                    size: glyph.size,
                    color,
                    font_id: glyph.font_id,
                    glyph_id: glyph.glyph_id,
                    clip: None,
                });
            }
        });
    }
}

/// A column header: the table's first-row cell vocabulary (M6).
pub struct ColumnHeader {
    text: String,
    column: usize,
    /// The app's text-color override; `None` borrows the theme's muted
    /// role at paint time (ADR-0079).
    color: Option<Color>,
    font_size: f32,
    style: Style,
    /// The sort direction tier's word (M112, ADR-0116): the order the
    /// column holds its rows in — `None` the honest absence.
    sort_direction: Option<alacer_core::SortDirection>,
}

impl ColumnHeader {
    /// A header for zero-based `column` displaying `text`.
    #[must_use]
    pub fn new(text: impl Into<String>, column: usize) -> Self {
        Self {
            text: text.into(),
            column,
            color: None,
            font_size: 13.0,
            style: Style {
                padding: 6.0,
                ..Style::default()
            },
            sort_direction: None,
        }
    }

    /// Mounts the sorted column (M112, ADR-0116): the order the column
    /// holds its rows in, said by the tree from the first frame —
    /// AccessKit's `SortDirection` property (ARIA's `aria-sort`) names
    /// the order to the AT beside the position the M6 slot already
    /// carried. The word prices nothing but the word: the column's
    /// doors answer exactly as they always did — the word names the
    /// order, it walls nothing. The app's live door rides
    /// [`ColumnHeader::set_sort`].
    #[must_use]
    pub fn sort(mut self, sort_direction: alacer_core::SortDirection) -> Self {
        self.sort_direction = Some(sort_direction);
        self
    }

    /// The sort door (M112, ADR-0116): the app's column order moved —
    /// the word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). `None` restores the
    /// honest absence. Report whether the word moved — the caller
    /// requests `A11Y` only when this answers `true`, so a write that
    /// lands where the word already is queues nothing (the M21
    /// idle-frame property, at the sort direction tier).
    pub fn set_sort(&mut self, sort_direction: Option<alacer_core::SortDirection>) -> bool {
        if self.sort_direction == sort_direction {
            return false;
        }
        self.sort_direction = sort_direction;
        true
    }

    /// The column's declared order (test introspection).
    #[must_use]
    pub const fn sort_direction(&self) -> Option<alacer_core::SortDirection> {
        self.sort_direction
    }

    /// Text color. Without it the header borrows the theme's muted
    /// role at paint time (ADR-0079).
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Replaces the header text in place (view reconciliation).
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    fn intrinsic(&self) -> alacer_core::geom::Size {
        alacer_core::text_seam::with_seam(|seam| {
            let text = seam.measure(
                &TextSpec {
                    text: self.text.clone(),
                    font_id: 0,
                    font_size: self.font_size,
                },
                None,
            );
            alacer_core::geom::Size::new(text.width + 12.0, text.height.max(16.0) + 10.0)
        })
    }
}

impl Widget for ColumnHeader {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::ColumnHeader
    }
    fn label(&self) -> LabelSource {
        LabelSource::text(self.text.clone())
    }
    fn style(&self) -> Style {
        self.style.clone()
    }
    fn table_slot(&self) -> Option<TableSlot> {
        Some(TableSlot::Header {
            column: self.column,
        })
    }
    fn sort_direction(&self) -> Option<alacer_core::SortDirection> {
        self.sort_direction
    }
    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(self.intrinsic()))
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        // The borrowed role resolves per paint pass (ADR-0079).
        let color = self.color.unwrap_or(ctx.palette().text_muted);
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.text, self.font_size, None);
            let text_origin = Point::new(6.0, (ctx.size().height - layout.height).max(0.0) * 0.5);
            for glyph in &layout.glyphs {
                ctx.push_glyph(alacer_core::scene::Glyph {
                    position: Point::new(text_origin.x + glyph.x, text_origin.y + glyph.y),
                    size: glyph.size,
                    color,
                    font_id: glyph.font_id,
                    glyph_id: glyph.glyph_id,
                    clip: None,
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{ColumnHeader, Table, TableCell, TableRow};
    use alacer_core::style::MeasureSpec;
    use alacer_core::widget::{LabelSource, Role, TableSlot, Widget};

    /// The probe seam: deterministic metrics, no glyphs (the widget layer
    /// cannot reach `alacer-text`; the real seam installs at assembly).
    struct ProbeSeam;
    impl alacer_core::text_seam::TextSeam for ProbeSeam {
        fn measure(
            &self,
            spec: &alacer_core::style::TextSpec,
            _max_width: Option<f32>,
        ) -> alacer_core::geom::Size {
            alacer_core::geom::Size::new(
                alacer_core::num::u32_to_f32(
                    u32::try_from(spec.text.chars().count()).unwrap_or(u32::MAX),
                ) * 7.0,
                16.0,
            )
        }

        fn layout_glyphs(
            &self,
            _text: &str,
            _font_size: f32,
            _max_width: Option<f32>,
        ) -> alacer_core::text_seam::GlyphRun {
            alacer_core::text_seam::GlyphRun {
                width: 0.0,
                height: 16.0,
                glyphs: Vec::new(),
            }
        }

        /// The probe's segmentation: alphanumeric runs are word-like,
        /// everything else separates — deterministic for the ASCII probe
        /// values; the real tables ride the engine (alacer-text).
        fn word_segments(&self, text: &str) -> Vec<alacer_core::text_seam::WordSegment> {
            let mut segments = Vec::new();
            let (mut start, mut word_like, mut units) = (0_usize, false, 0_usize);
            for ch in text.chars() {
                let like = ch.is_alphanumeric();
                if units > 0 && like != word_like {
                    segments.push(alacer_core::text_seam::WordSegment {
                        start,
                        end: units,
                        word_like,
                    });
                    start = units;
                }
                word_like = like;
                units += ch.len_utf16();
            }
            if units > start {
                segments.push(alacer_core::text_seam::WordSegment {
                    start,
                    end: units,
                    word_like,
                });
            }
            segments
        }
    }

    fn install_seam() {
        if !alacer_core::text_seam::is_installed() {
            alacer_core::text_seam::install(std::rc::Rc::new(ProbeSeam));
        }
    }

    #[test]
    fn table_reports_its_grid_geometry() {
        let table = Table::new("Inventory", 3);
        assert_eq!(table.role(), Role::Table);
        assert_eq!(table.label().resolve(), "Inventory");
        assert_eq!(
            table.table_slot(),
            Some(TableSlot::Table {
                rows: 0,
                columns: 3
            })
        );
    }

    #[test]
    fn row_count_tracks_the_row_api() {
        let mut table = Table::new("Inventory", 2);
        table.set_row_count(7);
        assert_eq!(table.row_count(), 7);
        assert_eq!(
            table.table_slot(),
            Some(TableSlot::Table {
                rows: 7,
                columns: 2
            })
        );
    }

    #[test]
    fn cells_carry_positions_and_names() {
        let cell = TableCell::new("Nails", 2, 1);
        assert_eq!(cell.role(), Role::TableCell);
        assert_eq!(cell.label().resolve(), "Nails");
        assert_eq!(
            cell.table_slot(),
            Some(TableSlot::Cell { row: 2, column: 1 })
        );
    }

    #[test]
    fn cells_renumber_in_place() {
        let mut cell = TableCell::new("Nails", 0, 0);
        cell.set_slot(3, 2);
        cell.set_text("Screws");
        assert_eq!(cell.slot(), (3, 2));
        assert_eq!(cell.label().resolve(), "Screws");
        assert_eq!(
            cell.table_slot(),
            Some(TableSlot::Cell { row: 3, column: 2 })
        );
    }

    #[test]
    fn headers_carry_their_column() {
        let header = ColumnHeader::new("Name", 0);
        assert_eq!(header.role(), Role::ColumnHeader);
        assert_eq!(header.table_slot(), Some(TableSlot::Header { column: 0 }));
    }

    #[test]
    fn headers_carry_their_sort_word() {
        // M112, ADR-0116: the constructor word is the trait door's
        // answer — the order the AT reads beside the header's column
        // position — and the door moves it live, the idle-frame
        // property on the write that lands where the word already is.
        use alacer_core::SortDirection;
        let mut header = ColumnHeader::new("Price", 2).sort(SortDirection::Ascending);
        assert_eq!(
            header.sort_direction(),
            Some(SortDirection::Ascending),
            "the constructor's word is the trait door's answer"
        );
        assert!(
            header.set_sort(Some(SortDirection::Descending)),
            "the door reports the word moved"
        );
        assert_eq!(
            header.sort_direction(),
            Some(SortDirection::Descending),
            "the door moved the word the tree says"
        );
        assert!(
            !header.set_sort(Some(SortDirection::Descending)),
            "the write that lands where the word already is queues nothing"
        );
        assert!(header.set_sort(None), "the door carries the word away");
        assert!(
            header.sort_direction().is_none(),
            "the honest absence restored"
        );
        assert!(
            !header.set_sort(None),
            "clearing the absence twice queues nothing"
        );
    }

    #[test]
    fn headers_without_a_word_inherit_the_honest_absence() {
        // M112, ADR-0116: a header that declared nothing announces no
        // order — ARIA's `none` spelled by `None`, never a guessed
        // direction.
        let header = ColumnHeader::new("Name", 0);
        assert!(header.sort_direction().is_none());
    }

    #[test]
    fn rows_carry_their_level_word() {
        // M113, ADR-0117: the constructor word is the trait door's
        // answer — the zero-based depth the AT reads beside the row's
        // grid position — and the door moves it live, the idle-frame
        // property on the write that lands where the word already is.
        // The read goes through the trait explicitly: the inherent
        // builder shares the door's name, the way every constructor
        // word does.
        let mut row = TableRow::new().level(2);
        assert_eq!(
            Widget::level(&row),
            Some(2),
            "the constructor's word is the trait door's answer — zero-based, AccessKit's own base"
        );
        assert!(row.set_level(Some(1)), "the door reports the word moved");
        assert_eq!(
            Widget::level(&row),
            Some(1),
            "the door moved the word the tree says"
        );
        assert!(
            !row.set_level(Some(1)),
            "the write that lands where the word already is queues nothing"
        );
        assert!(row.set_level(None), "the door carries the word away");
        assert!(Widget::level(&row).is_none(), "the honest absence restored");
        assert!(
            !row.set_level(None),
            "clearing the absence twice queues nothing"
        );
    }

    #[test]
    fn rows_without_a_word_inherit_the_honest_absence() {
        // M113, ADR-0117: a row that declared nothing announces no
        // depth — the flat row, never a guessed `0`.
        let row = TableRow::new();
        assert!(Widget::level(&row).is_none());
    }

    #[test]
    fn rows_carry_their_place_word() {
        // M119, ADR-0123: the set tier's member word — the row's
        // declared place in the collection, zero-based (AccessKit's
        // own base, beside ARIA's one-based `aria-posinset`).
        let mut row = TableRow::new();
        assert!(Widget::position_in_set(&row).is_none(), "the absence first");
        row = row.position_in_set(2);
        assert_eq!(Widget::position_in_set(&row), Some(2));
        assert!(
            row.set_position_in_set(Some(1)),
            "the door reports the word moved"
        );
        assert_eq!(Widget::position_in_set(&row), Some(1));
        assert!(
            !row.set_position_in_set(Some(1)),
            "the write that lands where the word already is queues nothing"
        );
        assert!(
            row.set_position_in_set(None),
            "the door carries the word away"
        );
        assert!(
            Widget::position_in_set(&row).is_none(),
            "the honest absence restored"
        );
        assert!(
            !row.set_position_in_set(None),
            "clearing the absence twice queues nothing"
        );
    }

    #[test]
    fn tables_carry_their_live_word() {
        // M120, ADR-0124: the live tier's declared word — the region
        // whose changes the AT speaks, the politeness the announcement
        // seam's own vocabulary carries.
        use alacer_core::announce::Politeness;
        let mut table = Table::new("Ledger", 2);
        assert!(Widget::live(&table).is_none(), "the absence first");
        table = table.live(Politeness::Assertive);
        assert_eq!(Widget::live(&table), Some(Politeness::Assertive));
        assert!(
            table.set_live(Some(Politeness::Polite)),
            "the door reports the word moved"
        );
        assert_eq!(Widget::live(&table), Some(Politeness::Polite));
        assert!(
            !table.set_live(Some(Politeness::Polite)),
            "the write that lands where the word already is queues nothing"
        );
        assert!(table.set_live(None), "the door carries the word away");
        assert!(
            Widget::live(&table).is_none(),
            "the honest absence restored"
        );
        assert!(
            !table.set_live(None),
            "clearing the absence twice queues nothing"
        );
    }

    #[test]
    fn rows_read_through_their_cells() {
        let row = TableRow::new();
        assert_eq!(row.role(), Role::TableRow);
        assert!(matches!(row.label(), LabelSource::FromChildren));
        assert_eq!(row.label().resolve(), "", "rows are nameless containers");
    }

    #[test]
    fn tables_carry_their_size_word() {
        // M119, ADR-0123: the set tier's container word — the
        // collection's declared item count (AccessKit's own placement:
        // the size rides the container, where ARIA's `aria-setsize`
        // is set on each item).
        let mut table = Table::new("Ledger", 2);
        assert!(Widget::size_of_set(&table).is_none(), "the absence first");
        table = table.size_of_set(500);
        assert_eq!(Widget::size_of_set(&table), Some(500));
        assert!(
            table.set_size_of_set(Some(120)),
            "the door reports the word moved"
        );
        assert_eq!(Widget::size_of_set(&table), Some(120));
        assert!(
            !table.set_size_of_set(Some(120)),
            "the write that lands where the word already is queues nothing"
        );
        assert!(
            table.set_size_of_set(None),
            "the door carries the word away"
        );
        assert!(
            Widget::size_of_set(&table).is_none(),
            "the honest absence restored"
        );
        assert!(
            !table.set_size_of_set(None),
            "clearing the absence twice queues nothing"
        );
    }

    #[test]
    fn cells_expose_a_measure_spec() {
        install_seam();
        let cell = TableCell::new("Nails", 0, 0);
        assert!(matches!(
            cell.measure_spec(),
            Some(MeasureSpec::Fixed(size)) if size.width > 12.0
        ));
        let header = ColumnHeader::new("Name", 0);
        assert!(header.measure_spec().is_some());
    }
}
