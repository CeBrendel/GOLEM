

use std::{hash::Hash, str::FromStr};

use crate::{
    board::{
        File, Rank, Square,
        Board, Move, Piece, ColoredPiece,
        FILES, RANKS
    },
    search::{
        Searchable, Status, Value,
        move_ordering::MVVLVAScorer,
        transposition_table::Hashable,
        quiescence::Quiescenceable
    }
};

use chess;

const SQUARES: [chess::Square; 64] = [
    chess::Square::A8, chess::Square::B8, chess::Square::C8, chess::Square::D8, chess::Square::E8, chess::Square::F8, chess::Square::G8, chess::Square::H8,
    chess::Square::A7, chess::Square::B7, chess::Square::C7, chess::Square::D7, chess::Square::E7, chess::Square::F7, chess::Square::G7, chess::Square::H7,
    chess::Square::A6, chess::Square::B6, chess::Square::C6, chess::Square::D6, chess::Square::E6, chess::Square::F6, chess::Square::G6, chess::Square::H6,
    chess::Square::A5, chess::Square::B5, chess::Square::C5, chess::Square::D5, chess::Square::E5, chess::Square::F5, chess::Square::G5, chess::Square::H5,
    chess::Square::A4, chess::Square::B4, chess::Square::C4, chess::Square::D4, chess::Square::E4, chess::Square::F4, chess::Square::G4, chess::Square::H4,
    chess::Square::A3, chess::Square::B3, chess::Square::C3, chess::Square::D3, chess::Square::E3, chess::Square::F3, chess::Square::G3, chess::Square::H3,
    chess::Square::A2, chess::Square::B2, chess::Square::C2, chess::Square::D2, chess::Square::E2, chess::Square::F2, chess::Square::G2, chess::Square::H2,
    chess::Square::A1, chess::Square::B1, chess::Square::C1, chess::Square::D1, chess::Square::E1, chess::Square::F1, chess::Square::G1, chess::Square::H1
];

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WrappedMove {
    pub r#move: chess::ChessMove
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UndoInformation {
    board: chess::Board,
    material_strength: i32
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WrappedBoard {
    pub board: chess::Board,
    material_strength: i32,
    history: Vec<UndoInformation>
}

impl WrappedBoard {

    fn get_piece_value(piece: chess::Piece, color: chess::Color) -> i32 {

        // get value of piece
        let piece_value = match piece {
            chess::Piece::Pawn   => 100,
            chess::Piece::Knight => 300,
            chess::Piece::Bishop => 300,
            chess::Piece::Rook   => 500,
            chess::Piece::Queen  => 900,
            chess::Piece::King   => 0
        };

        // return value with a possible sign flip
        return match color {
            chess::Color::White => piece_value,
            chess::Color::Black => -piece_value
        };

    }

    fn initialize_material_stength(&mut self) {

        // reset value
        self.material_strength = 0;

        // loop through all squares and record any material
        for square in SQUARES {

            // get piece on square if there is any
            let piece = match self.board.piece_on(square) {
                Option::None               => continue,
                Option::Some(piece) => piece
            };

            // get color of piece
            let color = match self.board.color_on(square) {
                Option::None               => panic!("Piece on given square had no color!"),
                Option::Some(color) => color
            };

            // record piece value
            self.material_strength += WrappedBoard::get_piece_value(piece, color)

        }

    }

    fn map_piece(piece: chess::Piece) -> Piece {
        return match piece {
            chess::Piece::Pawn   => Piece::Pawn,
            chess::Piece::Knight => Piece::Knight,
            chess::Piece::Bishop => Piece::Bishop,
            chess::Piece::Rook   => Piece::Rook,
            chess::Piece::Queen  => Piece::Queen,
            chess::Piece::King   => Piece::King
        };
    }

    fn file_from_own_file(file: File) -> chess::File {
        return match file {
            File::A => chess::File::A,
            File::B => chess::File::B,
            File::C => chess::File::C,
            File::D => chess::File::D,
            File::E => chess::File::E,
            File::F => chess::File::F,
            File::G => chess::File::G,
            File::H => chess::File::H
        }
    }

    fn own_file_from_file(file: chess::File) -> File {
        return match file {
            chess::File::A => File::A,
            chess::File::B => File::B,
            chess::File::C => File::C,
            chess::File::D => File::D,
            chess::File::E => File::E,
            chess::File::F => File::F,
            chess::File::G => File::G,
            chess::File::H => File::H
        }
    }

    fn rank_from_own_rank(rank: Rank) -> chess::Rank {
        return match rank {
            Rank::One   => chess::Rank::First,
            Rank::Two   => chess::Rank::Second,
            Rank::Three => chess::Rank::Third,
            Rank::Four  => chess::Rank::Fourth,
            Rank::Five  => chess::Rank::Fifth,
            Rank::Six   => chess::Rank::Sixth,
            Rank::Seven => chess::Rank::Seventh,
            Rank::Eight => chess::Rank::Eighth
        }
    }

    fn own_rank_from_rank(rank: chess::Rank) -> Rank {
        return match rank {
            chess::Rank::First   => Rank::One,
            chess::Rank::Second  => Rank::Two,
            chess::Rank::Third   => Rank::Three,
            chess::Rank::Fourth  => Rank::Four,
            chess::Rank::Fifth   => Rank::Five,
            chess::Rank::Sixth   => Rank::Six,
            chess::Rank::Seventh => Rank::Seven,
            chess::Rank::Eighth  => Rank::Eight
        }
    }

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
       self.initialize_material_stength();
       self.history = Vec::new();
    }

    fn put_into_fen(&mut self, fen: &str) {
        self.board = chess::Board::from_str(fen).expect("Invalid FEN!");
        self.initialize_material_stength();
        self.history = Vec::new();
    }

    fn make_move(&mut self, r#move: WrappedMove) {

        // clone current board and material strength, make undo information
        let board_clone = self.board.clone();
        let material_strength_clone = self.material_strength.clone();
        let undo_information = UndoInformation {
            board: board_clone,
            material_strength: material_strength_clone
        };

        // adjust material strength if the move captures
        let destination = r#move.r#move.get_dest();
        match self.board.piece_on(destination) {
            Option::None => {

                // check for en passant
                match self.board.en_passant() {
                    Option::None                    => {},
                    Option::Some(ep_square) => {

                        // if the destination is the en passant square, then a capture happened
                        if destination == ep_square {
                            let color = self.board.side_to_move();
                            let pawn_value = WrappedBoard::get_piece_value(chess::Piece::Pawn, !color);
                            self.material_strength -= pawn_value;
                        }
                    }
                }

            },
            Option::Some(victim) => {
                
                // get value of victim
                let color = self.board.side_to_move();
                let victim_value = WrappedBoard::get_piece_value(victim, !color);
                
                // adjust material
                self.material_strength -= victim_value;
            }
        }

        // adjust material strength if the move promotes
        match r#move.r#move.get_promotion() {
            Option::None               => {}
            Option::Some(piece) => {
                let color = self.board.side_to_move();
                let piece_value = WrappedBoard::get_piece_value(piece, color);
                let pawn_value = WrappedBoard::get_piece_value(chess::Piece::Pawn, color);
                self.material_strength += piece_value - pawn_value
            }
        }

        // make move on original board
        board_clone.make_move(r#move.r#move, &mut self.board);

        // remember old board
        self.history.push(undo_information);

    }

    fn visualize(&self) {
        vis_board(self);
    }

}

impl Value for i32 {
    const MIN: Self = i32::MIN + 1;  // we have to add 1 because -i32::MIN does not exist!
    const ZERO: Self = 0;
    const MATE_TRHESHOLD: Self = 16_384;
    const MATE: Self = 30_000;
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

        // get information from history needed for undoing move
        let undo_information = self.history.pop().expect("Cannot unmake moves on an empty history!");
        
        // undo move
        self.material_strength = undo_information.material_strength;
        self.board = undo_information.board;

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
            chess::BoardStatus::Checkmate => Status::Checkmate
        };
    }

    fn evaluate(&self) -> i32 {

        // offset to introduce some noise to thhe evaluation
        let hash = self.board.get_hash();
        let last_bit_of_hash = hash & 1;
        let some_other_bits = hash & 0b1110;
        let sign = if last_bit_of_hash == 1 {1} else {-1};
        let body = some_other_bits as i32;
        let offset = sign * body;

        // check if we are in a stale- or checkmate
        return match self.board.status() {
            chess::BoardStatus::Ongoing   => self.material_strength + offset,
            chess::BoardStatus::Stalemate => 0,
            chess::BoardStatus::Checkmate => -i32::MATE
        }

    }

}


