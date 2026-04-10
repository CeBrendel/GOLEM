
pub mod wrapped_board;

use std::fmt;

#[derive(Clone, Copy)]
pub enum Piece {
    Pawn = 0,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

pub enum ColoredPiece {
    WhitePawn = 0,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
}

#[derive(Clone, Copy)]
pub enum Rank {
    One=0, Two, Three, Four, Five, Six, Seven, Eight
}

static RANKS: [Rank; 8] = [Rank::One, Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven, Rank::Eight];

#[derive(Clone, Copy)]
pub enum File {
    A=0, B, C, D, E, F, G, H
}

static FILES: [File; 8] = [File::A, File::B, File::C, File::D, File::E, File::F, File::G, File::H];

pub type Square = (File, Rank);


pub trait Move: fmt::Debug + Clone + Copy + Default + PartialEq + Eq + Send + Sync + 'static {
    fn as_string(&self) -> String;
    fn from_algebraic(s: &str) -> Self;
}

pub trait Board<M: Move + 'static>: Send + 'static {
    fn put_into_startpos(&mut self);
    fn put_into_fen(&mut self, fen: &str);
    fn make_move(&mut self, r#move: M);
    fn visualize(&self);
}
