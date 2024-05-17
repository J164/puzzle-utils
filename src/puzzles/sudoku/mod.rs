mod mask;

use ab_glyph::FontRef;
use image::RgbImage;
use imageproc::drawing::draw_text_mut;
use thiserror::Error;

use crate::{
    cloudflare_image::SolutionPair,
    util::{RgbBuffer, BLACK_PIXEL, ROBOTO_MEDIUM, WHITE_PIXEL},
};

use self::mask::Mask;

const GRID_SIZE: usize = 9;

#[derive(Debug, Error)]
pub enum SudokuError {
    #[error("invalid integer `{0}`, must be integer 1-9 (use `0` for an empty space)")]
    InvalidInteger(char),
    #[error("sudoku must by 9 x 9, got {0} entries")]
    InvalidSize(usize),
    #[error("sudoku has no solution")]
    NoSolution,
}

pub fn solve_sudoku(puzzle: &str) -> Result<SolutionPair, SudokuError> {
    let original = parse_sudoku(puzzle)?;
    let solved = solve(&original)?;
    Ok(SolutionPair::new(
        print_sudoku(original),
        print_sudoku(solved),
    ))
}

fn parse_sudoku(puzzle: &str) -> Result<Vec<u8>, SudokuError> {
    let puzzle = puzzle
        .chars()
        .map(|char| {
            char.to_digit(10)
                .map(|x| x as u8)
                .ok_or(SudokuError::InvalidInteger(char))
        })
        .collect::<Result<Vec<u8>, SudokuError>>()?;

    if puzzle.len() != GRID_SIZE * GRID_SIZE {
        return Err(SudokuError::InvalidSize(puzzle.len()));
    }

    Ok(puzzle)
}

fn solve(puzzle: &Vec<u8>) -> Result<Vec<u8>, SudokuError> {
    let mut puzzle = puzzle.to_owned();
    let mut stack = Vec::with_capacity(GRID_SIZE * GRID_SIZE);

    let mut mask = Mask::new();

    for (index, &value) in puzzle.iter().enumerate() {
        if value == 0 {
            continue;
        }

        mask.set(index, value);
    }

    if let Some(square) = next_blank(&puzzle, &mask, 0) {
        stack.push(square);
    } else {
        return Ok(puzzle);
    }

    while !stack.is_empty() {
        let Square { index, candidates } = stack.last_mut().expect("stack should be non-empty");

        let previous = puzzle[*index];
        if previous != 0 {
            puzzle[*index] = 0;
            mask.clear(*index, previous);
        }

        if candidates.is_empty() {
            stack.pop();
            continue;
        }

        let value = candidates.pop().expect("candidates should be non-empty");
        puzzle[*index] = value;
        mask.set(*index, value);

        if let Some(square) = next_blank(&puzzle, &mask, *index) {
            stack.push(square);
        } else {
            return Ok(puzzle);
        }
    }

    Err(SudokuError::NoSolution)
}

struct Square {
    index: usize,
    candidates: Vec<u8>,
}

fn next_blank(puzzle: &[u8], mask: &Mask, start: usize) -> Option<Square> {
    puzzle[start..]
        .iter()
        .enumerate()
        .find_map(|(index, &value)| {
            if value == 0 {
                Some(Square {
                    index: index + start,
                    candidates: mask.candidates(index + start),
                })
            } else {
                None
            }
        })
}

