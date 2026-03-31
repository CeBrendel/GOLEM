
use crate::board::{Move, Board};

use std::ops::{Add, Sub, Neg};


pub trait Value: Eq + PartialOrd + Clone + Copy + From<u8> + Add<Self, Output=Self> + Sub<Self, Output=Self> + Neg<Output=Self> + ToString + Send + 'static {
    const MIN: Self;
    const WHITE_IS_DEAD: Self;
    const ZERO: Self;
    const BLACK_IS_DEAD: Self;
    const MAX: Self;
}

pub enum Status {
    Ongoing,
    Stalemate,
    WhiteIsDead,
    BlackIsDead
}

pub trait Searchable<M: Move, V: Value>: Board<M> {
    fn whites_turn(&self) -> bool;
    fn unmake_move(&mut self);
    fn get_legal_moves(&self) -> Vec<M>;
    fn status(&self) -> Status;
    fn evaluate(&self) -> V;
}
