
pub mod dummy_search;
pub mod minimax;
pub mod traits;
pub mod generics;

use std::fmt;
use std::sync::mpsc::{Sender, Receiver};

use crate::search::traits::{Status, Searchable, Value};
use crate::{
    uci::Response,
    board::{Move, Board}
};


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
    pub principal_variation_line: Option<Vec<M>>
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
            principal_variation_line: Option::None
        };
    }
}

#[derive(Clone)]
pub struct SearchResult<M: Move> {
    // todo: ponder
    pub bestmove: M
}

pub type IterableSearch<V, M, B> = fn(&mut B, u8, &Receiver<()>, &mut SearchInfo<M, V>) -> Result<(Option<M>, V), ()>;
pub type Search<V, M, B> = fn(&mut B, SearchInstruction, &Receiver<()>, &Sender<Response<M, V>>) -> SearchResult<M>;


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
        let pv_line: Option<String> = match &self.principal_variation_line {
            Option::None => Option::None,
            Option::Some(v) => Option::Some(v.iter().map(|r#move| r#move.as_string()).collect::<Vec<_>>().join(" "))
        };

        println!("\nSearchInfo:");
        maybe_write!(f, "depth:    {}", self.depth);
        maybe_write!(f, "time:     {}", self.time);
        maybe_write!(f, "nodes:    {}", Option::Some(self.nodes_searched));
        maybe_write!(f, "score:    {}", score);
        maybe_write!(f, "pv line:  {:?}", &pv_line);
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