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
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    branch: Branch,
    selected: Option<Branch>,
    workers: ThreadPool,
    active_receivers: Vec<Receiver<(Branch, Vec<usize>)>>,
}
impl Engine {
    pub fn init() {
        chess_backend::init();
    }
    pub fn new(board: Board, n_workers: usize) -> Self {
        Self {
            branch: Branch::from(board),
            selected: None,
            workers: ThreadPool::new(n_workers),
            active_receivers: Vec::new(),
        }
    }
    pub fn get_current(&self) -> Board {
        self.branch.board
    }
    fn search(&mut self, rx: Receiver<Command>) {
        self.add_job(self.branch.clone(), vec![]);
        while rx.try_recv().is_err() {
            for receiver in &self.active_receivers {
                if let Ok((res_branch, location)) = receiver.try_recv() {
                    self.branch.insert_branch(res_branch, location.as_slice());
                }
            }
        }

        // When an answer has been demanded, let the current threads finish
        self.workers.join();
        // Then choose the best branch from the explored tree
        let start = SystemTime::now();
        let best = self
            .branch
            .get_best(self.branch.board.side_to_move() == Colour::White);
        let stop = start.elapsed().unwrap();
        println!("Fixing took {}us", stop.as_micros());
        self.selected = Some(best);
    }
    fn add_job(&mut self, mut node: Branch, location: Vec<usize>) {
        let (tx, rx) = channel();
        self.active_receivers.push(rx);
        self.workers.execute(move || {
            let maximize = node.board.side_to_move() == Colour::White;
            node.run_node(3, maximize);
            let best = node.get_best(maximize);
            tx.send((node, location));
        });
    }
}
impl Default for Engine {
    fn default() -> Self {
        Self {
            branch: Branch::from(Board::default()),
            selected: None,
            workers: ThreadPool::default(),
            active_receivers: Vec::new(),
        }
    }
}
