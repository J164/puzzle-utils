mod recursive_backtrack;

use std::collections::VecDeque;

use image::RgbImage;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    puzzles::maze::recursive_backtrack::recursive_backtrack,
    util::{BLACK_PIXEL, RED_PIXEL, WHITE_PIXEL},
    RgbBuffer,
};

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

pub fn print_maze(width: u32, height: u32, grid: &[MazeNode]) -> Result<RgbBuffer, MazeError> {
    if width as usize * height as usize != grid.len() {
        return Err(MazeError::InvalidDimensions);
    }

    let mut image = RgbImage::from_pixel(width * 10 + 1, height * 10 + 1, WHITE_PIXEL);

    for row in 0..image.height() {
        image.put_pixel(0, row, BLACK_PIXEL);
    }

    for col in 10..image.width() {
        image.put_pixel(col, 0, BLACK_PIXEL);
    }

    for (i, node) in grid.iter().enumerate() {
        let idx = i as u32;
        let x = idx % width;
        let y = idx / width;

        if node.right {
            for k in 0..=10 {
                image.put_pixel((x + 1) * 10, y * 10 + k, BLACK_PIXEL);
            }
        }

        if node.down {
            for k in 0..=10 {
                image.put_pixel(x * 10 + k, (y + 1) * 10, BLACK_PIXEL);
            }
        }
    }

    Ok(image)
}

pub fn print_maze_solution(
    mut unsolved: RgbBuffer,
    solution: &[MazeDirection],
) -> Result<RgbBuffer, MazeError> {
    if unsolved.width() < 6 || unsolved.height() < 6 {
        return Err(MazeError::InvalidDimensions);
    }

    let mut x = 0;
    let mut y = 0;

    for k in 0..=5 {
        unsolved.put_pixel(x + 5, y + k, RED_PIXEL);
    }

    for step in solution.iter().rev() {
        match step {
            MazeDirection::Right => {
                if unsolved.width() < x * 10 + 16 || unsolved.height() < y * 10 + 6 {
                    return Err(MazeError::InvalidSolution);
                }

                for k in 0..=10 {
                    unsolved.put_pixel(x * 10 + k + 5, y * 10 + 5, RED_PIXEL);
                }

                x += 1;
            }
            MazeDirection::Down => {
                if unsolved.width() < x * 10 + 6 || unsolved.height() < y * 10 + 16 {
                    return Err(MazeError::InvalidSolution);
                }

                for k in 0..=10 {
                    unsolved.put_pixel(x * 10 + 5, y * 10 + k + 5, RED_PIXEL);
                }

                y += 1;
            }
            MazeDirection::Left => {
                if x * 10 < 15 || unsolved.height() < y * 10 + 6 {
                    return Err(MazeError::InvalidSolution);
                }

                for k in 0..=10 {
                    unsolved.put_pixel(x * 10 - k + 5, y * 10 + 5, RED_PIXEL);
                }

                x -= 1;
            }
            MazeDirection::Up => {
                if unsolved.width() < x * 10 + 6 || y * 10 < 15 {
                    return Err(MazeError::InvalidSolution);
                }

                for k in 0..=10 {
                    unsolved.put_pixel(x * 10 + 5, y * 10 - k + 5, RED_PIXEL);
                }

                y -= 1;
            }
        }
    }

    if unsolved.width() < x * 10 + 6 || unsolved.height() < y * 10 + 16 {
        return Err(MazeError::InvalidSolution);
    }

    for k in 1..=5 {
        unsolved.put_pixel(x * 10 + 5, y * 10 + k + 5, RED_PIXEL);
    }

    Ok(unsolved)
}
