use ab_glyph::FontRef;
use image::ImageBuffer;
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut},
    rect::Rect,
};
use thiserror::Error;

use crate::{
    cloudflare_image::SolutionPair,
    util::{RgbBuffer, BLACK_PIXEL, GRAY_PIXEL, ROBOTO_MEDIUM, WHITE_PIXEL},
};

#[derive(Debug, Error)]
pub enum NonogramError {
    #[error("puzzle cannot be empty")]
    EmptyPuzzle,
    #[error("invalid rule `{0}`")]
    InvalidRule(String),
    #[error("invalid rule dimension")]
    InvalidRuleDimension,
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

#[derive(Debug)]
struct Rule {
    values: Vec<usize>,
    size: usize,
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

            Ok(Rule { values, size })
        })
        .collect::<Result<Vec<Rule>, NonogramError>>()
}

#[derive(Debug, Clone)]
enum Square {
    Blank,
    Filled,
    Blocked,
}

fn solve(col: Vec<Rule>, row: Vec<Rule>) -> Result<Vec<Square>, NonogramError> {
    let width = col.len();
    let height = row.len();

    let mut grid = vec![Square::Blank; width * height];

    // TODO: solve nonogram
    /* IDEA:
        Each square on grid has three states (blank, filled, blocked)
        Grid is WIDTH x HEIGHT

        Narrowing:

        1. Define the variance of a rule as the bound of a rule minus its size
        2. For any value in a rule that is greater than the variance of that rule there are (value - variance) guaranteed squares
        3. Skip variance squares and fill guaranteed squares

        Recursive backtrack:

        A is the rule set with the least rules (col by default)
        B is the other rule set

        1. Iterate through max heap of A rules by size (index -> i, rule)
        2. IF no fulfilments exist, revert to previous state
        3. Iterate through fulfilments of rule that replace all blanks with filled or blocked // Because of row rule fulfilments, the effective board height is B.SIZE - i
            a. Attempt to solve all row rules of size <= (WIDTH - (i + 1)) // should have exactly one solution if puzzle is valid so far
            b. IF no valid solution exists for a rule, break back to (3)
            c. ELSE accept fulfilment and continue
    */

    narrow(&mut grid, &col, &row);
    recursive_backtrack(&mut grid, &col, &row);

    Ok(grid)
}

