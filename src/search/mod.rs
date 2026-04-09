
pub mod iterative_deepening;
pub mod minimax;
pub mod alpha_beta;
pub mod generics;
pub mod pv_table;


use std::{
    fmt,
    sync::mpsc::{Sender, Receiver},
    ops::{Add, Sub, Neg}
};

use crate::{
    uci::Response,
    board::{Move, Board},
    search::pv_table::PVTable
};


pub trait Value: Eq + PartialOrd + Clone + Copy + From<u8> + Add<Self, Output=Self> + Sub<Self, Output=Self> + Neg<Output=Self> + ToString + Send + 'static {
    const MIN: Self;
    const WHITE_IS_DEAD: Self;
    const ZERO: Self;
    const BLACK_IS_DEAD: Self;
    const MAX: Self;
}

pub enum Status {
    Ongoing,
    Stalemate,
    WhiteIsDead,
    BlackIsDead
}

pub trait Searchable<M: Move, V: Value>: Board<M> {
    fn whites_turn(&self) -> bool;
    fn unmake_move(&mut self);
    fn get_legal_moves(&self) -> Vec<M>;
    fn status(&self) -> Status;
    fn evaluate(&self) -> V;
}


pub fn evaluate_wrt_root<V: Value, M: Move, B: Board<M> + Searchable<M, V>>(board: &mut B, distance_to_root: u8) -> V {
    return match board.status() {
        Status::WhiteIsDead => board.evaluate() + V::from(distance_to_root),
        Status::BlackIsDead => board.evaluate() - V::from(distance_to_root),
        _                   => board.evaluate(),
    };
}


#[derive(Default, Clone)]
pub struct SearchInstruction {
    // TODO: ponder, nodes, mate
    pub searchmoves: Option<Vec<String>>,
    pub wtime_in_ms: Option<usize>,
    pub btime_in_ms: Option<usize>,
    pub winc_in_ms: Option<usize>,
    pub binc_in_ms: Option<usize>,
    pub movestogo: Option<usize>,
    pub depth: Option<u8>,
    pub movetime_in_ms: Option<usize>,
    pub infinite: bool
}

#[derive(Clone, Copy)]
pub struct SearchInfo<M: Move, V: Value> {
    // todo: seldepth, multipv, currmove, currmovenumber, hasfull, nps, tbhits, cpuload, string
    pub depth: Option<u8>,
    pub time: Option<u128>,
    pub nodes_searched: usize,
    pub was_stopped: bool,
    pub bestmove: Option<M>,
    pub evaluation: Option<V>,
    pub pv_table: PVTable<M>,
    pub fail_high_counter: usize,
    pub fail_high_on_first_counter: usize
}

impl<M: Move, V: Value> Default for SearchInfo<M, V> {
    fn default() -> Self {
        return Self {
            depth: Option::None,
            time: Option::None,
            nodes_searched: 0,
            was_stopped: false,
            bestmove: Option::None,
            evaluation: Option::None,
            pv_table: PVTable::new(),
            fail_high_counter: 0,
            fail_high_on_first_counter: 0
        };
    }
}

impl<V: Value, M: Move> SearchInfo<M, V> {

    fn from_pv_table(pv_table: &PVTable<M>) -> Self {
        return Self {
            depth: Option::None,
            time: Option::None,
            nodes_searched: 0,
            was_stopped: false,
            bestmove: Option::None,
            evaluation: Option::None,
            pv_table: pv_table.clone(),
            fail_high_counter: 0,
            fail_high_on_first_counter: 0
        }
    }

    fn partial_reset(&mut self) {
        self.depth = Option::None;
        self.time = Option::None;
        self.was_stopped = false;
    }
}

pub type Search<V, M, B> = fn(&mut B, SearchInstruction, &Receiver<()>, &Sender<Response<M, V>>) -> SearchInfo<M, V>;

macro_rules! implSearch {
    (<$V: ident, $B: ident, $M: ident>) => {
        impl 'static + Sync + Send + Fn(&mut B, SearchInstruction, &Receiver<()>, &Sender<Response<M, V>>) -> SearchInfo<M, V>
    };
}

pub(crate) use implSearch;


// for easier optional printing/formatting
macro_rules! maybe_write {
    ($f: expr, $description: expr, $option: expr) => {
        match $option {
            Option::None    => {},
            Option::Some(t) => writeln!($f, $description, t)?
        }
    };
}

impl fmt::Debug for SearchInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        println!("\nSearchInstructions:");
        maybe_write!(f, "searchmoves: {:?}", &self.searchmoves);
            writeln!(f, "   infinite: {}", self.infinite)?;
        maybe_write!(f, "      depth: {}", self.depth);
        maybe_write!(f, "   movetime: {}", self.movetime_in_ms);
        maybe_write!(f, "      wtime: {}", self.wtime_in_ms);
        maybe_write!(f, "      btime: {}", self.btime_in_ms);
        maybe_write!(f, "       winc: {}", self.winc_in_ms);
        maybe_write!(f, "       binc: {}", self.binc_in_ms);
        maybe_write!(f, "  movestogo: {}", self.movestogo);
        println!();

        return Ok(());
        
    }
}

impl<M: Move, V: Value + ToString> fmt::Debug for SearchInfo<M, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        // turn score into a string
        let score = match &self.evaluation {
            Option::None        => Option::None,
            Option::Some(score) => Option::Some(score.to_string())
        };

        // concat PV line into a single String (if there is any)
        let pv_line = self
            .pv_table.get_pv_line()
            .iter()
            .map(|r#move| r#move.as_string())
            .collect::<Vec<_>>()
            .join(" ");

        // calculate the quotient of fail highs on the first move, if possible
        let fhf_quotient = if self.fail_high_counter != 0 {
            Option::Some(self.fail_high_on_first_counter as f64 / self.fail_high_counter as f64)
        } else {
            Option::None
        };

        println!("\nSearchInfo:");
        maybe_write!(f, "         depth: {}", self.depth);
        maybe_write!(f, "          time: {}", self.time);
            writeln!(f, "       pv line: {:?}", pv_line)?;
        maybe_write!(f, "         score: {}", score);
            writeln!(f, "         nodes: {}", self.nodes_searched)?;
            writeln!(f, "    fail highs: {:?}", self.fail_high_counter)?;
            writeln!(f, "f.h.s on first: {:?}", self.fail_high_on_first_counter)?;
        maybe_write!(f, "  fhf-quotient: {:.3}", fhf_quotient);
        println!();

        return Ok(());
        
    }
}
