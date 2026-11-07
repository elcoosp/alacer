//! The data-table canon (M6): `Table` / `TableRow` / `TableCell` /
//! `ColumnHeader` as retained composition.
//!
//! A `Table` is a column container carrying its grid geometry — rows ×
//! columns, kept in step by the row-count API as the app mounts and removes
//! rows. `ColumnHeader` and `TableCell` are named text cells that report
//! their zero-based grid positions through [`Widget::table_slot`], so the
//! AccessKit update carries a *navigable* grid (row/column counts and
//! indexes), not just a pile of roles.

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
    header_fill: Color,
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
            header_fill: crate::theme::PANEL,
        }
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

    /// Header row fill.
    #[must_use]
    pub fn header_fill(mut self, color: Color) -> Self {
        self.header_fill = color;
        self
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
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        // The table's own chrome is none; headers carry their fill. Painting
        // a hairline table background keeps rows visually grouped.
        crate::paint_background(
            ctx,
            (ctx.size().width, ctx.size().height),
            self.header_fill,
            4.0,
            (crate::theme::BORDER, 1.0),
        );
    }
}

/// One data-table row: a horizontal container of cells.
pub struct TableRow {
    style: Style,
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
        }
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
}

/// A text cell: a table's leaf content with its grid position (M6).
pub struct TableCell {
    text: String,
    slot: (usize, usize),
    color: Color,
    font_size: f32,
    style: Style,
    strong: bool,
}

impl TableCell {
    /// A cell at zero-based `(row, column)` displaying `text`.
    #[must_use]
    pub fn new(text: impl Into<String>, row: usize, column: usize) -> Self {
        Self {
            text: text.into(),
            slot: (row, column),
            color: crate::theme::TEXT,
            font_size: crate::theme::FONT_SIZE,
            style: Style {
                padding: 6.0,
                ..Style::default()
            },
            strong: false,
        }
    }

    /// Emphasis styling (header-like data cells).
    #[must_use]
    pub fn strong(mut self, strong: bool) -> Self {
        self.strong = strong;
        self
    }

    /// Text color.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
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
    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(self.intrinsic()))
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.text, self.font_size, None);
            let text_origin = Point::new(6.0, (ctx.size().height - layout.height).max(0.0) * 0.5);
            for glyph in &layout.glyphs {
                ctx.push_glyph(alacer_core::scene::Glyph {
                    position: Point::new(text_origin.x + glyph.x, text_origin.y + glyph.y),
                    size: glyph.size,
                    color: self.color,
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
    color: Color,
    font_size: f32,
    style: Style,
}

impl ColumnHeader {
    /// A header for zero-based `column` displaying `text`.
    #[must_use]
    pub fn new(text: impl Into<String>, column: usize) -> Self {
        Self {
            text: text.into(),
            column,
            color: crate::theme::TEXT_MUTED,
            font_size: 13.0,
            style: Style {
                padding: 6.0,
                ..Style::default()
            },
        }
    }

    /// Text color.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
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
    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(self.intrinsic()))
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.text, self.font_size, None);
            let text_origin = Point::new(6.0, (ctx.size().height - layout.height).max(0.0) * 0.5);
            for glyph in &layout.glyphs {
                ctx.push_glyph(alacer_core::scene::Glyph {
                    position: Point::new(text_origin.x + glyph.x, text_origin.y + glyph.y),
                    size: glyph.size,
                    color: self.color,
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
    fn rows_read_through_their_cells() {
        let row = TableRow::new();
        assert_eq!(row.role(), Role::TableRow);
        assert!(matches!(row.label(), LabelSource::FromChildren));
        assert_eq!(row.label().resolve(), "", "rows are nameless containers");
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
