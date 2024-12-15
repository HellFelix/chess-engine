use chess_backend::{init, Board, Colour, Piece, SanMove};
use std::{fmt::Display, time::SystemTime};

#[test]
fn basic_notation() {
    init();
    let mut board = Board::default();

    let m = SanMove::new(
        Piece::Pawn(Colour::White),
        false,
        12,
        (false, false),
        28,
        None,
        None,
    );
    board.make_san_move(m);

    println!("{board}");
    println!("{m}");
}

#[test]
fn file_disambiguation() {
    init();
    let fen = "1k6/ppp5/8/7R/8/8/PPP5/1K2R3 w - - 0 1";

    let board = Board::from(fen);

    for m in board
        .generate_legal_moves()
        .iter()
        .filter(|m| m.base.destination_square.unwrap() == 36)
    {
        println!("{}", board.get_san(&m.board));
    }
}

#[test]
fn rank_disambiguation() {
    init();
    let fen = "1k6/ppp5/8/7R/8/8/PPP5/1K5R w - - 0 1";

    let board = Board::from(fen);

    for m in board
        .generate_legal_moves()
        .iter()
        .filter(|m| m.base.destination_square.unwrap() == 15)
    {
        println!("{}", board.get_san(&m.board));
    }
}

#[test]
fn double_disambiguation() {
    init();

    let fen = "1k6/ppp5/5N1N/8/8/8/PPP4N/1K6 w - - 0 1";

    let board = Board::from(fen);

    for m in board
        .generate_legal_moves()
        .iter()
        .filter(|m| m.base.destination_square.unwrap() == 30)
    {
        println!("{}", board.get_san(&m.board));
    }
}

#[test]
fn kingside_castle() {
    init();

    let fen = "1k6/ppp5/8/8/8/8/8/4K2R w K - 0 1";

    let board = Board::from(fen);

    for m in board.generate_legal_moves() {
        println!("{}", board.get_san(&m.board));
    }
}

#[test]
fn queenside_castle() {
    init();

    let fen = "1k6/ppp5/8/8/8/8/P7/R3K3 w Q - 0 1";

    let board = Board::from(fen);

    for m in board.generate_legal_moves() {
        println!("{}", board.get_san(&m.board));
    }
}

#[test]
fn promotion_san() {
    init();
    let fen = "1k6/ppp3P1/8/8/8/8/8/4K3 w - - 0 1";

    let board = Board::from(fen);

    for m in board.generate_legal_moves() {
        println!("{}", board.get_san(&m.board));
    }
}
