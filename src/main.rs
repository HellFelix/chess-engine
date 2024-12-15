mod tests;

use std::{thread, time::Duration};

mod engine;
use chess_backend::{Board, START_POSITION};
use engine::EngineController;

fn main() {
    book_move_testing();

    //engine_play();
}

fn book_move_testing() {
    EngineController::init();

    let mut controller = EngineController::default();
    controller.pick_move(Duration::from_secs(1));
    controller.show_board();
}

fn engine_play() {
    EngineController::init();

    let mut controller = EngineController::new(
        Board::from("rn2kb1r/pp3bpp/4p3/5pBq/3P4/3B2Q1/PPP2PPP/R3K2R w KQkq - 0 1"),
        2,
    );

    controller.show_board();
    loop {
        if controller.is_over() {
            break;
        } else {
            controller.pick_move(Duration::from_secs(1));
            controller.show_board();
        }
    }
    println!("Game ended with {:?}", controller.get_game_state());
}
