
use crate::board::traits::{Move, Board};
use crate::search::SearchInfo;
use crate::uci::SearchInstruction;
use crate::uci::channeling::{Sender, Receiver};

use std::thread;

pub fn dummy_search<M: Move, B: Board<M>>(
    _locked_board: &mut B,
    _search_instruction: SearchInstruction,
    stop_rx: &Receiver<bool>,
    search_info_tx: &Sender<SearchInfo<M>>
) {
    
    let mut pv: Vec<M> = Vec::new();
    let mut bestmove: M;
    let mut counter: usize = 2;
    let wait_duration = std::time::Duration::from_millis(3000);
    let mut stopped: bool = false;
    loop {

        // pretend that this loop takes some time
        let mut inner_counter = 0;
        loop {

            // check if search should be stopped
            match stop_rx.recv() {
                Option::None    => (),
                Option::Some(b) => {println!("  >> search-thread: search stopped!"); stopped = true; break}
            }


            // wait a bit
            thread::sleep(wait_duration/100);

            // break inner loop after 100 waits
            inner_counter += 1;
            if inner_counter >= 100 {
                break;
            }
        }

        // if we recorded a stop signal from the inner loop, break this outer loop
        if stopped {break}

        // make fake search info
        let bestmove_str = String::from("e2e") + counter.to_string().as_str();
        println!("  >> search-thread: bestmove {} at depth {}", bestmove_str, counter);
        counter += 1;
        let bestmove = M::from_algebraic(&bestmove_str);
        pv.push(bestmove);
        let search_info = SearchInfo {
            r#move: Option::Some(M::from_algebraic(&bestmove_str)),
            principal_variation_line: Option::Some(pv.clone()),
        };

        // send current search info
        search_info_tx.send(search_info);

    }
}