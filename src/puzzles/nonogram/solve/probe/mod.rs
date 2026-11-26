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
    Incomplete((Grid, Grid)),
}

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
    ) -> Result<Progress, Contradiction> {
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

            for (i, candidate) in candidates.iter_mut().enumerate() {
                if !matches!(grid.get_cell(i), Cell::Unknown) {
                    continue;
                }

                let start = self.log.len();
                candidate.probe(i, propagator, puzzle, grid, &mut self.log)?;

                if grid.is_solved() {
                    return Ok(Progress::Solved);
                }

                made_progress |= self.log.len() > start;
            }

            if !made_progress {
                break;
            }
        }

        if grid.is_solved() {
            return Ok(Progress::Solved);
        }

        let best = (0..n)
            .filter(|&i| matches!(grid.get_cell(i), Cell::Unknown))
            .max_by(|&a, &b| {
                candidates[a]
                    .branch_score(grid)
                    .total_cmp(&candidates[b].branch_score(grid))
            })
            .expect("an unsolved grid has an unknown cell");

        Ok(Progress::Incomplete(candidates[best].take_grids()))
    }
}
