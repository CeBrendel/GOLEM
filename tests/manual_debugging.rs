
use golem::{
    board::{Board, wrapped_board::WrappedBoard},
    search::{
        SearchInfo, negamax::negamax, transposition_table::TransTable
    }
};

// a history of awefulness
// static FEN_OF_INTEREST: &'static str = "r4k1r/2pQ1pp1/p4q1p/2N3N1/1p3P2/8/PP3PPP/4R1K1 w - - 1 0";
// static EXPECTED_MOVES: &[&'static str] = &["Qe8+", "Rxe8", "Nd7+", "Kg8", "Rxe8#"];
// static FEN_OF_INTEREST: &'static str = "5r2/7p/3R4/p3pk2/1p2N2p/1P2BP2/6PK/4r3 w - - 1 0";
// static EXPECTED_MOVES: &[&'static str] = &["g4+", "h4g3", "Nxg3#"];
// static FEN_OF_INTEREST: &'static str = "6k1/5p1p/2Q1p1p1/5n1r/N7/1B3P1P/1PP3PK/4q3 b - - 0 1";
// static EXPECTED_MOVES: &[&'static str] = &["Rxh3+", "gxh3", "Qf2+", "Kh1", "Ng3#"];
static FEN_OF_INTEREST: &'static str = "5rk1/pR4pp/4p2r/2p1n2q/2P1p3/P1Q1P1P1/1P3P1P/R1B2NK1 b - - 0 1";
static EXPECTED_MOVES: &[&'static str] = &["Nf3+", "Kh1", "Qxh2+", "Nxh2", "Rxh2#"];

const ONLY_FIRST_SEARCH: bool = false;

#[test]
fn debug() {

    // make dummy receiver
    let (_, stop_rx) = std::sync::mpsc::channel::<()>();

    // make transposition table
    let mut transposition_table = TransTable::new();

    // make board from FEN
    let mut board = WrappedBoard::default();
    board.put_into_fen(FEN_OF_INTEREST);
        

    for (idx, &move_str) in EXPECTED_MOVES.iter().enumerate() {

        // parse expected move
        let expected_move = board.make_san_move(move_str);

        // clear table for next search
        transposition_table.clear();

        // get depth to which we should search
        let depth: u8 = (EXPECTED_MOVES.len() - idx) as u8;

        // do search and get move found by it
        let mut search_info = SearchInfo::default();
        negamax(&mut board, depth, &mut transposition_table, &stop_rx, &mut search_info).expect("Search failed!");
        let r#move = search_info.bestmove.expect("Search did not return a bestmove!");
        let evaluation = search_info.evaluation.expect("Search did not return an evaluation!");
        
        // convert found move and expected move to long algebraic notation
        let found_move_in_long_algebraic = r#move.r#move.to_string();
        let expected_move_in_long_algebraic = expected_move.r#move.to_string();
        
        // print results to console
        println!("\nIndex {idx}, found move: {found_move_in_long_algebraic} with evaluation {evaluation}.");
        println!("      Expected move: {expected_move_in_long_algebraic}.");
        println!("{search_info:?}");

        // maybe break the loop
        if ONLY_FIRST_SEARCH {
            break;
        }

        // make move so that next search is shallower
        board.make_move(expected_move);

    }

}
