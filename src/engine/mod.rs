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

#[derive(Debug, PartialEq)]
enum Command {
    Demand,
}

struct Searcher {
    sender: Sender<Command>,
    handle: JoinHandle<Engine>,
}
impl Searcher {
    pub fn new(sender: Sender<Command>, handle: JoinHandle<Engine>) -> Self {
        Self { sender, handle }
    }
}

pub struct EngineController {
    engine: Option<Engine>,
    searcher: Option<Searcher>,
}
impl EngineController {
    pub fn new(engine: Engine) -> Self {
        Self {
            engine: Some(engine),
            searcher: None,
        }
    }
    pub fn show_board(&self) {
        if let Some(e) = &self.engine {
            println!("{}", e.get_current());
        } else {
            println!("Engine is unavailable");
        }
    }
    pub fn begin_search(&mut self) {
        let mut eng = mem::take(&mut self.engine).unwrap();
        let (tx, rx) = channel();
        self.searcher = Some(Searcher::new(
            tx,
            thread::spawn(move || {
                eng.search(rx);
                return eng;
            }),
        ));
    }
    pub fn demand_answer(&mut self) -> Option<Branch> {
        if let Some(s) = mem::take(&mut self.searcher) {
            s.sender.send(Command::Demand).unwrap();
            if let Ok(engine) = s.handle.join() {
                let selected = engine.selected.clone();
                self.engine = Some(engine);
                selected
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn force_move(&mut self) {
        let answer = self.demand_answer().unwrap();
        if let Some(e) = &mut self.engine {
            e.branch = answer;
            e.selected = None;
            (e.sender_model, e.receiver) = channel();
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    branch: Branch,
    selected: Option<Branch>,
    workers: ThreadPool,
    sender_model: Sender<(Branch, Vec<usize>, Vec<usize>)>,
    receiver: Receiver<(Branch, Vec<usize>, Vec<usize>)>,
}
impl Engine {
    pub fn init() {
        chess_backend::init();
    }
    pub fn new(board: Board, n_workers: usize) -> Self {
        let (sender_model, receiver) = channel();
        Self {
            branch: Branch::from(board),
            selected: None,
            workers: ThreadPool::new(n_workers),
            sender_model,
            receiver,
        }
    }
    pub fn get_current(&self) -> Board {
        self.branch.board
    }
    fn search(&mut self, rx: Receiver<Command>) {
        self.add_job(vec![]);
        let mut i = 0;
        while rx.try_recv().is_err() {
            for (res_branch, location, next_location) in self.receiver.try_iter() {
                self.branch.insert_branch(res_branch, location.as_slice());
                self.add_job(next_location);
            }
        }

        // When an answer has been demanded, let the current threads finish
        self.workers.join();
        thread::sleep(Duration::from_secs(1));
        // Then choose the best branch from the explored tree
        let start = SystemTime::now();
        let best = self
            .branch
            .get_best(self.branch.board.side_to_move() == Colour::White);
        let stop = start.elapsed().unwrap();
        println!("Fixing took {}us", stop.as_micros());
        self.selected = Some(best);
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
            selected: None,
            workers: ThreadPool::default(),
            sender_model,
            receiver,
        }
    }
}
