use crate::{structures::disjoint_set::DisjointSet, util::choose_random};

use super::{MazeDirection, MazeNode};

pub fn recursive_backtrack(width: usize, height: usize) -> Vec<MazeNode> {
    let mut maze = vec![MazeNode::new(); width * height];
    let mut connections = DisjointSet::with_size(width * height);

    let mut path = vec![0];
    let mut can_visit = vec![
        vec![
            MazeDirection::Right,
            MazeDirection::Down,
            MazeDirection::Left,
            MazeDirection::Up
        ];
        width * height
    ];
    while !path.is_empty() {
        let coordinate = path[path.len() - 1];

        match visit_next(
            coordinate,
            width,
            height,
            &mut maze,
            &mut connections,
            &mut can_visit[coordinate],
        ) {
            Some(next) => path.push(next),
            None => {
                path.pop();
            }
        };
    }

    maze
}

fn visit_next(
    coordinate: usize,
    width: usize,
    height: usize,
    maze: &mut [MazeNode],
    connections: &mut DisjointSet,
    visitable: &mut Vec<MazeDirection>,
) -> Option<usize> {
    while !visitable.is_empty() {
        let rand_idx = choose_random(visitable).expect("visitable should be non-empty");
        match rand_idx {
            MazeDirection::Right => {
                if (coordinate % width) == (width - 1) {
                    continue;
                }

                let next = coordinate + 1;

                if connections
                    .common_set(coordinate, next)
                    .expect("coordinate and next should be present in the set")
                {
                    continue;
                }

                maze[coordinate].right = false;
                connections
                    .union(coordinate, next)
                    .expect("coordinate and next should be present in the set");
                return Some(next);
            }
            MazeDirection::Down => {
                if coordinate >= width * (height - 1) {
                    continue;
                }

                let next = coordinate + width;

                if connections
                    .common_set(coordinate, next)
                    .expect("coordinate and next should be present in the set")
                {
                    continue;
                }

                maze[coordinate].down = false;
                connections
                    .union(coordinate, next)
                    .expect("coordinate and next should be present in the set");
                return Some(next);
            }
            MazeDirection::Left => {
                if coordinate % width == 0 {
                    continue;
                }

                let next = coordinate - 1;

                if connections
                    .common_set(coordinate, next)
                    .expect("coordinate and next should be present in the set")
                {
                    continue;
                }

                maze[next].right = false;
                connections
                    .union(coordinate, next)
                    .expect("coordinate and next should be present in the set");
                return Some(next);
            }
            MazeDirection::Up => {
                if coordinate < width {
                    continue;
                }

                let next = coordinate - width;

                if connections
                    .common_set(coordinate, next)
                    .expect("coordinate and next should be present in the set")
                {
                    continue;
                }

                maze[next].down = false;
                connections
                    .union(coordinate, next)
                    .expect("coordinate and next should be present in the set");
                return Some(next);
            }
        }
    }

    None
}
