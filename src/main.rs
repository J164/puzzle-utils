mod puzzles;
mod structures;
mod util;

use std::env;

use puzzles::maze::{generate_maze, MazeAlgorithm, MazeError};

fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    println!("Please pass the width (and optionally height) of the maze to generate");
    return;
  }

  let width = match args[1].parse() {
      Ok(width) => width,
      Err(_) => { println!("Invalid width"); return; }
  };

  let height = if args.len() < 3 { width } else { match args[2].parse() {
      Ok(height) => height,
      Err(_) => { println!("Invalid height"); return; }
  } };

  let maze = match generate_maze(width, height, MazeAlgorithm::RecursiveBacktrack) {
    Ok(maze) => maze,
    Err(MazeError::InvalidDimension) => { println!("Invalid maze dimensions"); return; } 
  };
  
  maze.unsolved.save("maze.png").unwrap();
  maze.solved.save("solution.png").unwrap();
}
