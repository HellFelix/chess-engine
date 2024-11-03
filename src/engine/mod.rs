use std::{
    mem,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use chess_backend::{Board, Colour};
use num_cpus;
use threadpool::ThreadPool;
use tree::Branch;

pub mod heuristics;
pub mod tree;

pub struct EngineController {
    board: Board,
    n_workers: usize,
}
impl EngineController {
    pub fn init() {
        chess_backend::init();
    }
    pub fn new(board: Board, n_workers: usize) -> Self {
        Self { board, n_workers }
    }
    pub fn search_move(&mut self, time_limit: Duration) {
        let mut engine = Engine::new(self.board, self.n_workers);
        self.board = engine.search(time_limit);
    }

    pub fn show_board(&self) {
        println!("{}", self.board);
    }
}

#[derive(Debug)]
struct Engine {
    branch: Branch,
    workers: ThreadPool,
    sender_model: Sender<(Branch, Vec<usize>, Vec<usize>)>,
    receiver: Receiver<(Branch, Vec<usize>, Vec<usize>)>,
}
impl Engine {
    pub fn new(board: Board, n_workers: usize) -> Self {
        let (sender_model, receiver) = channel();
        Self {
            branch: Branch::from(board),
            workers: ThreadPool::new(n_workers),
            sender_model,
            receiver,
        }
    }
    fn search(&mut self, time_limit: Duration) -> Board {
        self.add_job(vec![]);
        let start_time = SystemTime::now();
        while start_time.elapsed().unwrap() < time_limit {
            for (res_branch, location, next_location) in self.receiver.try_iter() {
                println!("Finished node");
                self.branch.insert_branch(res_branch, location.as_slice());
                self.add_job(next_location);
            }
        }
        // When an answer has been demanded, let the current threads finish
        println!("Joining workers");
        self.workers.join();
        // Then choose the best branch from the explored tree
        println!("Completing tree");
        self.branch.show_branch(0);

        let best = self
            .branch
            .get_best(self.branch.board.side_to_move() == Colour::White);

        if let Some(chosen) = best {
            chosen.board
        } else {
            println!("Incomplete");
            self.search(time_limit * 2)
        }
    }
    fn add_job(&self, location: Vec<usize>) {
        let tx = self.sender_model.clone();
        let mut node = self.branch.find_branch(&location.as_slice()).clone();
        self.workers.execute(move || {
            let maximize = node.board.side_to_move() == Colour::White;
            let next_location = node.run_node(3, location.as_slice(), maximize);
            tx.send((node, location.clone(), next_location))
                .expect("Failed to send finished branch");
        });
    }
}
impl Default for Engine {
    fn default() -> Self {
        let (sender_model, receiver) = channel();
        Self {
            branch: Branch::from(Board::default()),
            workers: ThreadPool::default(),
            sender_model,
            receiver,
        }
    }
}
