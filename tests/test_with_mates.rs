#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

mod read_fens;

use read_fens::read_mates;

use golem::{
    board::{
        Board, Move,
        wrapped_board::{WrappedBoard, WrappedMove}
    },
    search::{
        SearchInfo, SearchInstruction, Searchable, Status, Value,
        minimax::minimax,
        alpha_beta::alpha_beta,
        negamax::negamax,
        iterative_deepening::IterableSearch,
        transposition_table::TransTable
    },
    uci::Response
};

use chess;

static MATES_IN_TWO: &str = "tests/fens/MATES_IN_TWO";
static MATES_IN_THREE: &str = "tests/fens/MATES_IN_THREE";


#[test]
fn minimax_for_mate_in_two() {
    read_and_test(MATES_IN_TWO, 3, minimax);
}

#[test]
fn alpha_beta_for_mate_in_two() {
    read_and_test(MATES_IN_TWO, 3, alpha_beta);
}

#[test]
fn negamax_for_mate_in_two() {
    read_and_test(MATES_IN_TWO, 3, negamax);
}

#[test]
fn negamax_for_mate_in_three() {
    read_and_test(MATES_IN_THREE, 5, negamax);
}


fn read_and_test(
    str_of_mates: &str,
    ply_depth_to_mate: u8,
    search: IterableSearch<i32, WrappedMove, WrappedBoard>
) {

    let fens_and_moves = read_mates(str_of_mates);
    
    // make dummy receiver
    let (_, stop_rx) = std::sync::mpsc::channel::<()>();

    // make transposition table
    let mut transposition_table = TransTable::new();

    let count = fens_and_moves.len() - 1;
    for (idx, (fen, moves)) in fens_and_moves.into_iter().enumerate() {

        println!("\nTesting {idx}/{count}, FEN {fen},\nbestmoves are {moves}");

        // make board from FEN
        let mut board = WrappedBoard::default();
        board.put_into_fen(&fen);
        
        // parse moves
        let bestmove_san = moves
            .split_whitespace()
            .nth(1)
            .expect("Didn't find bestmove!");
        let bestmove = board.make_san_move(bestmove_san);

        // clear table for next search
        transposition_table.clear();
        
        // do search and get move found by it
        let mut search_info = SearchInfo::default();
        search(&mut board, ply_depth_to_mate, &mut transposition_table, &stop_rx, &mut search_info).expect("Search failed!");
        let r#move = search_info.bestmove.expect("Search did not return a bestmove!");
        let evaluation = search_info.evaluation.expect("Search did not return an evaluation!");
        
        // convert found move and best move to long algebraic notation
        let found_move_in_long_algebraic = r#move.r#move.to_string();
        let bestmove_in_long_algebraic = bestmove.r#move.to_string();
        
        // get distance to mate
        let distance_to_root = i32::MATE - evaluation;
        
        println!("Found move: {} with evaluation {}.", found_move_in_long_algebraic, evaluation);
        println!("Depth of mate to root: {}.", distance_to_root);
        
        // do asserts
        assert_eq!(found_move_in_long_algebraic, bestmove_in_long_algebraic);
        assert_eq!(distance_to_root, ply_depth_to_mate as i32);

    }

}
