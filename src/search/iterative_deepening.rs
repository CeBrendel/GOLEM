
use std::{sync::mpsc::{Receiver, Sender}, time::Instant};

use crate::{
    board::Move,
    search::{
        SearchInstruction, SearchInfo,
        Value, Searchable, implSearch
    },
    uci::Response
};


pub type IterableSearch<V, M, B> = fn(&mut B, u8, &Receiver<()>, &mut SearchInfo<M, V>) -> Result<(), ()>;


fn inner_iterative_deepening<V: Value, M: Move, B: Searchable<M, V>>(
    board: &mut B,
    search_instruction: SearchInstruction,
    stop_rx: &Receiver<()>,
    write_request_tx: &Sender<Response<M, V>>,
    iterable_search: IterableSearch<V, M, B>
) -> SearchInfo<M, V> {

    // get maximum depth (either infinite or some fixed number)
    let max_depth = match (search_instruction.infinite, search_instruction.depth) {
        (true, _)                    => u8::MAX,
        (false, Option::Some(d)) => d,
        _                            => panic!("Received neither a depth nor the infinite keyword!")
    };

    // loop through various depths
    let mut search_info_of_last_completed_search: Option<SearchInfo<M, V>> = Option::None;
    for depth in 1..=max_depth {

        // construct new search info, that gets modified in the search
        // clone PVTable of previous search info (if there is any)
        let mut search_info = match search_info_of_last_completed_search {
            Option::None                             => SearchInfo::default(),
            Option::Some(previous) => SearchInfo::from_pv_table(&previous.pv_table)
        };

        // set depth
        search_info.depth = Option::Some(depth);

        // for timing the search
        let now = Instant::now();

        // do search to the current depth
        match iterable_search(board, depth, stop_rx, &mut search_info) {
            Err(_) => break,  // is the search was stopped and returned an Err, break the loop
            Ok(_)  => {}
        };

        // time the search and remember the result
        search_info.time = Option::Some(now.elapsed().as_millis());

        println!("SearchInfo after depth {}:\n{:?}", depth, search_info);

        // remember search info
        search_info_of_last_completed_search = Option::Some(search_info);

        // send search info
        write_request_tx.send(Response::Info(search_info)).expect("Sending search info failed!");

    }

    return search_info_of_last_completed_search.expect("Iterative deepening could not complete a search!");

}


pub fn iterative_deepening<V: Value, M: Move, B: Searchable<M, V>>(
    iterable_search: IterableSearch<V, M, B>
) -> implSearch!(<V, M, B>) {
    
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
