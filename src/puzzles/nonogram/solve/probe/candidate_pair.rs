use std::mem::replace;

use crate::puzzles::nonogram::{
    Cell, Contradiction, Grid, NonogramPuzzle,
    grid::Change,
    solve::{DirtyLines, Propagator},
};

#[derive(Debug, Clone)]
pub(super) struct CandidatePair {
    empty: Grid,
    filled: Grid,
    dirty: DirtyLines,
    synced: usize,
    initialized: bool,
}

impl CandidatePair {
    pub(super) fn new() -> CandidatePair {
        CandidatePair {
            empty: Grid::new(),
            filled: Grid::new(),
            dirty: DirtyLines::new(),
            synced: 0,
            initialized: false,
        }
    }

    pub(super) fn init(&mut self, grid: &Grid, pixel: usize, log_length: usize) {
        if self.initialized {
            return;
        }

        let width = grid.width();
        let height = grid.height();

        self.empty.init_from_grid(grid);
        self.filled.init_from_grid(grid);
        self.dirty.init(width, height, false);

        self.empty.set_cell(pixel, Cell::Empty);
        self.filled.set_cell(pixel, Cell::Filled);
        self.dirty.mark_cell(pixel / width, pixel % width);

        self.synced = log_length;
        self.initialized = true;
    }

    fn sync(&mut self, log: &[Change]) {
        let new = &log[self.synced..];

        self.empty.apply(new);
        self.filled.apply(new);

        let (rows, cols) = self.dirty.rows_cols_mut();
        for change in new {
            rows[change.row] = true;
            cols[change.col] = true;
        }

        self.synced = log.len();
    }

    pub(super) fn probe(
        &mut self,
        propagator: &mut Propagator,
        puzzle: &NonogramPuzzle,
        grid: &mut Grid,
        log: &mut Vec<Change>,
    ) -> Result<(), Contradiction> {
        self.sync(log);
        if !self.dirty.is_dirty() {
            return Ok(());
        }

        let empty_result = propagator.solve(&mut self.empty, puzzle, &self.dirty);
        let filled_result = propagator.solve(&mut self.filled, puzzle, &self.dirty);

        let (a, b) = match (empty_result, filled_result) {
            (Ok(_), Err(_)) => (&self.empty, &self.empty),
            (Err(_), Ok(_)) => (&self.filled, &self.filled),
            (Ok(_), Ok(_)) => (&self.empty, &self.filled),
            (Err(_), Err(_)) => return Err(Contradiction),
        };
        grid.merge_from(a, b, log);

        self.dirty.clear();

        Ok(())
    }

    pub(super) fn branch_score(&self, grid: &Grid) -> f64 {
        let p0 = grid.unknown() - self.empty.unknown();
        let p1 = grid.unknown() - self.filled.unknown();

        p0.min(p1) as f64 + (p0.abs_diff(p1) as f64).ln_1p()
    }

    pub(super) fn take_grids(&mut self) -> (Grid, Grid) {
        self.initialized = false;

        (
            replace(&mut self.empty, Grid::new()),
            replace(&mut self.filled, Grid::new()),
        )
    }

    pub(super) fn reset(&mut self) {
        self.initialized = false;
    }
}
