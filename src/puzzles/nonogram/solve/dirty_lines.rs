use crate::puzzles::nonogram::grid::Change;

#[derive(Debug, Clone)]
pub(super) struct DirtyLines {
    rows: Vec<bool>,
    cols: Vec<bool>,
}

impl DirtyLines {
    pub(super) fn new() -> DirtyLines {
        DirtyLines {
            rows: Vec::new(),
            cols: Vec::new(),
        }
    }

    pub(super) fn with_capacity(width: usize, height: usize) -> DirtyLines {
        DirtyLines {
            rows: Vec::with_capacity(height),
            cols: Vec::with_capacity(width),
        }
    }

    pub(super) fn init(&mut self, width: usize, height: usize, value: bool) {
        self.rows.clear();
        self.rows.resize(height, value);
        self.cols.clear();
        self.cols.resize(width, value);
    }

    pub(super) fn mark_cell(&mut self, i: usize, j: usize) {
        self.rows[i] = true;
        self.cols[j] = true;
    }

    pub(super) fn mark_changes(&mut self, changes: &[Change]) {
        for change in changes {
            self.rows[change.row] = true;
            self.cols[change.col] = true;
        }
    }

    pub(super) fn clear(&mut self) {
        self.rows.fill(false);
        self.cols.fill(false);
    }

    pub(super) fn rows(&self) -> &[bool] {
        &self.rows
    }

    pub(super) fn cols(&self) -> &[bool] {
        &self.cols
    }
}
