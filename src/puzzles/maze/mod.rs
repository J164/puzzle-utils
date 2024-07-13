mod recursive_backtrack;

use std::collections::VecDeque;

use image::RgbImage;
use thiserror::Error;

use crate::{
    puzzles::maze::recursive_backtrack::recursive_backtrack,
    util::{RgbBuffer, SolutionPair, BLACK_PIXEL, RED_PIXEL, WHITE_PIXEL},
};

const MAX_DIMENSION: usize = 100;

#[derive(Debug, Error)]
pub enum MazeError {
    #[error("invalid dimension, max dimension is `{}`", MAX_DIMENSION)]
    InvalidDimension,
}

pub enum MazeAlgorithm {
    RecursiveBacktrack,
}

pub fn generate_maze(
    width: usize,
    height: usize,
    algorithm: MazeAlgorithm,
) -> Result<SolutionPair, MazeError> {
    if !(1..=MAX_DIMENSION).contains(&width) || !(1..=MAX_DIMENSION).contains(&height) {
        return Err(MazeError::InvalidDimension);
    }

    let (grid, solution) = create_maze(width, height, algorithm);

    let unsolved = print_maze(width as u32, height as u32, grid);
    let solved = print_solution(unsolved.clone(), solution);

    Ok(SolutionPair::new(unsolved, solved))
}

#[derive(Clone)]
struct Node {
    right: bool,
    down: bool,
}

impl Node {
    fn new() -> Self {
        Node {
            right: true,
            down: true,
        }
    }
}

#[derive(Clone)]
enum Direction {
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

fn create_maze(
    width: usize,
    height: usize,
    algorithm: MazeAlgorithm,
) -> (Vec<Node>, Vec<Direction>) {
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
                        Direction::Left
                    } else if parent == current + width {
                        Direction::Up
                    } else if parent == current - 1 {
                        Direction::Right
                    } else {
                        Direction::Down
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

fn print_maze(width: u32, height: u32, grid: Vec<Node>) -> RgbBuffer {
    let mut img = RgbImage::from_pixel(width * 10 + 1, height * 10 + 1, WHITE_PIXEL);

    for row in 0..img.height() {
        img.put_pixel(0, row, BLACK_PIXEL);
    }

    for col in 10..img.width() {
        img.put_pixel(col, 0, BLACK_PIXEL);
    }

    for (i, node) in grid.iter().enumerate() {
        let idx = i as u32;
        let x = idx % width;
        let y = idx / width;

        if node.right {
            for k in 0..=10 {
                img.put_pixel((x + 1) * 10, y * 10 + k, BLACK_PIXEL);
            }
        }

        if node.down {
            for k in 0..=10 {
                img.put_pixel(x * 10 + k, (y + 1) * 10, BLACK_PIXEL);
            }
        }
    }

    img
}

fn print_solution(mut image: RgbBuffer, solution: Vec<Direction>) -> RgbBuffer {
    let mut x = 0;
    let mut y = 0;

    for k in 0..=5 {
        image.put_pixel(x + 5, y + k, RED_PIXEL);
    }

    for step in solution.iter().rev() {
        match step {
            Direction::Right => {
                for k in 0..=10 {
                    image.put_pixel(x * 10 + k + 5, y * 10 + 5, RED_PIXEL);
                }

                x += 1;
            }
            Direction::Down => {
                for k in 0..=10 {
                    image.put_pixel(x * 10 + 5, y * 10 + k + 5, RED_PIXEL);
                }

                y += 1;
            }
            Direction::Left => {
                for k in 0..=10 {
                    image.put_pixel(x * 10 - k + 5, y * 10 + 5, RED_PIXEL);
                }

                x -= 1;
            }
            Direction::Up => {
                for k in 0..=10 {
                    image.put_pixel(x * 10 + 5, y * 10 - k + 5, RED_PIXEL);
                }

                y -= 1;
            }
        }
    }

    for k in 1..=5 {
        image.put_pixel(x * 10 + 5, y * 10 + k + 5, RED_PIXEL);
    }

    image
}
