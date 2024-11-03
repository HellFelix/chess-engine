mod tests;

use std::{thread, time::Duration};

mod engine;
use chess_backend::{Board, START_POSITION};
use engine::{
    heuristics::{self, eval_position},
    EngineController,
};

fn main() {
    EngineController::init();

    let mut controller = EngineController::new(Board::from(START_POSITION), 1);

    loop {
        controller.search_move(Duration::from_secs(1));
        controller.show_board();
    }
}
