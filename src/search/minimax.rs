
use crate::{
    board::{Board, Move},
    search::{
        evaluate_wrt_root,
        SearchInfo,
        SearchInstruction,
        SearchResult,
        generics::{Bool, False, Maximizer, Minimizer, Optimizer, True},
        traits::{Status, Searchable, Value}
    }, uci::Response
};

use std::{
    sync::mpsc::{Receiver, Sender}, time::{Duration, SystemTime}
};

pub fn minimax<V: Value, M: Move, B: Board<M> + Searchable<M, V>>(
    board: &mut B,
    depth: u8,
    stop_rx: &Receiver<()>,
    search_info: &mut SearchInfo<M, V>
) -> Result<(Option<M>, V), ()> {

    // will be called with the correct (const) arguments to accomplish the search
    fn inner_minimax<
        O: Optimizer,  // whether the call should maximizer or minimize the evaluation
        IsEntry: Bool,  // whether this is the entrypoint of the recursion (only then we write to the move buffer)
        V: Value,
        M: Move,
        B: Board<M> + Searchable<M, V>
    >(
        board: &mut B,
        depth_left: u8,
        distance_to_root: u8,
        stop_rx: &Receiver<()>,
        search_info: &mut SearchInfo<M, V>
    ) -> V {

        // check if search should be stopped
        // TODO: Factor this modulus out as a configurable
        if search_info.nodes_searched % 4096 == 0 {
            match stop_rx.try_recv() {
                Err(_) => {},
                Ok(_)   => {search_info.was_stopped = true;}
            }
        }

        // check if search was stopped
        if search_info.was_stopped {
            return V::ZERO;
        }

        // increment nodes counter
        search_info.nodes_searched += 1;

        // base case of the recursion
        // TODO: If we are in check, we should increase depth anyways!
        if depth_left == 0 {
            return evaluate_wrt_root(board, distance_to_root);
        }

        // get legal moves in current position
        let legal_moves = board.get_legal_moves();

        // if there are no legal moves to make, simply return the evaluation of the board
        if legal_moves.len() == 0 {
            return evaluate_wrt_root(board, distance_to_root);
        }

        // iterate over all moves and evaluate the resulting position via a recursive call
        let mut optimal_value: V = if O::IS_MAXIMIZER {V::MIN} else {V::MAX};
        for r#move in legal_moves {

            // make move
            board.make_move(r#move.clone());  // TODO: MAybe remove this clone by enforcing M: Copy?

            // recursive call
            let child_evaluation = inner_minimax::<
                O::Enemy,  // start a new search from the enemies point of view
                False,  // this search will never be the entrypoint of the main search
                V, M, B
            >(
                board,
                depth_left - 1,  // search one depth less
                distance_to_root + 1, // search one depth durther from the root
                stop_rx,
                search_info
            );

            // unmake move to restore previous position
            board.unmake_move();

            // compare values to decide if we have found a better move
            let new_move_is_better = if O::IS_MAXIMIZER {child_evaluation > optimal_value} else {child_evaluation < optimal_value};
            if new_move_is_better {

                // remember better evaluation
                optimal_value = child_evaluation;

                // if we are in the entrypoint to the main search, also remember the move and it evaluation
                if IsEntry::VALUE {
                    search_info.evaluation = Option::Some(optimal_value);
                    search_info.bestmove = Option::Some(r#move);
                }
            }

        }

        // return evaluation of the best move found
        return optimal_value;

    }

    // manual dispatch into the right implementation of inner_minimax
    let evaluation = match board.whites_turn() {
        true  => inner_minimax::<Maximizer, True, V, M, B>(board, depth, 0, stop_rx, search_info),
        false => inner_minimax::<Minimizer, True, V, M, B>(board, depth, 0, stop_rx, search_info)
    };

    // if search was stopped early, return an Err
    if search_info.was_stopped {
        return Err(());
    }

    // return best move and its evaluation
    return Ok((search_info.bestmove.clone(), evaluation));
    
}

pub fn alpha_beta<V: Value, M: Move, B: Board<M> + Searchable<M, V>>(
    board: &mut B,
    depth: u8,
    stop_rx: &Receiver<()>,
    search_info: &mut SearchInfo<M, V>
) -> Result<(Option<M>, V), ()> {

    // will be called with the correct (const) arguments to accomplish the search
    fn inner_alpha_beta<
        O: Optimizer,  // whether the call should maximizer or minimize the evaluation
        IsEntry: Bool,  // whether this is the entrypoint of the recursion (only then we write to the move buffer)
        V: Value,
        M: Move,
        B: Board<M> + Searchable<M, V>
    >(
        board: &mut B,
        depth_left: u8,
        distance_to_root: u8,
        mut alpha: V,
        mut beta: V,
        stop_rx: &Receiver<()>,
        search_info: &mut SearchInfo<M, V>
    ) -> V {

        // check if search should be stopped
        // TODO: Factor this modulus out as a configurable
        if search_info.nodes_searched % 4096 == 0 {
            match stop_rx.try_recv() {
                Err(_) => {},
                Ok(_)   => {search_info.was_stopped = true;}
            }
        }

        // check if search was stopped
        if search_info.was_stopped {
            return V::ZERO;
        }

        // increment nodes counter
        search_info.nodes_searched += 1;

        // base case of the recursion
        // TODO: If we are in check, we should increase depth anyways!
        if depth_left == 0 {
            return evaluate_wrt_root(board, distance_to_root);
        }

        // get legal moves in current position
        let legal_moves = board.get_legal_moves();

        // if there are no legal moves to make, simply return the evaluation of the board
        if legal_moves.len() == 0 {
            return evaluate_wrt_root(board, distance_to_root);
        }

        // iterate over all moves and evaluate the resulting position via a recursive call
        let mut optimal_value: V = if O::IS_MAXIMIZER {V::MIN} else {V::MAX};
        for r#move in legal_moves {

            // make move
            board.make_move(r#move.clone());  // TODO: MAybe remove this clone by enforcing M: Copy?

            // recursive call
            let child_evaluation = inner_alpha_beta::<
                O::Enemy,  // start a new search from the enemies point of view
                False,  // this search will never be the entrypoint of the main search
                V, M, B
            >(
                board,
                depth_left - 1,  // search one depth less
                distance_to_root + 1, // search one depth durther from the root
                alpha,
                beta,
                stop_rx,
                search_info
            );

            // unmake move to restore previous position
            board.unmake_move();

            // compare values to decide if we have found a better move
            let new_move_is_better = if O::IS_MAXIMIZER {child_evaluation > optimal_value} else {child_evaluation < optimal_value};
            if new_move_is_better {

                // remember better evaluation
                optimal_value = child_evaluation;

                // if we are in the entrypoint to the main search, also remember the move and it evaluation
                if IsEntry::VALUE {
                    search_info.evaluation = Option::Some(optimal_value);
                    search_info.bestmove = Option::Some(r#move);
                }

                // adjust alpha/beta
                if O::IS_MAXIMIZER {
                    if child_evaluation > alpha {
                        alpha = child_evaluation;
                    }
                } else {
                    if child_evaluation < beta {
                        beta = child_evaluation;
                    }
                }

                // check for cutoff
                let cutoff = if O::IS_MAXIMIZER {
                    if child_evaluation >= beta {
                        break;
                    }
                } else {
                    if alpha >= child_evaluation {
                        break;
                    }
                };
                
            }

        }

        // return evaluation of the best move found
        return optimal_value;

    }

    // manual dispatch into the right implementation of inner_minimax
    let evaluation = match board.whites_turn() {
        true  => inner_alpha_beta::<Maximizer, True, V, M, B>(board, depth, 0, V::MIN, V::MAX, stop_rx, search_info),
        false => inner_alpha_beta::<Minimizer, True, V, M, B>(board, depth, 0, V::MIN, V::MAX, stop_rx, search_info)
    };

    // if search was stopped early, return an Err
    if search_info.was_stopped {
        return Err(());
    }

    // return best move and its evaluation
    return Ok((search_info.bestmove.clone(), evaluation));
    
}


pub fn iterative_deepening<V: Value, M: Move, B: Board<M> + Searchable<M, V>>(
    board: &mut B,
    search_instruction: SearchInstruction,
    stop_rx: &Receiver<()>,
    write_request_tx: &Sender<Response<M, V>>,
    // search: FixedDepthSearch<V, M, B>
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
        let (bestmove, evaluation) = match minimax(board, depth, stop_rx, &mut search_info) {
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