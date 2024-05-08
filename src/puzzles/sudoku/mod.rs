use ab_glyph::FontRef;
use image::RgbImage;
use imageproc::drawing::draw_text_mut;

use crate::util::{RgbBuffer, BLACK_PIXEL, ROBOTO_MEDIUM, WHITE_PIXEL};

const NUM_MIN: u8 = 1;
const NUM_MAX: u8 = 9;

pub const GRID_SIZE: usize = 9;
const BOX_SIZE: usize = 3;

pub fn parse_sudoku(puzzle: &str) -> Option<Vec<u8>> {
    puzzle
        .chars()
        .map(|x| {
            let value = x.to_digit(10)?;

            if (0..=9).contains(&value) {
                return Some(value as u8);
            }

            None
        })
        .collect::<Option<Vec<u8>>>()
}

pub fn solve_sudoku(original: Vec<u8>) -> Option<(RgbBuffer, RgbBuffer)> {
    let solved = solve(&original)?;
    Some((print_sudoku(original), print_sudoku(solved)))
}

fn solve(original: &Vec<u8>) -> Option<Vec<u8>> {
    let mut sudoku = original.to_owned();
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

    for (i, number) in sudoku.iter().enumerate() {
        if *number == 0 {
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
        let original = vec![
            4, 1, 5, 8, 3, 0, 0, 9, 0, 0, 0, 3, 0, 0, 9, 1, 0, 4, 0, 0, 2, 1, 5, 0, 0, 0, 6, 9, 0,
            0, 7, 8, 3, 0, 0, 0, 2, 0, 0, 0, 0, 0, 3, 8, 1, 5, 0, 0, 0, 1, 2, 4, 0, 0, 0, 0, 4, 9,
            0, 0, 0, 6, 3, 3, 8, 0, 5, 0, 0, 0, 4, 0, 0, 0, 9, 3, 0, 7, 5, 0, 0,
        ];
        let expected = vec![
            4, 1, 5, 8, 3, 6, 2, 9, 7, 8, 6, 3, 2, 7, 9, 1, 5, 4, 7, 9, 2, 1, 5, 4, 8, 3, 6, 9, 4,
            1, 7, 8, 3, 6, 2, 5, 2, 7, 6, 4, 9, 5, 3, 8, 1, 5, 3, 8, 6, 1, 2, 4, 7, 9, 1, 5, 4, 9,
            2, 8, 7, 6, 3, 3, 8, 7, 5, 6, 1, 9, 4, 2, 6, 2, 9, 3, 4, 7, 5, 1, 8,
        ];

        let solution = solve(&original).expect("should be some");

        assert_eq!(solution, expected);

        let mut unsolved_image = Vec::new();
        let write_result = print_sudoku(original)
            .write_to(&mut Cursor::new(&mut unsolved_image), ImageFormat::Png);
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
        let original = vec![
            5, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 9, 0, 0, 0, 0, 2, 7, 4, 0, 0, 1, 0, 5, 0, 0, 9, 2, 0,
            0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 6, 0, 4, 9, 0, 0, 0, 3, 0, 0, 0,
            2, 7, 9, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4, 0, 1, 2,
        ];
        let expected = vec![
            5, 7, 8, 9, 6, 2, 3, 4, 1, 6, 1, 9, 4, 8, 3, 5, 2, 7, 4, 2, 3, 1, 7, 5, 8, 6, 9, 2, 9,
            4, 3, 1, 8, 6, 7, 5, 8, 3, 7, 2, 5, 6, 1, 9, 4, 1, 5, 6, 7, 4, 9, 2, 3, 8, 3, 4, 1, 5,
            2, 7, 9, 8, 6, 7, 8, 2, 6, 9, 1, 4, 5, 3, 9, 6, 5, 8, 3, 4, 7, 1, 2,
        ];

        let solution = solve(&original).expect("should be some");

        assert_eq!(solution, expected);

        let mut unsolved_image = Vec::new();
        let write_result = print_sudoku(original)
            .write_to(&mut Cursor::new(&mut unsolved_image), ImageFormat::Png);
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
        let original = vec![
            0, 0, 0, 0, 3, 0, 4, 0, 0, 9, 0, 0, 4, 0, 0, 3, 0, 0, 3, 0, 0, 0, 0, 0, 0, 7, 2, 0, 0,
            9, 0, 0, 5, 0, 0, 0, 8, 0, 0, 0, 1, 0, 0, 0, 0, 7, 0, 0, 6, 0, 0, 5, 2, 9, 0, 0, 0, 1,
            0, 0, 7, 0, 0, 6, 0, 1, 0, 5, 0, 0, 0, 8, 0, 4, 0, 0, 0, 0, 0, 1, 0,
        ];
        let expected = vec![
            1, 7, 2, 5, 3, 8, 4, 9, 6, 9, 8, 6, 4, 7, 2, 3, 5, 1, 3, 5, 4, 9, 6, 1, 8, 7, 2, 4, 6,
            9, 3, 2, 5, 1, 8, 7, 8, 2, 5, 7, 1, 9, 6, 3, 4, 7, 1, 3, 6, 8, 4, 5, 2, 9, 2, 9, 8, 1,
            4, 3, 7, 6, 5, 6, 3, 1, 2, 5, 7, 9, 4, 8, 5, 4, 7, 8, 9, 6, 2, 1, 3,
        ];

        let solution = solve(&original).expect("should be some");

        assert_eq!(solution, expected);

        let mut unsolved_image = Vec::new();
        let write_result = print_sudoku(original)
            .write_to(&mut Cursor::new(&mut unsolved_image), ImageFormat::Png);
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
        let original = vec![
            8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 6, 0, 0, 0, 0, 0, 0, 7, 0, 0, 9, 0, 2, 0, 0, 0, 5,
            0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 4, 5, 7, 0, 0, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 1, 0,
            0, 0, 0, 6, 8, 0, 0, 8, 5, 0, 0, 0, 1, 0, 0, 9, 0, 0, 0, 0, 4, 0, 0,
        ];

        let expected = vec![
            8, 1, 2, 7, 5, 3, 6, 4, 9, 9, 4, 3, 6, 8, 2, 1, 7, 5, 6, 7, 5, 4, 9, 1, 2, 8, 3, 1, 5,
            4, 2, 3, 7, 8, 9, 6, 3, 6, 9, 8, 4, 5, 7, 2, 1, 2, 8, 7, 1, 6, 9, 5, 3, 4, 5, 2, 1, 9,
            7, 4, 3, 6, 8, 4, 3, 8, 5, 2, 6, 9, 1, 7, 7, 9, 6, 3, 1, 8, 4, 5, 2,
        ];

        let solution = solve(&original).expect("should be some");

        assert_eq!(solution, expected);

        let mut unsolved_image = Vec::new();
        let write_result = print_sudoku(original)
            .write_to(&mut Cursor::new(&mut unsolved_image), ImageFormat::Png);
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
        let original = vec![
            1, 2, 0, 3, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 3, 0, 5, 0, 0, 0, 0, 0, 0,
            4, 2, 0, 0, 5, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 9, 0, 6, 0, 0, 0, 5, 0, 7, 0, 0, 0, 1, 5,
            0, 0, 2, 0, 0, 0, 0, 0, 0, 9, 0, 0, 6, 0, 0, 0, 0, 0, 0, 7, 0, 0, 8,
        ];

        let expected = vec![
            1, 2, 5, 3, 7, 4, 8, 9, 6, 4, 7, 9, 6, 1, 8, 3, 2, 5, 6, 8, 3, 9, 5, 2, 7, 1, 4, 7, 1,
            4, 2, 6, 9, 5, 8, 3, 5, 3, 2, 7, 8, 1, 6, 4, 9, 9, 6, 8, 4, 3, 5, 1, 7, 2, 8, 9, 1, 5,
            4, 6, 2, 3, 7, 2, 5, 7, 8, 9, 3, 4, 6, 1, 3, 4, 6, 1, 2, 7, 9, 5, 8,
        ];

        let solution = solve(&original).expect("should be some");

        assert_eq!(solution, expected);

        let mut unsolved_image = Vec::new();
        let write_result = print_sudoku(original)
            .write_to(&mut Cursor::new(&mut unsolved_image), ImageFormat::Png);
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
}
