use std::{collections::BTreeMap, mem::take, slice::Iter};

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
        let mut old_state = BTreeMap::from([(0, vec![])]);
        let mut new_state = BTreeMap::new();

        'outer: loop {
            let square = grid.next();

            for (&state, old_matches) in &old_state {
                let curr_state = &self.states[state];
                let next_state = &self.states[state + 1];

                match curr_state {
                    Node::Start | Node::Fill => (),
                    Node::Space | Node::End => {
                        if !matches!(square, Some(Square::Filled)) {
                            let mut matches = old_matches.clone();
                            matches.push(state);
                            new_state.insert(state, matches);
                        }
                    }
                }

                match next_state {
                    Node::Start => unreachable!(),
                    Node::Fill => {
                        if !matches!(square, Some(Square::Blocked)) {
                            let mut matches = old_matches.clone();
                            matches.push(state + 1);
                            new_state.insert(state + 1, matches);
                        }
                    }
                    Node::Space => {
                        if !matches!(square, Some(Square::Filled)) {
                            let mut matches = old_matches.clone();
                            matches.push(state + 1);
                            new_state.insert(state + 1, matches);
                        }
                    }
                    Node::End => {
                        if !matches!(square, Some(Square::Filled)) {
                            let mut matches = old_matches.clone();
                            matches.push(state + 1);
                            new_state.insert(state + 1, matches);
                        }

                        match self.states.get(state + 2) {
                            Some(Node::Fill) => {
                                if !matches!(square, Some(Square::Blocked)) {
                                    let mut matches = old_matches.clone();
                                    matches.push(state + 2);
                                    new_state.insert(state + 2, matches);
                                }
                            }
                            Some(Node::Start | Node::Space | Node::End) => unreachable!(),
                            None => {
                                let mut matches = old_matches.clone();
                                matches.push(state + 2);
                                new_state.insert(state + 2, matches);
                                break 'outer;
                            }
                        }
                    }
                }
            }

            if new_state.is_empty() {
                return None;
            }

            old_state = take(&mut new_state);
        }

        new_state
            .get_mut(&self.states.len())
            .map(|solution| take(solution))
    }
}
