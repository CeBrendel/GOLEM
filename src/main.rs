
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod board;
pub mod search;
pub mod uci;

use board::{
    dummy_board::{DummyMove, DummyBoard},
    wrapped_board::{WrappedMove, WrappedBoard}
};

use uci::uci_loop;
use search::{
    dummy_search::dummy_search,
    minimax::iterative_deepening
};


enum KindOfBoard {
    Dummy,
    Wrapped
}

enum KindOfSearch {
    DummySearch,
    Minimax
}

const BOARD: KindOfBoard = KindOfBoard::Wrapped;
const SEARCH: KindOfSearch = KindOfSearch::Minimax;

pub fn main() {

    // start UCI loop
    match (BOARD, SEARCH) {
        (KindOfBoard::Dummy, KindOfSearch::DummySearch)   => uci_loop::<i32, DummyMove, DummyBoard>(dummy_search),
        (KindOfBoard::Dummy, KindOfSearch::Minimax)       => panic!(),
        (KindOfBoard::Wrapped, KindOfSearch::DummySearch) => uci_loop::<i32, WrappedMove, WrappedBoard>(dummy_search),
        (KindOfBoard::Wrapped, KindOfSearch::Minimax)     => uci_loop::<i32, WrappedMove, WrappedBoard>(iterative_deepening),
    }
    
}