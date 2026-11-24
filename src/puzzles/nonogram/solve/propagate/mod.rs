mod line_solver;

use std::cmp::max;

use crate::puzzles::nonogram::{
    NonogramPuzzle,
    grid::{Axis, Grid},
    solve::{Contradiction, Progress, SolveResult, propagate::line_solver::LineSolver},
};

pub(super) struct Propagator {
    line_solver: LineSolver,
    row_changed: Vec<bool>,
    col_changed: Vec<bool>,
}

impl Propagator {
    pub(super) fn new() -> Propagator {
        Propagator {
            line_solver: LineSolver::new(),
            row_changed: Vec::new(),
            col_changed: Vec::new(),
        }
    }

    pub(super) fn with_capacity(width: usize, height: usize) -> Propagator {
        Propagator {
            line_solver: LineSolver::with_capacity(max(width, height)),
            row_changed: Vec::with_capacity(height),
            col_changed: Vec::with_capacity(width),
        }
    }

    pub(super) fn solve(&mut self, grid: &mut Grid, puzzle: &NonogramPuzzle) -> SolveResult {
        self.row_changed.clear();
        self.row_changed.resize(grid.height(), true);
        self.col_changed.clear();
        self.col_changed.resize(grid.width(), true);

        loop {
            self.sweep::<true>(grid, puzzle)?;

            if !self.sweep::<false>(grid, puzzle)? {
                break;
            }
        }

        Ok(if grid.is_solved() {
            Progress::Solved
        } else {
            Progress::Incomplete
        })
    }

    fn sweep<const ROW: bool>(
        &mut self,
        grid: &mut Grid,
        puzzle: &NonogramPuzzle,
    ) -> Result<bool, Contradiction> {
        let (axis, clues, dirty, crossing_dirty) = if ROW {
            (
                Axis::Row,
                &puzzle.row_clues,
                &mut self.row_changed,
                &mut self.col_changed,
            )
        } else {
            (
                Axis::Col,
                &puzzle.col_clues,
                &mut self.col_changed,
                &mut self.row_changed,
            )
        };

        let mut crossing_changed = false;
        for (i, dirty) in dirty.iter_mut().enumerate() {
            if !*dirty {
                continue;
            }

            let updated_line = self.line_solver.solve(grid.line(axis, i), &clues[i])?;

            for ((grid_cell, &updated_cell), crossing_dirty) in grid
                .line_mut(axis, i)
                .zip(updated_line)
                .zip(crossing_dirty.iter_mut())
            {
                let changed = *grid_cell != updated_cell;
                *crossing_dirty |= changed;
                crossing_changed |= changed;
                *grid_cell = updated_cell;
            }

            *dirty = false;
        }

        Ok(crossing_changed)
    }
}
