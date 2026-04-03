
use crate::{
    board::{Board, Move},
    search::{
        evaluate_wrt_root,
        SearchInfo,
        SearchInstruction,
        SearchResult,
        Search,
        generics::{Bool, False, Maximizer, Minimizer, Optimizer, True},
        traits::{Status, Searchable, Value}
    }, uci::Response
};

use std::{
    sync::mpsc::{Receiver, Sender}, time::{Duration, SystemTime}
};

pub type IterableSearch<V, M, B> = fn(&mut B, u8, &Receiver<()>, &mut SearchInfo<M, V>) -> Result<(Option<M>, V), ()>;

fn inner_iterative_deepening<V: Value, M: Move, B: Board<M> + Searchable<M, V>>(
    board: &mut B,
    search_instruction: SearchInstruction,
    stop_rx: &Receiver<()>,
    write_request_tx: &Sender<Response<M, V>>,
    iterable_search: IterableSearch<V, M, B>
) -> SearchResult<M> {

    // get maximum depth (either infinite or some fixed number)
    let max_depth = match (search_instruction.infinite, search_instruction.depth) {
        (true, _)                    => u8::MAX,
        (false, Option::Some(d)) => d,
        _                            => panic!("Received neither a depth nor the infinite keyword!")
    };

    // loop through various depths
    let mut maybe_bestmove: Option<M> = Option::None;
    for depth in 1..max_depth {

        // construct new search info, that gets modified in the search
        let mut search_info = SearchInfo::default();

        // do search to the current depth
        let (bestmove, evaluation) = match iterable_search(board, depth, stop_rx, &mut search_info) {
            Err(_)                     => break,  // is the search was stopped and returned an Err, break the loop
            Ok(result) => result
        };

        println!("SearchInfo after depth {}:\n{:?}", depth, search_info);

        // remember best move
        maybe_bestmove = bestmove;
        search_info.evaluation = Option::Some(evaluation);

        // send search info
        write_request_tx.send(Response::Info(search_info)).expect("Sending search info failed!");

    }

    return SearchResult{
        bestmove: maybe_bestmove.expect("Iterative deepening could not identifiy a move!")
    };

}


macro_rules! search {
    (<$V: ident, $B: ident, $M: ident>) => {
        impl 'static + Sync + Send + Fn(&mut B, SearchInstruction, &Receiver<()>, &Sender<Response<M, V>>) -> SearchResult<M>
    };
}

pub(crate) use search;


pub fn iterative_deepening<V: Value, M: Move, B: Board<M> + Searchable<M, V>>(
    iterable_search: IterableSearch<V, M, B>
) -> search!(<V, M, B>) {
    
    let partial_iterative_deepening = move |
        board: &mut B,
        search_instruction: SearchInstruction,
        stop_rx: &Receiver<()>,
        write_request_tx: &Sender<Response<M, V>>,
    | {
        return inner_iterative_deepening(board, search_instruction, stop_rx, write_request_tx, iterable_search);
    };

    return partial_iterative_deepening;
}
