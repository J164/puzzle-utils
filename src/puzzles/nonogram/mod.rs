use serde::ser::{Serialize, SerializeStruct, Serializer};

pub struct Nonogram {
    width: usize,
    height: usize,
    grid: Vec<bool>,
}

impl Serialize for Nonogram {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Maze", 3)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        state.serialize_field("grid", &self.grid)?;
        state.end()
    }
}

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

pub fn solve_nonogram(row: &str, col: &str) -> Option<Nonogram> {
    let height = row.split(';').count();
    let width = col.split(';').count();

    if height == 0 || width == 0 {
        return None;
    }

    let row = parse_rule(row, width)?;
    let col = parse_rule(col, height)?;

    let mut grid = vec![false; height * width];

    Some(Nonogram {
        width,
        height,
        grid,
    })
}
