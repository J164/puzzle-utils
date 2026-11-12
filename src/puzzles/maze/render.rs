use image::RgbImage;

use super::{MazeDirection, MazeError, MazeNode};
use crate::render::{BLACK_PIXEL, RED_PIXEL, RgbBuffer, WHITE_PIXEL};

pub fn print_maze(width: u32, height: u32, grid: &[MazeNode]) -> Result<RgbBuffer, MazeError> {
    if width as usize * height as usize != grid.len() {
        return Err(MazeError::InvalidDimensions);
    }

    let mut image = RgbImage::from_pixel(width * 10 + 1, height * 10 + 1, WHITE_PIXEL);

    for row in 0..image.height() {
        image.put_pixel(0, row, BLACK_PIXEL);
    }

    for col in 10..image.width() {
        image.put_pixel(col, 0, BLACK_PIXEL);
    }

    for (i, node) in grid.iter().enumerate() {
        let idx = i as u32;
        let x = idx % width;
        let y = idx / width;

        if node.right {
            for k in 0..=10 {
                image.put_pixel((x + 1) * 10, y * 10 + k, BLACK_PIXEL);
            }
        }

        if node.down {
            for k in 0..=10 {
                image.put_pixel(x * 10 + k, (y + 1) * 10, BLACK_PIXEL);
            }
        }
    }

    Ok(image)
}

pub fn print_maze_solution(
    mut unsolved: RgbBuffer,
    solution: &[MazeDirection],
) -> Result<RgbBuffer, MazeError> {
    if unsolved.width() < 6 || unsolved.height() < 6 {
        return Err(MazeError::InvalidDimensions);
    }

    let mut x = 0;
    let mut y = 0;

    for k in 0..=5 {
        unsolved.put_pixel(x + 5, y + k, RED_PIXEL);
    }

    for step in solution.iter().rev() {
        match step {
            MazeDirection::Right => {
                if unsolved.width() < x * 10 + 16 || unsolved.height() < y * 10 + 6 {
                    return Err(MazeError::InvalidSolution);
                }

                for k in 0..=10 {
                    unsolved.put_pixel(x * 10 + k + 5, y * 10 + 5, RED_PIXEL);
                }

                x += 1;
            }
            MazeDirection::Down => {
                if unsolved.width() < x * 10 + 6 || unsolved.height() < y * 10 + 16 {
                    return Err(MazeError::InvalidSolution);
                }

                for k in 0..=10 {
                    unsolved.put_pixel(x * 10 + 5, y * 10 + k + 5, RED_PIXEL);
                }

                y += 1;
            }
            MazeDirection::Left => {
                if x * 10 < 15 || unsolved.height() < y * 10 + 6 {
                    return Err(MazeError::InvalidSolution);
                }

                for k in 0..=10 {
                    unsolved.put_pixel(x * 10 - k + 5, y * 10 + 5, RED_PIXEL);
                }

                x -= 1;
            }
            MazeDirection::Up => {
                if unsolved.width() < x * 10 + 6 || y * 10 < 15 {
                    return Err(MazeError::InvalidSolution);
                }

                for k in 0..=10 {
                    unsolved.put_pixel(x * 10 + 5, y * 10 - k + 5, RED_PIXEL);
                }

                y -= 1;
            }
        }
    }

    if unsolved.width() < x * 10 + 6 || unsolved.height() < y * 10 + 16 {
        return Err(MazeError::InvalidSolution);
    }

    for k in 1..=5 {
        unsolved.put_pixel(x * 10 + 5, y * 10 + k + 5, RED_PIXEL);
    }

    Ok(unsolved)
}
