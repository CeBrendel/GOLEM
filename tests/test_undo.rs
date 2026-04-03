
mod read_fens;

use read_fens::read_fens;

use std::sync::mpsc;

use golem::{
    board::{Board, wrapped_board::WrappedBoard},
    search::{SearchInfo, minimax::minimax}
};


#[test]
fn test_undo() {

    // read fens
    let fens = read_fens();
    let n_fens = fens.len();

    // loop through (some) fens
    for (idx, fen) in fens.iter().enumerate() {

        // occasionally print progress
        if (idx + 1) % 500 == 0 {
            println!("Handling FEN {} out of {n_fens}.", idx + 1)
        }

        // make board and put it into position
        let mut board = WrappedBoard::default();
        board.put_into_fen(&fen);

        // record state
        let record = board.clone();

        //, do a shallow search (lots of making and unmaking of moves)
        let (_, stop_rx) = mpsc::channel::<()>();
        let mut search_info = SearchInfo::default();
        let _ = minimax(&mut board, 2, &stop_rx, &mut search_info);

        // assert that board agrees with record
        assert_eq!(board, record);

    }
}