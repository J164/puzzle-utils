mod candidate_pair;

use std::cmp::max;

use crate::puzzles::nonogram::{
    Cell, Contradiction, Grid, NonogramPuzzle,
    grid::Change,
    solve::{Propagator, probe::candidate_pair::CandidatePair},
};

pub(super) struct Prober {
    candidates: Vec<CandidatePair>,
    log: Vec<Change>,
}

#[derive(Debug)]
pub(super) enum Progress {
    Solved,
    Incomplete,
}

pub(super) type SolveResult = Result<Progress, Contradiction>;

impl Prober {
    pub(super) fn new() -> Prober {
        Prober {
            candidates: Vec::new(),
            log: Vec::new(),
        }
    }

    pub(super) fn solve(
        &mut self,
        propagator: &mut Propagator,
        grid: &mut Grid,
        puzzle: &NonogramPuzzle,
    ) -> SolveResult {
        let n = grid.width() * grid.height();

        self.candidates
            .resize(max(self.candidates.len(), n), CandidatePair::new());

        let candidates = &mut self.candidates[..n];
        for candidate in candidates.iter_mut() {
            candidate.reset();
        }

        self.log.clear();

        loop {
            let mut made_progress = false;

            for i in 0..n {
                if !matches!(grid.get_cell(i), Cell::Unknown) {
                    continue;
                }

                let start = self.log.len();
                candidates[i].init(grid, i, start);
                candidates[i].probe(propagator, puzzle, grid, &mut self.log)?;

                if grid.is_solved() {
                    return Ok(Progress::Solved);
                }

                made_progress |= self.log.len() > start;
            }

            if !made_progress {
                break;
            }
        }

        Ok(if grid.is_solved() {
            Progress::Solved
        } else {
            Progress::Incomplete
        })
    }
}
