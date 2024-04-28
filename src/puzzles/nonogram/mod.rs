use image::{ImageBuffer, Rgb, RgbImage};

use crate::util::{BLACK_PIXEL, WHITE_PIXEL};

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

pub fn solve_nonogram(row: &str, col: &str) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
    let height = row.split(';').count();
    let width = col.split(';').count();

    if height == 0 || width == 0 {
        return None;
    }

    let row = parse_rule(row, width)?;
    let col = parse_rule(col, height)?;

    let mut grid = vec![false; height * width];

    Some(print_nonogram(width, height, &grid))
}

pub fn print_nonogram(width: usize, height: usize, grid: &[bool]) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let width = width as u32;
    let height = height as u32;

    let mut img = RgbImage::from_pixel(width * 10, height * 10, WHITE_PIXEL);

    for (i, _) in grid.iter().enumerate().filter(|(_, &x)| x) {
        let idx = i as u32;
        let x = idx % width;
        let y = idx / width;

        for row in 0..10 {
            for col in 0..10 {
                img.put_pixel(x * 10 + col, y * 10 + row, BLACK_PIXEL);
            }
        }
    }

    img
}
