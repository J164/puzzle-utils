mod right_left;

use std::cmp::max;

use ab_glyph::FontRef;
use image::ImageBuffer;
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut},
    rect::Rect,
};
use thiserror::Error;

use crate::{
    util::{BLACK_PIXEL, GRAY_PIXEL, ROBOTO_MEDIUM, WHITE_PIXEL},
    RgbBuffer,
};

use self::right_left::RuleMachine;

#[derive(Debug, Error)]
pub enum NonogramError {
    #[error("puzzle cannot be empty")]
    EmptyPuzzle,
    #[error("invalid rule `{0}`")]
    InvalidRule(Box<str>),
    #[error("invalid rule dimension")]
    InvalidRuleDimension,
    #[error("puzzle has no solution")]
    NoSolution,
    #[error("invalid dimensions")]
    InvalidDimensions,
}

pub fn parse_nonogram_rules(rules: &str, bound: usize) -> Result<Vec<Vec<usize>>, NonogramError> {
    rules
        .split(';')
        .map(|rule| {
            let mut size = 0;

            let values = rule
                .split(',')
                .map(|x| {
                    let value = x
                        .parse::<usize>()
                        .or(Err(NonogramError::InvalidRule(x.into())))?;

                    size += value;

                    Ok(value)
                })
                .collect::<Result<Vec<usize>, NonogramError>>()?;

            size += values.len() - 1;

            if values.is_empty() || size > bound {
                return Err(NonogramError::InvalidRuleDimension);
            }

            Ok(values)
        })
        .collect::<Result<Vec<Vec<usize>>, NonogramError>>()
}

#[derive(Debug, Clone, PartialEq)]
enum Square {
    Blank,
    Filled,
    Blocked,
}

pub fn solve_nonogram(col: &[Vec<usize>], row: &[Vec<usize>]) -> Result<Vec<bool>, NonogramError> {
    let width = col.len();
    let height = row.len();

    let mut grid = vec![Square::Blank; width * height];

    right_left(&mut grid, col, row)?;
    recursive_backtrack(&mut grid, col, row);

    Ok(grid
        .iter()
        .map(|square| matches!(square, Square::Filled))
        .collect())
}

fn right_left(
    grid: &mut [Square],
    col: &[Vec<usize>],
    row: &[Vec<usize>],
) -> Result<(), NonogramError> {
    let width = col.len();

    let col_machines: Vec<RuleMachine> = col.iter().map(|rule| RuleMachine::new(rule)).collect();
    let row_machines: Vec<RuleMachine> = row.iter().map(|rule| RuleMachine::new(rule)).collect();

    loop {
        let mut changed = false;

        for (index, machine) in col_machines.iter().enumerate() {
            changed |= machine.right_left(grid[index..].iter_mut().step_by(width).collect())?;
        }

        for (index, machine) in row_machines.iter().enumerate() {
            changed |= machine.right_left(
                grid[width * index..width * (index + 1)]
                    .iter_mut()
                    .collect(),
            )?;
        }

        if !changed {
            break;
        }
    }

    Ok(())
}

fn recursive_backtrack(grid: &mut [Square], col: &[Vec<usize>], row: &[Vec<usize>]) {}

