
use crate::board::{Move, Board};
use crate::search::{SearchInstruction, SearchInfo, SearchResult};
use crate::uci::Response;

use std::sync::mpsc::{Sender, Receiver};
use std::thread;

pub fn dummy_search<M: Move, B: Board<M>>(
    _locked_board: &mut B,
    search_instruction: SearchInstruction,
    stop_rx: &Receiver<()>,
    write_request_tx: &Sender<Response<M>>
) -> SearchResult<M> {
    
    let mut pv: Vec<M> = Vec::new();
    let mut bestmove: M = M::from_algebraic("e2e4");
    let mut counter = 1;
    let wait_duration = std::time::Duration::from_millis(200);
    let mut stopped: bool = false;

    loop {

        // pretend that this loop takes some time
        let mut inner_counter = 0;
        loop {

            // check if search should be stopped
            match stop_rx.try_recv() {
                Err(_) => (),  // no stop signal was supplied
                Ok(_)  => {stopped = true; break}
            }


            // wait a bit
            thread::sleep(wait_duration/100 * counter);

            // break inner loop after 100 waits
            inner_counter += 1;
            if inner_counter >= 100 {
                break;
            }
        }

        // if we recorded a stop signal from the inner loop, break this outer loop
        if stopped {break}

        // make fake search info
        let bestmove_str = String::from("e2e") + (counter + 1).to_string().as_str();
        bestmove = M::from_algebraic(&bestmove_str);
        pv.push(bestmove.clone());
        let search_info = SearchInfo {
            score: Option::None,
            depth: Option::Some(counter as usize),
            nodes: Option::None,
            time: Option::None,
            principal_variation_line: Option::Some(pv.clone()),
        };
        counter += 1;
        
        // send current search info
        write_request_tx.send(Response::Info(search_info)).expect("Sending of search info failed!");
        
        // maybe break search at the right depth
        match search_instruction.depth {
            Option::None               => (),
            Option::Some(depth) => if counter >= ((depth + 1) as u32) {break}
        }

    }

    return SearchResult {bestmove};

}