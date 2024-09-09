use std::f32::INFINITY;

use chess_backend::{Board, ChessMove, Colour, FinishedState, GameState, Piece, Pieces};

pub fn eval_position(board: &Board, mobility: usize) -> f32 {
    match board.get_unchecked_game_state(mobility) {
        GameState::Ongoing => eval_heuristic(board, mobility),
        GameState::Finished(state) => match state {
            // With a finished state, the evaluation is absolute.
            FinishedState::Win(c, _) => match c {
                Colour::White => INFINITY,
                Colour::Black => -INFINITY,
            },
            FinishedState::Draw(_) => 0.,
        },
    }
}

fn eval_heuristic(board: &Board, mobility: usize) -> f32 {
    let mut res = 0.;

    res +=
        eval_pieces(Pieces::from(board.base.white)) - eval_pieces(Pieces::from(board.base.black));

    res
}

const PAWN_VAL: f32 = 1.;
const KNIGHT_VAL: f32 = 3.;
const BISHOP_VAL: f32 = 3.;
const ROOK_VAL: f32 = 5.;
const QUEEN_VAL: f32 = 9.;

fn eval_pieces(pieces: Pieces) -> f32 {
    PAWN_VAL * pieces.pawns.len() as f32
        + KNIGHT_VAL * pieces.knights.len() as f32
        + BISHOP_VAL * pieces.bishops.len() as f32
        + ROOK_VAL * pieces.rooks.len() as f32
        + QUEEN_VAL * pieces.queens.len() as f32
}