fn print_sudoku(sudoku: Vec<u8>) -> RgbBuffer {
    const IMAGE_SIZE: u32 = GRID_SIZE as u32 * 100;

    let mut image = RgbImage::from_pixel(IMAGE_SIZE, IMAGE_SIZE, WHITE_PIXEL);

    let font = FontRef::try_from_slice(ROBOTO_MEDIUM).expect("Font should be valid");

    for grid_pos in 0..(GRID_SIZE as u32) {
        for line_coord in 0..IMAGE_SIZE {
            let grid_coord = grid_pos * 100;

            if grid_pos % 3 == 0 && grid_pos != 0 && grid_pos != GRID_SIZE as u32 - 1 {
                image.put_pixel(line_coord, grid_coord + 1, BLACK_PIXEL);
                image.put_pixel(line_coord, grid_coord - 1, BLACK_PIXEL);

                image.put_pixel(grid_coord + 1, line_coord, BLACK_PIXEL);
                image.put_pixel(grid_coord - 1, line_coord, BLACK_PIXEL);
            }

            image.put_pixel(line_coord, grid_coord, BLACK_PIXEL);
            image.put_pixel(grid_coord, line_coord, BLACK_PIXEL);
        }
    }

    for (i, &number) in sudoku.iter().enumerate() {
        if number == 0 {
            continue;
        }

        let x = (i % GRID_SIZE) * 100 + 25;
        let y = (i / GRID_SIZE) * 100 + 5;

        draw_text_mut(
            &mut image,
            BLACK_PIXEL,
            x as i32,
            y as i32,
            100.0,
            &font,
            &number.to_string(),
        );
    }

    image
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::ImageFormat;

    use super::*;

    #[test]
    fn easy_solve() {
        let puzzle_string =
            "415830090003009104002150006900783000200000381500012400004900063380500040009307500";

        let expected_puzzle = vec![
            4, 1, 5, 8, 3, 0, 0, 9, 0, 0, 0, 3, 0, 0, 9, 1, 0, 4, 0, 0, 2, 1, 5, 0, 0, 0, 6, 9, 0,
            0, 7, 8, 3, 0, 0, 0, 2, 0, 0, 0, 0, 0, 3, 8, 1, 5, 0, 0, 0, 1, 2, 4, 0, 0, 0, 0, 4, 9,
            0, 0, 0, 6, 3, 3, 8, 0, 5, 0, 0, 0, 4, 0, 0, 0, 9, 3, 0, 7, 5, 0, 0,
        ];
        let expected_solution = vec![
            4, 1, 5, 8, 3, 6, 2, 9, 7, 8, 6, 3, 2, 7, 9, 1, 5, 4, 7, 9, 2, 1, 5, 4, 8, 3, 6, 9, 4,
            1, 7, 8, 3, 6, 2, 5, 2, 7, 6, 4, 9, 5, 3, 8, 1, 5, 3, 8, 6, 1, 2, 4, 7, 9, 1, 5, 4, 9,
            2, 8, 7, 6, 3, 3, 8, 7, 5, 6, 1, 9, 4, 2, 6, 2, 9, 3, 4, 7, 5, 1, 8,
        ];

        let puzzle = parse_sudoku(puzzle_string).expect("should be ok");

        assert_eq!(puzzle, expected_puzzle);

        let solution = solve(&puzzle).expect("should be ok");

        assert_eq!(solution, expected_solution);

        let mut unsolved_image = Vec::new();
        let write_result =
            print_sudoku(puzzle).write_to(&mut Cursor::new(&mut unsolved_image), ImageFormat::Png);
        assert!(write_result.is_ok());

        assert_eq!(
            unsolved_image,
            include_bytes!("../../../tests/sudoku/unsolved/easy.png")
        );

        let mut solved_image = Vec::new();
        let write_result =
            print_sudoku(solution).write_to(&mut Cursor::new(&mut solved_image), ImageFormat::Png);
        assert!(write_result.is_ok());

        assert_eq!(
            solved_image,
            include_bytes!("../../../tests/sudoku/solved/easy.png")
        );
    }

    #[test]
    fn medium_solve() {
        let puzzle_string =
            "500000300009000027400105009200000070000006000006049000300027900080600000000034012";

        let expected_puzzle = vec![
            5, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 9, 0, 0, 0, 0, 2, 7, 4, 0, 0, 1, 0, 5, 0, 0, 9, 2, 0,
            0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 6, 0, 4, 9, 0, 0, 0, 3, 0, 0, 0,
            2, 7, 9, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4, 0, 1, 2,
        ];
        let expected_solution = vec![
            5, 7, 8, 9, 6, 2, 3, 4, 1, 6, 1, 9, 4, 8, 3, 5, 2, 7, 4, 2, 3, 1, 7, 5, 8, 6, 9, 2, 9,
            4, 3, 1, 8, 6, 7, 5, 8, 3, 7, 2, 5, 6, 1, 9, 4, 1, 5, 6, 7, 4, 9, 2, 3, 8, 3, 4, 1, 5,
            2, 7, 9, 8, 6, 7, 8, 2, 6, 9, 1, 4, 5, 3, 9, 6, 5, 8, 3, 4, 7, 1, 2,
        ];

        let puzzle = parse_sudoku(puzzle_string).expect("should be ok");

        assert_eq!(puzzle, expected_puzzle);

        let solution = solve(&puzzle).expect("should be ok");

        assert_eq!(solution, expected_solution);

        let mut unsolved_image = Vec::new();
        let write_result =
            print_sudoku(puzzle).write_to(&mut Cursor::new(&mut unsolved_image), ImageFormat::Png);
        assert!(write_result.is_ok());

        assert_eq!(
            unsolved_image,
            include_bytes!("../../../tests/sudoku/unsolved/medium.png")
        );

        let mut solved_image = Vec::new();
        let write_result =
            print_sudoku(solution).write_to(&mut Cursor::new(&mut solved_image), ImageFormat::Png);
        assert!(write_result.is_ok());

        assert_eq!(
            solved_image,
            include_bytes!("../../../tests/sudoku/solved/medium.png")
        );
    }

    #[test]
    fn hard_solve1() {
        let puzzle_string =
            "000030400900400300300000072009005000800010000700600529000100700601050008040000010";

        let expected_puzzle = vec![
            0, 0, 0, 0, 3, 0, 4, 0, 0, 9, 0, 0, 4, 0, 0, 3, 0, 0, 3, 0, 0, 0, 0, 0, 0, 7, 2, 0, 0,
            9, 0, 0, 5, 0, 0, 0, 8, 0, 0, 0, 1, 0, 0, 0, 0, 7, 0, 0, 6, 0, 0, 5, 2, 9, 0, 0, 0, 1,
            0, 0, 7, 0, 0, 6, 0, 1, 0, 5, 0, 0, 0, 8, 0, 4, 0, 0, 0, 0, 0, 1, 0,
        ];
        let expected_solution = vec![
            1, 7, 2, 5, 3, 8, 4, 9, 6, 9, 8, 6, 4, 7, 2, 3, 5, 1, 3, 5, 4, 9, 6, 1, 8, 7, 2, 4, 6,
            9, 3, 2, 5, 1, 8, 7, 8, 2, 5, 7, 1, 9, 6, 3, 4, 7, 1, 3, 6, 8, 4, 5, 2, 9, 2, 9, 8, 1,
            4, 3, 7, 6, 5, 6, 3, 1, 2, 5, 7, 9, 4, 8, 5, 4, 7, 8, 9, 6, 2, 1, 3,
        ];

        let puzzle = parse_sudoku(puzzle_string).expect("should be ok");

        assert_eq!(puzzle, expected_puzzle);

        let solution = solve(&puzzle).expect("should be ok");

        assert_eq!(solution, expected_solution);

        let mut unsolved_image = Vec::new();
        let write_result =
            print_sudoku(puzzle).write_to(&mut Cursor::new(&mut unsolved_image), ImageFormat::Png);
        assert!(write_result.is_ok());

        assert_eq!(
            unsolved_image,
            include_bytes!("../../../tests/sudoku/unsolved/hard1.png")
        );

        let mut solved_image = Vec::new();
        let write_result =
            print_sudoku(solution).write_to(&mut Cursor::new(&mut solved_image), ImageFormat::Png);
        assert!(write_result.is_ok());

        assert_eq!(
            solved_image,
            include_bytes!("../../../tests/sudoku/solved/hard1.png")
        );
    }

    #[test]
    fn hard_solve2() {
        let puzzle_string =
            "800000000003600000070090200050007000000045700000100030001000068008500010090000400";

        let expected_puzzle = vec![
            8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 6, 0, 0, 0, 0, 0, 0, 7, 0, 0, 9, 0, 2, 0, 0, 0, 5,
            0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 4, 5, 7, 0, 0, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 1, 0,
            0, 0, 0, 6, 8, 0, 0, 8, 5, 0, 0, 0, 1, 0, 0, 9, 0, 0, 0, 0, 4, 0, 0,
        ];

        let expected_solution = vec![
            8, 1, 2, 7, 5, 3, 6, 4, 9, 9, 4, 3, 6, 8, 2, 1, 7, 5, 6, 7, 5, 4, 9, 1, 2, 8, 3, 1, 5,
            4, 2, 3, 7, 8, 9, 6, 3, 6, 9, 8, 4, 5, 7, 2, 1, 2, 8, 7, 1, 6, 9, 5, 3, 4, 5, 2, 1, 9,
            7, 4, 3, 6, 8, 4, 3, 8, 5, 2, 6, 9, 1, 7, 7, 9, 6, 3, 1, 8, 4, 5, 2,
        ];

        let puzzle = parse_sudoku(puzzle_string).expect("should be ok");

        assert_eq!(puzzle, expected_puzzle);

        let solution = solve(&puzzle).expect("should be ok");

        assert_eq!(solution, expected_solution);

        let mut unsolved_image = Vec::new();
        let write_result =
            print_sudoku(puzzle).write_to(&mut Cursor::new(&mut unsolved_image), ImageFormat::Png);
        assert!(write_result.is_ok());

        assert_eq!(
            unsolved_image,
            include_bytes!("../../../tests/sudoku/unsolved/hard2.png")
        );

        let mut solved_image = Vec::new();
        let write_result =
            print_sudoku(solution).write_to(&mut Cursor::new(&mut solved_image), ImageFormat::Png);
        assert!(write_result.is_ok());

        assert_eq!(
            solved_image,
            include_bytes!("../../../tests/sudoku/solved/hard2.png")
        );
    }

    #[test]
    fn hard_solve3() {
        let puzzle_string =
            "120300000400000300003050000004200500000080009060005070001500200000090060000007008";

        let expected_puzzle = vec![
            1, 2, 0, 3, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 3, 0, 5, 0, 0, 0, 0, 0, 0,
            4, 2, 0, 0, 5, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 9, 0, 6, 0, 0, 0, 5, 0, 7, 0, 0, 0, 1, 5,
            0, 0, 2, 0, 0, 0, 0, 0, 0, 9, 0, 0, 6, 0, 0, 0, 0, 0, 0, 7, 0, 0, 8,
        ];

        let expected_solution = vec![
            1, 2, 5, 3, 7, 4, 8, 9, 6, 4, 7, 9, 6, 1, 8, 3, 2, 5, 6, 8, 3, 9, 5, 2, 7, 1, 4, 7, 1,
            4, 2, 6, 9, 5, 8, 3, 5, 3, 2, 7, 8, 1, 6, 4, 9, 9, 6, 8, 4, 3, 5, 1, 7, 2, 8, 9, 1, 5,
            4, 6, 2, 3, 7, 2, 5, 7, 8, 9, 3, 4, 6, 1, 3, 4, 6, 1, 2, 7, 9, 5, 8,
        ];

        let puzzle = parse_sudoku(puzzle_string).expect("should be ok");

        assert_eq!(puzzle, expected_puzzle);

        let solution = solve(&puzzle).expect("should be ok");

        assert_eq!(solution, expected_solution);

        let mut unsolved_image = Vec::new();
        let write_result =
            print_sudoku(puzzle).write_to(&mut Cursor::new(&mut unsolved_image), ImageFormat::Png);
        assert!(write_result.is_ok());

        assert_eq!(
            unsolved_image,
            include_bytes!("../../../tests/sudoku/unsolved/hard3.png")
        );

        let mut solved_image = Vec::new();
        let write_result =
            print_sudoku(solution).write_to(&mut Cursor::new(&mut solved_image), ImageFormat::Png);
        assert!(write_result.is_ok());

        assert_eq!(
            solved_image,
            include_bytes!("../../../tests/sudoku/solved/hard3.png")
        );
    }

    #[test]
    fn no_solution() {
        let puzzle = vec![
            1, 2, 0, 3, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 3, 0, 5, 0, 0, 0, 0, 0, 0,
            4, 2, 0, 0, 5, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 9, 0, 6, 0, 0, 0, 5, 0, 7, 0, 0, 0, 1, 5,
            0, 0, 2, 0, 0, 0, 0, 0, 0, 9, 0, 0, 6, 0, 0, 0, 0, 0, 0, 7, 0, 8, 8,
        ];

        let error = solve(&puzzle).expect_err("should be err");
        assert!(matches!(error, SudokuError::NoSolution));
    }

    #[test]
    fn invalid_size() {
        let puzzle_string =
            "1203000004000003000030500000042005000000800090600050700015002000000900600000070080";

        let error = parse_sudoku(puzzle_string).expect_err("should be err");
        assert!(matches!(error, SudokuError::InvalidSize(82)));

        let puzzle_string =
            "12030000040000030000305000000420050000080009060005070001500200000090060000007008";

        let error = parse_sudoku(puzzle_string).expect_err("should be err");
        assert!(matches!(error, SudokuError::InvalidSize(80)));
    }

    #[test]
    fn invalid_integer() {
        let puzzle_string =
            "1203000004000003000030500000042005000a0080009060005070001500200000090060000007008";

        let error = parse_sudoku(puzzle_string).expect_err("should be err");
        assert!(matches!(error, SudokuError::InvalidInteger('a')));

        let puzzle_string =
            "1203000004000003000030500000042005000-0080009060005070001500200000090060000007008";

        let error = parse_sudoku(puzzle_string).expect_err("should be err");
        assert!(matches!(error, SudokuError::InvalidInteger('-')));
    }
}
