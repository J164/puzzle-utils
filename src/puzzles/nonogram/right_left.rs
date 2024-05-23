use std::mem::{swap, take};

use super::{NonogramError, Square};

#[derive(Clone)]
enum Node {
    Start,
    Fill,
    Space,
    End,
}

pub struct RuleMachine {
    left_states: Vec<Node>,
    right_states: Vec<Node>,
}

impl RuleMachine {
    pub fn new(rule: &[usize]) -> Self {
        let mut left_states = vec![Node::Start, Node::End];

        for &value in rule {
            left_states.extend(vec![Node::Fill; value].into_iter());
            left_states.push(Node::Space);
        }

        *left_states.last_mut().expect("states should not be empty") = Node::End;

        let mut right_states = vec![Node::Start, Node::End];

        for &value in rule.iter().rev() {
            right_states.extend(vec![Node::Fill; value].into_iter());
            right_states.push(Node::Space);
        }

        *right_states.last_mut().expect("states should not be empty") = Node::End;

        RuleMachine {
            left_states,
            right_states,
        }
    }

    pub fn right_left(&self, mut grid: Vec<&mut Square>) -> Result<bool, NonogramError> {
        let left = find_left(&self.left_states, grid.iter()).ok_or(NonogramError::NoSolution)?;
        let right =
            find_left(&self.right_states, grid.iter().rev()).ok_or(NonogramError::NoSolution)?;

        let mut changed = false;

        let mut left_block = 0;
        let mut right_block = 0;

        let mut left_state = false;
        let mut right_state = false;

        for i in 0..left.len() {
            let left = left[i];
            let right = right[right.len() - 1 - i];

            if left != left_state {
                left_block += 1;
                left_state = left;
            }

            if right != right_state {
                right_block += 1;
                right_state = right;
            }

            if !matches!(grid[i], Square::Blank) {
                continue;
            }

            if left_block == right_block && left == right {
                changed = true;
                *grid[i] = if left {
                    Square::Filled
                } else {
                    Square::Blocked
                };
            }
        }

        Ok(changed)
    }
}

fn find_left<'a>(
    states: &[Node],
    mut grid: impl Iterator<Item = &'a &'a mut Square> + Clone,
) -> Option<Vec<bool>> {
    let mut old_state = vec![None; states.len() + 1];
    let mut new_state = vec![None; states.len() + 1];

    old_state[0] = Some(vec![]);

    'outer: loop {
        let square = grid.next();

        'matches: for (state, mut old_matches) in old_state
            .iter_mut()
            .enumerate()
            .filter_map(|(index, matches)| Some((index, take(matches)?)))
        {
            let curr_state = &states[state];
            let next_state = &states[state + 1];

            match curr_state {
                Node::Start | Node::Fill => (),
                Node::Space | Node::End => {
                    if !matches!(square, Some(Square::Filled)) {
                        let mut old_matches = old_matches.clone();
                        old_matches.push(state);
                        new_state[state] = Some(old_matches);
                    }
                }
            }

            match next_state {
                Node::Start => unreachable!(),
                Node::Fill => {
                    if !matches!(square, Some(Square::Blocked)) {
                        old_matches.push(state + 1);
                        new_state[state + 1] = Some(old_matches);
                    }
                }
                Node::Space => {
                    if !matches!(square, Some(Square::Filled)) {
                        old_matches.push(state + 1);
                        new_state[state + 1] = Some(old_matches);
                    }
                }
                Node::End => match states.get(state + 2) {
                    Some(Node::Fill) => {
                        if !matches!(square, Some(Square::Filled)) {
                            let mut old_matches = old_matches.clone();
                            old_matches.push(state + 1);
                            new_state[state + 1] = Some(old_matches);
                        }
                        if !matches!(square, Some(Square::Blocked)) {
                            old_matches.push(state + 2);
                            new_state[state + 2] = Some(old_matches);
                        }
                    }
                    Some(Node::Start | Node::Space | Node::End) => unreachable!(),
                    None => {
                        if let Some(mut square) = square {
                            let mut grid = grid.clone();
                            loop {
                                if matches!(square, Square::Filled) {
                                    continue 'matches;
                                }

                                old_matches.push(state + 1);
                                match grid.next() {
                                    Some(_square) => square = _square,
                                    None => break,
                                }
                            }
                        }

                        new_state[state + 2] = Some(old_matches);
                        break 'outer;
                    }
                },
            }
        }

        if square.is_none() {
            break;
        }

        swap(&mut old_state, &mut new_state);
    }

    Some(
        new_state[states.len()]
            .as_ref()?
            .iter()
            .map(|&state| match states[state] {
                Node::Start => unreachable!(),
                Node::Fill => true,
                Node::Space | Node::End => false,
            })
            .collect(),
    )
}
