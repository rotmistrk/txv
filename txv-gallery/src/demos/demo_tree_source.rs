//! Demo tree data source for TreeTableView.

use txv_widgets::tree_table_source::{CellValidator, ColAlign, TreeTableSource};

use super::demo_row::DemoRow;

/// A simple flat tree source with sample data for the gallery demo.
pub(crate) struct DemoTreeSource {
    rows: Vec<DemoRow>,
}

impl DemoTreeSource {
    pub(crate) fn new() -> Self {
        let rows = vec![
            DemoRow {
                label: "Widgets".into(),
                depth: 0,
                cells: ["Count".into(), "Status".into()],
            },
            DemoRow {
                label: "StatusBar".into(),
                depth: 1,
                cells: ["1".into(), "ok".into()],
            },
            DemoRow {
                label: "InputLine".into(),
                depth: 1,
                cells: ["2".into(), "ok".into()],
            },
            DemoRow {
                label: "Frame".into(),
                depth: 1,
                cells: ["1".into(), "ok".into()],
            },
        ];
        Self { rows }
    }
}

impl TreeTableSource for DemoTreeSource {
    fn visible_count(&self) -> usize {
        self.rows.len()
    }

    fn label(&self, row: usize) -> &str {
        self.rows.get(row).map_or("", |r| &r.label)
    }

    fn depth(&self, row: usize) -> usize {
        self.rows.get(row).map_or(0, |r| r.depth)
    }

    fn is_expandable(&self, _row: usize) -> bool {
        false
    }

    fn is_expanded(&self, _row: usize) -> bool {
        false
    }

    fn toggle(&mut self, _row: usize) {}

    fn column_count(&self) -> usize {
        2
    }

    fn cell(&self, row: usize, col: usize) -> &str {
        self.rows
            .get(row)
            .and_then(|r| r.cells.get(col))
            .map_or("", |s| s.as_str())
    }

    fn column_align(&self, col: usize) -> ColAlign {
        if col == 0 {
            ColAlign::Right
        } else {
            ColAlign::Left
        }
    }

    fn column_validator(&self, _col: usize) -> Option<&dyn CellValidator> {
        None
    }
}
