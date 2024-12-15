use std::{
    slice::Iter,
    sync::mpsc::{channel, Receiver, Sender},
    time::{Duration, SystemTime},
};

use sqlite::{self, Connection};
const DB_PATH: &str = "openings.db";

use chess_backend::{Board, Colour, GameState};
use threadpool::ThreadPool;
use tree::Branch;
use utils::{eval::Eval, phase::GamePhase};

pub mod heuristics;
mod opening_book;
pub mod tree;
pub mod utils;

fn get_db_connection() -> Connection {
    Connection::open(DB_PATH).expect("Failed to connect to opening database")
}

pub struct EngineController {
    board: Board,
    n_workers: usize,
    db_conn: Connection,
    phase: Option<GamePhase>,
}
impl EngineController {
    pub fn init() {
        chess_backend::init();
    }
    pub fn new(board: Board, n_workers: usize) -> Self {
        Self {
            board,
            n_workers,
            db_conn: get_db_connection(),
            phase: None,
        }
    }
    pub fn pick_move(&mut self, time_limit: Duration) {
        let mut engine = Engine::new(self.board, self.n_workers, self.phase);
        (self.board, self.phase) = engine.begin_search(time_limit, self.phase, &self.db_conn);
    }

    pub fn show_board(&self) {
        println!("{}", self.board);
    }

    pub fn get_game_state(&self) -> GameState {
        self.board.get_game_state()
    }

    pub fn is_over(&self) -> bool {
        self.get_game_state() != GameState::Ongoing
    }
}
impl Default for EngineController {
    fn default() -> Self {
        Self {
            board: Board::default(),
            n_workers: num_cpus::get(),
            db_conn: get_db_connection(),
            phase: Some(GamePhase::Opening(1)),
        }
    }
}

#[derive(Debug)]
struct Engine {
    branch: Branch,
    workers: ThreadPool,
    sender_model: Sender<(Branch, Vec<usize>, [Option<(Eval, Vec<usize>)>; 3], bool)>,
    receiver: Receiver<(Branch, Vec<usize>, [Option<(Eval, Vec<usize>)>; 3], bool)>,
}
impl Engine {
    pub fn new(board: Board, n_workers: usize, phase: Option<GamePhase>) -> Self {
        let (sender_model, receiver) = channel();
        Self {
            branch: Branch::from_parent(board, phase),
            workers: ThreadPool::new(n_workers),
            sender_model,
            receiver,
        }
    }
    pub fn begin_search(
        &mut self,
        time_limit: Duration,
        phase: Option<GamePhase>,
        db_conn: &Connection,
    ) -> (Board, Option<GamePhase>) {
        if let Some(p) = phase {
            match p {
                GamePhase::Opening(id) => opening_book::find_bookmove(db_conn, id),
                _ => self.search(time_limit),
            }
        } else {
            // Likely the first search, meaning the phase has yet to be determined
            self.search(time_limit)
        }
    }
    fn search(&mut self, time_limit: Duration) -> (Board, Option<GamePhase>) {
        let criteria = if self.branch.board.side_to_move() == Colour::White {
            Eval::NegInfinity
        } else {
            Eval::Infinity
        };
        self.add_job([Some(vec![]), None, None].iter(), criteria);
        let start_time = SystemTime::now();
        while start_time.elapsed().unwrap() < time_limit {
            for (res_branch, location, res, continue_search) in self.receiver.try_iter() {
                println!("Finished node");
                self.branch.insert_branch(res_branch, location.as_slice());

                if continue_search {
                    if let Some(criteria) = res[0].clone() {
                        self.add_job(
                            [
                                Some(res[0].clone().unwrap().1),
                                if let Some(e) = res[1].clone() {
                                    Some(e.1)
                                } else {
                                    None
                                },
                                if let Some(e) = res[2].clone() {
                                    Some(e.1)
                                } else {
                                    None
                                },
                            ]
                            .iter(),
                            criteria.0,
                        );
                    }
                } else {
                    println!("Node at {location:?} failed to meet required criteria. Terminating search.");
                }
            }
        }
        // When an answer has been demanded, let the current threads finish
        println!("Joining workers");
        self.workers.join();
        // Then choose the best branch from the explored tree
        let best = self
            .branch
            .get_best(self.branch.board.side_to_move() == Colour::White);

        if let Some(chosen) = best {
            (chosen.board, chosen.phase)
        } else {
            println!("Incomplete");
            self.search(time_limit * 2)
        }
    }
    fn add_job(&self, mut locations: Iter<Option<Vec<usize>>>, criteria: Eval) {
        self.handle_primary(
            locations
                .next()
                .unwrap()
                .clone()
                .expect("Primary branch has no moves"),
        );
        for _ in 0..2 {
            if let Some(location) = locations.next().unwrap().clone() {
                self.handle_secondary(location, criteria);
            }
        }
    }

    fn handle_primary(&self, location: Vec<usize>) {
        let tx = self.sender_model.clone();
        let mut node = self.branch.find_branch(&location.as_slice()).clone();
        self.workers.execute(move || {
            let maximize = node.board.side_to_move() == Colour::White;
            let res = node.run_node(3, location.as_slice(), maximize);
            tx.send((node, location.clone(), res, true))
                .expect("Failed to send finished branch");
        });
    }
    // A secondary node must be rated higher than its primary after one cycle or we discontinue the
    // search
    fn handle_secondary(&self, location: Vec<usize>, criteria: Eval) {
        let tx = self.sender_model.clone();
        let mut node = self.branch.find_branch(&location.as_slice()).clone();
        self.workers.execute(move || {
            let maximize = node.board.side_to_move() == Colour::White;
            let next_location = node.run_node(3, location.as_slice(), maximize);
            let updated_value = node.simple_minimax(maximize);
            let continue_search =
                (maximize && updated_value > criteria) || (!maximize && updated_value < criteria);
            tx.send((node, location.clone(), next_location, continue_search))
                .expect("Failed to send finished branch");
        });
    }
}
impl Default for Engine {
    fn default() -> Self {
        let (sender_model, receiver) = channel();
        Self {
            branch: Branch::from_parent(Board::default(), Some(GamePhase::Opening(1))),
            workers: ThreadPool::default(),
            sender_model,
            receiver,
        }
    }
}
