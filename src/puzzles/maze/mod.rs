mod recursive_backtrack;

use std::collections::VecDeque;

use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::puzzles::maze::recursive_backtrack::recursive_backtrack;

const MAX_DIMENSION: usize = 5_000;

pub struct Maze {
    width: usize,
    height: usize,
    grid: Vec<Node>,
    solution: Vec<usize>,
}

impl Serialize for Maze {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Maze", 4)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        state.serialize_field("grid", &self.grid)?;
        state.serialize_field("solution", &self.solution)?;
        state.end()
    }
}

pub enum MazeAlgorithm {
    RecursiveBacktrack,
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

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Node", 2)?;
        state.serialize_field("right", &self.right)?;
        state.serialize_field("down", &self.down)?;
        state.end()
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

pub fn generate_maze(width: usize, height: usize, algorithm: MazeAlgorithm) -> Option<Maze> {
    if !(1..=MAX_DIMENSION).contains(&width) || !(1..=MAX_DIMENSION).contains(&height) {
        return None;
    }

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
                    solution.push(current);
                    current = parent;
                }

                return Some(Maze {
                    width,
                    height,
                    grid,
                    solution,
                });
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
