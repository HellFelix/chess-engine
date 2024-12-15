use std::mem;

use chess_backend::Board;

use crate::engine::utils::eval::Eval;
use crate::engine::utils::phase::GamePhase;

#[derive(Debug, Clone)]
pub struct Branch {
    pub board: Board,
    pub eval: Option<Eval>,
    pub phase: Option<GamePhase>,
    pub children: Vec<Branch>,
    pub is_terminal: bool,
}
impl Branch {
    pub fn populate(&mut self) {
        self.children = self
            .board
            .generate_legal_moves()
            .iter()
            .map(|m| Branch::from_parent(m.board, self.phase))
            .collect();
    }

    fn simple_alpha_beta<'a>(
        &'a mut self,
        current_depth: usize,
        desired_depth: usize,
        current_location: &'a [usize],
        alpha: Eval,
        beta: Eval,
        maximize: bool,
    ) -> (Eval, Vec<usize>) {
        self.populate();
        if current_depth == desired_depth || self.children.len() == 0 {
            let eval = self.eval_position(self.children.len(), current_depth);
            self.eval = Some(eval);
            self.is_terminal = true;
            return (eval, current_location.into());
        }

        if maximize {
            let mut max_eval = Eval::NegInfinity;
            let mut max_location = Vec::new();
            let mut alpha = alpha;
            for (relative_location, child) in &mut self.children.iter_mut().enumerate() {
                let (eval, inherited_location) = child.simple_alpha_beta(
                    current_depth + 1,
                    desired_depth,
                    &[current_location, &[relative_location]].concat(),
                    alpha,
                    beta,
                    false,
                );
                if eval > max_eval {
                    max_eval = eval;
                    max_location = inherited_location.clone();
                    self.eval = Some(max_eval);
                }
                alpha = alpha.max(eval);
                if beta < alpha {
                    break;
                }
            }
            (max_eval, max_location)
        } else {
            let mut min_eval = Eval::Infinity;
            let mut min_location = Vec::new();
            let mut beta = beta;
            for (relative_location, child) in &mut self.children.iter_mut().enumerate() {
                let (eval, inherited_location) = child.simple_alpha_beta(
                    current_depth + 1,
                    desired_depth,
                    &[current_location, &[relative_location]].concat(),
                    alpha,
                    beta,
                    true,
                );
                if eval < min_eval {
                    min_eval = eval;
                    min_location = inherited_location.clone();
                    self.eval = Some(min_eval);
                }
                beta = beta.min(eval);
                if beta < alpha {
                    break;
                }
            }
            (min_eval, min_location)
        }
    }

    pub fn run_node<'a>(
        &'a mut self,
        depth: usize,
        location: &'a [usize],
        maximize: bool,
    ) -> [Option<(Eval, Vec<usize>)>; 3] {
        self.is_terminal = false;
        self.simple_alpha_beta(
            0,
            depth,
            location,
            Eval::NegInfinity,
            Eval::Infinity,
            maximize,
        );
        self.get_top_three(location, maximize)
    }

    pub fn get_top_three(
        &self,
        location: &[usize],
        maximize: bool,
    ) -> [Option<(Eval, Vec<usize>)>; 3] {
        let mut top_three: [Option<(Eval, Vec<usize>)>; 3] = [None, None, None];
        for (relative_location, child) in self.children.iter().enumerate() {
            if let Some(eval) = child.eval {
                if let Some(current_first_value) = top_three[0].clone() {
                    if (maximize && eval > current_first_value.0)
                        || (!maximize && eval < current_first_value.0)
                    {
                        top_three[2] = mem::take(&mut top_three[1]);
                        top_three[1] = mem::take(&mut top_three[0]);
                        top_three[0] = Some((eval, [location, &[relative_location]].concat()));
                    } else if let Some(current_second_value) = top_three[1].clone() {
                        if (maximize && eval > current_second_value.0)
                            || (!maximize && eval < current_second_value.0)
                        {
                            top_three[2] = mem::take(&mut top_three[1]);
                            top_three[1] = Some((eval, [location, &[relative_location]].concat()));
                        } else if let Some(current_second_value) = top_three[2].clone() {
                            if (maximize && eval > current_second_value.0)
                                || (!maximize && eval < current_second_value.0)
                            {
                                top_three[2] =
                                    Some((eval, [location, &[relative_location]].concat()));
                            }
                        } else {
                            top_three[2] = Some((eval, [location, &[relative_location]].concat()));
                        }
                    } else {
                        top_three[1] = Some((eval, [location, &[relative_location]].concat()));
                    }
                } else {
                    top_three[0] = Some((eval, [location, &[relative_location]].concat()))
                }
            }
        }

        top_three
    }

    // Doesn't evaluate positions, simply rearanges with new information
    pub fn simple_minimax(&mut self, maximize: bool) -> Eval {
        if self.is_terminal {
            // Unwrap should be safe. All terminal nodes have been evaluated
            self.eval.unwrap()
        } else if maximize {
            let mut max_eval = Eval::NegInfinity;
            for child in &mut self.children {
                let eval = child.simple_minimax(false);
                max_eval = max_eval.max(eval);
                self.eval = Some(max_eval);
            }
            max_eval
        } else {
            let mut min_eval = Eval::Infinity;
            for child in &mut self.children {
                let eval = child.simple_minimax(true);
                min_eval = min_eval.min(eval);
                self.eval = Some(min_eval);
            }
            min_eval
        }
    }

    pub fn get_best(&mut self, maximize: bool) -> Option<&Branch> {
        // fix tree after expanded search
        self.simple_minimax(maximize);
        if maximize {
            self.children
                .iter()
                .max_by(|c1, c2| c1.eval.partial_cmp(&c2.eval).unwrap())
        } else {
            self.children
                .iter()
                .min_by(|c1, c2| c1.eval.partial_cmp(&c2.eval).unwrap())
        }
    }

    pub fn find_branch(&self, location: &[usize]) -> &Self {
        if location.len() == 0 {
            self
        } else {
            self.children[location[0]].find_branch(&location[1..])
        }
    }

    pub fn insert_branch(&mut self, input_branch: Branch, location: &[usize]) {
        if location.len() == 0 {
            *self = input_branch;
        } else {
            self.children[location[0]].insert_branch(input_branch, &location[1..]);
        }
    }

    pub fn show_branch(&self, depth: usize) {
        for _ in 0..depth {
            print!("|   ");
        }
        println!("{:?}", self.eval);
        for child in &self.children {
            if child.eval != None {
                child.show_branch(depth + 1);
            }
        }
    }

    /// Creates a new branch that should inherit the game phase from its parent.
    /// If the phase is None, it will be determined at the next evaluation
    pub fn from_parent(board: Board, parent_phase: Option<GamePhase>) -> Self {
        Self {
            board,
            eval: None,
            phase: parent_phase,
            children: Vec::new(),
            is_terminal: false,
        }
    }
}
