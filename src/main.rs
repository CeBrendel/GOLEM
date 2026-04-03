
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod board;
pub mod search;
pub mod uci;

use crate::{
    board::wrapped_board::{WrappedMove, WrappedBoard},
    uci::uci_loop,
    search::{
        iterative_deepening::iterative_deepening,
        minimax::minimax,
        alpha_beta::alpha_beta
    }
};

enum KindOfBoard {
    Wrapped
}

enum KindOfSearch {
    Minimax,
    AlphaBeta
}

const BOARD: KindOfBoard = KindOfBoard::Wrapped;
const SEARCH: KindOfSearch = KindOfSearch::AlphaBeta;

pub fn main() {

    // start UCI loop
    match (BOARD, SEARCH) {
        (KindOfBoard::Wrapped, KindOfSearch::Minimax)   => uci_loop::<i32, WrappedMove, WrappedBoard>(iterative_deepening(minimax)),
        (KindOfBoard::Wrapped, KindOfSearch::AlphaBeta) => uci_loop::<i32, WrappedMove, WrappedBoard>(iterative_deepening(alpha_beta)),
    }
    
}