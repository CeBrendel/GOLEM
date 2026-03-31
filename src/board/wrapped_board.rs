

use std::str::FromStr;

use crate::{
    board::{Board, Move},
    search::traits::{Status, Searchable, Value}
};

use chess;
use chess::Square;


const SQUARES: [Square; 64] = [
    Square::A8, Square::B8, Square::C8, Square::D8, Square::E8, Square::F8, Square::G8, Square::H8,
    Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7,
    Square::A6, Square::B6, Square::C6, Square::D6, Square::E6, Square::F6, Square::G6, Square::H6,
    Square::A5, Square::B5, Square::C5, Square::D5, Square::E5, Square::F5, Square::G5, Square::H5,
    Square::A4, Square::B4, Square::C4, Square::D4, Square::E4, Square::F4, Square::G4, Square::H4,
    Square::A3, Square::B3, Square::C3, Square::D3, Square::E3, Square::F3, Square::G3, Square::H3,
    Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2,
    Square::A1, Square::B1, Square::C1, Square::D1, Square::E1, Square::F1, Square::G1, Square::H1
];

#[derive(Clone, Copy, Debug)]
pub struct WrappedMove {
    pub r#move: chess::ChessMove
}

#[derive(Default)]
pub struct WrappedBoard {
    pub board: chess::Board,
    pub history: Vec<chess::Board>
}

impl WrappedBoard {
    pub fn make_san_move(&self, san_str: &str) -> WrappedMove {
        let parsed_move = chess::ChessMove::from_san(&self.board, san_str).expect("Invalid move!");
        return WrappedMove{r#move: parsed_move};
    }
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

    fn make_move(&mut self, r#move: WrappedMove) {

        // clone current board
        let old_board = self.board.clone();

        // make move on original board
        old_board.make_move(r#move.r#move, &mut self.board);

        // remember old board
        self.history.push(old_board);
    }

    fn visualize(&self) {
        println!("{}", self.board);
    }

}

impl Value for i32 {
    const MIN: Self = i32::MIN;
    const WHITE_IS_DEAD: Self = -30_000;
    const ZERO: Self = 0;
    const BLACK_IS_DEAD: Self = 31_000;
    const MAX: Self = i32::MAX;
}

impl Searchable<WrappedMove, i32> for WrappedBoard {

    fn whites_turn(&self) -> bool {
        match self.board.side_to_move() {
            chess::Color::White => true,
            chess::Color::Black => false
        }
    }

    fn unmake_move(&mut self) {
        self.board = self.history.pop().expect("Cannot unmake moves on an empty history!");
    }

    fn get_legal_moves(&self) -> Vec<WrappedMove> {
        return chess::MoveGen::new_legal(&self.board)
            .map(|r#move| WrappedMove { r#move })
            .collect();
    }

    fn status(&self) -> Status {
        return match self.board.status() {
            chess::BoardStatus::Ongoing   => Status::Ongoing,
            chess::BoardStatus::Stalemate => Status::Stalemate,
            chess::BoardStatus::Checkmate => if self.whites_turn() {Status::WhiteIsDead} else {return Status::BlackIsDead}
        };
    }

    fn evaluate(&self) -> i32 {

        // check if we are in a stale- or checkmate
        match self.board.status() {
            chess::BoardStatus::Ongoing   => {},
            chess::BoardStatus::Stalemate => {return 0;},
            chess::BoardStatus::Checkmate => {
                if self.whites_turn() {
                    return i32::WHITE_IS_DEAD
                } else {
                    return i32::BLACK_IS_DEAD
                }
            }
        }

        // loop through all squares and sum piece values
        let mut evaluation: i32 = 0;
        for square in SQUARES {

            // get piece on square if there is any
            let piece = match self.board.piece_on(square) {
                Option::None               => continue,
                Option::Some(piece) => piece
            };

            // get value of piece
            let piece_value = match piece {
                chess::Piece::Pawn   => 100,
                chess::Piece::Knight => 300,
                chess::Piece::Bishop => 300,
                chess::Piece::Rook   => 500,
                chess::Piece::Queen  => 900,
                chess::Piece::King   => 0
            };

            // get color of piece
            let color_sign = if self.board.color_on(square).unwrap() == chess::Color::White {1} else {-1};

            // add piece value (with the right sign)
            evaluation += color_sign * piece_value
        }

        return evaluation;

    }
    
}
