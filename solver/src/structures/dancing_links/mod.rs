mod node;

use std::{alloc::dealloc, mem::take, ptr::null_mut};

use node::{Node, NODE_LAYOUT};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DancingLinksError {
    #[error("Index out of bounds")]
    IndexError,
    #[error("Row cannot be added to solution")]
    InvalidRow,
    #[error("No solution exists")]
    NoSolution,
}

pub struct DancingMatrix {
    root: *mut Node, // Points to a dummy column header
    rows: Vec<*mut Node>,
    partial_solution: Vec<usize>,
}

impl DancingMatrix {
    pub fn new<'a>(
        constraints: impl Iterator<Item = impl ExactSizeIterator<Item = &'a usize>>,
    ) -> Self {
        let mut rows = Vec::new();

        let root = unsafe { Node::new_header(null_mut(), 0) };
        let mut curr = root;
        for constraint in constraints {
            let new = unsafe { Node::new_header(curr, constraint.len()) };

            let mut col_curr = new;
            for &index in constraint {
                if index >= rows.len() {
                    rows.resize(index + 1, null_mut());
                }

                let new = unsafe { Node::new(new, rows[index], col_curr, index) };

                rows[index] = new;
                col_curr = new;
            }

            curr = new;
        }

        DancingMatrix {
            root,
            rows,
            partial_solution: Vec::new(),
        }
    }

    pub fn add_solution(&mut self, row: usize) -> Result<(), DancingLinksError> {
        if row >= self.rows.len() {
            return Err(DancingLinksError::IndexError);
        }

        self.partial_solution.push(row);
        let row = self.rows[row];

        if row.is_null() {
            return Err(DancingLinksError::InvalidRow);
        }

        for node in unsafe { Node::iter_down(Node::header(row)).skip(1) } {
            self.rows[unsafe { Node::row(node) }] = null_mut();
        }

        for node in unsafe { Node::iter_right(row) } {
            unsafe { Node::cover_column(node) };
        }

        for node in unsafe { Node::iter_right(row).skip(1) } {
            unsafe { Node::free_chain(node) };
        }
        unsafe { Node::free_chain(row) };

        Ok(())
    }

    pub fn solve(mut self) -> Result<Vec<usize>, DancingLinksError> {
        if self.solve_helper() {
            Ok(take(&mut self.partial_solution))
        } else {
            Err(DancingLinksError::NoSolution)
        }
    }

    fn solve_helper(&mut self) -> bool {
        if self.is_empty() {
            return true;
        }

        let constraint = unsafe { Node::iter_right(self.root) }
            .skip(1)
            .min_by(|first, second| unsafe { Node::row(*first).cmp(&Node::row(*second)) })
            .expect("Iterator should be non empty");

        unsafe { Node::cover_column(constraint) };
        for row in unsafe { Node::iter_down(constraint).skip(1) } {
            self.partial_solution.push(unsafe { Node::row(row) });

            for node in unsafe { Node::iter_right(row).skip(1) } {
                unsafe { Node::cover_column(node) };
            }

            if self.solve_helper() {
                for node in unsafe { Node::iter_right(row).skip(1) } {
                    unsafe { Node::free_chain(node) };
                }
                unsafe { Node::free_chain(constraint) };

                return true;
            }

            for node in unsafe { Node::iter_left(row).skip(1) } {
                unsafe { Node::uncover_column(node) };
            }

            self.partial_solution.pop();
        }
        unsafe { Node::uncover_column(constraint) };

        false
    }

    fn is_empty(&self) -> bool {
        unsafe { Node::right(self.root) == self.root }
    }
}

impl Drop for DancingMatrix {
    fn drop(&mut self) {
        for header in unsafe { Node::iter_right(self.root) } {
            for node in unsafe { Node::iter_down(header) } {
                unsafe { dealloc(node as *mut u8, NODE_LAYOUT) };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn miri_empty() {
        let constraints: [[usize; 0]; 0] = [];

        let matrix =
            super::DancingMatrix::new(constraints.iter().map(|constraint| constraint.iter()));
        let solution = matrix.solve().expect("should be Some");

        assert_eq!(solution, Vec::<usize>::new());
    }

    #[test]
    fn miri_basic() {
        let constraints: [Vec<usize>; 7] = [
            vec![0, 1],
            vec![4, 5],
            vec![3, 4],
            vec![0, 1, 2],
            vec![2, 3],
            vec![3, 4],
            vec![0, 2, 4, 5],
        ];

        let matrix =
            super::DancingMatrix::new(constraints.iter().map(|constraint| constraint.iter()));
        let mut solution = matrix.solve().expect("should be Some");
        solution.sort_unstable();

        assert_eq!(solution, vec![1, 3, 5]);
    }

    #[test]
    fn miri_remove_empty_row() {
        let constraints: [Vec<usize>; 7] = [
            vec![0, 1],
            vec![4, 5],
            vec![3, 4],
            vec![0, 1, 2],
            vec![2, 3],
            vec![3, 4],
            vec![0, 2, 4, 5],
        ];

        let mut matrix =
            super::DancingMatrix::new(constraints.iter().map(|constraint| constraint.iter()));
        matrix.add_solution(1).expect("should be Ok");

        let actual = matrix.add_solution(0).expect_err("should be Err");

        assert!(matches!(actual, super::DancingLinksError::InvalidRow));
    }

    #[test]
    fn miri_add_solution() {
        let constraints: [Vec<usize>; 7] = [
            vec![0, 1],
            vec![4, 5],
            vec![3, 4],
            vec![0, 1, 2],
            vec![2, 3],
            vec![3, 4],
            vec![0, 2, 4, 5],
        ];

        let mut matrix =
            super::DancingMatrix::new(constraints.iter().map(|constraint| constraint.iter()));
        matrix.add_solution(3).expect("should be Ok");
        let mut solution = matrix.solve().expect("should be Some");
        solution.sort_unstable();

        assert_eq!(solution, vec![1, 3, 5]);
    }
}
