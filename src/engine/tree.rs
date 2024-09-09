use std::f32::INFINITY;

use chess_backend::{Board, ChessMove};

use super::heuristics::eval_position;

#[derive(Debug, Clone)]
pub struct Branch {
    pub board: Board,
    pub eval: Option<f32>,
    children: Vec<Branch>,
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

    pub fn simple_alpha_beta(
        &mut self,
        depth: usize,
        alpha: f32,
        beta: f32,
        maximize: bool,
    ) -> f32 {
        self.populate();
        if depth == 0 || self.children.len() == 0 {
            let eval = eval_position(&self.board, self.children.len());
            self.eval = Some(eval);
            return eval;
        }

        if maximize {
            let mut value = -INFINITY;
            let mut alpha = alpha;
            for child in &mut self.children {
                value = value.max(child.simple_alpha_beta(depth - 1, alpha, beta, false));
                if value > beta {
                    break; // beta cutoff
                }
                alpha = alpha.max(value);
                self.eval = Some(alpha);
            }
            return value;
        } else {
            let mut value = INFINITY;
            let mut beta = beta;
            for child in &mut self.children {
                value = value.min(child.simple_alpha_beta(depth - 1, alpha, beta, true));
                if value > alpha {
                    break; // alpha cutoff
                }
                beta = beta.min(value);
                self.eval = Some(beta);
            }
            return value;
        }
    }

    pub fn run_node(&mut self, depth: usize, maximize: bool) {
        self.simple_alpha_beta(depth, -INFINITY, INFINITY, maximize);
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
