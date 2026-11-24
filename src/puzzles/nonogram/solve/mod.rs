mod propagate;

use crate::puzzles::nonogram::{NonogramPuzzle, grid::Grid, solve::propagate::Propagator};

#[derive(Debug)]
pub struct Contradiction;

#[derive(Debug)]
pub(super) enum Progress {
    Solved,
    Incomplete,
}

pub(super) type SolveResult = Result<Progress, Contradiction>;

pub fn solve_nonogram(puzzle: &NonogramPuzzle) -> Result<Grid, Contradiction> {
    let width = puzzle.col_clues.len();
    let height = puzzle.row_clues.len();

    NonogramSolver::with_capacity(width, height).into_solution(puzzle)
}

pub struct NonogramSolver {
    grid: Grid,
    propagator: Propagator,
}

impl NonogramSolver {
    pub fn new() -> NonogramSolver {
        NonogramSolver {
            grid: Grid::new(),
            propagator: Propagator::new(),
        }
    }

    pub fn with_capacity(width: usize, height: usize) -> NonogramSolver {
        NonogramSolver {
            grid: Grid::with_capacity(width, height),
            propagator: Propagator::with_capacity(width, height),
        }
    }

    pub fn solve(&mut self, puzzle: &NonogramPuzzle) -> Result<&Grid, Contradiction> {
        let width = puzzle.col_clues.len();
        let height = puzzle.row_clues.len();

        self.grid.init(width, height);
        self.propagator.solve(&mut self.grid, puzzle)?;

        Ok(&self.grid)
    }

    pub fn into_solution(mut self, puzzle: &NonogramPuzzle) -> Result<Grid, Contradiction> {
        self.solve(puzzle)?;
        Ok(self.grid)
    }
}

impl Default for NonogramSolver {
    fn default() -> Self {
        Self::new()
    }
}