impl MVVLVAScorer<WrappedMove> for WrappedBoard {
    fn is_capture(&self, r#move: WrappedMove) -> bool {

        // get destination of move and check if there is a piece, if not check if it is the ep square
        let dest = r#move.r#move.get_dest();

        match self.board.piece_on(dest) {
            Option::None    => {

                // no piece on dest, but is it the ep-square?
                match self.board.en_passant() {
                    Option::None                    => {return false;},
                    Option::Some(ep_square) => {

                        let correct_square = ep_square == dest;

                        let source = r#move.r#move.get_source();
                        let piece = self.board.piece_on(source).expect("No piece on source of move!");
                        let correct_piece = piece == chess::Piece::Pawn;

                        return correct_square && correct_piece;
                    }
                }
            },
            Option::Some(_) => {return true;}
        }

    }

    fn get_victim_of(&self, r#move: WrappedMove) -> Piece {
        
        // get destination of move and check if there is a piece, if not then it is the en passant square
        let dest = r#move.r#move.get_dest();

        return match self.board.piece_on(dest) {
            Option::None               => Piece::Pawn,
            Option::Some(piece) => Self::map_piece(piece)
        };

    }

    fn get_attacker_of(&self, r#move: WrappedMove) -> Piece {

        // get destination of move and check if there is a piece, if not then it is the en passant square
        let source = r#move.r#move.get_source();

        return Self::map_piece(self.board.piece_on(source).expect("No piece on source of move!"));

    }

}


