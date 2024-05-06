use ab_glyph::FontRef;
use image::RgbImage;
use imageproc::drawing::draw_text_mut;

use crate::util::{RgbBuffer, BLACK_PIXEL, WHITE_PIXEL};

const NUM_MIN: u8 = 1;
const NUM_MAX: u8 = 9;

pub const GRID_SIZE: usize = 9;
const BOX_SIZE: usize = 3;

pub fn solve_sudoku(original: Vec<u8>) -> Option<(RgbBuffer, RgbBuffer)> {
    let mut sudoku = original.to_owned();
    let mut stack = Vec::with_capacity(GRID_SIZE * GRID_SIZE);

    if let Some(square) = next_blank(&sudoku, 0) {
        stack.push(square);
    } else {
        return Some((print_sudoku(original), print_sudoku(sudoku)));
    }

    while !stack.is_empty() {
        let Square { index, candidates } = stack.last_mut().expect("stack should be non-empty");

        if candidates.is_empty() {
            sudoku[*index] = 0;
            stack.pop();
            continue;
        }

        sudoku[*index] = candidates.pop().expect("candidates should be non-empty");

        if let Some(square) = next_blank(&sudoku, *index) {
            stack.push(square);
        } else {
            return Some((print_sudoku(original), print_sudoku(sudoku)));
        }
    }

    None
}

struct Square {
    index: usize,
    candidates: Vec<u8>,
}

fn next_blank(sudoku: &[u8], start: usize) -> Option<Square> {
    sudoku[start..]
        .iter()
        .enumerate()
        .find_map(|(index, &value)| {
            if value == 0 {
                Some(Square {
                    index: index + start,
                    candidates: candidates(sudoku, index + start),
                })
            } else {
                None
            }
        })
}

fn candidates(sudoku: &[u8], position: usize) -> Vec<u8> {
    let row = position / GRID_SIZE;
    let col = position % GRID_SIZE;
    let box_start = ((row / 3) * GRID_SIZE + col / 3) * BOX_SIZE;

    (NUM_MIN..=NUM_MAX)
        .filter(|&candidate| {
            let row_iter = (row * GRID_SIZE)..((row + 1) * GRID_SIZE);
            let col_iter = (col..=(col + (GRID_SIZE) * (GRID_SIZE - 1))).step_by(GRID_SIZE);
            let box_iter = (0..BOX_SIZE)
                .flat_map(|x| (box_start + x * GRID_SIZE)..(box_start + x * GRID_SIZE + BOX_SIZE));

            row_iter
                .chain(col_iter)
                .chain(box_iter)
                .all(|x| sudoku[x] != candidate)
        })
        .collect()
}

fn print_sudoku(sudoku: Vec<u8>) -> RgbBuffer {
    const IMAGE_SIZE: u32 = GRID_SIZE as u32 * 100;

    let mut image = RgbImage::from_pixel(IMAGE_SIZE, IMAGE_SIZE, WHITE_PIXEL);

    let font =
        FontRef::try_from_slice(include_bytes!("../../../resources/Roboto-Medium.ttf") as &[u8])
            .expect("Font should be valid");

    for grid_pos in 0..(GRID_SIZE as u32) {
        for line_coord in 0..IMAGE_SIZE {
            let grid_coord = grid_pos * 100;

            if grid_pos % 3 == 0 && grid_pos != 0 && grid_pos != GRID_SIZE as u32 - 1 {
                image.put_pixel(line_coord, grid_coord + 1, BLACK_PIXEL);
                image.put_pixel(line_coord, grid_coord - 1, BLACK_PIXEL);

                image.put_pixel(grid_coord + 1, line_coord, BLACK_PIXEL);
                image.put_pixel(grid_coord - 1, line_coord, BLACK_PIXEL);
            }

            image.put_pixel(line_coord, grid_coord, BLACK_PIXEL);
            image.put_pixel(grid_coord, line_coord, BLACK_PIXEL);
        }
    }

    for (i, number) in sudoku.iter().enumerate() {
        if *number == 0 {
            continue;
        }

        let x = (i % GRID_SIZE) * 100 + 25;
        let y = (i / GRID_SIZE) * 100 + 5;

        draw_text_mut(
            &mut image,
            BLACK_PIXEL,
            x as i32,
            y as i32,
            100.0,
            &font,
            &number.to_string(),
        );
    }

    image
}
