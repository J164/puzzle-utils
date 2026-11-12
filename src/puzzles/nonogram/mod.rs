#[cfg(feature = "render")]
mod render;
mod right_left;

use thiserror::Error;

use self::right_left::RuleMachine;

#[cfg(feature = "render")]
pub use self::render::{print_nonogram, print_nonogram_solution};

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

#[cfg(test)]
mod tests {
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
}
