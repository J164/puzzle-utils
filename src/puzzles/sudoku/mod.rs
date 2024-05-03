const NUM_MIN: u8 = 1;
const NUM_MAX: u8 = 9;

pub const GRID_SIZE: usize = 9;
const BOX_SIZE: usize = 3;

pub fn solve_sudoku(sudoku: &[u8]) -> Option<Vec<u8>> {
    let mut sudoku = sudoku.to_owned();
    let mut stack = Vec::with_capacity(GRID_SIZE * GRID_SIZE);

    if let Some(square) = next_blank(&sudoku, 0) {
        stack.push(square);
    } else {
        return Some(sudoku);
    }

    while !stack.is_empty() {
        let Square { index, candidates } = stack.last_mut().expect("stack should be non-empty");

        if candidates.is_empty() {
            sudoku[*index] = 0;
            stack.pop();
            continue;
        }

        sudoku[*index] = candidates.pop().expect("candidates should be non-empty");

        if let Some(square) = next_blank(&sudoku, *index) {
            stack.push(square);
        } else {
            return Some(sudoku);
        }
    }

    None
}

struct Square {
    index: usize,
    candidates: Vec<u8>,
}

fn next_blank(sudoku: &[u8], start: usize) -> Option<Square> {
    sudoku[start..]
        .iter()
        .enumerate()
        .find_map(|(index, &value)| {
            if value == 0 {
                Some(Square {
                    index: index + start,
                    candidates: candidates(sudoku, index + start),
                })
            } else {
                None
            }
        })
}

fn candidates(sudoku: &[u8], position: usize) -> Vec<u8> {
    let row = position / GRID_SIZE;
    let col = position % GRID_SIZE;
    let box_start = ((row / 3) * GRID_SIZE + col / 3) * BOX_SIZE;

    (NUM_MIN..=NUM_MAX)
        .filter(|&candidate| {
            let row_iter = (row * GRID_SIZE)..((row + 1) * GRID_SIZE);
            let col_iter = (col..=(col + (GRID_SIZE) * (GRID_SIZE - 1))).step_by(GRID_SIZE);
            let box_iter = (0..BOX_SIZE)
                .flat_map(|x| (box_start + x * GRID_SIZE)..(box_start + x * GRID_SIZE + BOX_SIZE));

            row_iter
                .chain(col_iter)
                .chain(box_iter)
                .all(|x| sudoku[x] != candidate)
        })
        .collect()
}

pub fn print_sudoku(sudoku: &[u8]) -> String {
    sudoku.iter().map(|x| x.to_string()).collect()
}
