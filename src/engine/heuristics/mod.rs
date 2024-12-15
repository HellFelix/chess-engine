use chess_backend::{Colour, FinishedState, GameState, Pieces};

use crate::engine::tree::Branch;
use crate::engine::utils::eval::Eval;
use crate::engine::utils::phase::GamePhase;
mod piece_square_table;

const MOBILITY_MOD: f32 = 0.1;
const POSITIONAL_MOD: f32 = 0.01;
// Piece values
const PAWN_VAL: f32 = 1.;
const KNIGHT_VAL: f32 = 3.;
const BISHOP_VAL: f32 = 3.;
const ROOK_VAL: f32 = 5.;
const QUEEN_VAL: f32 = 9.;

impl Branch {
    pub fn eval_position(&mut self, mobility: usize, depth: usize) -> Eval {
        match self.board.get_unchecked_game_state(mobility) {
            GameState::Ongoing => self.eval_heuristic(mobility),
            GameState::Finished(state) => match state {
                // With a finished state, the evaluation is absolute.
                FinishedState::Win(c, _) => match c {
                    Colour::White => Eval::Mate(depth, Colour::White),
                    Colour::Black => Eval::Mate(depth, Colour::Black),
                },
                FinishedState::Draw(_) => Eval::Numeric(0.),
            },
        }
    }

    pub fn eval_heuristic(&mut self, mobility: usize) -> Eval {
        let mut res = 0.;

        let white_pieces = Pieces::from(self.board.base.white);
        let black_pieces = Pieces::from(self.board.base.black);

        // Before evaluation, we should first update the current game phase
        // Evaluation may depend on the game phase
        self.phase = Some(GamePhase::determine_phase(
            self.phase,
            &white_pieces,
            &black_pieces,
        ));

        let piece_value_white = self.eval_pieces(&white_pieces);
        let piece_value_black = self.eval_pieces(&black_pieces);

        res += piece_value_white - piece_value_black;

        // unwrapping here should be safe since phase has previously been determined
        res += (piece_square_table::positional_evaluation(
            Colour::White,
            &white_pieces,
            self.phase.unwrap(),
        ) - piece_square_table::positional_evaluation(
            Colour::Black,
            &black_pieces,
            self.phase.unwrap(),
        )) * POSITIONAL_MOD;

        Eval::Numeric(res)
    }

    fn eval_pieces(&self, pieces: &Pieces) -> f32 {
        PAWN_VAL * pieces.pawns.len() as f32
            + KNIGHT_VAL * pieces.knights.len() as f32
            + BISHOP_VAL * pieces.bishops.len() as f32
            + ROOK_VAL * pieces.rooks.len() as f32
            + QUEEN_VAL * pieces.queens.len() as f32
    }
}