static RANK_CHARS: [char; 8]  = ['1', '2', '3', '4', '5', '6', '7', '8'];
static FILE_CHARS: [char; 8]  = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
static EMPTY_SQUARE_CHAR: char = '.';
static PIECE_CHARS: [char; 12] = ['P','N','B','R','Q','K', 'p','n','b','r','q','k'];  //['♙','♘','♗','♖','♕','♔', '♟','♞','♝','♜','♛','♚'];
static PLAYER_CHARS: [char; 2]  = ['w', 'b'];


pub trait Visualizable {
    fn whites_turn(&self) -> bool;
    fn piece_at(&self, square: Square) -> Option<ColoredPiece>;
    fn en_passant_square(&self) -> Option<Square>;
    fn zobrist_hash(&self) -> Option<u64>;
    fn polykey(&self) -> Option<u64>;
}

fn get_repr_for_square(square: Square) -> String {
    let (file, rank) = square;
    return format!("{}{}", FILE_CHARS[file as usize], RANK_CHARS[rank as usize]);
}

pub fn vis_board<B: Visualizable>(board: &B) {

    let mut repr: String = String::new();

    repr += "\n    ";
    for file in FILES {
        repr += &format!("{} ", FILE_CHARS[file as usize]);
    }

    repr += "\n    ";
    for _ in 0..7 {
        repr += "__";
    }
    repr += "_";

    repr += "\n";

    for rank in RANKS.into_iter().rev() {

        repr += &format!("{}  |", RANK_CHARS[rank as usize]);

        for file in FILES {

            // get char for piece on that square (no pice is explicitly covered)
            let maybe_piece_char = match board.piece_at((file, rank)) {
                Option::None                      => EMPTY_SQUARE_CHAR,
                Option::Some(piece) => PIECE_CHARS[piece as usize]
            };

            repr += &format!("{maybe_piece_char} ");
        }
        repr += "\n";
    }
    
    repr += "\n";

    // show which player's turn it is
    let player_idx = if board.whites_turn() {0} else {1};
    repr += &format!("side to move: {}", PLAYER_CHARS[player_idx]);
    
    // show en passant square (if any)
    let maybe_square_repr = match board.en_passant_square() {
        Option::None                       => "-",
        Option::Some(square) => &get_repr_for_square(square)
    };
    repr += &format!("\nen passent on: {maybe_square_repr}");

    // // show castling permissions
    // let K_perm;
    // let Q_perm;
    // let k_perm;
    // let q_perm;
    // // if (board.castle_perm & CastlingRights::WkR as u8) != 0 {'K'} else {'-'},
    // // if (board.castle_perm & CastlingRights::WqR as u8) != 0 {'Q'} else {'-'},
    // // if (board.castle_perm & CastlingRights::BkR as u8) != 0 {'k'} else {'-'},
    // // if (board.castle_perm & CastlingRights::BqR as u8) != 0 {'q'} else {'-'},
    // repr += &format!("\ncastle permissions: {K_perm}{Q_perm}{k_perm}{q_perm}");

    // maybe show Zobrist hash
    match board.zobrist_hash() {
        Option::None            => {},
        Option::Some(hash) => {repr += &format!("\nboard key: {hash:x?}");}
    }
    
    // maybe show polykey
    match board.polykey() {
        Option::None           => {},
        Option::Some(key) => {repr += &format!("\npoly key: {key:x?}");}
    }

    repr += &format!("\n\n");

    println!("{repr}");

}


