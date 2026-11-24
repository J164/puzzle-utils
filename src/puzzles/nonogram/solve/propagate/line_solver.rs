use std::{cmp::min, hint::assert_unchecked};

use crate::puzzles::nonogram::{grid::Cell, solve::Contradiction};

pub(super) struct LineSolver {
    line: Vec<Cell>,
    number_row: Vec<usize>,
    bool_row: Vec<bool>,
    table: Vec<TableEntry>,
}

#[derive(Debug, Clone)]
struct TableEntry {
    fix0: bool,
    fix1: bool,
    fix: bool,
    reached: bool,
}

impl LineSolver {
    pub(super) fn new() -> LineSolver {
        LineSolver {
            line: Vec::new(),
            number_row: Vec::new(),
            bool_row: Vec::new(),
            table: Vec::new(),
        }
    }

    pub(super) fn with_capacity(capacity: usize) -> LineSolver {
        LineSolver {
            line: Vec::with_capacity(capacity),
            number_row: Vec::with_capacity(capacity),
            bool_row: Vec::with_capacity(capacity),
            table: Vec::with_capacity(capacity),
        }
    }

    pub(super) fn solve(
        &mut self,
        cells: impl IntoIterator<Item = Cell>,
        clues: &[usize],
    ) -> Result<&[Cell], Contradiction> {
        self.setup_line(cells);

        let line_length = self.line.len();
        let clues_length = clues.len();

        let table_size = line_length * (clues_length + 1);

        self.number_row.clear();
        self.number_row.resize(line_length, 0);

        self.bool_row.clear();
        self.bool_row.resize(line_length, false);

        self.table.clear();
        self.table.resize(
            table_size,
            TableEntry {
                fix0: false,
                fix1: false,
                fix: false,
                reached: false,
            },
        );

        // SAFETY: line_length was set to self.line.len() and self.line's size has not been modified since
        unsafe {
            assert_unchecked(self.line.len() == line_length);
        }
        // SAFETY: self.number_row was resized to line_length and its size has not been modified since
        unsafe {
            assert_unchecked(self.number_row.len() == line_length);
        }
        // SAFETY: self.bool_row was resized to line_length and its size has not been modified since
        unsafe {
            assert_unchecked(self.bool_row.len() == line_length);
        }
        // SAFETY: self.table was resized to table_size and its size has not been modified since
        unsafe {
            assert_unchecked(self.table.len() == table_size);
        }

        let line = &mut self.line[..line_length];
        let number_row = &mut self.number_row[..line_length];
        let bool_row = &mut self.bool_row[..line_length];
        let table = &mut self.table[..table_size];

        fix(line, clues, number_row, table)?;
        paint(line, clues, number_row, bool_row, table);

        Ok(&line[1..])
    }

    #[inline(always)]
    fn setup_line(&mut self, cells: impl IntoIterator<Item = Cell>) {
        self.line.clear();
        self.line.push(Cell::Empty);
        self.line.extend(cells);

        // SAFETY: a sentinel empty cell was pushed to line so it must not be empty
        unsafe { assert_unchecked(!self.line.is_empty()) };
    }
}

#[inline(always)]
fn fix0(cell: Cell, previous_fix: bool) -> bool {
    !matches!(cell, Cell::Filled) & previous_fix
}

#[inline(always)]
fn fix(
    line: &[Cell],
    clues: &[usize],
    run: &mut [usize],
    table: &mut [TableEntry],
) -> Result<(), Contradiction> {
    let line_length = line.len();

    for i in 1..line_length {
        run[i] = (!matches!(line[i], Cell::Empty) as usize) * (run[i - 1] + 1);
    }

    let mut rows = table.chunks_exact_mut(line_length);
    // SAFETY: the table must have at least one row for the subproblem where there are no clues
    let first_row = unsafe { rows.next().unwrap_unchecked() };

    first_row[0] = TableEntry {
        fix0: true,
        fix1: false,
        fix: true,
        reached: false,
    };
    let mut previous_fix = true;
    for i in 1..line_length {
        let fix0 = fix0(line[i], previous_fix);
        first_row[i] = TableEntry {
            fix0,
            fix1: false,
            fix: fix0,
            reached: false,
        };

        previous_fix = fix0;
    }

    let mut previous = first_row;
    for (&clue, current) in clues.iter().zip(rows) {
        let mut previous_fix = false;

        for i in 1..line_length {
            let fix0 = fix0(line[i], previous_fix);
            let fix1 = i >= clue && ((run[i] >= clue) & previous[i - clue].fix0);
            let fix = fix0 | fix1;
            current[i] = TableEntry {
                fix0,
                fix1,
                fix,
                reached: false,
            };

            previous_fix = fix;
        }

        previous = current;
    }

    previous[line_length - 1].fix.ok_or(Contradiction)
}

#[inline(always)]
fn paint(
    line: &mut [Cell],
    clues: &[usize],
    sequence_start: &mut [usize],
    possibly_zero: &mut [bool],
    table: &mut [TableEntry],
) {
    let line_length = line.len();

    sequence_start.fill(usize::MAX);

    let mut rows = table.rchunks_exact_mut(line_length);
    // SAFETY: the table must have at least one row for the subproblem where there are no clues
    let mut current = unsafe { rows.next().unwrap_unchecked() };

    current[line_length - 1].reached = true;
    for (&clue, next) in clues.iter().rev().zip(rows) {
        for i in (1..line_length).rev() {
            if !current[i].reached {
                continue;
            }

            if current[i].fix0 {
                possibly_zero[i] = true;

                current[i - 1].reached = true;
            }

            if current[i].fix1 && i >= clue {
                possibly_zero[i - clue] = true;
                sequence_start[i] = min(sequence_start[i], i - clue + 1);

                if i > clue {
                    next[i - clue - 1].reached = true;
                }
            }
        }

        current = next;
    }

    for i in (1..line_length).rev() {
        if current[i].reached && current[i].fix0 {
            possibly_zero[i] = true;
            current[i - 1].reached = true;
        }
    }

    let mut lowest = line_length;
    for i in (1..line_length).rev() {
        lowest = min(lowest, sequence_start[i]);

        let can_be_zero = possibly_zero[i] as u8;
        let can_be_one = (lowest <= i) as u8;

        line[i] = match can_be_one << 1 | can_be_zero {
            0 | 1 => Cell::Empty,
            2 => Cell::Filled,
            _ => Cell::Unknown,
        };
    }
}
