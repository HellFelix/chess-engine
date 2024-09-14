mod tests;

use std::{thread, time::Duration};

mod engine;
use chess_backend::Board;
use engine::{heuristics::eval_position, Engine, EngineController};

fn main() {
    Engine::init();

    // let board = Board::from("2k5/8/1K6/8/8/8/3Q4/8 b - - 0 1");

    // let moves = board.generate_legal_moves();

    // for m in moves {
    //     let board = m.board;
    //     let mobility = board.generate_legal_moves().len();
    //     let state = board.get_unchecked_game_state(mobility);

    //     println!("{board}");
    //     println!("This position has state {state:?}");
    //     println!("This position has eval {}", eval_position(&board, mobility));
    // }

    let engine = Engine::new(
        Board::from("5rk1/5Nqp/4p3/3p4/3p4/P4Q2/1P5R/6RK w - - 0 1"),
        1,
    );
    println!("{}", engine.get_current());
    let mut controller = EngineController::new(engine);

    loop {
        controller.begin_search();
        thread::sleep(Duration::from_secs(3));
        controller.force_move();
        controller.show_board();
    }
    // controller.begin_search();
    // thread::sleep(Duration::from_secs(1));
    // controller.force_move();
    // thread::sleep(Duration::from_secs(1));

    // let mut controller = EngineController::new(engine);
    // controller.begin_search();
    // thread::sleep(Duration::from_secs(1));
    // controller.force_move();
    // controller.show_board();
    // thread::sleep(Duration::from_secs(1));
}
