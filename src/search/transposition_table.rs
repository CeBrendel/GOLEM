
use crate::{
    board::Move,
    search::{Searchable, Value, Status}
};

const DEBUG: bool = false;

pub trait Hashable<V: Value, M: Move>: Searchable<M, V> {
    fn get_zobrist_hash(&self) -> usize;
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum TransFlag {
    #[default]Invalid,
    EvalIsUpperbound,
    EvalIsLowerbound,
    EvalIsExact
}

#[derive(Default, Clone, Copy)]
pub struct TransTableEntry<V, M> {
    pub flag: TransFlag,
    pub position_key: usize,
    pub bestmove: M,
    pub status: Status,
    pub evaluation: V,
    pub depth_searched: u8
}

// calculate size of transposition table
pub const TT_MEMORY_IN_BYTES: usize = 256 * 1024 * 1024;
const UPPER_BOUND_ON_SIZE_OF_TT_ENTRY: usize = std::mem::size_of::<TransTableEntry<i32, u32>>();
const TT_SIZE: usize = TT_MEMORY_IN_BYTES / UPPER_BOUND_ON_SIZE_OF_TT_ENTRY;

pub struct TransTable<V, M> {
    entries: Vec<TransTableEntry<V, M>>,
    pub num_entries: usize
}

pub enum ProbeResult<V, M> {
    Miss,
    HitTooShallow(M),
    HitWrongWindow(M),
    HitLowerbound((M, V)),
    HitUpperbound((M, V)),
    HitExact((M, V))
}

impl<V: Value, M: Move> TransTable<V, M> {

    pub fn new() -> Self {
        return Self {
            entries: vec![TransTableEntry::default(); TT_SIZE],
            num_entries: 0
        }
    }

    pub fn clear(&mut self) {
        // make every entry of the transposition table invalid
        self.entries.iter_mut().for_each(|entry| {entry.flag = TransFlag::Invalid;});
    }

    pub fn store<B: Hashable<V, M>>(
        &mut self,
        board: &B,
        r#move: M,
        mut evaluation: V,
        depth_searched: u8,
        distance_to_root: u8,
        flag: TransFlag,
    ) {

        if DEBUG {
            let is_mate = (evaluation < -V::MATE_TRHESHOLD) || (evaluation > V::MATE_TRHESHOLD);
            if is_mate {
                println!("    Store, eval {evaluation:?}, depth searched {depth_searched}, plies {distance_to_root}");
            }
        }

        // get hash
        let hash = board.get_zobrist_hash();
        
        // use the first few bits of the hash for indexing
        let index: usize = hash % TT_SIZE;

        // get status of game
        let status = board.status();

        // adjust mate counter (mate score is relative to root, but should be relative to node)
        evaluation = evaluation.make_relative_for_storing(distance_to_root);

        // store information
        self.entries[index] = TransTableEntry {
            position_key: hash,
            bestmove: r#move,
            status: status,
            evaluation: evaluation,
            depth_searched: depth_searched,
            flag: flag
        };
        
        // increase counter
        self.num_entries += 1;

    }

    pub fn probe<B: Hashable<V, M>>(
        &self, board: &B, alpha: V, beta: V, required_depth: u8, distance_to_root: u8
    ) -> ProbeResult<V, M> {
        
        // get hash of board
        let hash = board.get_zobrist_hash();

        // calculate TT index from hash
        let index = hash % TT_SIZE;

        // get entry at index
        let entry = self.entries[index];

        // if entry is invalid or hash does not agree, report a miss
        if entry.flag == TransFlag::Invalid || entry.position_key != hash {
            return ProbeResult::Miss;
        }

        // get bestmove
        let bestmove = entry.bestmove;
        
        // if entries search was not deep enough, we can reuse the stored move for move ordering
        // TODO: Maybe also include "depth discrepancy" as quality of that move? Might be better than PV if it is close to full depth
        if entry.depth_searched < required_depth {
            return ProbeResult::HitTooShallow(entry.bestmove);
        }

        // get evaluation and adjust to ply depth (mate score is relative to node but should be relative to root of search)
        let evaluation = entry.evaluation.make_absolute_for_probing(distance_to_root);

        if DEBUG {
            let is_mate = (evaluation < -V::MATE_TRHESHOLD) || (evaluation > V::MATE_TRHESHOLD);
            if is_mate {
                println!("    Probe, eval {evaluation:?}, req. depth {required_depth}, plies {distance_to_root}");
            }
        }

        // check for cutoff flags and adjust evaluation, keep track if we have a hit
        match entry.flag {
            TransFlag::EvalIsLowerbound => {
                if evaluation >= beta {
                    return ProbeResult::HitLowerbound((bestmove, evaluation));
                }
            },
            TransFlag::EvalIsUpperbound => {
                if evaluation <= alpha {
                    return ProbeResult::HitUpperbound((bestmove, evaluation));
                }
            },
            TransFlag::EvalIsExact => {
                return ProbeResult::HitExact((bestmove, evaluation));
            },
            TransFlag::Invalid => panic!("Near-hit was invalid!"),
        }
        
        // if we have reached hear, then we had a near-hit but the bounding wondow was wrong
        return ProbeResult::HitWrongWindow(bestmove);

    }

}
