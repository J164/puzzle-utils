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
    let original = parse(puzzle)?;
    let solved = solve(&original)?;
    Ok(SolutionPair::new(print(original), print(solved)))
}

fn parse(puzzle: &str) -> Result<Vec<u8>, SudokuError> {
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

fn print(sudoku: Vec<u8>) -> RgbBuffer {
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

    // Easy
    const EASY_STRING: &str =
        "415830090003009104002150006900783000200000381500012400004900063380500040009307500";
    const EASY_UNSOLVED: [u8; 81] = [
        4, 1, 5, 8, 3, 0, 0, 9, 0, 0, 0, 3, 0, 0, 9, 1, 0, 4, 0, 0, 2, 1, 5, 0, 0, 0, 6, 9, 0, 0,
        7, 8, 3, 0, 0, 0, 2, 0, 0, 0, 0, 0, 3, 8, 1, 5, 0, 0, 0, 1, 2, 4, 0, 0, 0, 0, 4, 9, 0, 0,
        0, 6, 3, 3, 8, 0, 5, 0, 0, 0, 4, 0, 0, 0, 9, 3, 0, 7, 5, 0, 0,
    ];
    const EASY_SOLVED: [u8; 81] = [
        4, 1, 5, 8, 3, 6, 2, 9, 7, 8, 6, 3, 2, 7, 9, 1, 5, 4, 7, 9, 2, 1, 5, 4, 8, 3, 6, 9, 4, 1,
        7, 8, 3, 6, 2, 5, 2, 7, 6, 4, 9, 5, 3, 8, 1, 5, 3, 8, 6, 1, 2, 4, 7, 9, 1, 5, 4, 9, 2, 8,
        7, 6, 3, 3, 8, 7, 5, 6, 1, 9, 4, 2, 6, 2, 9, 3, 4, 7, 5, 1, 8,
    ];

    // Medium
    const MEDIUM_STRING: &str =
        "500000300009000027400105009200000070000006000006049000300027900080600000000034012";
    const MEDIUM_UNSOLVED: [u8; 81] = [
        5, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 9, 0, 0, 0, 0, 2, 7, 4, 0, 0, 1, 0, 5, 0, 0, 9, 2, 0, 0,
        0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 6, 0, 4, 9, 0, 0, 0, 3, 0, 0, 0, 2, 7,
        9, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4, 0, 1, 2,
    ];
    const MEDIUM_SOLVED: [u8; 81] = [
        5, 7, 8, 9, 6, 2, 3, 4, 1, 6, 1, 9, 4, 8, 3, 5, 2, 7, 4, 2, 3, 1, 7, 5, 8, 6, 9, 2, 9, 4,
        3, 1, 8, 6, 7, 5, 8, 3, 7, 2, 5, 6, 1, 9, 4, 1, 5, 6, 7, 4, 9, 2, 3, 8, 3, 4, 1, 5, 2, 7,
        9, 8, 6, 7, 8, 2, 6, 9, 1, 4, 5, 3, 9, 6, 5, 8, 3, 4, 7, 1, 2,
    ];

    // Hard 1
    const HARD1_STRING: &str =
        "000030400900400300300000072009005000800010000700600529000100700601050008040000010";
    const HARD1_UNSOLVED: [u8; 81] = [
        0, 0, 0, 0, 3, 0, 4, 0, 0, 9, 0, 0, 4, 0, 0, 3, 0, 0, 3, 0, 0, 0, 0, 0, 0, 7, 2, 0, 0, 9,
        0, 0, 5, 0, 0, 0, 8, 0, 0, 0, 1, 0, 0, 0, 0, 7, 0, 0, 6, 0, 0, 5, 2, 9, 0, 0, 0, 1, 0, 0,
        7, 0, 0, 6, 0, 1, 0, 5, 0, 0, 0, 8, 0, 4, 0, 0, 0, 0, 0, 1, 0,
    ];
    const HARD1_SOLVED: [u8; 81] = [
        1, 7, 2, 5, 3, 8, 4, 9, 6, 9, 8, 6, 4, 7, 2, 3, 5, 1, 3, 5, 4, 9, 6, 1, 8, 7, 2, 4, 6, 9,
        3, 2, 5, 1, 8, 7, 8, 2, 5, 7, 1, 9, 6, 3, 4, 7, 1, 3, 6, 8, 4, 5, 2, 9, 2, 9, 8, 1, 4, 3,
        7, 6, 5, 6, 3, 1, 2, 5, 7, 9, 4, 8, 5, 4, 7, 8, 9, 6, 2, 1, 3,
    ];

    // Hard 2
    const HARD2_STRING: &str =
        "800000000003600000070090200050007000000045700000100030001000068008500010090000400";
    const HARD2_UNSOLVED: [u8; 81] = [
        8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 6, 0, 0, 0, 0, 0, 0, 7, 0, 0, 9, 0, 2, 0, 0, 0, 5, 0,
        0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 4, 5, 7, 0, 0, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0,
        0, 6, 8, 0, 0, 8, 5, 0, 0, 0, 1, 0, 0, 9, 0, 0, 0, 0, 4, 0, 0,
    ];
    const HARD2_SOLVED: [u8; 81] = [
        8, 1, 2, 7, 5, 3, 6, 4, 9, 9, 4, 3, 6, 8, 2, 1, 7, 5, 6, 7, 5, 4, 9, 1, 2, 8, 3, 1, 5, 4,
        2, 3, 7, 8, 9, 6, 3, 6, 9, 8, 4, 5, 7, 2, 1, 2, 8, 7, 1, 6, 9, 5, 3, 4, 5, 2, 1, 9, 7, 4,
        3, 6, 8, 4, 3, 8, 5, 2, 6, 9, 1, 7, 7, 9, 6, 3, 1, 8, 4, 5, 2,
    ];

    // Hard 3
    const HARD3_STRING: &str =
        "120300000400000300003050000004200500000080009060005070001500200000090060000007008";
    const HARD3_UNSOLVED: [u8; 81] = [
        1, 2, 0, 3, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 3, 0, 5, 0, 0, 0, 0, 0, 0, 4,
        2, 0, 0, 5, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 9, 0, 6, 0, 0, 0, 5, 0, 7, 0, 0, 0, 1, 5, 0, 0,
        2, 0, 0, 0, 0, 0, 0, 9, 0, 0, 6, 0, 0, 0, 0, 0, 0, 7, 0, 0, 8,
    ];
    const HARD3_SOLVED: [u8; 81] = [
        1, 2, 5, 3, 7, 4, 8, 9, 6, 4, 7, 9, 6, 1, 8, 3, 2, 5, 6, 8, 3, 9, 5, 2, 7, 1, 4, 7, 1, 4,
        2, 6, 9, 5, 8, 3, 5, 3, 2, 7, 8, 1, 6, 4, 9, 9, 6, 8, 4, 3, 5, 1, 7, 2, 8, 9, 1, 5, 4, 6,
        2, 3, 7, 2, 5, 7, 8, 9, 3, 4, 6, 1, 3, 4, 6, 1, 2, 7, 9, 5, 8,
    ];

    #[test]
    fn parse() {
        // Easy
        let easy_expected = EASY_UNSOLVED.to_vec();

        let easy_actual = super::parse(EASY_STRING).expect("should be ok");
        assert_eq!(easy_actual, easy_expected);

        // Medium
        let medium_expected = MEDIUM_UNSOLVED.to_vec();

        let medium_actual = super::parse(MEDIUM_STRING).expect("should be ok");
        assert_eq!(medium_actual, medium_expected);

        // Hard 1
        let hard1_expected = HARD1_UNSOLVED.to_vec();

        let hard1_actual = super::parse(HARD1_STRING).expect("should be ok");
        assert_eq!(hard1_actual, hard1_expected);

        // Hard 2
        let hard2_expected = HARD2_UNSOLVED.to_vec();

        let hard2_actual = super::parse(HARD2_STRING).expect("should be ok");
        assert_eq!(hard2_actual, hard2_expected);

        // Hard 3
        let hard3_expected = HARD3_UNSOLVED.to_vec();

        let hard3_actual = super::parse(HARD3_STRING).expect("should be ok");
        assert_eq!(hard3_actual, hard3_expected);
    }

    #[test]
    fn solve() {
        // Easy
        let easy_puzzle = EASY_UNSOLVED.to_vec();

        let easy_expected = EASY_SOLVED.to_vec();

        let easy_actual = super::solve(&easy_puzzle).expect("should be ok");
        assert_eq!(easy_actual, easy_expected);

        // Medium
        let medium_puzzle = MEDIUM_UNSOLVED.to_vec();

        let medium_expected = MEDIUM_SOLVED.to_vec();

        let medium_actual = super::solve(&medium_puzzle).expect("should be ok");
        assert_eq!(medium_actual, medium_expected);

        // Hard 1
        let hard1_puzzle = HARD1_UNSOLVED.to_vec();

        let hard1_expected = HARD1_SOLVED.to_vec();

        let hard1_actual = super::solve(&hard1_puzzle).expect("should be ok");
        assert_eq!(hard1_actual, hard1_expected);

        // Hard 2
        let hard2_puzzle = HARD2_UNSOLVED.to_vec();

        let hard2_expected = HARD2_SOLVED.to_vec();

        let hard2_actual = super::solve(&hard2_puzzle).expect("should be ok");
        assert_eq!(hard2_actual, hard2_expected);

        // Hard 3
        let hard3_puzzle = HARD3_UNSOLVED.to_vec();

        let hard3_expected = HARD3_SOLVED.to_vec();

        let hard3_actual = super::solve(&hard3_puzzle).expect("should be ok");
        assert_eq!(hard3_actual, hard3_expected);
    }

    #[test]
    fn print() {
        // Easy
        let easy_unsolved = EASY_UNSOLVED.to_vec();

        let easy_unsolved_expected = include_bytes!("../../../tests/sudoku/unsolved/easy.png");

        let mut easy_unsolved_actual = Vec::new();
        let write_result = super::print(easy_unsolved).write_to(
            &mut Cursor::new(&mut easy_unsolved_actual),
            ImageFormat::Png,
        );

        assert!(write_result.is_ok());
        assert_eq!(easy_unsolved_actual, easy_unsolved_expected);

        let easy_solved = EASY_SOLVED.to_vec();

        let easy_solved_expected = include_bytes!("../../../tests/sudoku/solved/easy.png");

        let mut easy_solved_actual = Vec::new();
        let write_result = super::print(easy_solved)
            .write_to(&mut Cursor::new(&mut easy_solved_actual), ImageFormat::Png);

        assert!(write_result.is_ok());
        assert_eq!(easy_solved_actual, easy_solved_expected);

        // Medium
        let medium_unsolved = MEDIUM_UNSOLVED.to_vec();

        let medium_unsolved_expected = include_bytes!("../../../tests/sudoku/unsolved/medium.png");

        let mut medium_unsolved_actual = Vec::new();
        let write_result = super::print(medium_unsolved).write_to(
            &mut Cursor::new(&mut medium_unsolved_actual),
            ImageFormat::Png,
        );

        assert!(write_result.is_ok());
        assert_eq!(medium_unsolved_actual, medium_unsolved_expected);

        let medium_solved = MEDIUM_SOLVED.to_vec();

        let medium_solved_expected = include_bytes!("../../../tests/sudoku/solved/medium.png");

        let mut medium_solved_actual = Vec::new();
        let write_result = super::print(medium_solved).write_to(
            &mut Cursor::new(&mut medium_solved_actual),
            ImageFormat::Png,
        );

        assert!(write_result.is_ok());
        assert_eq!(medium_solved_actual, medium_solved_expected);

        // Hard 1
        let hard1_unsolved = HARD1_UNSOLVED.to_vec();

        let hard1_unsolved_expected = include_bytes!("../../../tests/sudoku/unsolved/hard1.png");

        let mut hard1_unsolved_actual = Vec::new();
        let write_result = super::print(hard1_unsolved).write_to(
            &mut Cursor::new(&mut hard1_unsolved_actual),
            ImageFormat::Png,
        );

        assert!(write_result.is_ok());
        assert_eq!(hard1_unsolved_actual, hard1_unsolved_expected);

        let hard1_solved = HARD1_SOLVED.to_vec();

        let hard1_solved_expected = include_bytes!("../../../tests/sudoku/solved/hard1.png");

        let mut hard1_solved_actual = Vec::new();
        let write_result = super::print(hard1_solved)
            .write_to(&mut Cursor::new(&mut hard1_solved_actual), ImageFormat::Png);

        assert!(write_result.is_ok());
        assert_eq!(hard1_solved_actual, hard1_solved_expected);

        // Hard 2
        let hard2_unsolved = HARD2_UNSOLVED.to_vec();

        let hard2_unsolved_expected = include_bytes!("../../../tests/sudoku/unsolved/hard2.png");

        let mut hard2_unsolved_actual = Vec::new();
        let write_result = super::print(hard2_unsolved).write_to(
            &mut Cursor::new(&mut hard2_unsolved_actual),
            ImageFormat::Png,
        );

        assert!(write_result.is_ok());
        assert_eq!(hard2_unsolved_actual, hard2_unsolved_expected);

        let hard2_solved = HARD2_SOLVED.to_vec();

        let hard2_solved_expected = include_bytes!("../../../tests/sudoku/solved/hard2.png");

        let mut hard2_solved_actual = Vec::new();
        let write_result = super::print(hard2_solved)
            .write_to(&mut Cursor::new(&mut hard2_solved_actual), ImageFormat::Png);

        assert!(write_result.is_ok());
        assert_eq!(hard2_solved_actual, hard2_solved_expected);

        // hard3
        let hard3_unsolved = HARD3_UNSOLVED.to_vec();

        let hard3_unsolved_expected = include_bytes!("../../../tests/sudoku/unsolved/hard3.png");

        let mut hard3_unsolved_actual = Vec::new();
        let write_result = super::print(hard3_unsolved).write_to(
            &mut Cursor::new(&mut hard3_unsolved_actual),
            ImageFormat::Png,
        );

        assert!(write_result.is_ok());
        assert_eq!(hard3_unsolved_actual, hard3_unsolved_expected);

        let hard3_solved = HARD3_SOLVED.to_vec();

        let hard3_solved_expected = include_bytes!("../../../tests/sudoku/solved/hard3.png");

        let mut hard3_solved_actual = Vec::new();
        let write_result = super::print(hard3_solved)
            .write_to(&mut Cursor::new(&mut hard3_solved_actual), ImageFormat::Png);

        assert!(write_result.is_ok());
        assert_eq!(hard3_solved_actual, hard3_solved_expected);
    }
}
