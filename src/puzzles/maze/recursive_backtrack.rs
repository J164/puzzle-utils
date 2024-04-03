use crate::{structures::disjoint_set::DisjointSet, util::choose_random};

use super::Node;

pub fn recursive_backtrack(width: u32, height: u32) -> Vec<Node> {
  let width: usize = width.try_into().unwrap();
  let height: usize = height.try_into().unwrap();
      
  let mut maze = vec![Node::new(); width * height];
  let mut connections = DisjointSet::with_size(width * height);
              
  let mut path = vec![0];
  let mut can_visit: Vec<Vec<u8>> = vec![vec![0,1,2,3]; width * height];
  while !path.is_empty() {
    let coordinate = path[path.len() - 1];

    match visit_next(coordinate, width, height, &mut maze, &mut connections, &mut can_visit[coordinate]) {
      Some(next) => path.push(next),
      None => { path.pop(); }
    };
  }
      
  maze
}

fn visit_next(coordinate: usize, width: usize, height: usize, maze: &mut Vec<Node>, connections: &mut DisjointSet, visitable: &mut Vec<u8>) -> Option<usize> {
  while !visitable.is_empty() {
    let rand_idx = choose_random(visitable);
    match rand_idx {
      0 => {
        if (coordinate % width) == (width - 1) {
          continue;
        }
  
        let next = coordinate + 1;
  
        if connections.find(coordinate).unwrap() == connections.find(next).unwrap() {
          continue;
        }
  
        maze[coordinate].right = false;
        connections.union(coordinate, next).unwrap();
        return Some(next);
      } 
      1 => {
        if coordinate >= width * (height - 1) {
          continue;
        }
  
        let next = coordinate + width;
  
        if connections.find(coordinate).unwrap() == connections.find(next).unwrap() {
          continue;
        }
  
        maze[coordinate].down = false;
        connections.union(coordinate, next).unwrap();
        return Some(next);
      }
      2 => {
        if coordinate % width == 0 {
          continue;
        }
  
        let next = coordinate - 1;
  
        if connections.find(coordinate).unwrap() == connections.find(next).unwrap() {
          continue;
        }
  
        maze[next].right = false;
        connections.union(coordinate, next).unwrap();
        return Some(next);
      }
      3 => {
        if coordinate < width {
          continue;
        }
  
        let next = coordinate - width;
  
        if connections.find(coordinate).unwrap() == connections.find(next).unwrap() {
          continue;
        }
  
        maze[next].down = false;
        connections.union(coordinate, next).unwrap();
        return Some(next);
      }
      _ => panic!("unreachable")
    }
  }

  None
}