use std::{
    mem::{swap, take},
    slice::Iter,
};

use super::Square;

#[derive(Clone)]
enum Node {
    Start,
    Fill,
    Space,
    End,
}

struct RuleMachine {
    states: Vec<Node>,
}

impl RuleMachine {
    fn new(rule: &[usize]) -> Self {
        let mut states = vec![Node::Start, Node::End];

        for &value in rule {
            states.extend(vec![Node::Fill; value].into_iter());
            states.push(Node::Space);
        }

        *states.last_mut().expect("states should not be empty") = Node::End;

        RuleMachine { states }
    }

    fn find_left(&self, mut grid: Iter<Square>) -> Option<Vec<usize>> {
        let mut old_state = vec![None; self.states.len() + 1];
        let mut new_state = vec![None; self.states.len() + 1];

        old_state[0] = Some(vec![]);

        'outer: loop {
            let square = grid.next();
            let mut empty = true;

            for (state, mut old_matches) in old_state
                .iter_mut()
                .enumerate()
                .filter_map(|(index, matches)| Some((index, take(matches)?)))
            {
                empty = false;

                let curr_state = &self.states[state];
                let next_state = &self.states[state + 1];

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
                    Node::End => {
                        if !matches!(square, Some(Square::Filled)) {
                            let mut old_matches = old_matches.clone();
                            old_matches.push(state + 1);
                            new_state[state + 1] = Some(old_matches);
                        }

                        match self.states.get(state + 2) {
                            Some(Node::Fill) => {
                                if !matches!(square, Some(Square::Blocked)) {
                                    old_matches.push(state + 2);
                                    new_state[state + 2] = Some(old_matches);
                                }
                            }
                            Some(Node::Start | Node::Space | Node::End) => unreachable!(),
                            None => {
                                old_matches.push(state + 2);
                                new_state[state + 2] = Some(old_matches);
                                break 'outer;
                            }
                        }
                    }
                }
            }

            if empty {
                return None;
            }

            swap(&mut old_state, &mut new_state);
        }

        take(&mut new_state[self.states.len()])
    }
}