fn narrow(grid: &mut [Square], col: &[Rule], row: &[Rule]) {
    let width = col.len();
    let height = row.len();

    for (x, rule) in col.iter().enumerate() {
        let variance = height - rule.size;

        if variance > height / 2 {
            continue;
        }

        let mut y = 0;
        for &value in &rule.values {
            if value > variance {
                for k in variance..value {
                    grid[(y + k) * width + x] = Square::Filled;
                }
            }
            y += value + 1;
        }
    }

    for (y, rule) in row.iter().enumerate() {
        let variance = width - rule.size;

        if variance > width / 2 {
            continue;
        }

        let mut x = 0;
        for &value in &rule.values {
            if value > variance {
                for k in variance..value {
                    grid[y * width + x + k] = Square::Filled;
                }
            }
            x += value + 1;
        }
    }
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

    use super::Square;

    impl PartialEq for super::Rule {
        fn eq(&self, other: &Self) -> bool {
            self.values == other.values && self.size == other.size
        }
    }

    impl PartialEq for super::Square {
        fn eq(&self, other: &Self) -> bool {
            matches!(
                (self, other),
                (Self::Blank, Self::Blank)
                    | (Self::Blocked, Self::Blocked)
                    | (Self::Filled, Self::Filled)
            )
        }
    }

    // two x two
    const TWO_TWO_WIDTH: usize = 2;
    const TWO_TWO_HEIGHT: usize = 2;

    const TWO_TWO_COL_STRING: &str = "2;1";
    fn two_two_col() -> Vec<super::Rule> {
        vec![
            super::Rule {
                values: vec![2],
                size: 2,
            },
            super::Rule {
                values: vec![1],
                size: 1,
            },
        ]
    }

    const TWO_TWO_ROW_STRING: &str = "2;1";
    fn two_two_row() -> Vec<super::Rule> {
        vec![
            super::Rule {
                values: vec![2],
                size: 2,
            },
            super::Rule {
                values: vec![1],
                size: 1,
            },
        ]
    }

    fn two_two_narrowed() -> Vec<Square> {
        vec![
            Square::Filled,
            Square::Filled,
            Square::Filled,
            Square::Blank,
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

    // two x three
    const TWO_THREE_WIDTH: usize = 2;
    const TWO_THREE_HEIGHT: usize = 3;

    const TWO_THREE_COL_STRING: &str = "1,1;2";
    fn two_three_col() -> Vec<super::Rule> {
        vec![
            super::Rule {
                values: vec![1, 1],
                size: 3,
            },
            super::Rule {
                values: vec![2],
                size: 2,
            },
        ]
    }

    const TWO_THREE_ROW_STRING: &str = "1;1;2";
    fn two_three_row() -> Vec<super::Rule> {
        vec![
            super::Rule {
                values: vec![1],
                size: 1,
            },
            super::Rule {
                values: vec![1],
                size: 1,
            },
            super::Rule {
                values: vec![2],
                size: 2,
            },
        ]
    }

    fn two_three_narrowed() -> Vec<Square> {
        vec![
            Square::Filled,
            Square::Blank,
            Square::Blank,
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

    // three x three
    const THREE_THREE_WIDTH: usize = 3;
    const THREE_THREE_HEIGHT: usize = 3;

    const THREE_THREE_COL_STRING: &str = "2;1;1";
    fn three_three_col() -> Vec<super::Rule> {
        vec![
            super::Rule {
                values: vec![2],
                size: 2,
            },
            super::Rule {
                values: vec![1],
                size: 1,
            },
            super::Rule {
                values: vec![1],
                size: 1,
            },
        ]
    }

    const THREE_THREE_ROW_STRING: &str = "1;1;2";
    fn three_three_row() -> Vec<super::Rule> {
        vec![
            super::Rule {
                values: vec![1],
                size: 1,
            },
            super::Rule {
                values: vec![1],
                size: 1,
            },
            super::Rule {
                values: vec![2],
                size: 2,
            },
        ]
    }

    fn three_three_narrowed() -> Vec<Square> {
        vec![
            Square::Blank,
            Square::Blank,
            Square::Blank,
            Square::Filled,
            Square::Blank,
            Square::Blank,
            Square::Blank,
            Square::Filled,
            Square::Blank,
        ]
    }

    fn three_three_backtracked() -> Vec<Square> {
        vec![
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
            Square::Blocked,
            Square::Filled,
            Square::Filled,
            Square::Blocked,
        ]
    }

    const THREE_THREE_UNSOLVED_IMAGE: &[u8] =
        include_bytes!("../../../tests/nonogram/unsolved/three_three.png");

    fn test_parse(string: &str, expected: Vec<super::Rule>, bound: usize) {
        let actual = super::parse(string, bound).expect("should be ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_two_two() {
        test_parse(TWO_TWO_COL_STRING, two_two_col(), TWO_TWO_HEIGHT);
        test_parse(TWO_TWO_ROW_STRING, two_two_row(), TWO_TWO_WIDTH);
    }

    #[test]
    fn parse_two_three() {
        test_parse(TWO_THREE_COL_STRING, two_three_col(), TWO_THREE_HEIGHT);
        test_parse(TWO_THREE_ROW_STRING, two_three_row(), TWO_THREE_WIDTH);
    }

    #[test]
    fn parse_three_three() {
        test_parse(
            THREE_THREE_COL_STRING,
            three_three_col(),
            THREE_THREE_HEIGHT,
        );
        test_parse(THREE_THREE_ROW_STRING, three_three_row(), THREE_THREE_WIDTH);
    }

    fn test_narrow(col: Vec<super::Rule>, row: Vec<super::Rule>, expected: Vec<super::Square>) {
        let mut actual = vec![Square::Blank; col.len() * row.len()];
        super::narrow(&mut actual, &col, &row);
        assert_eq!(actual, expected);
    }

    #[test]
    fn narrow_two_two() {
        test_narrow(two_two_col(), two_two_row(), two_two_narrowed());
    }

    #[test]
    fn narrow_two_three() {
        test_narrow(two_three_col(), two_three_row(), two_three_narrowed());
    }

    #[test]
    fn narrow_three_three() {
        test_narrow(three_three_col(), three_three_row(), three_three_narrowed());
    }

    fn test_backtrack(
        mut actual: Vec<Square>,
        col: Vec<super::Rule>,
        row: Vec<super::Rule>,
        expected: Vec<super::Square>,
    ) {
        super::recursive_backtrack(&mut actual, &col, &row);
        assert_eq!(actual, expected);
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
    fn recursive_backtrack_two_three() {
        test_backtrack(
            two_three_narrowed(),
            two_three_col(),
            two_three_row(),
            two_three_backtracked(),
        );
    }

    #[test]
    fn recursive_backtrack_three_three() {
        test_backtrack(
            three_three_narrowed(),
            three_three_col(),
            three_three_row(),
            three_three_backtracked(),
        );
    }

    fn test_print(col: Vec<super::Rule>, row: Vec<super::Rule>, expected: &[u8]) {
        let mut actual = Vec::new();
        super::print(col.len() as u32, row.len() as u32, &col, &row)
            .write_to(&mut Cursor::new(&mut actual), ImageFormat::Png)
            .expect("should be ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn print_two_two() {
        test_print(two_two_col(), two_two_row(), TWO_TWO_UNSOLVED_IMAGE);
    }

    #[test]
    fn print_two_three() {
        test_print(two_three_col(), two_three_row(), TWO_THREE_UNSOLVED_IMAGE);
    }

    #[test]
    fn print_three_three() {
        test_print(
            three_three_col(),
            three_three_row(),
            THREE_THREE_UNSOLVED_IMAGE,
        );
    }
}
