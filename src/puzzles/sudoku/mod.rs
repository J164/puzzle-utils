use ab_glyph::FontRef;
use image::RgbImage;
use imageproc::drawing::draw_text_mut;
use thiserror::Error;

use crate::{
    structures::dancing_links::{DancingLinksError, DancingMatrix},
    util::{RgbBuffer, SolutionPair, BLACK_PIXEL, ROBOTO_MEDIUM, WHITE_PIXEL},
};

const GRID_SIZE: usize = 9;

const SUDOKU_CONSTRAINTS: [[usize; 9]; 324] = const {
    let mut constraints = [[0; 9]; 324];

    let mut index = 0;

    while index < 81 {
        let mut inner_index = 0;
        while inner_index < 9 {
            constraints[index][inner_index] = index * 9 + inner_index;
            constraints[index + 81][inner_index] = (index % 9) + inner_index * 9 + (index / 9) * 81;
            constraints[index + 162][inner_index] = index + inner_index * 81;
            constraints[index + 243][inner_index] = (index % 9)
                + (inner_index % 3) * 9
                + (inner_index / 3) * 81
                + ((index % 27) / 9) * 27
                + (index / 27) * 243;
            inner_index += 1;
        }
        index += 1;
    }

    constraints
};

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

fn solve(puzzle: &[u8]) -> Result<Vec<u8>, SudokuError> {
    let mut matrix = DancingMatrix::new(
        SUDOKU_CONSTRAINTS
            .iter()
            .map(|constraint| constraint.iter()),
    );

    for (index, &value) in puzzle.iter().enumerate() {
        if value == 0 {
            continue;
        }

        let result = matrix.add_solution(index * 9 + (value as usize) - 1);

        match result {
            Err(DancingLinksError::InvalidRow) => Err(SudokuError::NoSolution),
            _ => Ok(()),
        }?;
    }

    let mut solution = matrix.solve().ok_or(SudokuError::NoSolution)?;
    solution.sort();
    Ok(solution.iter().map(|num| (num % 9) as u8 + 1).collect())
}

