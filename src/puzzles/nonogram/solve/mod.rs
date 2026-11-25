mod dirty_lines;
mod probe;
mod propagate;

use std::mem::replace;

use crate::puzzles::nonogram::{
    NonogramPuzzle,
    grid::Grid,
    solve::{dirty_lines::DirtyLines, probe::Prober, propagate::Propagator},
};

#[derive(Debug)]
pub struct Contradiction;

pub fn solve_nonogram(puzzle: &NonogramPuzzle) -> Result<Grid, Contradiction> {
    let width = puzzle.col_clues.len();
    let height = puzzle.row_clues.len();

    NonogramSolver::with_capacity(width, height).into_solution(puzzle)
}

pub struct NonogramSolver {
    grid: Grid,
    dirty: DirtyLines,
    propagator: Propagator,
    prober: Prober,
    stack: Vec<Grid>,
}

impl NonogramSolver {
    pub fn new() -> NonogramSolver {
        NonogramSolver {
            grid: Grid::new(),
            dirty: DirtyLines::new(),
            propagator: Propagator::new(),
            prober: Prober::new(),
            stack: Vec::new(),
        }
    }

    pub fn with_capacity(width: usize, height: usize) -> NonogramSolver {
        NonogramSolver {
            grid: Grid::with_capacity(width, height),
            dirty: DirtyLines::with_capacity(width, height),
            propagator: Propagator::with_capacity(width, height),
            prober: Prober::new(),
            stack: Vec::new(),
        }
    }

    pub fn solve(&mut self, puzzle: &NonogramPuzzle) -> Result<&Grid, Contradiction> {
        let width = puzzle.col_clues.len();
        let height = puzzle.row_clues.len();

        self.grid.init(width, height);
        self.dirty.init(width, height, true);

        let progress = self.propagator.solve(&mut self.grid, puzzle, &self.dirty)?;

        if matches!(progress, propagate::Progress::Solved) {
            return Ok(&self.grid);
        }

        self.stack.clear();
        self.stack.push(replace(&mut self.grid, Grid::new()));
        while let Some(mut grid) = self.stack.pop() {
            let result = self.prober.solve(&mut self.propagator, &mut grid, puzzle);

            match result {
                Ok(probe::Progress::Solved) => {
                    self.grid = grid;
                    return Ok(&self.grid);
                }
                Ok(probe::Progress::Incomplete((empty, filled))) => {
                    self.stack.push(empty);
                    self.stack.push(filled);
                }
                Err(Contradiction) => self.grid = grid,
            };
        }

        Err(Contradiction)
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
