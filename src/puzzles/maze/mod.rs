mod recursive_backtrack;

use std::collections::VecDeque;

use image::{ImageBuffer, Rgb, RgbImage};

use crate::puzzles::maze::recursive_backtrack::recursive_backtrack;

const WHITE_PIXEL: Rgb<u8> = Rgb([255, 255, 255]);
const BLACK_PIXEL: Rgb<u8> = Rgb([0, 0, 0]);
const RED_PIXEL: Rgb<u8> = Rgb([255, 0, 0]);

const MAX_DIMENSION: u32 = 1_000;

pub struct Maze {
  pub unsolved: ImageBuffer<Rgb<u8>, Vec<u8>>,
  pub solved: ImageBuffer<Rgb<u8>, Vec<u8>>
}

pub enum MazeAlgorithm {
  RecursiveBacktrack
}

#[derive(Debug)]
pub enum MazeError {
  InvalidDimension
}

pub fn generate_maze(width: u32, height: u32, algorithm: MazeAlgorithm) -> Result<Maze, MazeError> {
  if !(1..=MAX_DIMENSION).contains(&width) || !(1..=MAX_DIMENSION).contains(&height) {
    return Err(MazeError::InvalidDimension);
  }

  let unsolved_maze = match algorithm {
    MazeAlgorithm::RecursiveBacktrack => recursive_backtrack(width, height)
  };

  let mut unsolved = print_maze(width, height, &unsolved_maze);
  let solution = solve_maze(width, height, &unsolved_maze);
  let solved = print_solution(&solution, &mut unsolved);

  Ok(Maze {
    unsolved,
    solved
  })
}

#[derive(Clone)]
struct Node {
  right: bool,
  down: bool
}

impl Node {
  fn new() -> Self {
    Node {
      right: true,
      down: true
    }
  }
}

#[derive(Clone)]
enum Direction {
  Right,
  Down,
  Left,
  Up
}

#[derive(Clone)]
enum PathNode {
  Start,
  Path(usize)
}

fn solve_maze(width: u32, height: u32, maze: &[Node]) -> Vec<Direction> {
  let width = width as usize;
  let height: usize = height as usize;
  
  let mut path_tree = vec![PathNode::Start; width * height];

  let mut traversal = VecDeque::new();
  traversal.push_back(0);

  let mut found_paths = 0;

  let mut paths: Vec<Vec<Direction>> = vec![Vec::new(); width];
  let mut seen_node = vec![false; height * width];

  while found_paths < width {
    let coordinate = traversal.pop_front().unwrap();
    seen_node[coordinate] = true;

    if (coordinate / width) == (height - 1) {
      found_paths += 1;
      let mut current = coordinate;

      while let PathNode::Path(parent) = path_tree[current] {
        paths[coordinate % width].push(
          if parent == current + 1 { Direction::Left } 
          else if parent == current + width { Direction::Up }
          else if parent == current - 1 { Direction::Right } 
          else if parent == current - width { Direction::Down } 
          else { panic!("unreachable") }
        );

        current = parent;
      }
    }

    let right = coordinate + 1;
    if !maze[coordinate].right && !seen_node[right] {
      path_tree[right] = PathNode::Path(coordinate);
      traversal.push_back(right);
    }

    let down = coordinate + width;
    if !maze[coordinate].down && !seen_node[down] {
      path_tree[down] = PathNode::Path(coordinate);
      traversal.push_back(down);
    }

    if let Some(left) = coordinate.checked_sub(1) {
      if !maze[left].right && !seen_node[left] {
        path_tree[left] = PathNode::Path(coordinate);
        traversal.push_back(left);
      }
    }

    if let Some(up) = coordinate.checked_sub(width) {
      if !maze[up].down && !seen_node[up] {
        path_tree[up] = PathNode::Path(coordinate);
        traversal.push_back(up);
      }
    }
  }

  let mut max_path = paths.iter().max_by(
    |x, y| x.len().cmp(&y.len())
  ).unwrap().to_vec();

  max_path.reverse();

  max_path
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

fn print_solution(solution: &Vec<Direction>, unsolved: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
  let mut solved = unsolved.clone();

  let mut x = 0;
  let mut y = 0;

  for k in 0..=5 {
    solved.put_pixel(x + 5, y + k, RED_PIXEL);
  }

  for step in solution {
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