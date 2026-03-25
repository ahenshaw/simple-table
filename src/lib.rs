//! # simple_table
//!
//! An easy-to-use egui table widget built on [`egui_table`] v0.7.
//!
//! Features:
//! - Scrollable in both horizontal and vertical dimensions
//! - Fixed (sticky) header row that stays visible while scrolling vertically
//! - Fixed (sticky) row-header column that stays visible while scrolling horizontally
//! - Separate styling for header cells vs data cells
//! - Click selection with visual highlight — row and cell callbacks via [`TableResponse`]
//! - Simple data-driven API — just provide `Vec<String>` column names and a 2-D data array
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use simple_table::{FancyTable, CellStyle, HeaderStyle};
//! use egui::Color32;
//!
//! let columns = vec![
//!     "Row Label".to_string(),
//!     "Alpha".to_string(),
//!     "Beta".to_string(),
//! ];
//!
//! let data = vec![
//!     vec!["Row 1".to_string(), "1.0".to_string(), "2.0".to_string()],
//!     vec!["Row 2".to_string(), "3.0".to_string(), "4.0".to_string()],
//! ];
//!
//! let mut table = FancyTable::new("my_table", columns, data);
//!
//! // In your egui update() / show() callback:
//! // let response = table.show(ui);
//! // if let Some(cell) = response.clicked_cell {
//! //     println!("clicked row={} col={} value={}", cell.row, cell.col, cell.value);
//! // }
//! // if let Some(row) = response.clicked_row {
//! //     println!("clicked row={}", row.row);
//! // }
//! ```

use egui::{Color32, CornerRadius, Frame, Margin, Sense, Stroke, Ui};
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate};

// ─────────────────────────────────────────────────────────────────────────────
// Public callback / response types
// ─────────────────────────────────────────────────────────────────────────────

/// Information about a row that was clicked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowClick {
    /// Zero-based data row index (does not count the header row).
    pub row: usize,
}

/// Information about a cell that was clicked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellClick {
    /// Zero-based data row index.
    pub row: usize,
    /// Zero-based column index (0 = row-header column).
    pub col: usize,
    /// The text value of the clicked cell.
    pub value: String,
}

