use std::{
    f32::INFINITY,
    mem,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use chess_backend::{Board, ChessMove, Colour};
use threadpool::ThreadPool;
use tree::Branch;

pub mod heuristics;
mod tree;

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
        }
    }
    pub fn get_current(&self) -> Board {
        self.branch.board
    }
    fn search(&mut self, rx: Receiver<Command>) {
        let maximize = self.branch.board.side_to_move() == Colour::White;
        println!("Began search");
        let start = SystemTime::now();
        self.branch.run_node(4, maximize);
        let best = self.branch.get_best(maximize);
        let eval = best.eval;
        self.selected = Some(best);
        let stop = start.elapsed();
        println!("Finished searching");

        println!("It took {}ms", stop.unwrap().as_millis());
        println!("Evaluation is {eval:?}");
        while rx.try_recv().is_err() {
            thread::sleep(Duration::from_secs(1));
            println!("Searching");
        }
        self.workers.join();
    }
}
impl Default for Engine {
    fn default() -> Self {
        Self {
            branch: Branch::from(Board::default()),
            selected: None,
            workers: ThreadPool::default(),
        }
    }
}
