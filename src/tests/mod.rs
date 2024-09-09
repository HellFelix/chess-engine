#[cfg(test)]
mod move_gen {
    use std::time::SystemTime;

    use chess_backend::{
        init, Board, CASTLE_KINGSIDE_POSITION, CASTLE_QUEENSIDE_POSITION, CHECK_POSITION,
        PROMOTION_POSITION, START_POSITION,
    };

    #[test]
    fn standard_position() {
        init();
        let board = Board::from(START_POSITION);
        let start = SystemTime::now();
        let moves = board.generate_legal_moves();
        let stop = start.elapsed();
        for m in moves {
            println!("{}", m.board);
        }
        println!("It took {}us to generate moves", stop.unwrap().as_micros());
    }

    #[test]
    fn castle_kingside_position() {
        init();
        let board = Board::from(CASTLE_KINGSIDE_POSITION);
        for m in board.generate_legal_moves() {
            println!("{}", m.board);
        }
    }

    #[test]
    fn castle_queenside_position() {
        init();
        let board = Board::from(CASTLE_QUEENSIDE_POSITION);
        for m in board.generate_legal_moves() {
            println!("{}", m.board);
        }
    }

    #[test]
    fn promotion_position() {
        init();
        let board = Board::from(PROMOTION_POSITION);
        for m in board.generate_legal_moves() {
            println!("{}", m.board);
        }
    }

    #[test]
    fn check_position() {
        init();
        let board = Board::from(CHECK_POSITION);
        for m in board.generate_legal_moves() {
            println!("{}", m.board);
        }
    }
}

#[cfg(test)]
mod engine;
