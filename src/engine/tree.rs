use std::{cmp::Ordering, f32::INFINITY, fmt::Display};

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

    fn simple_alpha_beta(
        &mut self,
        current_depth: usize,
        desired_depth: usize,
        alpha: Eval,
        beta: Eval,
        maximize: bool,
    ) -> Eval {
        self.populate();
        if current_depth == desired_depth || self.children.len() == 0 {
            let eval = eval_position(&self.board, self.children.len(), current_depth);
            self.eval = Some(eval);
            return eval;
        }

        if maximize {
            let mut max_eval = Eval::NegInfinity;
            let mut alpha = alpha;
            for child in &mut self.children {
                let eval =
                    child.simple_alpha_beta(current_depth + 1, desired_depth, alpha, beta, false);
                max_eval = max_eval.max(eval);
                self.eval = Some(max_eval);
                alpha = alpha.max(eval);
                if beta < alpha {
                    break;
                }
            }
            max_eval
        } else {
            let mut min_eval = Eval::Infinity;
            let mut beta = beta;
            for child in &mut self.children {
                let eval =
                    child.simple_alpha_beta(current_depth + 1, desired_depth, alpha, beta, true);
                min_eval = min_eval.min(eval);
                self.eval = Some(min_eval);
                beta = beta.min(eval);
                if beta < alpha {
                    break;
                }
            }
            min_eval
        }
    }

    pub fn run_node(&mut self, depth: usize, maximize: bool) {
        self.simple_alpha_beta(0, depth, Eval::NegInfinity, Eval::Infinity, maximize);
    }

    pub fn get_best(&self, maximize: bool) -> Branch {
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
        }
    }
}
