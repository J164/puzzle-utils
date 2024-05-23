mod narrower;

use std::cmp::max;

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

    let solved = print_solution(width as u32, height as u32, unsolved.clone(), grid);

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
        let y = (y as u32) * 50 + rule_height + 10;

        draw_text_mut(
            &mut image,
            BLACK_PIXEL,
            10,
            y as i32,
            30.0,
            &font,
            &rule
                .values
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

    image
}

fn print_solution(width: u32, height: u32, mut image: RgbBuffer, grid: Vec<Square>) -> RgbBuffer {
    for (i, square) in grid.iter().enumerate() {
        if !matches!(square, Square::Filled) {
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

    image
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::ImageFormat;

    use crate::util::RgbBuffer;

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

    fn test_print(col: Vec<Rule>, row: Vec<Rule>, expected: &[u8]) -> RgbBuffer {
        let mut actual = Vec::new();
        let image = super::print(col.len() as u32, row.len() as u32, &col, &row);
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
        grid: Vec<Square>,
        expected: &[u8],
    ) {
        let mut actual = Vec::new();
        super::print_solution(width as u32, height as u32, image, grid)
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

    const TWO_TWO_SOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/solved/two_two.png");

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
        let unsolved = test_print(two_two_col(), two_two_row(), TWO_TWO_UNSOLVED_IMAGE);
        test_print_solution(
            TWO_TWO_WIDTH,
            TWO_TWO_HEIGHT,
            unsolved,
            two_two_backtracked(),
            TWO_TWO_SOLVED_IMAGE,
        );
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

    const TWO_THREE_SOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/solved/two_three.png");

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
        let unsolved = test_print(two_three_col(), two_three_row(), TWO_THREE_UNSOLVED_IMAGE);
        test_print_solution(
            TWO_THREE_WIDTH,
            TWO_THREE_HEIGHT,
            unsolved,
            two_three_backtracked(),
            TWO_THREE_SOLVED_IMAGE,
        );
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

    const FIVE_FIVE_SOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/solved/five_five.png");

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
        let unsolved = test_print(five_five_col(), five_five_row(), FIVE_FIVE_UNSOLVED_IMAGE);
        test_print_solution(
            FIVE_FIVE_WIDTH,
            FIVE_FIVE_HEIGHT,
            unsolved,
            five_five_backtracked(),
            FIVE_FIVE_SOLVED_IMAGE,
        );
    }

    // Large
    const LARGE_WIDTH: usize = 25;
    const LARGE_HEIGHT: usize = 25;

    const LARGE_COL_STRING: &str = "2,3,4,3;1,3,2;7,2,3;8,1,5;4,6,6;4,1,1,3,5;4,1,3,1,3;7,2,1;3,1,1,4,2;1,1,3,3;7,1,3;5,3;4,1,1,1,3,1;2,4,3,2;3,5,3,3;5,3,2,4;2,1,3,3,4;2,6,4;2,1,8,3;2,1,11,3;2,1,3,2,3,3;2,1,3,15;1,1,1,15;6,3,3;4,3,1";
    fn large_col() -> Vec<Rule> {
        vec![
            Rule {
                values: vec![2, 3, 4, 3],
            },
            Rule {
                values: vec![1, 3, 2],
            },
            Rule {
                values: vec![7, 2, 3],
            },
            Rule {
                values: vec![8, 1, 5],
            },
            Rule {
                values: vec![4, 6, 6],
            },
            Rule {
                values: vec![4, 1, 1, 3, 5],
            },
            Rule {
                values: vec![4, 1, 3, 1, 3],
            },
            Rule {
                values: vec![7, 2, 1],
            },
            Rule {
                values: vec![3, 1, 1, 4, 2],
            },
            Rule {
                values: vec![1, 1, 3, 3],
            },
            Rule {
                values: vec![7, 1, 3],
            },
            Rule { values: vec![5, 3] },
            Rule {
                values: vec![4, 1, 1, 1, 3, 1],
            },
            Rule {
                values: vec![2, 4, 3, 2],
            },
            Rule {
                values: vec![3, 5, 3, 3],
            },
            Rule {
                values: vec![5, 3, 2, 4],
            },
            Rule {
                values: vec![2, 1, 3, 3, 4],
            },
            Rule {
                values: vec![2, 6, 4],
            },
            Rule {
                values: vec![2, 1, 8, 3],
            },
            Rule {
                values: vec![2, 1, 11, 3],
            },
            Rule {
                values: vec![2, 1, 3, 2, 3, 3],
            },
            Rule {
                values: vec![2, 1, 3, 15],
            },
            Rule {
                values: vec![1, 1, 1, 15],
            },
            Rule {
                values: vec![6, 3, 3],
            },
            Rule {
                values: vec![4, 3, 1],
            },
        ]
    }

    const LARGE_ROW_STRING: &str = "9,1,7;1,7,3,7;14;6,7,2,2;4,5,2,4;8,3,1,2;5,4,2,6;3,2,3,3,1,1;1,2,7,3;1,3,1,1,8;9,9;3,4,6;1,8;1,2,4;4,1,7;5,6,4;15,2;5,3,2;3,2,6;3,7;1,1,7;1,4,2;1,4,3;1,3,3;1,1,3,3";
    fn large_row() -> Vec<Rule> {
        vec![
            Rule {
                values: vec![9, 1, 7],
            },
            Rule {
                values: vec![1, 7, 3, 7],
            },
            Rule { values: vec![14] },
            Rule {
                values: vec![6, 7, 2, 2],
            },
            Rule {
                values: vec![4, 5, 2, 4],
            },
            Rule {
                values: vec![8, 3, 1, 2],
            },
            Rule {
                values: vec![5, 4, 2, 6],
            },
            Rule {
                values: vec![3, 2, 3, 3, 1, 1],
            },
            Rule {
                values: vec![1, 2, 7, 3],
            },
            Rule {
                values: vec![1, 3, 1, 1, 8],
            },
            Rule { values: vec![9, 9] },
            Rule {
                values: vec![3, 4, 6],
            },
            Rule { values: vec![1, 8] },
            Rule {
                values: vec![1, 2, 4],
            },
            Rule {
                values: vec![4, 1, 7],
            },
            Rule {
                values: vec![5, 6, 4],
            },
            Rule {
                values: vec![15, 2],
            },
            Rule {
                values: vec![5, 3, 2],
            },
            Rule {
                values: vec![3, 2, 6],
            },
            Rule { values: vec![3, 7] },
            Rule {
                values: vec![1, 1, 7],
            },
            Rule {
                values: vec![1, 4, 2],
            },
            Rule {
                values: vec![1, 4, 3],
            },
            Rule {
                values: vec![1, 3, 3],
            },
            Rule {
                values: vec![1, 1, 3, 3],
            },
        ]
    }

    fn large_narrowed() -> Vec<Square> {
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
        large_narrowed()
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
    fn narrow_large() {
        test_narrow(large_col(), large_row(), large_narrowed());
    }

    #[test]
    fn recursive_backtrack_large() {
        test_backtrack(
            large_narrowed(),
            large_col(),
            large_row(),
            large_backtracked(),
        );
    }

    #[test]
    fn print_large() {
        let unsolved = test_print(large_col(), large_row(), LARGE_UNSOLVED_IMAGE);
        test_print_solution(
            LARGE_WIDTH,
            LARGE_HEIGHT,
            unsolved,
            large_backtracked(),
            LARGE_SOLVED_IMAGE,
        );
    }
}
