mod narrower;

use ab_glyph::FontRef;
use image::ImageBuffer;
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut},
    rect::Rect,
};
use thiserror::Error;

use crate::util::{RgbBuffer, SolutionPair, BLACK_PIXEL, GRAY_PIXEL, ROBOTO_MEDIUM, WHITE_PIXEL};

use self::narrower::RuleMachine;

#[derive(Debug, Error)]
pub enum NonogramError {
    #[error("puzzle cannot be empty")]
    EmptyPuzzle,
    #[error("invalid rule `{0}`")]
    InvalidRule(String),
    #[error("invalid rule dimension")]
    InvalidRuleDimension,
    #[error("puzzle has no solution")]
    NoSolution,
}

pub fn solve_nonogram(col: &str, row: &str) -> Result<SolutionPair, NonogramError> {
    let width = col.split(';').count();
    let height = row.split(';').count();

    if width == 0 || height == 0 {
        return Err(NonogramError::EmptyPuzzle);
    }

    let col = parse(col, height)?;
    let row = parse(row, width)?;

    let unsolved = print(width as u32, height as u32, &row, &col);

    let grid = solve(col, row)?;

    let solved = print_solution(width as u32, unsolved.clone(), grid);

    Ok(SolutionPair::new(unsolved, solved))
}

#[derive(Debug, PartialEq)]
struct Rule {
    values: Vec<usize>,
}

fn parse(rule: &str, bound: usize) -> Result<Vec<Rule>, NonogramError> {
    rule.split(';')
        .map(|rule| {
            let mut size = 0;

            let values = rule
                .split(',')
                .map(|x| {
                    let value = x
                        .parse::<usize>()
                        .or(Err(NonogramError::InvalidRule(x.to_string())))?;

                    size += value;

                    Ok(value)
                })
                .collect::<Result<Vec<usize>, NonogramError>>()?;

            size += values.len() - 1;

            if values.is_empty() || size > bound {
                return Err(NonogramError::InvalidRuleDimension);
            }

            Ok(Rule { values })
        })
        .collect::<Result<Vec<Rule>, NonogramError>>()
}

#[derive(Debug, Clone, PartialEq)]
enum Square {
    Blank,
    Filled,
    Blocked,
}

fn solve(col: Vec<Rule>, row: Vec<Rule>) -> Result<Vec<Square>, NonogramError> {
    let width = col.len();
    let height = row.len();

    let mut grid = vec![Square::Blank; width * height];

    narrow(&mut grid, &col, &row)?;
    recursive_backtrack(&mut grid, &col, &row);

    Ok(grid)
}

fn narrow(grid: &mut [Square], col: &[Rule], row: &[Rule]) -> Result<(), NonogramError> {
    let width = col.len();

    let col_machines: Vec<RuleMachine> = col
        .iter()
        .map(|rule| RuleMachine::new(&rule.values))
        .collect();
    let row_machines: Vec<RuleMachine> = row
        .iter()
        .map(|rule| RuleMachine::new(&rule.values))
        .collect();

    loop {
        let mut changed = false;

        for (index, machine) in col_machines.iter().enumerate() {
            changed |= machine.find_guaranteed(
                grid[index..]
                    .iter_mut()
                    .step_by(width)
                    .map(|square| square)
                    .collect(),
            )?;
        }

        for (index, machine) in row_machines.iter().enumerate() {
            changed |= machine.find_guaranteed(
                grid[width * index..width * (index + 1)]
                    .iter_mut()
                    .map(|square| square)
                    .collect(),
            )?;
        }

        if !changed {
            break;
        }
    }

    Ok(())
}

fn recursive_backtrack(grid: &mut [Square], col: &[Rule], row: &[Rule]) {}

