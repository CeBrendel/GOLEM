
use std::{collections::BTreeSet, sync::mpsc::Receiver};

use crate::{
    board::Move,
    search::{
        SearchInfo, Searchable, Status, Value, evaluate_wrt_root,
        generics::{Bool, False, True},
        move_ordering::{MVVLVAScorer, MoveIterator},
        transposition_table::{Hashable, ProbeResult, TransFlag, TransTable},
        quiescence::{Quiescenceable, quiescence_search}
    }
};


const DEBUG: bool = false;
const USE_TT: bool = true;
const USE_QUIESCENCE: bool = true;


pub fn negamax<V: Value, M: Move, B: Searchable<M, V> + MVVLVAScorer<M> + Hashable<V, M> + Quiescenceable<V, M>>(
    board: &mut B,
    depth: u8,
    transposition_table: &mut TransTable<V, M>,
    stop_rx: &Receiver<()>,
    search_info: &mut SearchInfo<M, V>,
) -> Result<(), ()> {

    // will be called with the correct (const) arguments to accomplish the search
    fn inner_negamax<
        IsEntry: Bool,  // whether this is the entrypoint of the recursion (only then we write to the move buffer)
        V: Value,
        M: Move,
        B: Searchable<M, V> + MVVLVAScorer<M> + Hashable<V, M> + Quiescenceable<V, M>
    >(
        board: &mut B,
        depth_left: u8,
        distance_to_root: u8,
        mut alpha: V,
        mut beta: V,
        transposition_table: &mut TransTable<V, M>,
        stop_rx: &Receiver<()>,
        search_info: &mut SearchInfo<M, V>
    ) -> V {

        // check if search should be stopped
        // TODO: Factor this modulus out as a configurable
        if search_info.nodes_searched % 4096 == 0 {
            match stop_rx.try_recv() {
                Err(_) => {},
                Ok(_)  => {search_info.was_stopped = true;}
            }
        }

        // check if search was stopped
        if search_info.was_stopped {
            return V::ZERO;
        }

        // increment nodes counter
        search_info.nodes_searched += 1;

        // probe transposition table
        let mut transposition_move = Option::None;
        if USE_TT {
            match transposition_table.probe(board, alpha, beta, depth_left, distance_to_root) {
                ProbeResult::Miss => {},

                ProbeResult::HitTooShallow(r#move) => {
                    search_info.transposition_near_hits += 1;
                    transposition_move = Option::Some(r#move);
                },

                ProbeResult::HitWrongWindow(r#move) => {
                    search_info.transposition_near_hits += 1;
                    transposition_move = Option::Some(r#move);
                },

                ProbeResult::HitLowerbound((r#move, evaluation)) => {

                    // use evaluation (a lower bound on the true evaluation) to adjust alpha
                    if evaluation > alpha {
                        search_info.transposition_hits += 1;
                        alpha = evaluation;
                    };

                    // cutoff?
                    if alpha >= beta {
                        return evaluation;
                    }

                    // remember move
                    transposition_move = Option::Some(r#move);
                },

                ProbeResult::HitUpperbound((r#move, evaluation)) => {

                    // use evaluation (an upperbound on the true evaluation) to adjust beta
                    if evaluation < beta {
                        search_info.transposition_hits += 1;
                        beta = evaluation;
                    };

                    // cutoff?
                    if alpha >= beta {
                        return evaluation;
                    }

                    // remember move
                    transposition_move = Option::Some(r#move);
                },

                ProbeResult::HitExact((r#move, evaluation)) => {
                    search_info.transposition_hits += 1;
                    if IsEntry::VALUE {
                        search_info.evaluation = Option::Some(evaluation);
                        search_info.bestmove = Option::Some(r#move);
                    }
                    return evaluation;
                }

            };
        }

        // base case of the recursion
        if depth_left == 0 {
            if USE_QUIESCENCE {
                return quiescence_search(
                    board, 16,
                    distance_to_root,
                    alpha,
                    beta,
                    stop_rx,
                    search_info
                )
            } else {
                return evaluate_wrt_root(board, distance_to_root);
            }
        }

        // get legal moves in current position
        let legal_moves = board.get_legal_moves();
        
        // if there are no legal moves to make, simply return the evaluation of the board
        if legal_moves.len() == 0 {
            return evaluate_wrt_root(board, distance_to_root);
        }

        // but them into an iterator sorting them heuristically; afterwards clear pv table for current depth
        let legal_moves = MoveIterator::from_vec(legal_moves, search_info, board, transposition_move);
        search_info.pv_table.clear_at(distance_to_root as usize);

        // remember old alpha to check (after the following loop) if search increased alpha
        let old_alpha = alpha;

        // iterate over all moves and evaluate the resulting position via a recursive call
        let mut bestmove: Option<M> = Option::None;
        let mut evaluation: V = V::MIN;
        let mut is_first_iteration_of_loop: bool = true;
        for r#move in legal_moves {

            if DEBUG && IsEntry::VALUE {
                print!("\nSearching move {}:\n", r#move.as_string());
            }

            // make move
            board.make_move(r#move);

            // recursive call
            let child_evaluation = -inner_negamax::<
                False,  // this search will never be the entrypoint of the main search
                V, M, B
            >(
                board,
                depth_left - 1,  // search one depth less
                distance_to_root + 1, // search one depth durther from the root
                -beta,
                -alpha,
                transposition_table,
                stop_rx,
                search_info
            );

            // unmake move to restore previous position
            board.unmake_move();

            if DEBUG && IsEntry::VALUE {
                print!("  Child eval {child_evaluation:?} current eval {evaluation:?} alpha {alpha:?} beta {beta:?}.");
            }

            // compare values to decide if we have found a better move
            if child_evaluation > evaluation {

                // remember better evaluation
                evaluation = child_evaluation;
                bestmove = Option::Some(r#move);

                // if we are in the entrypoint to the main search, also remember the move and it evaluation in search_info
                if IsEntry::VALUE {
                    search_info.evaluation = Option::Some(evaluation);
                    search_info.bestmove = Option::Some(r#move);
                    if DEBUG {print!(" Increased alpha!");}
                }

                // adjust alpha
                if child_evaluation > alpha {
                    alpha = child_evaluation;
                    search_info.pv_table.store(r#move, distance_to_root as usize);
                }

                // check for beta cutoff
                if child_evaluation >= beta {
                    search_info.fail_high_counter += 1;
                    if is_first_iteration_of_loop {
                        search_info.fail_high_on_first_counter += 1;
                    }
                    if DEBUG && IsEntry::VALUE {print!(" Cutoff!");}
                    break;
                }
                
            }

            // we have completed the first iteration
            is_first_iteration_of_loop = false;

        }

        // if search increaed alpha, store position and search result in the transposition table
        if USE_TT {
            transposition_table.store(
                board,
                bestmove.expect("Couldn't unwrap bestmove!"),
                evaluation,
                depth_left,  // the node to store was searched to depth "depth_left"
                distance_to_root,
                if evaluation <= old_alpha {
                    TransFlag::EvalIsUpperbound
                } else if evaluation >= beta {
                    TransFlag::EvalIsLowerbound
                } else {
                    TransFlag::EvalIsExact
                }
            );
        }

        // return evaluation of the best move found
        return evaluation;

    }

    // manual dispatch into the right implementation of inner_minimax
    (match board.whites_turn() {
        true  => inner_negamax::<True, V, M, B>,
        false => inner_negamax::<True, V, M, B>
    })(
        board, depth, 0, V::MIN, V::MAX, transposition_table, stop_rx, search_info
    );

    // if search was stopped early, return an Err
    if search_info.was_stopped {
        return Err(());
    }

    // end search
    return Ok(());
    
}