mod grid;
mod parse;
mod solve;

use thiserror::Error;

pub use grid::{Cell, Grid};
pub use solve::{Contradiction, NonogramSolver, solve_nonogram};

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum NonogramParseError {
    #[error("invalid clue `{0}`, a clue containing `0` must be exactly `[0]`")]
    MixedZeroClue(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonogramPuzzle {
    row_clues: Vec<Vec<usize>>,
    col_clues: Vec<Vec<usize>>,
}

impl NonogramPuzzle {
    pub fn new(
        row_clues: Vec<Vec<usize>>,
        col_clues: Vec<Vec<usize>>,
    ) -> Result<NonogramPuzzle, NonogramParseError> {
        for clues in col_clues.iter().chain(row_clues.iter()) {
            if clues.len() > 1 && clues.contains(&0) {
                return Err(NonogramParseError::MixedZeroClue(format!("{clues:?}")));
            }
        }

        Ok(NonogramPuzzle {
            row_clues,
            col_clues,
        })
    }

    pub fn row_clues(&self) -> &[Vec<usize>] {
        &self.row_clues
    }

    pub fn col_clues(&self) -> &[Vec<usize>] {
        &self.col_clues
    }
}
