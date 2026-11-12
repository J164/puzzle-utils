pub mod puzzles;
#[cfg(feature = "render")]
mod render;
mod structures;
#[cfg(feature = "maze")]
mod util;

#[cfg(feature = "maze")]
pub use crate::puzzles::maze::{MazeAlgorithm, MazeDirection, MazeError, MazeNode, create_maze};
#[cfg(all(feature = "maze", feature = "render"))]
pub use crate::puzzles::maze::{print_maze, print_maze_solution};

#[cfg(feature = "nonogram")]
pub use crate::puzzles::nonogram::{NonogramError, parse_nonogram_rules, solve_nonogram};
#[cfg(all(feature = "nonogram", feature = "render"))]
pub use crate::puzzles::nonogram::{print_nonogram, print_nonogram_solution};

#[cfg(all(feature = "sudoku", feature = "render"))]
pub use crate::puzzles::sudoku::print_sudoku;
#[cfg(feature = "sudoku")]
pub use crate::puzzles::sudoku::{SudokuError, parse_sudoku, solve_sudoku};

#[cfg(feature = "render")]
pub use crate::render::{RgbBuffer, image_to_png_bytes};
