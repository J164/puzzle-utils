use std::cmp::max;

use ab_glyph::FontRef;
use image::ImageBuffer;
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut},
    rect::Rect,
};

use super::NonogramError;
use crate::render::{BLACK_PIXEL, GRAY_PIXEL, ROBOTO_MEDIUM, RgbBuffer, WHITE_PIXEL};

pub fn print_nonogram(
    width: u32,
    height: u32,
    col: &[Vec<usize>],
    row: &[Vec<usize>],
) -> Result<RgbBuffer, NonogramError> {
    if width as usize != col.len() || height as usize != row.len() {
        return Err(NonogramError::InvalidDimensions);
    }

    let rule_width = max(150, width * 10);
    let rule_height = max(150, height * 10);

    let mut image = ImageBuffer::from_pixel(
        width * 50 + rule_width,
        height * 50 + rule_height,
        WHITE_PIXEL,
    );

    let font = FontRef::try_from_slice(ROBOTO_MEDIUM).expect("Font should be valid");

    for (x, rule) in col.iter().enumerate() {
        let x = (x as u32) * 50 + rule_width + 15;

        for (y, rule) in rule.iter().enumerate() {
            let y = (y as u32) * 30 + 10;

            draw_text_mut(
                &mut image,
                BLACK_PIXEL,
                x as i32,
                y as i32,
                30.0,
                &font,
                &rule.to_string(),
            );
        }
    }

    for (y, rule) in row.iter().enumerate() {
        let y = (y as u32) * 50 + rule_height + 10;

        draw_text_mut(
            &mut image,
            BLACK_PIXEL,
            10,
            y as i32,
            30.0,
            &font,
            &rule
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join("  "),
        );
    }

    for x in 0..width {
        for y in 0..(height * 50 + rule_height) {
            image.put_pixel(x * 50 + rule_width, y, GRAY_PIXEL);

            if x % 5 == 0 {
                image.put_pixel(x * 50 + rule_width + 1, y, GRAY_PIXEL);
                image.put_pixel(x * 50 + rule_width - 1, y, GRAY_PIXEL);
            }
        }
    }

    for y in 0..height {
        for x in 0..(width * 50 + rule_width) {
            image.put_pixel(x, y * 50 + rule_height, GRAY_PIXEL);

            if y % 5 == 0 {
                image.put_pixel(x, y * 50 + rule_height + 1, GRAY_PIXEL);
                image.put_pixel(x, y * 50 + rule_height - 1, GRAY_PIXEL);
            }
        }
    }

    Ok(image)
}

pub fn print_nonogram_solution(
    width: u32,
    height: u32,
    mut image: RgbBuffer,
    grid: &[bool],
) -> Result<RgbBuffer, NonogramError> {
    let rule_width = max(150, width * 10);
    let rule_height = max(150, height * 10);

    if grid.len() != width as usize * height as usize
        || image.width() != width * 50 + rule_width
        || image.height() != height * 50 + rule_height
    {
        return Err(NonogramError::InvalidDimensions);
    }

    for (i, square) in grid.iter().enumerate() {
        if !square {
            continue;
        }

        let x = (i as u32 % width) * 50 + max(150, width * 10) + 1;
        let y = (i as u32 / width) * 50 + max(150, height * 10) + 1;

        draw_filled_rect_mut(
            &mut image,
            Rect::at(x as i32, y as i32).of_size(49, 49),
            BLACK_PIXEL,
        );
    }

    Ok(image)
}
