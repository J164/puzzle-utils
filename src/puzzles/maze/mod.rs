mod recursive_backtrack;
#[cfg(feature = "render")]
mod render;

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::puzzles::maze::recursive_backtrack::recursive_backtrack;

#[cfg(feature = "render")]
pub use self::render::{print_maze, print_maze_solution};

#[derive(Debug, Error)]
pub enum MazeError {
    #[error("maze dimensions are invalid")]
    InvalidDimensions,
    #[error("maze solution is invalid")]
    InvalidSolution,
}

#[derive(Debug, Clone)]
pub enum MazeAlgorithm {
    RecursiveBacktrack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazeNode {
    right: bool,
    down: bool,
}

impl MazeNode {
    fn new() -> Self {
        MazeNode {
            right: true,
            down: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MazeDirection {
    Right,
    Down,
    Left,
    Up,
}

#[derive(Clone)]
enum PathNode {
    Start,
    Path(usize),
    Unvisited,
}

pub fn create_maze(
    width: usize,
    height: usize,
    algorithm: MazeAlgorithm,
) -> (Vec<MazeNode>, Vec<MazeDirection>) {
    let mut grid = match algorithm {
        MazeAlgorithm::RecursiveBacktrack => recursive_backtrack(width, height),
    };

    let mut path_tree = vec![PathNode::Unvisited; width * height];
    path_tree[0] = PathNode::Start;

    let mut traversal = VecDeque::new();
    traversal.push_back(0);

    let mut found_paths = 0;

    loop {
        let coordinate = traversal
            .pop_front()
            .expect("the traversal should be non-empty");

        if (coordinate / width) == (height - 1) {
            found_paths += 1;

            if found_paths == width {
                grid[coordinate].down = false;
                let mut current = coordinate;

                let mut solution = Vec::new();
                while let PathNode::Path(parent) = path_tree[current] {
                    solution.push(if parent == current + 1 {
                        MazeDirection::Left
                    } else if parent == current + width {
                        MazeDirection::Up
                    } else if parent == current - 1 {
                        MazeDirection::Right
                    } else {
                        MazeDirection::Down
                    });

                    current = parent;
                }

                return (grid, solution);
            }
        }

        let right = coordinate + 1;
        if !grid[coordinate].right && matches!(path_tree[right], PathNode::Unvisited) {
            path_tree[right] = PathNode::Path(coordinate);
            traversal.push_back(right);
        }

        let down = coordinate + width;
        if !grid[coordinate].down && matches!(path_tree[down], PathNode::Unvisited) {
            path_tree[down] = PathNode::Path(coordinate);
            traversal.push_back(down);
        }

        if let Some(left) = coordinate.checked_sub(1) {
            if !grid[left].right && matches!(path_tree[left], PathNode::Unvisited) {
                path_tree[left] = PathNode::Path(coordinate);
                traversal.push_back(left);
            }
        }

        if let Some(up) = coordinate.checked_sub(width) {
            if !grid[up].down && matches!(path_tree[up], PathNode::Unvisited) {
                path_tree[up] = PathNode::Path(coordinate);
                traversal.push_back(up);
            }
        }
    }
}