pub fn print_nonogram(
    width: u32,
    height: u32,
    col: &[Vec<usize>],
    row: &[Vec<usize>],
) -> Result<RgbBuffer, NonogramError> {
    if width as usize != col.len() || height as usize != row.len() {
        return Err(NonogramError::InvalidDimensions);
    }

    let rule_width = max(150, width * 10);
    let rule_height = max(150, height * 10);

    let mut image = ImageBuffer::from_pixel(
        width * 50 + rule_width,
        height * 50 + rule_height,
        WHITE_PIXEL,
    );

    let font = FontRef::try_from_slice(ROBOTO_MEDIUM).expect("Font should be valid");

    for (x, rule) in col.iter().enumerate() {
        let x = (x as u32) * 50 + rule_width + 15;

        for (y, rule) in rule.iter().enumerate() {
            let y = (y as u32) * 30 + 10;

            draw_text_mut(
                &mut image,
                BLACK_PIXEL,
                x as i32,
                y as i32,
                30.0,
                &font,
                &rule.to_string(),
            );
        }
    }

    for (y, rule) in row.iter().enumerate() {
        let y = (y as u32) * 50 + rule_height + 10;

        draw_text_mut(
            &mut image,
            BLACK_PIXEL,
            10,
            y as i32,
            30.0,
            &font,
            &rule
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join("  "),
        );
    }

    for x in 0..width {
        for y in 0..(height * 50 + rule_height) {
            image.put_pixel(x * 50 + rule_width, y, GRAY_PIXEL);

            if x % 5 == 0 {
                image.put_pixel(x * 50 + rule_width + 1, y, GRAY_PIXEL);
                image.put_pixel(x * 50 + rule_width - 1, y, GRAY_PIXEL);
            }
        }
    }

    for y in 0..height {
        for x in 0..(width * 50 + rule_width) {
            image.put_pixel(x, y * 50 + rule_height, GRAY_PIXEL);

            if y % 5 == 0 {
                image.put_pixel(x, y * 50 + rule_height + 1, GRAY_PIXEL);
                image.put_pixel(x, y * 50 + rule_height - 1, GRAY_PIXEL);
            }
        }
    }

    Ok(image)
}

