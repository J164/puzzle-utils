const NUM_MIN: u8 = 1;
const NUM_MAX: u8 = 9;

const GRID_SIZE: usize = 9;
const BOX_SIZE: usize = 3;

#[derive(Debug)]
pub enum SudokuError {
    NoSolution,
}

pub fn solve_sudoku(sudoku: &[Option<u8>]) -> Result<Vec<Option<u8>>, SudokuError> {
    let mut sudoku = sudoku.to_owned();
    let mut stack = Vec::with_capacity(GRID_SIZE * GRID_SIZE);

    if let Some(square) = next_blank(&sudoku, 0) {
        stack.push(square);
    } else {
        return Ok(sudoku);
    }

    while !stack.is_empty() {
        let Square { index, candidates } = stack.last_mut().unwrap();
        
        if candidates.is_empty() {
            sudoku[*index] = None;
            stack.pop();
            continue;
        }

        sudoku[*index] = candidates.pop();

        if let Some(square) = next_blank(&sudoku, *index) {
            stack.push(square);
        } else {
            return Ok(sudoku);
        }
    }

    Err(SudokuError::NoSolution)
}

struct Square {
    index: usize,
    candidates: Vec<u8>,
}

fn next_blank(sudoku: &[Option<u8>], start: usize) -> Option<Square> {
    sudoku[start..]
        .iter()
        .enumerate()
        .find_map(|(index, value)| match value {
            Some(_) => None,
            None => Some(Square {
                index: index + start,
                candidates: candidates(sudoku, index + start),
            }),
        })
}

pub fn candidates(sudoku: &[Option<u8>], position: usize) -> Vec<u8> {
    let row = position / GRID_SIZE;
    let col = position % GRID_SIZE;
    let box_start = ((row / 3) * GRID_SIZE + col / 3) * BOX_SIZE;

    (NUM_MIN..=NUM_MAX)
        .filter(|candidate| {
            let row_iter = (row * GRID_SIZE)..((row + 1) * GRID_SIZE);
            let col_iter = (col..=(col + (GRID_SIZE) * (GRID_SIZE - 1))).step_by(GRID_SIZE);
            let box_iter = (0..BOX_SIZE)
                .flat_map(|x| (box_start + x * GRID_SIZE)..(box_start + x * GRID_SIZE + BOX_SIZE));

            row_iter
                .chain(col_iter)
                .chain(box_iter)
                .all(|x| sudoku[x] != Some(*candidate))
        })
        .collect()
}

pub fn print_sudoku(sudoku: &[Option<u8>]) {
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            print!(
                "{}{}",
                match sudoku[row * 9 + col] {
                    Some(x) => x.to_string(),
                    None => " ".to_string(),
                },
                if col % GRID_SIZE == GRID_SIZE - 1 {
                    '\n'
                } else if col % BOX_SIZE == BOX_SIZE - 1 {
                    '|'
                } else {
                    ' '
                }
            );
        }

        if row % BOX_SIZE == BOX_SIZE - 1 && row % GRID_SIZE != GRID_SIZE - 1 {
            for col in 0..GRID_SIZE {
                print!("—{}", if col == GRID_SIZE - 1 { '\n' } else { ' ' });
            }
        }
    }
}
