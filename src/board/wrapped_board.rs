

use std::str::FromStr;
use crate::board::{Move, Board};
use chess;


#[derive(Clone)]
pub struct WrappedMove {
    r#move: chess::ChessMove
}

#[derive(Default)]
pub struct WrappedBoard {
    board: chess::Board,
    history: Vec<chess::Board>
}

fn get_rank_from_char(char: char) -> chess::Rank {
    return match char {
        '1' => chess::Rank::First,
        '2' => chess::Rank::Second,
        '3' => chess::Rank::Third,
        '4' => chess::Rank::Fourth,
        '5' => chess::Rank::Fifth,
        '6' => chess::Rank::Sixth,
        '7' => chess::Rank::Seventh,
        '8' => chess::Rank::Eighth,
        _ => panic!("Invalid rank!")
    };
}

fn get_file_from_char(char: char) -> chess::File {
    return match char {
        'a' => chess::File::A,
        'b' => chess::File::B,
        'c' => chess::File::C,
        'd' => chess::File::D,
        'e' => chess::File::E,
        'f' => chess::File::F,
        'g' => chess::File::G,
        'h' => chess::File::H,
        _ => panic!("Invalid file!")
    };
}

fn get_square_from_str(square_str: &str) -> chess::Square {

        let chars = square_str.chars().collect::<Vec<char>>();
        let (file_char, rank_char) = (chars[0], chars[1]);
        let file = get_file_from_char(file_char);
        let rank = get_rank_from_char(rank_char);

        return chess::Square::make_square(rank, file);

}

fn get_promotion_piece_from_char(char: char) -> chess::Piece {
    return match char {
        'n' => chess::Piece::Knight,
        'b' => chess::Piece::Bishop,
        'r' => chess::Piece::Rook,
        'q' => chess::Piece::Queen,
        _ => panic!("Invalid promotion piece!")
    };
}

impl Move for WrappedMove {

    fn from_algebraic(s: &str) -> Self {
        
        let start_sq = get_square_from_str(&s[..2]);
        let end_sq = get_square_from_str(&s[2..]);

        let maybe_promotion_char = s.chars().nth(5);
        let maybe_promotion_piece = match maybe_promotion_char {
            Option::None          => Option::None,
            Option::Some(c) => Option::Some(get_promotion_piece_from_char(c))
        };

        let r#move = chess::ChessMove::new(start_sq, end_sq, maybe_promotion_piece);

        return Self {r#move: r#move};
    }

    fn as_string(&self) -> String {
        self.r#move.to_string()
    }
}

impl Board<WrappedMove> for WrappedBoard {

    fn put_into_startpos(&mut self) {
       self.board = chess::Board::default();
       self.history = Vec::new();
    }

    fn put_into_fen(&mut self, fen: &str) {
        self.board = chess::Board::from_str(fen).expect("Invalid FEN!");
        self.history = Vec::new();
    }

    fn make(&mut self, r#move: WrappedMove) {

        // clone current board
        let old_board = self.board.clone();

        // make move on original board
        old_board.make_move(r#move.r#move, &mut self.board);

        // remember old board
        self.history.push(old_board);
    }

    fn unmake(&mut self) {
        self.board = self.history.pop().expect("Cannot unmake moves on an empty history!");
    }

    fn visualize(&self) {
        println!("{}", self.board);
    }

}
