mod recursive_backtrack;

use std::collections::VecDeque;

use image::{ImageBuffer, Rgb, RgbImage};

use crate::puzzles::maze::recursive_backtrack::recursive_backtrack;

const WHITE_PIXEL: Rgb<u8> = Rgb([255, 255, 255]);
const BLACK_PIXEL: Rgb<u8> = Rgb([0, 0, 0]);
const RED_PIXEL: Rgb<u8> = Rgb([255, 0, 0]);

const MAX_DIMENSION: u32 = 5_000;

pub struct Maze {
    pub unsolved: ImageBuffer<Rgb<u8>, Vec<u8>>,
    pub solved: ImageBuffer<Rgb<u8>, Vec<u8>>,
}

pub enum MazeAlgorithm {
    RecursiveBacktrack,
}

#[derive(Debug)]
pub enum MazeError {
    InvalidDimension,
}

pub fn generate_maze(width: u32, height: u32, algorithm: MazeAlgorithm) -> Result<Maze, MazeError> {
    if !(1..=MAX_DIMENSION).contains(&width) || !(1..=MAX_DIMENSION).contains(&height) {
        return Err(MazeError::InvalidDimension);
    }

    let unsolved_maze = match algorithm {
        MazeAlgorithm::RecursiveBacktrack => recursive_backtrack(width, height),
    };

    let mut unsolved = print_maze(width, height, &unsolved_maze);
    let solution = solve_maze(width, height, &unsolved_maze);
    let solved = print_solution(&solution, &mut unsolved);

    Ok(Maze { unsolved, solved })
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
    Path(u32),
    Unvisited,
}

fn solve_maze(width: u32, height: u32, maze: &[Node]) -> Vec<Direction> {
    let mut path_tree = vec![PathNode::Unvisited; (width * height) as usize];
    path_tree[0] = PathNode::Start;

    let mut traversal = VecDeque::new();
    traversal.push_back(0);

    let mut found_paths = 0;

    loop {
        let coordinate = traversal.pop_front().unwrap();

        if (coordinate / width) == (height - 1) {
            found_paths += 1;

            if found_paths == width {
                let mut current = coordinate;

                let mut path = Vec::new();
                while let PathNode::Path(parent) = path_tree[current as usize] {
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
        if !maze[coordinate as usize].right
            && matches!(path_tree[right as usize], PathNode::Unvisited)
        {
            path_tree[right as usize] = PathNode::Path(coordinate);
            traversal.push_back(right);
        }

        let down = coordinate + width;
        if !maze[coordinate as usize].down
            && matches!(path_tree[down as usize], PathNode::Unvisited)
        {
            path_tree[down as usize] = PathNode::Path(coordinate);
            traversal.push_back(down);
        }

        if let Some(left) = coordinate.checked_sub(1) {
            if !maze[left as usize].right && matches!(path_tree[left as usize], PathNode::Unvisited)
            {
                path_tree[left as usize] = PathNode::Path(coordinate);
                traversal.push_back(left);
            }
        }

        if let Some(up) = coordinate.checked_sub(width) {
            if !maze[up as usize].down && matches!(path_tree[up as usize], PathNode::Unvisited) {
                path_tree[up as usize] = PathNode::Path(coordinate);
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