fn print(puzzle: Vec<u8>) -> RgbBuffer {
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

    for (i, &number) in puzzle.iter().enumerate() {
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

    fn test_parse(string: &str, expected: Vec<u8>) {
        let actual = super::parse(string).expect("should be ok");
        assert_eq!(actual, expected);
    }

    fn test_solve(puzzle: &[u8], expected: &[u8]) {
        let acutal = super::solve(puzzle).expect("should be ok");
        assert_eq!(acutal, expected);
    }

    fn test_print(puzzle: Vec<u8>, expected: &[u8]) {
        let mut actual = Vec::new();
        super::print(puzzle)
            .write_to(&mut Cursor::new(&mut actual), ImageFormat::Png)
            .expect("should be ok");
        assert_eq!(actual, expected);
    }

    //// GENERAL TESTS

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

    const EASY_UNSOLVED_IMAGE: &[u8] = include_bytes!("../../../tests/sudoku/unsolved/easy.png");
    const EASY_SOLVED_IMAGE: &[u8] = include_bytes!("../../../tests/sudoku/solved/easy.png");

    #[test]
    fn parse_easy() {
        test_parse(EASY_STRING, EASY_UNSOLVED.to_vec());
    }

    #[test]
    fn miri_solve_easy() {
        test_solve(&EASY_UNSOLVED, &EASY_SOLVED);
    }

    #[test]
    fn print_easy() {
        test_print(EASY_UNSOLVED.to_vec(), EASY_UNSOLVED_IMAGE);
        test_print(EASY_SOLVED.to_vec(), EASY_SOLVED_IMAGE);
    }

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

    const MEDIUM_UNSOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/sudoku/unsolved/medium.png");
    const MEDIUM_SOLVED_IMAGE: &[u8] = include_bytes!("../../../tests/sudoku/solved/medium.png");

    #[test]
    fn parse_medium() {
        test_parse(MEDIUM_STRING, MEDIUM_UNSOLVED.to_vec());
    }

    #[test]
    fn miri_solve_medium() {
        test_solve(&MEDIUM_UNSOLVED, &MEDIUM_SOLVED);
    }

    #[test]
    fn print_medium() {
        test_print(MEDIUM_UNSOLVED.to_vec(), MEDIUM_UNSOLVED_IMAGE);
        test_print(MEDIUM_SOLVED.to_vec(), MEDIUM_SOLVED_IMAGE);
    }

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

    const HARD1_UNSOLVED_IMAGE: &[u8] = include_bytes!("../../../tests/sudoku/unsolved/hard1.png");
    const HARD1_SOLVED_IMAGE: &[u8] = include_bytes!("../../../tests/sudoku/solved/hard1.png");

    #[test]
    fn parse_hard1() {
        test_parse(HARD1_STRING, HARD1_UNSOLVED.to_vec());
    }

    #[test]
    fn miri_solve_hard1() {
        test_solve(&HARD1_UNSOLVED, &HARD1_SOLVED);
    }

    #[test]
    fn print_hard1() {
        test_print(HARD1_UNSOLVED.to_vec(), HARD1_UNSOLVED_IMAGE);
        test_print(HARD1_SOLVED.to_vec(), HARD1_SOLVED_IMAGE);
    }

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

    const HARD2_UNSOLVED_IMAGE: &[u8] = include_bytes!("../../../tests/sudoku/unsolved/hard2.png");
    const HARD2_SOLVED_IMAGE: &[u8] = include_bytes!("../../../tests/sudoku/solved/hard2.png");

    #[test]
    fn parse_hard2() {
        test_parse(HARD2_STRING, HARD2_UNSOLVED.to_vec());
    }

    #[test]
    fn miri_solve_hard2() {
        test_solve(&HARD2_UNSOLVED, &HARD2_SOLVED);
    }

    #[test]
    fn print_hard2() {
        test_print(HARD2_UNSOLVED.to_vec(), HARD2_UNSOLVED_IMAGE);
        test_print(HARD2_SOLVED.to_vec(), HARD2_SOLVED_IMAGE);
    }

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

    const HARD3_UNSOLVED_IMAGE: &[u8] = include_bytes!("../../../tests/sudoku/unsolved/hard3.png");
    const HARD3_SOLVED_IMAGE: &[u8] = include_bytes!("../../../tests/sudoku/solved/hard3.png");

    #[test]
    fn parse_hard3() {
        test_parse(HARD3_STRING, HARD3_UNSOLVED.to_vec());
    }

    #[test]
    fn miri_solve_hard3() {
        test_solve(&HARD3_UNSOLVED, &HARD3_SOLVED);
    }

    #[test]
    fn print_hard3() {
        test_print(HARD3_UNSOLVED.to_vec(), HARD3_UNSOLVED_IMAGE);
        test_print(HARD3_SOLVED.to_vec(), HARD3_SOLVED_IMAGE);
    }

    // Impossible
    const IMPOSSIBLE_STRING: &str =
        "731000008000500042400009700020304000005000400000180006000708005090020100006090000";

    const IMPOSSIBLE_UNSOLVED: [u8; 81] = [
        7, 3, 1, 0, 0, 0, 0, 0, 8, 0, 0, 0, 5, 0, 0, 0, 4, 2, 4, 0, 0, 0, 0, 9, 7, 0, 0, 0, 2, 0,
        3, 0, 4, 0, 0, 0, 0, 0, 5, 0, 0, 0, 4, 0, 0, 0, 0, 0, 1, 8, 0, 0, 0, 6, 0, 0, 0, 7, 0, 8,
        0, 0, 5, 0, 9, 0, 0, 2, 0, 1, 0, 0, 0, 0, 6, 0, 9, 0, 0, 0, 0,
    ];

    const IMPOSSIBLE_UNSOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/sudoku/unsolved/impossible.png");

    #[test]
    fn parse_impossible() {
        test_parse(IMPOSSIBLE_STRING, IMPOSSIBLE_UNSOLVED.to_vec());
    }

    #[test]
    fn miri_solve_impossible() {
        let actual = super::solve(&IMPOSSIBLE_UNSOLVED).expect_err("should be Err");
        assert!(matches!(actual, super::SudokuError::NoSolution));
    }

    #[test]
    fn print_impossible() {
        test_print(IMPOSSIBLE_UNSOLVED.to_vec(), IMPOSSIBLE_UNSOLVED_IMAGE);
    }
}
