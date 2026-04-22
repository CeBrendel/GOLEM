
use std::sync::mpsc::Receiver;

use crate::{
    board::{Board, Move},
    search::{
        Value, Searchable, SearchInfo,
        generics::Bool,
        evaluate_wrt_root
    }
};

pub trait Quiescenceable<V: Value, M: Move>: Board<M> + Searchable<M, V> {
    fn loud_moves(&self) -> Vec<M>;
}

pub fn quiescence_search<
    V: Value,
    M: Move,
    B: Quiescenceable<V, M>
>(
    board: &mut B,
    depth_left: u8,
    distance_to_root: u8,
    mut alpha: V,
    beta: V,
    stop_rx: &Receiver<()>,
    search_info: &mut SearchInfo<M, V>
) -> V {

    // check if search should be stopped
    // TODO: Factor this modulus out as a configurable
    if search_info.nodes_searched % 8192 == 0 {
        match stop_rx.try_recv() {
            Err(_) => {},
            Ok(_)  => {search_info.was_stopped = true;}
        }
    }

    // check if search was stopped
    if search_info.was_stopped {
        return V::ZERO;
    }

    // increment counter
    search_info.quiescence_nodes += 1;
    search_info.nodes_searched += 1;

    // base case of recursion
    if depth_left == 0 {
        return evaluate_wrt_root(board, distance_to_root);
    }

    // use heuristic evaluation
    let static_eval = evaluate_wrt_root(board, distance_to_root);

    // stand Pat
    let mut best_value = static_eval;

    // fail high?
    if best_value >= beta {
        search_info.quiescence_fail_high_counter += 1;
        return best_value;
    }

    // maybe adjust alpha
    if best_value > alpha {
        alpha = best_value;
    }

    for r#move in board.loud_moves() {
        
        // make move
        board.make_move(r#move);

        // evaluate child
        let child_evaluation = -quiescence_search(
            board,
            depth_left - 1,
            distance_to_root + 1,
            -beta,
            -alpha,
            stop_rx,
            search_info
        );

        // unmake move
        board.unmake_move();

        // compare values to decide if we have found a better move
        if child_evaluation > best_value {

            // remember better evaluation
            best_value = child_evaluation;

            // adjust alpha
            if child_evaluation > alpha {
                alpha = child_evaluation;
            }

            // check for cutoff
            if child_evaluation >= beta {
                break;
            }
            
        }

    }

    return best_value;

}
