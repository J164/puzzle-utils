mod node;

use std::{alloc::dealloc, ptr::null_mut};

use node::{Node, NODE_LAYOUT};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DancingLinksError {
    #[error("At least one constraint must be provided")]
    ZeroConstraints,
}

pub struct DancingMatrix {
    root: *mut Node, // Points to a dummy column header
}

impl DancingMatrix {
    pub fn new(constraints: Vec<Vec<usize>>) -> Result<Self, DancingLinksError> {
        if constraints.is_empty() {
            return Err(DancingLinksError::ZeroConstraints);
        }

        let num_rows = *constraints
            .iter()
            .map(|constraint| constraint.iter().max().unwrap_or(&0))
            .max()
            .expect("constraints should be non-empty")
            + 1;
        let mut rows = vec![null_mut::<Node>(); num_rows];

        let root = unsafe { Node::new_header(null_mut()) };
        let mut curr = root;
        for constraint in constraints {
            let new = unsafe { Node::new_header(curr) };

            let mut col_curr = new;
            for index in constraint {
                let new = unsafe { Node::new(new, rows[index], col_curr) };

                rows[index] = new;
                col_curr = new;
            }

            curr = new;
        }

        Ok(DancingMatrix { root })
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
    fn unsafe_new() {
        let constraints = vec![
            vec![0, 3, 6],
            vec![1, 4, 7],
            vec![2, 5, 8],
            vec![0, 1, 2],
            vec![3, 4, 5],
            vec![6, 7, 8],
        ];

        let matrix = super::DancingMatrix::new(constraints).expect("should be ok");
    }
}
