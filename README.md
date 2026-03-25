# simple-table

An easy-to-use egui table widget built on [`egui_table`] v0.7.

LLM-generated

Features:
- Scrollable in both horizontal and vertical dimensions
- Fixed (sticky) header row that stays visible while scrolling vertically
- Fixed (sticky) row-header column that stays visible while scrolling horizontally
- Separate styling for header cells vs data cells
- Click selection with visual highlight — row and cell callbacks via [`TableResponse`]
- Simple data-driven API — just provide `Vec<String>` column names and a 2-D data array
## Quick start
```rust,no_run
use simple_table::{FancyTable, CellStyle, HeaderStyle};
use egui::Color32;
let columns = vec![
    "Row Label".to_string(),
    "Alpha".to_string(),
    "Beta".to_string(),
];
let data = vec![
    vec!["Row 1".to_string(), "1.0".to_string(), "2.0".to_string()],
    vec!["Row 2".to_string(), "3.0".to_string(), "4.0".to_string()],
];
let mut table = FancyTable::new("my_table", columns, data);

// In your egui update() / show() callback:
// let response = table.show(ui);
// if let Some(cell) = response.clicked_cell {
//     println!("clicked row={} col={} value={}",
//     cell.row, cell.col, cell.value);
// }
// if let Some(row) = response.clicked_row {
//     println!("clicked row={}", row.row);
// }
```