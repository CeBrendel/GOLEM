
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod uci;
pub mod board;

use uci::parse_uci_command;
use board::dummy_board::DummyBoard;

/*
TODO:
    - Default for Board
    - assert that all in- or outgoing command end with "\n"
*/

pub fn main() -> std::io::Result<()> {
    
    let stdin = std::io::stdin();
    let board: &mut DummyBoard = &mut DummyBoard{fen_like_base_position: String::from("empty"), pushed_moves: Vec::new()};

    loop {

        // read command from stdin
        let command = &mut String::new();
        stdin.read_line(command)?;

        // parse command
        parse_uci_command(command, board);

    }
}