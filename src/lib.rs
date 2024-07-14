pub mod puzzles;
mod structures;
mod util;

use std::io::Cursor;

use image::{ImageBuffer, ImageError, ImageFormat, Rgb};

pub use crate::puzzles::maze::{
    create_maze, print_maze, print_maze_solution, MazeAlgorithm, MazeDirection, MazeError, MazeNode,
};
pub use crate::puzzles::nonogram::{
    parse_nonogram_rules, print_nonogram, print_nonogram_solution, solve_nonogram, NonogramError,
};
pub use crate::puzzles::sudoku::{parse_sudoku, print_sudoku, solve_sudoku, SudokuError};

pub type RgbBuffer = ImageBuffer<Rgb<u8>, Vec<u8>>;

/// Converts a RgbBuffer to a Vec of bytes representing a PNG
pub fn image_to_png_bytes(image: &RgbBuffer) -> Result<Vec<u8>, ImageError> {
    let mut bytes = Vec::new();
    image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;

    Ok(bytes)
}
