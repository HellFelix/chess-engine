use super::tree::Eval;
use chess_backend::{Board, Colour, FinishedState, GameState, Pieces};

mod piece_square_table;

pub fn eval_position(board: &Board, mobility: usize, depth: usize) -> Eval {
    match board.get_unchecked_game_state(mobility) {
        GameState::Ongoing => eval_heuristic(board, mobility),
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

const MOBILITY_MOD: f32 = 0.1;
const POSITIONAL_MOD: f32 = 0.01;
// Piece values
const PAWN_VAL: f32 = 1.;
const KNIGHT_VAL: f32 = 3.;
const BISHOP_VAL: f32 = 3.;
const ROOK_VAL: f32 = 5.;
const QUEEN_VAL: f32 = 9.;

#[derive(Clone, Copy, Debug)]
pub enum GamePhase {
    MiddleGame,
    EndGame,
}

pub fn eval_heuristic(board: &Board, mobility: usize) -> Eval {
    let mut res = 0.;

    let white_pieces = Pieces::from(board.base.white);
    let black_pieces = Pieces::from(board.base.black);

    let piece_value_white = eval_pieces(&white_pieces);
    let piece_value_black = eval_pieces(&black_pieces);

    res += piece_value_white - piece_value_black;

    let game_phase = if piece_value_white + piece_value_black > 34. {
        GamePhase::MiddleGame
    } else {
        GamePhase::EndGame
    };

    res += POSITIONAL_MOD
        * (piece_square_table::positional_evaluation(Colour::White, &white_pieces, game_phase)
            - piece_square_table::positional_evaluation(Colour::Black, &black_pieces, game_phase));

    Eval::Numeric(res)
}

fn eval_pieces(pieces: &Pieces) -> f32 {
    PAWN_VAL * pieces.pawns.len() as f32
        + KNIGHT_VAL * pieces.knights.len() as f32
        + BISHOP_VAL * pieces.bishops.len() as f32
        + ROOK_VAL * pieces.rooks.len() as f32
        + QUEEN_VAL * pieces.queens.len() as f32
}