/// Returned by [`FancyTable::show`] every frame.
///
/// Both fields fire on the same click — handle whichever is useful to you.
/// Neither fires for header-row clicks.
#[derive(Clone, Debug, Default)]
pub struct TableResponse {
    /// Set when the user clicks any cell in a data row.
    pub clicked_row: Option<RowClick>,
    /// Set when the user clicks a specific data cell.
    pub clicked_cell: Option<CellClick>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Public style types
// ─────────────────────────────────────────────────────────────────────────────

/// Visual style applied to every **header** cell (the top row and the
/// leftmost column).
#[derive(Clone, Debug)]
pub struct HeaderStyle {
    /// Background fill colour.
    pub bg_color: Color32,
    /// Text colour.
    pub text_color: Color32,
    /// Inner padding around the cell contents.
    pub padding: Margin,
    /// Optional border stroke drawn around the cell.
    pub border: Stroke,
    /// Corner rounding of the cell background.
    pub rounding: CornerRadius,
}

impl Default for HeaderStyle {
    fn default() -> Self {
        Self {
            bg_color: Color32::from_rgb(40, 80, 140),
            text_color: Color32::WHITE,
            padding: Margin::same(6),
            border: Stroke::new(1.0, Color32::from_rgb(20, 50, 100)),
            rounding: CornerRadius::ZERO,
        }
    }
}

/// Visual style applied to every regular **data** cell.
#[derive(Clone, Debug)]
pub struct CellStyle {
    /// Background fill colour.
    pub bg_color: Color32,
    /// Background fill colour for alternating (odd) rows — set to the same
    /// value as `bg_color` to disable striping.
    pub alt_bg_color: Color32,
    /// Text colour.
    pub text_color: Color32,
    /// Inner padding around the cell contents.
    pub padding: Margin,
    /// Optional border stroke.
    pub border: Stroke,
    /// Corner rounding of the cell background.
    pub rounding: CornerRadius,
}

impl Default for CellStyle {
    fn default() -> Self {
        Self {
            bg_color: Color32::from_rgb(245, 247, 250),
            alt_bg_color: Color32::from_rgb(230, 235, 242),
            text_color: Color32::from_rgb(20, 20, 30),
            padding: Margin::same(6),
            border: Stroke::new(0.5, Color32::from_rgb(200, 205, 215)),
            rounding: CornerRadius::ZERO,
        }
    }
}

/// Visual style used to highlight the selected row and cell.
#[derive(Clone, Debug)]
pub struct SelectionStyle {
    /// Background colour for the entire selected row.
    pub row_bg: Color32,
    /// Background colour for the specifically selected cell (drawn on top of
    /// the row highlight).
    pub cell_bg: Color32,
    /// Text colour override for the selected cell. `None` keeps the normal colour.
    pub text_color: Option<Color32>,
    /// Border drawn around the selected cell.
    pub cell_border: Stroke,
}

impl Default for SelectionStyle {
    fn default() -> Self {
        Self {
            row_bg: Color32::from_rgb(180, 210, 255),
            cell_bg: Color32::from_rgb(60, 130, 220),
            text_color: Some(Color32::WHITE),
            cell_border: Stroke::new(2.0, Color32::from_rgb(20, 80, 180)),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FancyTable – main public struct
// ─────────────────────────────────────────────────────────────────────────────

/// A scrollable egui table with a **fixed header row** and a **fixed
/// row-header column**, with click-selection support.
///
/// Construct with [`FancyTable::new`], customise styling with the builder
/// methods, then call [`FancyTable::show`] every frame and inspect the
/// returned [`TableResponse`].
pub struct FancyTable {
    id_salt: String,
    column_names: Vec<String>,
    data: Vec<Vec<String>>,
    header_height: f32,
    row_height: f32,
    min_col_width: f32,
    header_style: HeaderStyle,
    cell_style: CellStyle,
    selection_style: SelectionStyle,
    selected_row: Option<usize>,
    selected_cell: Option<(usize, usize)>,
}

impl FancyTable {
    // ── Construction ─────────────────────────────────────────────────────────

    /// Create a new `FancyTable`.
    ///
    /// * `id_salt`      – unique string so egui can persist column-width state.
    /// * `column_names` – column labels; the **first** name is the row-header
    ///                    column (sticky/fixed on the left).
    /// * `data`         – rows of cells; each inner `Vec` must be the same
    ///                    length as `column_names`.
    pub fn new(
        id_salt: impl Into<String>,
        column_names: Vec<String>,
        data: Vec<Vec<String>>,
    ) -> Self {
        Self {
            id_salt: id_salt.into(),
            column_names,
            data,
            header_height: 28.0,
            row_height: 22.0,
            min_col_width: 80.0,
            header_style: HeaderStyle::default(),
            cell_style: CellStyle::default(),
            selection_style: SelectionStyle::default(),
            selected_row: None,
            selected_cell: None,
        }
    }

    // ── Builder setters ───────────────────────────────────────────────────────

    /// Override the header style.
    pub fn header_style(mut self, style: HeaderStyle) -> Self {
        self.header_style = style;
        self
    }

    /// Override the data-cell style.
    pub fn cell_style(mut self, style: CellStyle) -> Self {
        self.cell_style = style;
        self
    }

    /// Override the selection highlight style.
    pub fn selection_style(mut self, style: SelectionStyle) -> Self {
        self.selection_style = style;
        self
    }

    /// Set the height of the header row (default: 28 px).
    pub fn header_height(mut self, height: f32) -> Self {
        self.header_height = height;
        self
    }

    /// Set the height of each data row (default: 22 px).
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    /// Set the minimum column width (default: 80 px).
    pub fn min_col_width(mut self, width: f32) -> Self {
        self.min_col_width = width;
        self
    }

    // ── Mutators ──────────────────────────────────────────────────────────────

    /// Replace the entire dataset.
    pub fn set_data(&mut self, data: Vec<Vec<String>>) {
        self.data = data;
    }

    /// Replace a single cell's text.
    pub fn set_cell(&mut self, row: usize, col: usize, value: impl Into<String>) {
        if let Some(row_data) = self.data.get_mut(row) {
            if let Some(cell) = row_data.get_mut(col) {
                *cell = value.into();
            }
        }
    }

    /// Programmatically select a row (clears cell selection).
    pub fn select_row(&mut self, row: Option<usize>) {
        self.selected_row = row;
        self.selected_cell = None;
    }

    /// Programmatically select a specific cell (also selects its row).
    pub fn select_cell(&mut self, row: usize, col: usize) {
        self.selected_row = Some(row);
        self.selected_cell = Some((row, col));
    }

    /// Clear all selection state.
    pub fn clear_selection(&mut self) {
        self.selected_row = None;
        self.selected_cell = None;
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    /// Number of data rows.
    pub fn num_rows(&self) -> usize { self.data.len() }

    /// Number of columns (including the row-header column).
    pub fn num_cols(&self) -> usize { self.column_names.len() }

    /// Currently selected row index, if any.
    pub fn selected_row(&self) -> Option<usize> { self.selected_row }

    /// Currently selected (row, col) cell, if any.
    pub fn selected_cell(&self) -> Option<(usize, usize)> { self.selected_cell }

    // ── Rendering ─────────────────────────────────────────────────────────────

    /// Render the table into `ui`.  Call this every frame.
    ///
    /// Inspect the returned [`TableResponse`] to react to clicks.
    pub fn show(&mut self, ui: &mut Ui) -> TableResponse {
        let num_cols = self.column_names.len();
        let num_rows = self.data.len() as u64;

        let columns: Vec<Column> = (0..num_cols)
            .map(|col| {
                Column::new(self.min_col_width)
                    .resizable(col==0)  // header column is resizable
                    .range(self.min_col_width..=f32::INFINITY)
            })
            .collect();

        let headers = vec![HeaderRow::new(self.header_height)];

        let mut delegate = FancyTableDelegate {
            column_names: &self.column_names,
            data: &self.data,
            header_style: &self.header_style,
            cell_style: &self.cell_style,
            selection_style: &self.selection_style,
            selected_row: self.selected_row,
            selected_cell: self.selected_cell,
            row_height: self.row_height,
            clicked_row: None,
            clicked_cell: None,
        };

        Table::new()
            .id_salt(&self.id_salt)
            .num_rows(num_rows)
            .columns(columns)
            .num_sticky_cols(1)
            .headers(headers)
            .show(ui, &mut delegate);

        // Collect results and update internal selection state.
        let response = TableResponse {
            clicked_row: delegate.clicked_row,
            clicked_cell: delegate.clicked_cell,
        };

        if let Some(ref c) = response.clicked_cell {
            self.selected_row = Some(c.row);
            self.selected_cell = Some((c.row, c.col));
        }

        response
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal egui_table delegate
// ─────────────────────────────────────────────────────────────────────────────

struct FancyTableDelegate<'a> {
    column_names: &'a [String],
    data: &'a [Vec<String>],
    header_style: &'a HeaderStyle,
    cell_style: &'a CellStyle,
    selection_style: &'a SelectionStyle,
    selected_row: Option<usize>,
    selected_cell: Option<(usize, usize)>,
    row_height: f32,
    // Written during cell_ui, read back by FancyTable::show after rendering.
    clicked_row: Option<RowClick>,
    clicked_cell: Option<CellClick>,
}

impl<'a> FancyTableDelegate<'a> {
    fn resolve_cell(
        &self,
        row: usize,
        col: usize,
    ) -> (Color32, Color32, Stroke, CornerRadius, Margin) {
        let hs = &self.header_style;
        let cs = &self.cell_style;
        let ss = &self.selection_style;
        let is_sel_cell = self.selected_cell == Some((row, col));
        let is_sel_row  = self.selected_row  == Some(row);

        if col == 0 {
            // Row-header column: base colours from HeaderStyle, but honour
            // selection highlights.
            let bg = if is_sel_cell {
                ss.cell_bg
            } else if is_sel_row {
                ss.row_bg
            } else {
                hs.bg_color
            };
            let text = if is_sel_cell {
                ss.text_color.unwrap_or(hs.text_color)
            } else {
                hs.text_color
            };
            let border = if is_sel_cell { ss.cell_border } else { hs.border };
            (bg, text, border, hs.rounding, hs.padding)
        } else {
            // Normal data cell.
            let bg = if is_sel_cell {
                ss.cell_bg
            } else if is_sel_row {
                ss.row_bg
            } else if row % 2 == 0 {
                cs.bg_color
            } else {
                cs.alt_bg_color
            };
            let text = if is_sel_cell {
                ss.text_color.unwrap_or(cs.text_color)
            } else {
                cs.text_color
            };
            let border = if is_sel_cell { ss.cell_border } else { cs.border };
            (bg, text, border, cs.rounding, cs.padding)
        }
    }
}

impl<'a> TableDelegate for FancyTableDelegate<'a> {
    // ── Header cells ─────────────────────────────────────────────────────────

    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &HeaderCellInfo) {
        let s = &self.header_style;
        let rect = ui.max_rect();
        ui.painter()
            .rect(rect, s.rounding, s.bg_color, s.border, egui::StrokeKind::Inside);
        Frame::new().inner_margin(s.padding).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            let label = self
                .column_names
                .get(cell.col_range.start)
                .map(|s| s.as_str())
                .unwrap_or("");
            ui.colored_label(s.text_color, label);
        });
    }

    // ── Data cells ────────────────────────────────────────────────────────────

    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo) {
        let row = cell.row_nr as usize;
        let col = cell.col_nr;

        let (bg, text_color, border, rounding, padding) = self.resolve_cell(row, col);

        let rect = ui.max_rect();

        // Background.
        ui.painter()
            .rect(rect, rounding, bg, border, egui::StrokeKind::Inside);

        // Invisible clickable overlay covering the whole cell rect.
        let interact = ui.interact(rect, ui.id().with((row, col)), Sense::click());

        if interact.clicked() {
            let value = self
                .data
                .get(row)
                .and_then(|r| r.get(col))
                .cloned()
                .unwrap_or_default();
            self.clicked_cell = Some(CellClick { row, col, value });
            self.clicked_row  = Some(RowClick { row });
        }

        // Subtle hover tint (skip if this cell is already selected).
        if interact.hovered() && self.selected_cell != Some((row, col)) {
            ui.painter()
                .rect_filled(rect, rounding, Color32::from_white_alpha(20));
        }

        // Cell content.
        Frame::new().inner_margin(padding).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            let text = self
                .data
                .get(row)
                .and_then(|r| r.get(col))
                .map(|s| s.as_str())
                .unwrap_or("");
            ui.colored_label(text_color, text);
        });
    }

    // ── Row geometry ─────────────────────────────────────────────────────────

    fn default_row_height(&self) -> f32 {
        self.row_height
    }
}
