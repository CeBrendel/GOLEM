
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod board;
pub mod search;
pub mod uci;

use board::{
    wrapped_board::{WrappedMove, WrappedBoard}
};

use uci::uci_loop;
use search::{
    dummy_search::dummy_search,
    iterative_deepening::iterative_deepening,
    minimax::minimax,
    alpha_beta::alpha_beta
};


enum KindOfBoard {
    Wrapped
}

enum KindOfSearch {
    DummySearch,
    Minimax,
    AlphaBeta
}

const BOARD: KindOfBoard = KindOfBoard::Wrapped;
const SEARCH: KindOfSearch = KindOfSearch::AlphaBeta;

pub fn main() {

    // start UCI loop
    match (BOARD, SEARCH) {
        (KindOfBoard::Wrapped, KindOfSearch::DummySearch) => uci_loop::<i32, WrappedMove, WrappedBoard>(dummy_search),
        (KindOfBoard::Wrapped, KindOfSearch::Minimax)     => uci_loop::<i32, WrappedMove, WrappedBoard>(iterative_deepening(minimax)),
        (KindOfBoard::Wrapped, KindOfSearch::AlphaBeta)   => uci_loop::<i32, WrappedMove, WrappedBoard>(iterative_deepening(alpha_beta)),
    }
    
}