fn print(width: u32, height: u32, col: &[Rule], row: &[Rule]) -> RgbBuffer {
    let mut image = ImageBuffer::from_pixel(width * 50 + 150, height * 50 + 150, WHITE_PIXEL);

    let font = FontRef::try_from_slice(ROBOTO_MEDIUM).expect("Font should be valid");

    for (x, rule) in col.iter().enumerate() {
        let x = (x as u32) * 50 + 165;

        for (y, rule) in rule.values.iter().enumerate() {
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
        let y = (y as u32) * 50 + 160;

        for (x, value) in rule.values.iter().enumerate() {
            let x = (x as u32) * 30 + 10;

            draw_text_mut(
                &mut image,
                BLACK_PIXEL,
                x as i32,
                y as i32,
                30.0,
                &font,
                &value.to_string(),
            );
        }
    }

    for x in 0..width {
        for y in 0..(height * 50 + 150) {
            image.put_pixel(x * 50 + 150, y, GRAY_PIXEL);
        }
    }

    for y in 0..height {
        for x in 0..(width * 50 + 150) {
            image.put_pixel(x, y * 50 + 150, GRAY_PIXEL);
        }
    }

    image
}

fn print_solution(width: u32, mut image: RgbBuffer, grid: Vec<Square>) -> RgbBuffer {
    for (i, square) in grid.iter().enumerate() {
        if !matches!(square, Square::Filled) {
            continue;
        }

        let x = (i as u32 % width) * 50 + 151;
        let y = (i as u32 / width) * 50 + 151;

        draw_filled_rect_mut(
            &mut image,
            Rect::at(x as i32, y as i32).of_size(49, 49),
            BLACK_PIXEL,
        );
    }

    image
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::ImageFormat;

    use super::{Rule, Square};

    fn test_parse(string: &str, expected: Vec<Rule>, bound: usize) {
        let actual = super::parse(string, bound).expect("should be ok");
        assert_eq!(actual, expected);
    }

    fn test_narrow(col: Vec<Rule>, row: Vec<Rule>, expected: Vec<Square>) {
        let mut actual = vec![Square::Blank; col.len() * row.len()];
        super::narrow(&mut actual, &col, &row).unwrap();
        assert_eq!(actual, expected);
    }

    fn test_backtrack(
        mut actual: Vec<Square>,
        col: Vec<Rule>,
        row: Vec<Rule>,
        expected: Vec<Square>,
    ) {
        super::recursive_backtrack(&mut actual, &col, &row);
        assert_eq!(actual, expected);
    }

    fn test_print(col: Vec<Rule>, row: Vec<Rule>, expected: &[u8]) {
        let mut actual = Vec::new();
        super::print(col.len() as u32, row.len() as u32, &col, &row)
            .write_to(&mut Cursor::new(&mut actual), ImageFormat::Png)
            .expect("should be ok");
        assert_eq!(actual, expected);
    }

    //// GENERAL TESTS

    // two x two
    const TWO_TWO_WIDTH: usize = 2;
    const TWO_TWO_HEIGHT: usize = 2;

    const TWO_TWO_COL_STRING: &str = "2;1";
    fn two_two_col() -> Vec<Rule> {
        vec![Rule { values: vec![2] }, Rule { values: vec![1] }]
    }

    const TWO_TWO_ROW_STRING: &str = "2;1";
    fn two_two_row() -> Vec<Rule> {
        vec![Rule { values: vec![2] }, Rule { values: vec![1] }]
    }

    fn two_two_narrowed() -> Vec<Square> {
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

    const TWO_TWO_UNSOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/unsolved/two_two.png");

    #[test]
    fn parse_two_two() {
        test_parse(TWO_TWO_COL_STRING, two_two_col(), TWO_TWO_HEIGHT);
        test_parse(TWO_TWO_ROW_STRING, two_two_row(), TWO_TWO_WIDTH);
    }

    #[test]
    fn narrow_two_two() {
        test_narrow(two_two_col(), two_two_row(), two_two_narrowed());
    }

    #[test]
    fn recursive_backtrack_two_two() {
        test_backtrack(
            two_two_narrowed(),
            two_two_col(),
            two_two_row(),
            two_two_backtracked(),
        );
    }

    #[test]
    fn print_two_two() {
        test_print(two_two_col(), two_two_row(), TWO_TWO_UNSOLVED_IMAGE);
    }

    // two x three
    const TWO_THREE_WIDTH: usize = 2;
    const TWO_THREE_HEIGHT: usize = 3;

    const TWO_THREE_COL_STRING: &str = "1,1;2";
    fn two_three_col() -> Vec<Rule> {
        vec![Rule { values: vec![1, 1] }, Rule { values: vec![2] }]
    }

    const TWO_THREE_ROW_STRING: &str = "1;1;2";
    fn two_three_row() -> Vec<Rule> {
        vec![
            Rule { values: vec![1] },
            Rule { values: vec![1] },
            Rule { values: vec![2] },
        ]
    }

    fn two_three_narrowed() -> Vec<Square> {
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

    const TWO_THREE_UNSOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/unsolved/two_three.png");

    #[test]
    fn parse_two_three() {
        test_parse(TWO_THREE_COL_STRING, two_three_col(), TWO_THREE_HEIGHT);
        test_parse(TWO_THREE_ROW_STRING, two_three_row(), TWO_THREE_WIDTH);
    }

    #[test]
    fn narrow_two_three() {
        test_narrow(two_three_col(), two_three_row(), two_three_narrowed());
    }

    #[test]
    fn recursive_backtrack_two_three() {
        test_backtrack(
            two_three_narrowed(),
            two_three_col(),
            two_three_row(),
            two_three_backtracked(),
        );
    }

    #[test]
    fn print_two_three() {
        test_print(two_three_col(), two_three_row(), TWO_THREE_UNSOLVED_IMAGE);
    }

    // five x five
    const FIVE_FIVE_WIDTH: usize = 5;
    const FIVE_FIVE_HEIGHT: usize = 5;

    const FIVE_FIVE_COL_STRING: &str = "1,2;3;4;2;1";
    fn five_five_col() -> Vec<Rule> {
        vec![
            Rule { values: vec![1, 2] },
            Rule { values: vec![3] },
            Rule { values: vec![4] },
            Rule { values: vec![2] },
            Rule { values: vec![1] },
        ]
    }

    const FIVE_FIVE_ROW_STRING: &str = "1,1;1;2;4;4";
    fn five_five_row() -> Vec<Rule> {
        vec![
            Rule { values: vec![1, 1] },
            Rule { values: vec![1] },
            Rule { values: vec![2] },
            Rule { values: vec![4] },
            Rule { values: vec![4] },
        ]
    }

    fn five_five_narrowed() -> Vec<Square> {
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

    const FIVE_FIVE_UNSOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/unsolved/five_five.png");

    #[test]
    fn parse_five_five() {
        test_parse(FIVE_FIVE_COL_STRING, five_five_col(), FIVE_FIVE_HEIGHT);
        test_parse(FIVE_FIVE_ROW_STRING, five_five_row(), FIVE_FIVE_WIDTH);
    }

    #[test]
    fn narrow_five_five() {
        test_narrow(five_five_col(), five_five_row(), five_five_narrowed());
    }

    #[test]
    fn recursive_backtrack_five_five() {
        test_backtrack(
            five_five_narrowed(),
            five_five_col(),
            five_five_row(),
            five_five_backtracked(),
        );
    }

    #[test]
    fn print_five_five() {
        test_print(five_five_col(), five_five_row(), FIVE_FIVE_UNSOLVED_IMAGE);
    }
}
