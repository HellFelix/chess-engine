use std::cmp::max_by;

use sqlite::{Connection, State};

use chess_backend::Board;

use crate::engine::utils::phase::GamePhase;

#[derive(Debug, Clone)]
struct BookMove {
    pub id: i64,
    pub parent_move: i64,
    pub san: String,
    pub eval: f64,
    pub freq: i64,
    pub terminal: bool,
}

pub fn find_bookmove(db_conn: &Connection, id: i64) -> (Board, Option<GamePhase>) {
    let chosen = find_best_by_parent(db_conn, id);

    println!("{chosen:?}");
    unimplemented!()
}

fn find_best_by_parent(db_conn: &Connection, id: i64) -> BookMove {
    let mut stm = db_conn
        .prepare("SELECT * FROM moves WHERE parent_move = :id AND frequency > 2")
        .unwrap();
    stm.bind((":id", id)).unwrap();

    let mut children = Vec::new();
    while let Ok(State::Row) = stm.next() {
        children.push(BookMove {
            id: stm.read::<i64, _>("id").unwrap(),
            parent_move: stm.read::<i64, _>("parent_move").unwrap(),
            san: stm.read::<String, _>("san").unwrap(),
            eval: stm.read::<f64, _>("eval").unwrap(),
            freq: stm.read::<i64, _>("frequency").unwrap(),
            terminal: stm.read::<i64, _>("terminal").unwrap() == 1,
        });
    }

    children
        .iter()
        .max_by(|bm1, bm2| bm1.freq.cmp(&bm2.freq))
        .unwrap()
        .clone()
}
