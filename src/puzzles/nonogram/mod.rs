use image::{ImageBuffer, Rgb, RgbImage};

use crate::util::{BLACK_PIXEL, WHITE_PIXEL};

#[derive(Debug)]
pub enum RuleParseError {
    EmptyRule,
    InvalidRule,
}

pub struct Rules {
    row: Vec<Vec<u32>>,
    col: Vec<Vec<u32>>,
}

fn parse_rule(rule: &str) -> Result<Vec<Vec<u32>>, RuleParseError> {
    rule.split(';')
        .map(|rule| {
            let rule = rule
                .split(',')
                .map(|x| x.parse::<u32>())
                .collect::<Result<Vec<u32>, _>>();

            if let Ok(rule) = rule {
                if rule.is_empty() {
                    return Err(RuleParseError::EmptyRule);
                }

                Ok(rule)
            } else {
                Err(RuleParseError::InvalidRule)
            }
        })
        .collect::<Result<Vec<Vec<u32>>, RuleParseError>>()
}

impl Rules {
    pub fn new(row: &str, col: &str) -> Result<Self, RuleParseError> {
        Ok(Rules {
            row: parse_rule(row)?,
            col: parse_rule(col)?,
        })
    }
}

#[derive(Debug)]
pub enum NonogramError {
    InvalidRuleSize,
}

pub struct Nonogram {
    width: usize,
    height: usize,
    rules: Rules,
    grid: Vec<bool>,
}

impl Nonogram {
    pub fn new(width: usize, height: usize, rules: Rules) -> Result<Self, NonogramError> {
        if rules.row.len() < height || rules.col.len() < width {
            return Err(NonogramError::InvalidRuleSize);
        }

        Ok(Nonogram {
            width,
            height,
            rules,
            grid: vec![false; width * height],
        })
    }
}

pub fn print_nonogram(puzzle: &Nonogram) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let width = puzzle.width as u32;
    let height = puzzle.height as u32;

    let mut img = RgbImage::from_pixel(width * 10, height * 10, WHITE_PIXEL);

    for (i, _) in puzzle.grid.iter().enumerate().filter(|(_, &x)| x) {
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
