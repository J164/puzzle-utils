use crate::{structures::disjoint_set::DisjointSet, util::choose_random};

use super::{Direction, Node};

pub fn recursive_backtrack(width: u32, height: u32) -> Vec<Node> {
    let mut maze = vec![Node::new(); (width * height) as usize];
    let mut connections = DisjointSet::with_size((width * height) as usize);

    let mut path = vec![0];
    let mut can_visit = vec![
        vec![
            Direction::Right,
            Direction::Down,
            Direction::Left,
            Direction::Up
        ];
        (width * height) as usize
    ];
    while !path.is_empty() {
        let coordinate = path[path.len() - 1];

        match visit_next(
            coordinate,
            width,
            height,
            &mut maze,
            &mut connections,
            &mut can_visit[coordinate as usize],
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
    coordinate: u32,
    width: u32,
    height: u32,
    maze: &mut [Node],
    connections: &mut DisjointSet,
    visitable: &mut Vec<Direction>,
) -> Option<u32> {
    while !visitable.is_empty() {
        let rand_idx = choose_random(visitable).expect("visitable should be non-empty");
        match rand_idx {
            Direction::Right => {
                if (coordinate % width) == (width - 1) {
                    continue;
                }

                let next = coordinate + 1;

                if connections
                    .common_set(coordinate as usize, next as usize)
                    .expect("coordinate and next should be present in the set")
                {
                    continue;
                }

                maze[coordinate as usize].right = false;
                connections
                    .union(coordinate as usize, next as usize)
                    .expect("coordinate and next should be present in the set");
                return Some(next);
            }
            Direction::Down => {
                if coordinate >= width * (height - 1) {
                    continue;
                }

                let next = coordinate + width;

                if connections
                    .common_set(coordinate as usize, next as usize)
                    .expect("coordinate and next should be present in the set")
                {
                    continue;
                }

                maze[coordinate as usize].down = false;
                connections
                    .union(coordinate as usize, next as usize)
                    .expect("coordinate and next should be present in the set");
                return Some(next);
            }
            Direction::Left => {
                if coordinate % width == 0 {
                    continue;
                }

                let next = coordinate - 1;

                if connections
                    .common_set(coordinate as usize, next as usize)
                    .expect("coordinate and next should be present in the set")
                {
                    continue;
                }

                maze[next as usize].right = false;
                connections
                    .union(coordinate as usize, next as usize)
                    .expect("coordinate and next should be present in the set");
                return Some(next);
            }
            Direction::Up => {
                if coordinate < width {
                    continue;
                }

                let next = coordinate - width;

                if connections
                    .common_set(coordinate as usize, next as usize)
                    .expect("coordinate and next should be present in the set")
                {
                    continue;
                }

                maze[next as usize].down = false;
                connections
                    .union(coordinate as usize, next as usize)
                    .expect("coordinate and next should be present in the set");
                return Some(next);
            }
        }
    }

    None
}
