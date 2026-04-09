
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
