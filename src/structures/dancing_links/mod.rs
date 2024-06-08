mod node;

use std::{alloc::dealloc, ptr::null_mut};

use node::{Node, NODE_LAYOUT};

pub struct DancingMatrix {
    root: *mut Node, // Points to a dummy column header
}

impl DancingMatrix {
    pub fn new(constraints: Vec<Vec<usize>>) -> Self {
        let num_rows = *constraints
            .iter()
            .map(|constraint| constraint.iter().max().unwrap_or(&0))
            .max()
            .expect("constraints should be non-empty")
            + 1;
        let mut rows = vec![null_mut::<Node>(); num_rows];

        let root = unsafe { Node::new_header(null_mut(), 0) };
        let mut curr = root;
        for constraint in constraints {
            let new = unsafe { Node::new_header(curr, constraint.len()) };

            let mut col_curr = new;
            for index in constraint {
                let new = unsafe { Node::new(new, rows[index], col_curr, index) };

                rows[index] = new;
                col_curr = new;
            }

            curr = new;
        }

        DancingMatrix { root }
    }

    pub fn solve(self) -> Option<Vec<usize>> {
        self.solve_helper(Vec::new())
    }

    fn solve_helper(&self, solution: Vec<usize>) -> Option<Vec<usize>> {
        if self.is_empty() {
            return Some(solution);
        }

        let constraint = unsafe { Node::iter_right(self.root) }
            .skip(1)
            .max_by(|first, second| unsafe { Node::row(*first).cmp(&Node::row(*second)) })
            .expect("Iterator should be non empty");

        unsafe { Node::cover_column(constraint) };
        for row in unsafe { Node::iter_down(constraint).skip(1) } {
            let mut solution = solution.clone();
            solution.push(unsafe { Node::row(row) });

            for node in unsafe { Node::iter_right(row).skip(1) } {
                unsafe { Node::cover_column(node) };
            }

            if let Some(solution) = self.solve_helper(solution) {
                for node in unsafe { Node::iter_right(row).skip(1) } {
                    unsafe { Node::free_chain(node) };
                }

                unsafe { Node::free_chain(constraint) };

                return Some(solution);
            }

            for node in unsafe { Node::iter_left(row).skip(1) } {
                unsafe { Node::uncover_column(node) };
            }
        }
        unsafe { Node::uncover_column(constraint) };

        None
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
    fn miri_basic() {
        let constraints = vec![
            vec![0, 1],
            vec![4, 5],
            vec![3, 4],
            vec![0, 1, 2],
            vec![2, 3],
            vec![3, 4],
            vec![0, 2, 4, 5],
        ];

        let matrix = super::DancingMatrix::new(constraints);
        let mut solution = matrix.solve().expect("should be Some");
        solution.sort();

        assert_eq!(solution, vec![1, 3, 5]);
    }
}
