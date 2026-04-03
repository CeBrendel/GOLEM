
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

#[derive(Clone)]
pub struct SearchInfo<M: Move, V: Value> {
    // todo: seldepth, multipv, currmove, currmovenumber, hasfull, nps, tbhits, cpuload, string
    pub depth: Option<u8>,
    pub time: Option<usize>,
    pub nodes_searched: usize,
    pub was_stopped: bool,
    pub bestmove: Option<M>,
    pub evaluation: Option<V>,
    pub pv_table: PVTable<M>
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
            pv_table: PVTable::new()
        };
    }
}

#[derive(Clone)]
pub struct SearchResult<M: Move> {
    // todo: ponder
    pub bestmove: M
}


pub type Search<V, M, B> = fn(&mut B, SearchInstruction, &Receiver<()>, &Sender<Response<M, V>>) -> SearchResult<M>;

macro_rules! implSearch {
    (<$V: ident, $B: ident, $M: ident>) => {
        impl 'static + Sync + Send + Fn(&mut B, SearchInstruction, &Receiver<()>, &Sender<Response<M, V>>) -> SearchResult<M>
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
        maybe_write!(f, "   infinite: {}", Option::Some(self.infinite));
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
            .pv_table.get_pv()
            .iter()
            .map(|r#move| r#move.as_string())
            .collect::<Vec<_>>()
            .join(" ");

        println!("\nSearchInfo:");
        maybe_write!(f, "depth:    {}", self.depth);
        maybe_write!(f, "time:     {}", self.time);
        maybe_write!(f, "nodes:    {}", Option::Some(self.nodes_searched));
        maybe_write!(f, "score:    {}", score);
        writeln!(f, "pv line:  {:?}", pv_line)?;
        println!();

        return Ok(());
        
    }
}

impl<M: Move> fmt::Debug for SearchResult<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        println!("\nSearchResult:");
        writeln!(f, "bestmove: {}", self.bestmove.as_string())?;
        println!();

        return Ok(());

    }
}