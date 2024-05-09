use ab_glyph::FontRef;
use image::ImageBuffer;
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut},
    rect::Rect,
};
use thiserror::Error;

use crate::{
    cloudflare_image::SolutionPair,
    util::{RgbBuffer, BLACK_PIXEL, GRAY_PIXEL, ROBOTO_MEDIUM, WHITE_PIXEL},
};

type Rule = Vec<Vec<usize>>;

#[derive(Debug, Error)]
pub enum NonogramError {
    #[error("puzzle cannot be empty")]
    EmptyPuzzle,
    #[error("invalid rule `{0}`")]
    InvalidRule(String),
    #[error("invalid rule dimension")]
    InvalidRuleDimension,
}

pub fn solve_nonogram(col: &str, row: &str) -> Result<SolutionPair, NonogramError> {
    let width = col.split(';').count();
    let height = row.split(';').count();

    if width == 0 || height == 0 {
        return Err(NonogramError::EmptyPuzzle);
    }

    let col = parse_rule(col, height)?;
    let row = parse_rule(row, width)?;

    let unsolved = print_nonogram(width as u32, height as u32, &row, &col);

    let grid = solve(width, height, col, row)?;

    let solved = print_solution(width as u32, unsolved.clone(), grid);

    Ok(SolutionPair::new(unsolved, solved))
}

fn parse_rule(rule: &str, rule_height: usize) -> Result<Rule, NonogramError> {
    rule.split(';')
        .map(|rule| {
            let rule = rule
                .split(',')
                .map(|x| {
                    x.parse::<usize>()
                        .or(Err(NonogramError::InvalidRule(x.to_string())))
                })
                .collect::<Result<Vec<usize>, NonogramError>>()?;

            if rule.is_empty() || rule.iter().sum::<usize>() + (rule.len() - 1) > rule_height {
                return Err(NonogramError::InvalidRuleDimension);
            }

            Ok(rule)
        })
        .collect::<Result<Rule, NonogramError>>()
}

fn solve(width: usize, height: usize, col: Rule, row: Rule) -> Result<Vec<bool>, NonogramError> {
    let mut grid = vec![false; width * height];
    // TODO: solve nonogram
    Ok(grid)
}

fn print_nonogram(width: u32, height: u32, row: &Rule, col: &Rule) -> RgbBuffer {
    let mut image = ImageBuffer::from_pixel(width * 50 + 150, height * 50 + 150, WHITE_PIXEL);

    let font = FontRef::try_from_slice(ROBOTO_MEDIUM).expect("Font should be valid");

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

fn print_solution(width: u32, mut image: RgbBuffer, grid: Vec<bool>) -> RgbBuffer {
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