pub fn print_nonogram_solution(
    width: u32,
    height: u32,
    mut image: RgbBuffer,
    grid: &[bool],
) -> Result<RgbBuffer, NonogramError> {
    let rule_width = max(150, width * 10);
    let rule_height = max(150, height * 10);

    if grid.len() != width as usize * height as usize
        || image.width() != width * 50 + rule_width
        || image.height() != height * 50 + rule_height
    {
        return Err(NonogramError::InvalidDimensions);
    }

    for (i, square) in grid.iter().enumerate() {
        if !square {
            continue;
        }

        let x = (i as u32 % width) * 50 + max(150, width * 10) + 1;
        let y = (i as u32 / width) * 50 + max(150, height * 10) + 1;

        draw_filled_rect_mut(
            &mut image,
            Rect::at(x as i32, y as i32).of_size(49, 49),
            BLACK_PIXEL,
        );
    }

    Ok(image)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::ImageFormat;

    use crate::RgbBuffer;

    use super::Square;

    fn test_parse(string: &str, expected: Vec<Vec<usize>>, bound: usize) {
        let actual = super::parse_nonogram_rules(string, bound).expect("should be ok");
        assert_eq!(actual, expected);
    }

    fn test_right_left(col: Vec<Vec<usize>>, row: Vec<Vec<usize>>, expected: Vec<Square>) {
        let mut actual = vec![Square::Blank; col.len() * row.len()];
        super::right_left(&mut actual, &col, &row).expect("should be ok");
        assert_eq!(actual, expected);
    }

    fn test_backtrack(
        mut actual: Vec<Square>,
        col: Vec<Vec<usize>>,
        row: Vec<Vec<usize>>,
        expected: Vec<Square>,
    ) {
        super::recursive_backtrack(&mut actual, &col, &row);
        assert_eq!(actual, expected);
    }

    fn test_solve(col: Vec<Vec<usize>>, row: Vec<Vec<usize>>, expected: Vec<bool>) {
        let actual = super::solve_nonogram(&col, &row).expect("should be ok");
        assert_eq!(actual, expected);
    }

    fn test_print(col: Vec<Vec<usize>>, row: Vec<Vec<usize>>, expected: &[u8]) -> RgbBuffer {
        let mut actual = Vec::new();
        let image = super::print_nonogram(col.len() as u32, row.len() as u32, &col, &row)
            .expect("should be ok");
        image
            .write_to(&mut Cursor::new(&mut actual), ImageFormat::Png)
            .expect("should be ok");
        assert_eq!(actual, expected);
        image
    }

    fn test_print_solution(
        width: usize,
        height: usize,
        image: RgbBuffer,
        grid: Vec<bool>,
        expected: &[u8],
    ) {
        let mut actual = Vec::new();
        super::print_nonogram_solution(width as u32, height as u32, image, &grid)
            .expect("should be ok")
            .write_to(&mut Cursor::new(&mut actual), ImageFormat::Png)
            .expect("should be ok");
        assert_eq!(actual, expected);
    }

    //// GENERAL TESTS

    // two x two
    const TWO_TWO_WIDTH: usize = 2;
    const TWO_TWO_HEIGHT: usize = 2;

    const TWO_TWO_COL_STRING: &str = "2;1";
    fn two_two_col() -> Vec<Vec<usize>> {
        vec![vec![2], vec![1]]
    }

    const TWO_TWO_ROW_STRING: &str = "2;1";
    fn two_two_row() -> Vec<Vec<usize>> {
        vec![vec![2], vec![1]]
    }

    fn two_two_right_left() -> Vec<Square> {
        vec![
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
        ]
    }

    fn two_two_backtracked() -> Vec<Square> {
        vec![
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
        ]
    }

    fn two_two_solved() -> Vec<bool> {
        vec![true, true, true, false]
    }

    const TWO_TWO_UNSOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/unsolved/two_two.png");

    const TWO_TWO_SOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/solved/two_two.png");

    #[test]
    fn parse_two_two() {
        test_parse(TWO_TWO_COL_STRING, two_two_col(), TWO_TWO_HEIGHT);
        test_parse(TWO_TWO_ROW_STRING, two_two_row(), TWO_TWO_WIDTH);
    }

    #[test]
    fn right_left_two_two() {
        test_right_left(two_two_col(), two_two_row(), two_two_right_left());
    }

    #[test]
    fn recursive_backtrack_two_two() {
        test_backtrack(
            two_two_right_left(),
            two_two_col(),
            two_two_row(),
            two_two_backtracked(),
        );
    }

    #[test]
    fn solve_two_two() {
        test_solve(two_two_col(), two_two_row(), two_two_solved());
    }

    #[test]
    fn print_two_two() {
        let unsolved = test_print(two_two_col(), two_two_row(), TWO_TWO_UNSOLVED_IMAGE);
        test_print_solution(
            TWO_TWO_WIDTH,
            TWO_TWO_HEIGHT,
            unsolved,
            two_two_solved(),
            TWO_TWO_SOLVED_IMAGE,
        );
    }

    // two x three
    const TWO_THREE_WIDTH: usize = 2;
    const TWO_THREE_HEIGHT: usize = 3;

    const TWO_THREE_COL_STRING: &str = "1,1;2";
    fn two_three_col() -> Vec<Vec<usize>> {
        vec![vec![1, 1], vec![2]]
    }

    const TWO_THREE_ROW_STRING: &str = "1;1;2";
    fn two_three_row() -> Vec<Vec<usize>> {
        vec![vec![1], vec![1], vec![2]]
    }

    fn two_three_right_left() -> Vec<Square> {
        vec![
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
        ]
    }

    fn two_three_backtracked() -> Vec<Square> {
        vec![
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
        ]
    }

    fn two_three_solved() -> Vec<bool> {
        vec![true, false, false, true, true, true]
    }

    const TWO_THREE_UNSOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/unsolved/two_three.png");

    const TWO_THREE_SOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/solved/two_three.png");

    #[test]
    fn parse_two_three() {
        test_parse(TWO_THREE_COL_STRING, two_three_col(), TWO_THREE_HEIGHT);
        test_parse(TWO_THREE_ROW_STRING, two_three_row(), TWO_THREE_WIDTH);
    }

    #[test]
    fn right_left_two_three() {
        test_right_left(two_three_col(), two_three_row(), two_three_right_left());
    }

    #[test]
    fn recursive_backtrack_two_three() {
        test_backtrack(
            two_three_right_left(),
            two_three_col(),
            two_three_row(),
            two_three_backtracked(),
        );
    }

    #[test]
    fn solve_two_three() {
        test_solve(two_three_col(), two_three_row(), two_three_solved());
    }

    #[test]
    fn print_two_three() {
        let unsolved = test_print(two_three_col(), two_three_row(), TWO_THREE_UNSOLVED_IMAGE);
        test_print_solution(
            TWO_THREE_WIDTH,
            TWO_THREE_HEIGHT,
            unsolved,
            two_three_solved(),
            TWO_THREE_SOLVED_IMAGE,
        );
    }

    // five x five
    const FIVE_FIVE_WIDTH: usize = 5;
    const FIVE_FIVE_HEIGHT: usize = 5;

    const FIVE_FIVE_COL_STRING: &str = "1,2;3;4;2;1";
    fn five_five_col() -> Vec<Vec<usize>> {
        vec![vec![1, 2], vec![3], vec![4], vec![2], vec![1]]
    }

    const FIVE_FIVE_ROW_STRING: &str = "1,1;1;2;4;4";
    fn five_five_row() -> Vec<Vec<usize>> {
        vec![vec![1, 1], vec![1], vec![2], vec![4], vec![4]]
    }

    fn five_five_right_left() -> Vec<Square> {
        vec![
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
        ]
    }

    fn five_five_backtracked() -> Vec<Square> {
        vec![
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
        ]
    }

    fn five_five_solved() -> Vec<bool> {
        vec![
            true, false, false, false, true, false, false, true, false, false, false, true, true,
            false, false, true, true, true, true, false, true, true, true, true, false,
        ]
    }

    const FIVE_FIVE_UNSOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/unsolved/five_five.png");

    const FIVE_FIVE_SOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/solved/five_five.png");

    #[test]
    fn parse_five_five() {
        test_parse(FIVE_FIVE_COL_STRING, five_five_col(), FIVE_FIVE_HEIGHT);
        test_parse(FIVE_FIVE_ROW_STRING, five_five_row(), FIVE_FIVE_WIDTH);
    }

    #[test]
    fn right_left_five_five() {
        test_right_left(five_five_col(), five_five_row(), five_five_right_left());
    }

    #[test]
    fn recursive_backtrack_five_five() {
        test_backtrack(
            five_five_right_left(),
            five_five_col(),
            five_five_row(),
            five_five_backtracked(),
        );
    }

    #[test]
    fn solve_five_five() {
        test_solve(five_five_col(), five_five_row(), five_five_solved());
    }

    #[test]
    fn print_five_five() {
        let unsolved = test_print(five_five_col(), five_five_row(), FIVE_FIVE_UNSOLVED_IMAGE);
        test_print_solution(
            FIVE_FIVE_WIDTH,
            FIVE_FIVE_HEIGHT,
            unsolved,
            five_five_solved(),
            FIVE_FIVE_SOLVED_IMAGE,
        );
    }

    // Large
    const LARGE_WIDTH: usize = 25;
    const LARGE_HEIGHT: usize = 25;

    const LARGE_COL_STRING: &str = "2,3,4,3;1,3,2;7,2,3;8,1,5;4,6,6;4,1,1,3,5;4,1,3,1,3;7,2,1;3,1,1,4,2;1,1,3,3;7,1,3;5,3;4,1,1,1,3,1;2,4,3,2;3,5,3,3;5,3,2,4;2,1,3,3,4;2,6,4;2,1,8,3;2,1,11,3;2,1,3,2,3,3;2,1,3,15;1,1,1,15;6,3,3;4,3,1";
    fn large_col() -> Vec<Vec<usize>> {
        vec![
            vec![2, 3, 4, 3],
            vec![1, 3, 2],
            vec![7, 2, 3],
            vec![8, 1, 5],
            vec![4, 6, 6],
            vec![4, 1, 1, 3, 5],
            vec![4, 1, 3, 1, 3],
            vec![7, 2, 1],
            vec![3, 1, 1, 4, 2],
            vec![1, 1, 3, 3],
            vec![7, 1, 3],
            vec![5, 3],
            vec![4, 1, 1, 1, 3, 1],
            vec![2, 4, 3, 2],
            vec![3, 5, 3, 3],
            vec![5, 3, 2, 4],
            vec![2, 1, 3, 3, 4],
            vec![2, 6, 4],
            vec![2, 1, 8, 3],
            vec![2, 1, 11, 3],
            vec![2, 1, 3, 2, 3, 3],
            vec![2, 1, 3, 15],
            vec![1, 1, 1, 15],
            vec![6, 3, 3],
            vec![4, 3, 1],
        ]
    }

    const LARGE_ROW_STRING: &str = "9,1,7;1,7,3,7;14;6,7,2,2;4,5,2,4;8,3,1,2;5,4,2,6;3,2,3,3,1,1;1,2,7,3;1,3,1,1,8;9,9;3,4,6;1,8;1,2,4;4,1,7;5,6,4;15,2;5,3,2;3,2,6;3,7;1,1,7;1,4,2;1,4,3;1,3,3;1,1,3,3";
    fn large_row() -> Vec<Vec<usize>> {
        vec![
            vec![9, 1, 7],
            vec![1, 7, 3, 7],
            vec![14],
            vec![6, 7, 2, 2],
            vec![4, 5, 2, 4],
            vec![8, 3, 1, 2],
            vec![5, 4, 2, 6],
            vec![3, 2, 3, 3, 1, 1],
            vec![1, 2, 7, 3],
            vec![1, 3, 1, 1, 8],
            vec![9, 9],
            vec![3, 4, 6],
            vec![1, 8],
            vec![1, 2, 4],
            vec![4, 1, 7],
            vec![5, 6, 4],
            vec![15, 2],
            vec![5, 3, 2],
            vec![3, 2, 6],
            vec![3, 7],
            vec![1, 1, 7],
            vec![1, 4, 2],
            vec![1, 4, 3],
            vec![1, 3, 3],
            vec![1, 1, 3, 3],
        ]
    }

    fn large_right_left() -> Vec<Square> {
        vec![
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
        ]
    }

    fn large_backtracked() -> Vec<Square> {
        large_right_left()
    }

    fn large_solved() -> Vec<bool> {
        large_right_left()
            .iter()
            .map(|square| matches!(square, Square::Filled))
            .collect()
    }

    const LARGE_UNSOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/unsolved/large.png");

    const LARGE_SOLVED_IMAGE: &[u8] = include_bytes!("../../../tests/nonogram/solved/large.png");

    #[test]
    fn parse_large() {
        test_parse(LARGE_COL_STRING, large_col(), LARGE_HEIGHT);
        test_parse(LARGE_ROW_STRING, large_row(), LARGE_WIDTH);
    }

    #[test]
    fn right_left_large() {
        test_right_left(large_col(), large_row(), large_right_left());
    }

    #[test]
    fn recursive_backtrack_large() {
        test_backtrack(
            large_right_left(),
            large_col(),
            large_row(),
            large_backtracked(),
        );
    }

    #[test]
    fn solve_large() {
        test_solve(large_col(), large_row(), large_solved());
    }

    #[test]
    fn print_large() {
        let unsolved = test_print(large_col(), large_row(), LARGE_UNSOLVED_IMAGE);
        test_print_solution(
            LARGE_WIDTH,
            LARGE_HEIGHT,
            unsolved,
            large_solved(),
            LARGE_SOLVED_IMAGE,
        );
    }
}
