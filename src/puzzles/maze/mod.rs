mod recursive_backtrack;

use std::collections::VecDeque;

use image::{ImageBuffer, Rgb, RgbImage};

use crate::{
    puzzles::maze::recursive_backtrack::recursive_backtrack,
    util::{BLACK_PIXEL, RED_PIXEL, WHITE_PIXEL},
};

const MAX_DIMENSION: usize = 5_000;

pub struct MazeImage {
    pub unsolved: ImageBuffer<Rgb<u8>, Vec<u8>>,
    pub solved: ImageBuffer<Rgb<u8>, Vec<u8>>,
}

pub enum MazeAlgorithm {
    RecursiveBacktrack,
}

pub fn generate_maze(width: usize, height: usize, algorithm: MazeAlgorithm) -> Option<MazeImage> {
    if !(1..=MAX_DIMENSION).contains(&width) || !(1..=MAX_DIMENSION).contains(&height) {
        return None;
    }

    let unsolved_maze = match algorithm {
        MazeAlgorithm::RecursiveBacktrack => recursive_backtrack(width, height),
    };

    let mut unsolved = print_maze(width as u32, height as u32, &unsolved_maze);
    let solution = solve_maze(width, height, &unsolved_maze);
    let solved = print_solution(&solution, &mut unsolved);

    Some(MazeImage { unsolved, solved })
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

fn solve_maze(width: usize, height: usize, maze: &[Node]) -> Vec<Direction> {
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
                let mut current = coordinate;

                let mut path = Vec::new();
                while let PathNode::Path(parent) = path_tree[current] {
                    path.push(if parent == current + 1 {
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

                return path;
            }
        }

        let right = coordinate + 1;
        if !maze[coordinate].right && matches!(path_tree[right], PathNode::Unvisited) {
            path_tree[right] = PathNode::Path(coordinate);
            traversal.push_back(right);
        }

        let down = coordinate + width;
        if !maze[coordinate].down && matches!(path_tree[down], PathNode::Unvisited) {
            path_tree[down] = PathNode::Path(coordinate);
            traversal.push_back(down);
        }

        if let Some(left) = coordinate.checked_sub(1) {
            if !maze[left].right && matches!(path_tree[left], PathNode::Unvisited) {
                path_tree[left] = PathNode::Path(coordinate);
                traversal.push_back(left);
            }
        }

        if let Some(up) = coordinate.checked_sub(width) {
            if !maze[up].down && matches!(path_tree[up], PathNode::Unvisited) {
                path_tree[up] = PathNode::Path(coordinate);
                traversal.push_back(up);
            }
        }
    }
}

fn print_maze(width: u32, height: u32, maze: &[Node]) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut img = RgbImage::from_pixel(width * 10 + 1, height * 10 + 1, WHITE_PIXEL);

    for row in 0..img.height() {
        img.put_pixel(0, row, BLACK_PIXEL);
    }

    for col in 10..img.width() {
        img.put_pixel(col, 0, BLACK_PIXEL);
    }

    for (i, node) in maze.iter().enumerate() {
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

fn print_solution(
    solution: &[Direction],
    unsolved: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut solved = unsolved.clone();

    let mut x = 0;
    let mut y = 0;

    for k in 0..=5 {
        solved.put_pixel(x + 5, y + k, RED_PIXEL);
    }

    for step in solution.iter().rev() {
        match step {
            Direction::Right => {
                for k in 0..=10 {
                    solved.put_pixel(x * 10 + k + 5, y * 10 + 5, RED_PIXEL);
                }

                x += 1;
            }
            Direction::Down => {
                for k in 0..=10 {
                    solved.put_pixel(x * 10 + 5, y * 10 + k + 5, RED_PIXEL);
                }

                y += 1;
            }
            Direction::Left => {
                for k in 0..=10 {
                    solved.put_pixel(x * 10 - k + 5, y * 10 + 5, RED_PIXEL);
                }

                x -= 1;
            }
            Direction::Up => {
                for k in 0..=10 {
                    solved.put_pixel(x * 10 + 5, y * 10 - k + 5, RED_PIXEL);
                }

                y -= 1;
            }
        }
    }

    for k in 1..10 {
        unsolved.put_pixel(x * 10 + k, (y + 1) * 10, WHITE_PIXEL);
        solved.put_pixel(x * 10 + k, (y + 1) * 10, WHITE_PIXEL);
    }

    for k in 1..=5 {
        solved.put_pixel(x * 10 + 5, y * 10 + k + 5, RED_PIXEL);
    }

    solved
}
