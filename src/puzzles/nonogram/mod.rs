use ab_glyph::FontRef;
use image::ImageBuffer;
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut},
    rect::Rect,
};

use crate::util::{RgbBuffer, BLACK_PIXEL, GRAY_PIXEL, WHITE_PIXEL};

fn parse_rule(rule: &str, rule_height: usize) -> Option<Vec<Vec<usize>>> {
    rule.split(';')
        .map(|rule| {
            let rule = rule
                .split(',')
                .map(|x| x.parse::<usize>())
                .collect::<Result<Vec<usize>, _>>()
                .ok()?;

            if rule.is_empty() || rule.iter().sum::<usize>() + (rule.len() - 1) > rule_height {
                return None;
            }
            Some(rule)
        })
        .collect::<Option<Vec<Vec<usize>>>>()
}

pub fn solve_nonogram(row: &str, col: &str) -> Option<(RgbBuffer, RgbBuffer)> {
    let height = row.split(';').count();
    let width = col.split(';').count();

    if height == 0 || width == 0 {
        return None;
    }

    let row = parse_rule(row, width)?;
    let col = parse_rule(col, height)?;

    let unsolved = print_nonogram(width as u32, height as u32, &row, &col);

    let mut grid = vec![false; height * width];
    // TODO: solve nonogram

    let solved = print_solution(width as u32, unsolved.clone(), &grid);

    Some((unsolved, solved))
}

fn print_nonogram(width: u32, height: u32, row: &[Vec<usize>], col: &[Vec<usize>]) -> RgbBuffer {
    let mut image = ImageBuffer::from_pixel(width * 50 + 150, height * 50 + 150, WHITE_PIXEL);

    let font =
        FontRef::try_from_slice(include_bytes!("../../../resources/Roboto-Medium.ttf") as &[u8])
            .expect("Font should be valid");

    for (y, rule_set) in row.iter().enumerate() {
        let y = (y as u32) * 50 + 160;

        for (x, rule) in rule_set.iter().enumerate() {
            let x = (x as u32) * 30 + 10;

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

    for (x, rule_set) in col.iter().enumerate() {
        let x = (x as u32) * 50 + 165;

        for (y, rule) in rule_set.iter().enumerate() {
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

    for x in 0..width {
        for y in 0..(height * 50 + 150) {
            image.put_pixel(x * 50 + 150, y, GRAY_PIXEL);
        }
    }

    for y in 0..height {
        for x in 0..(width * 50 + 150) {
            image.put_pixel(x, y * 50 + 150, GRAY_PIXEL);
        }
    }

    image
}

fn print_solution(width: u32, mut image: RgbBuffer, grid: &[bool]) -> RgbBuffer {
    for (i, square) in grid.iter().enumerate() {
        if !square {
            continue;
        }

        let x = (i as u32 % width) * 50 + 151;
        let y = (i as u32 / width) * 50 + 151;

        draw_filled_rect_mut(
            &mut image,
            Rect::at(x as i32, y as i32).of_size(49, 49),
            BLACK_PIXEL,
        );
    }

    image
}
