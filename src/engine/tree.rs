use std::cmp::Ordering;

use chess_backend::{Board, Colour};

use super::heuristics::eval_position;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Eval {
    Numeric(f32),
    Mate(usize, Colour),
    Infinity,
    NegInfinity,
}
impl Eval {
    pub fn max(self, other: Self) -> Self {
        if self >= other {
            self
        } else {
            other
        }
    }

    pub fn min(self, other: Self) -> Self {
        if self <= other {
            self
        } else {
            other
        }
    }
}
impl PartialOrd for Eval {
    fn lt(&self, other: &Self) -> bool {
        !self.ge(other)
    }
    fn le(&self, other: &Self) -> bool {
        !self.gt(other)
    }
    fn gt(&self, other: &Self) -> bool {
        match self {
            Self::Numeric(val_self) => match other {
                Self::Numeric(val_other) => val_self > val_other,
                Self::Mate(_, c) => match c {
                    Colour::White => false,
                    Colour::Black => true,
                },
                Self::Infinity => false,
                Self::NegInfinity => true,
            },
            Self::Mate(mate_self, c1) => match other {
                Self::Numeric(_) => match c1 {
                    Colour::White => true,
                    Colour::Black => false,
                },
                Self::Mate(mate_other, c2) => match c1 {
                    Colour::White => match c2 {
                        Colour::White => mate_self < mate_other,
                        Colour::Black => true,
                    },
                    Colour::Black => match c2 {
                        Colour::White => false,
                        Colour::Black => mate_self > mate_other,
                    },
                },
                Self::Infinity => false,
                Self::NegInfinity => true,
            },
            Self::Infinity => true,
            Self::NegInfinity => false,
        }
    }
    fn ge(&self, other: &Self) -> bool {
        match self {
            Self::Numeric(val_self) => match other {
                Self::Numeric(val_other) => val_self >= val_other,
                Self::Mate(_, c) => match c {
                    Colour::White => false,
                    Colour::Black => true,
                },
                Self::Infinity => false,
                Self::NegInfinity => true,
            },
            Self::Mate(mate_self, c1) => match other {
                Self::Numeric(_) => match c1 {
                    Colour::White => true,
                    Colour::Black => false,
                },
                Self::Mate(mate_other, c2) => match c1 {
                    Colour::White => match c2 {
                        Colour::White => mate_self <= mate_other,
                        Colour::Black => true,
                    },
                    Colour::Black => match c2 {
                        Colour::White => false,
                        Colour::Black => mate_self >= mate_other,
                    },
                },
                Self::Infinity => false,
                Self::NegInfinity => true,
            },
            Self::Infinity => true,
            Self::NegInfinity => false,
        }
    }
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self > other {
            Some(Ordering::Greater)
        } else if self < other {
            Some(Ordering::Less)
        } else if self == other {
            Some(Ordering::Equal)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub board: Board,
    pub eval: Option<Eval>,
    pub children: Vec<Branch>,
    pub is_terminal: bool,
}
impl Branch {
    pub fn populate(&mut self) {
        self.children = self
            .board
            .generate_legal_moves()
            .iter()
            .map(|m| Branch::from(m.board))
            .collect();
    }

    fn simple_alpha_beta<'a, 'b: 'a>(
        &'b mut self,
        current_depth: usize,
        desired_depth: usize,
        current_location: &'a [usize],
        alpha: Eval,
        beta: Eval,
        maximize: bool,
    ) -> (Eval, Vec<usize>) {
        self.populate();
        if current_depth == desired_depth || self.children.len() == 0 {
            let eval = eval_position(&self.board, self.children.len(), current_depth);
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
    ) -> Vec<usize> {
        self.is_terminal = false;
        self.simple_alpha_beta(
            0,
            depth,
            location,
            Eval::NegInfinity,
            Eval::Infinity,
            maximize,
        )
        .1
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

    pub fn get_best(&mut self, maximize: bool) -> Branch {
        // fix tree after expanded search
        self.simple_minimax(maximize);
        if maximize {
            self.children
                .iter()
                .max_by(|c1, c2| c1.eval.partial_cmp(&c2.eval).unwrap())
                .unwrap()
                .clone()
        } else {
            self.children
                .iter()
                .min_by(|c1, c2| c1.eval.partial_cmp(&c2.eval).unwrap())
                .unwrap()
                .clone()
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
}
impl From<Board> for Branch {
    fn from(value: Board) -> Self {
        Self {
            board: value,
            eval: None,
            children: Vec::new(),
            is_terminal: false,
        }
    }
}
