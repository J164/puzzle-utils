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

    fn init(&mut self, grid: &Grid, pixel: usize, log_length: usize) {
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

    pub(super) fn probe(
        &mut self,
        pixel: usize,
        propagator: &mut Propagator,
        puzzle: &NonogramPuzzle,
        grid: &mut Grid,
        log: &mut Vec<Change>,
    ) -> Result<(), Contradiction> {
        let fresh = !self.initialized;
        if fresh {
            self.init(grid, pixel, log.len());
        }

        let new = &log[self.synced..];
        if new.is_empty() && !fresh {
            return Ok(());
        }

        self.dirty.mark_changes(new);

        let empty_result = update_candidate(&mut self.empty, &self.dirty, new, puzzle, propagator);
        let filled_result =
            update_candidate(&mut self.filled, &self.dirty, new, puzzle, propagator);

        let (a, b) = match (empty_result, filled_result) {
            (Ok(_), Err(_)) => (&self.empty, &self.empty),
            (Err(_), Ok(_)) => (&self.filled, &self.filled),
            (Ok(_), Ok(_)) => (&self.empty, &self.filled),
            (Err(_), Err(_)) => return Err(Contradiction),
        };
        grid.merge_from(a, b, log);

        self.dirty.clear();
        self.synced = log.len();

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

fn update_candidate(
    candidate: &mut Grid,
    dirty: &DirtyLines,
    new: &[Change],
    puzzle: &NonogramPuzzle,
    propagator: &mut Propagator,
) -> Result<(), Contradiction> {
    if candidate.apply(new) {
        return Err(Contradiction);
    }

    propagator.solve(candidate, puzzle, dirty).map(|_| ())
}
