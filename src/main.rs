mod puzzles;
mod structures;
mod util;

use puzzles::maze::{generate_maze, MazeAlgorithm};

fn main() {
  let maze = generate_maze(100, 100, MazeAlgorithm::RecursiveBacktrack).unwrap();
  maze.unsolved.save("maze.png").unwrap();
  maze.solved.save("solution.png").unwrap();
}
