
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod board;
pub mod search;
pub mod uci;

use board::dummy_board::{DummyMove, DummyBoard};
use board::wrapped_board::{WrappedMove, WrappedBoard};

use uci::uci_loop;
use search::dummy_search::dummy_search;


enum BoardType {
    Dummy,
    Wrapped
}

const BOARD_TYPE: BoardType = BoardType::Wrapped;

pub fn main() {

    // start UCI loop using the Board chosen by BOARD_TYPE
    match BOARD_TYPE {
        BoardType::Dummy   => uci_loop::<DummyMove, DummyBoard>(dummy_search),
        BoardType::Wrapped => uci_loop::<WrappedMove, WrappedBoard>(dummy_search)
    }
    
}