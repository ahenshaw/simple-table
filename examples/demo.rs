//! Demo application for `simple_table`.
//!
//! Run with:  `cargo run -p demo`
//!
//! The window shows a large synthetic dataset (20 columns × 50 rows).
//! Click any cell to select it — the clicked row is highlighted in light blue
//! and the specific cell in darker blue.  The side panel shows the last
//! clicked row and cell so you can verify the callbacks work.
//!
//! A style-preset picker lets you switch between three colour themes.

use eframe::egui::{self, Color32, CornerRadius, Margin, Stroke};
use simple_table::{CellStyle, FancyTable, HeaderStyle, SelectionStyle};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("simple-table demo")
            .with_inner_size([1100.0, 650.0]),
        ..Default::default()
    };
    eframe::run_native(
        "simple-table demo",
        options,
        Box::new(|_cc| Ok(Box::new(DemoApp::new()))),
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Style presets
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum StylePreset {
    Blue,
    Dark,
    Warm,
}

impl StylePreset {
    fn label(self) -> &'static str {
        match self {
            Self::Blue => "Blue (default)",
            Self::Dark => "Dark",
            Self::Warm => "Warm amber",
        }
    }

    fn header_style(self) -> HeaderStyle {
        match self {
            Self::Blue => HeaderStyle::default(),
            Self::Dark => HeaderStyle {
                bg_color: Color32::from_rgb(30, 30, 35),
                text_color: Color32::from_rgb(200, 210, 255),
                padding: Margin::same(6),
                border: Stroke::new(1.0, Color32::from_rgb(60, 60, 80)),
                rounding: CornerRadius::ZERO,
            },
            Self::Warm => HeaderStyle {
                bg_color: Color32::from_rgb(160, 80, 20),
                text_color: Color32::WHITE,
                padding: Margin::same(7),
                border: Stroke::new(1.0, Color32::from_rgb(120, 50, 10)),
                rounding: CornerRadius::same(3),
            },
        }
    }

    fn cell_style(self) -> CellStyle {
        match self {
            Self::Blue => CellStyle::default(),
            Self::Dark => CellStyle {
                bg_color: Color32::from_rgb(40, 42, 54),
                alt_bg_color: Color32::from_rgb(48, 50, 65),
                text_color: Color32::from_rgb(215, 218, 230),
                padding: Margin::same(6),
                border: Stroke::new(0.5, Color32::from_rgb(60, 62, 80)),
                rounding: CornerRadius::ZERO,
            },
            Self::Warm => CellStyle {
                bg_color: Color32::from_rgb(255, 248, 235),
                alt_bg_color: Color32::from_rgb(255, 238, 210),
                text_color: Color32::from_rgb(60, 30, 5),
                padding: Margin::same(7),
                border: Stroke::new(0.5, Color32::from_rgb(200, 160, 100)),
                rounding: CornerRadius::ZERO,
            },
        }
    }

    fn selection_style(self) -> SelectionStyle {
        match self {
            Self::Blue => SelectionStyle::default(),
            Self::Dark => SelectionStyle {
                row_bg: Color32::from_rgb(60, 65, 100),
                cell_bg: Color32::from_rgb(80, 120, 220),
                text_color: Some(Color32::WHITE),
                cell_border: Stroke::new(2.0, Color32::from_rgb(100, 160, 255)),
            },
            Self::Warm => SelectionStyle {
                row_bg: Color32::from_rgb(255, 210, 140),
                cell_bg: Color32::from_rgb(200, 100, 20),
                text_color: Some(Color32::WHITE),
                cell_border: Stroke::new(2.0, Color32::from_rgb(140, 60, 0)),
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Data generation
// ─────────────────────────────────────────────────────────────────────────────

const NUM_DATA_COLS: usize = 50; // + 1 row-header column
const NUM_ROWS: usize = 2000;

fn make_column_names() -> Vec<String> {
    let mut names = vec!["Metric".to_string()];
    for i in 0..NUM_DATA_COLS {
        names.push(format!("Col {}", i + 1));
    }
    names
}

fn make_data() -> Vec<Vec<String>> {
    (0..NUM_ROWS)
        .map(|row| {
            let mut cells = vec![format!("Row {:>2}", row + 1)];
            for col in 0..NUM_DATA_COLS {
                let value = ((row * NUM_DATA_COLS + col) as f64 * 1.23456).sin() * 1000.0;
                cells.push(format!("{:.2}", value));
            }
            cells
        })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// App state
// ─────────────────────────────────────────────────────────────────────────────

struct DemoApp {
    table: FancyTable,
    selected_preset: StylePreset,
    // Last click info for the side panel display.
    last_row_click: Option<usize>,
    last_cell_click: Option<(usize, usize, String)>,
}

impl DemoApp {
    fn new() -> Self {
        let preset = StylePreset::Blue;
        let table = Self::build_table(preset);
        Self {
            table,
            selected_preset: preset,
            last_row_click: None,
            last_cell_click: None,
        }
    }

    fn build_table(preset: StylePreset) -> FancyTable {
        FancyTable::new("demo_table", make_column_names(), make_data())
            .header_style(preset.header_style())
            .cell_style(preset.cell_style())
            .selection_style(preset.selection_style())
            .header_height(30.0)
            .row_height(22.0)
            .min_col_width(90.0)
    }

    fn apply_preset(&mut self, preset: StylePreset) {
        self.selected_preset = preset;
        // Preserve selection across theme change.
        let sel_row  = self.table.selected_row();
        let sel_cell = self.table.selected_cell();
        self.table = Self::build_table(preset);
        if let Some((r, c)) = sel_cell {
            self.table.select_cell(r, c);
        } else if let Some(r) = sel_row {
            self.table.select_row(Some(r));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// eframe App impl
// ─────────────────────────────────────────────────────────────────────────────

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Side panel ───────────────────────────────────────────────────────
        egui::SidePanel::left("controls")
            .resizable(false)
            .exact_width(200.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading("simple-table demo");
                ui.add_space(12.0);

                ui.label("Style preset:");
                ui.add_space(4.0);
                for preset in [StylePreset::Blue, StylePreset::Dark, StylePreset::Warm] {
                    if ui
                        .selectable_label(self.selected_preset == preset, preset.label())
                        .clicked()
                        && self.selected_preset != preset
                    {
                        self.apply_preset(preset);
                    }
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(format!("Rows: {}", self.table.num_rows()));
                ui.label(format!("Cols: {}", self.table.num_cols()));

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // ── Selection state ──────────────────────────────────────────
                ui.label(egui::RichText::new("Selection").strong());
                ui.add_space(4.0);

                match self.table.selected_row() {
                    Some(r) => { ui.label(format!("Row:  {}", r)); }
                    None    => { ui.label("Row:  —"); }
                }
                match self.table.selected_cell() {
                    Some((r, c)) => { ui.label(format!("Cell: ({}, {})", r, c)); }
                    None         => { ui.label("Cell: —"); }
                }

                ui.add_space(8.0);
                if ui.button("Clear selection").clicked() {
                    self.table.clear_selection();
                    self.last_row_click  = None;
                    self.last_cell_click = None;
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // ── Last click callbacks ─────────────────────────────────────
                ui.label(egui::RichText::new("Last click callbacks").strong());
                ui.add_space(4.0);

                match &self.last_row_click {
                    Some(r) => { ui.label(format!("clicked_row:\n  row={}", r)); }
                    None    => { ui.label("clicked_row: —"); }
                }
                ui.add_space(4.0);
                match &self.last_cell_click {
                    Some((r, c, v)) => {
                        ui.label(format!("clicked_cell:\n  row={} col={}\n  value={}", r, c, v));
                    }
                    None => { ui.label("clicked_cell: —"); }
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.small("• Header row: fixed (sticky)");
                ui.small("• Column 0: fixed (sticky)");
                ui.small("• Click any cell to select");
                ui.small("• Drag column edges to resize");
            });

        // ── Central panel: the table ─────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            let response = self.table.show(ui);

            // Handle callbacks.
            if let Some(row) = response.clicked_row {
                self.last_row_click = Some(row.row);
            }
            if let Some(cell) = response.clicked_cell {
                self.last_cell_click = Some((cell.row, cell.col, cell.value));
            }
        });
    }
}
