
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod uci;
pub mod board;
pub mod search;

use uci::uci_loop;
use board::dummy_board::{DummyMove, DummyBoard};
use search::dummy_search::dummy_search;

pub fn main() {
    
    // start UCI loop using DummyBoard, DummyMove and the dummy_search
    uci_loop::<DummyMove, DummyBoard>(dummy_search).unwrap();

}