impl Visualizable for WrappedBoard {

    fn whites_turn(&self) -> bool {
        return Searchable::whites_turn(self);
    }

    fn piece_at(&self, square: super::Square) -> Option<super::ColoredPiece> {

        let (file, rank) = square;
        let file = Self::file_from_own_file(file);
        let rank = Self::rank_from_own_rank(rank);
        let square = chess::Square::make_square(rank, file);

        let piece = match self.board.piece_on(square) {
            Option::None               => return Option::None,
            Option::Some(piece) => piece
        };
        
        let color = self.board.color_on(square).unwrap();

        let parsed_piece = match (piece, color) {
            (chess::Piece::Pawn,   chess::Color::White) => super::ColoredPiece::WhitePawn,
            (chess::Piece::Knight, chess::Color::White) => super::ColoredPiece::WhiteKnight,
            (chess::Piece::Bishop, chess::Color::White) => super::ColoredPiece::WhiteBishop,
            (chess::Piece::Rook,   chess::Color::White) => super::ColoredPiece::WhiteRook,
            (chess::Piece::Queen,  chess::Color::White) => super::ColoredPiece::WhiteQueen,
            (chess::Piece::King,   chess::Color::White) => super::ColoredPiece::WhiteKing,
            (chess::Piece::Pawn,   chess::Color::Black) => super::ColoredPiece::BlackPawn,
            (chess::Piece::Knight, chess::Color::Black) => super::ColoredPiece::BlackKnight,
            (chess::Piece::Bishop, chess::Color::Black) => super::ColoredPiece::BlackBishop,
            (chess::Piece::Rook,   chess::Color::Black) => super::ColoredPiece::BlackRook,
            (chess::Piece::Queen,  chess::Color::Black) => super::ColoredPiece::BlackQueen,
            (chess::Piece::King,   chess::Color::Black) => super::ColoredPiece::BlackKing,
        };

        return Option::Some(parsed_piece);

    }

    fn en_passant_square(&self) -> Option<super::Square> {
        return match self.board.en_passant() {
            Option::None                 => Option::None,
            Option::Some(square) => {
                let file = Self::own_file_from_file(square.get_file());
                let rank = Self::own_rank_from_rank(square.get_rank());
                Option::Some((file, rank))
            }
        }
    }
    fn zobrist_hash(&self) -> Option<u64> {
        return Option::None;
    }
    fn polykey(&self) -> Option<u64> {
        return Option::None;
    }
}

impl Hashable<i32, WrappedMove> for WrappedBoard {
    fn get_zobrist_hash(&self) -> usize {
        return self.board.get_hash() as usize;
    }
}

impl Quiescenceable<i32, WrappedMove> for WrappedBoard {
    fn loud_moves(&self) -> Vec<WrappedMove> {
        return self.get_legal_moves()
            .into_iter()
            .filter(|&r#move| MVVLVAScorer::is_capture(self, r#move))
            .collect();
    }
}
