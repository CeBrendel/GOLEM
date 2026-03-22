
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod board;
pub mod channeling;
pub mod search;
pub mod uci;

use board::dummy_board::{DummyMove, DummyBoard};
use uci::uci_loop;
use search::dummy_search::dummy_search;

pub fn main() {
    
    // start UCI loop using DummyBoard, DummyMove and the dummy_search
    uci_loop::<DummyMove, DummyBoard>(dummy_search);

}