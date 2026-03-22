
pub mod dummy_search;

use std::fmt;
use std::sync::mpsc::{Sender, Receiver};

use crate::board::Move;
use crate::uci::Response;


#[derive(Default, Clone)]
pub struct SearchInstruction {
    // TODO: ponder, nodes, mate
    pub searchmoves: Option<Vec<String>>,
    pub wtime_in_ms: Option<usize>,
    pub btime_in_ms: Option<usize>,
    pub winc_in_ms: Option<usize>,
    pub binc_in_ms: Option<usize>,
    pub movestogo: Option<usize>,
    pub depth: Option<usize>,
    pub movetime_in_ms: Option<usize>,
    pub infinite: bool
}

#[derive(Clone)]
pub enum Score {
    // todo: Lowerbound, upperbound
    Centipawn(i32),
    Mate(u8),
}

impl Score {
    pub fn as_str(&self) -> String {
        return match self {
            Self::Centipawn(v) => format!("cp {}", v),
            Self::Mate(c)       => format!("mate {}", c),
        }
    }
}

#[derive(Clone)]
pub struct SearchInfo<M: Move> {
    // todo: seldepth, multipv, currmove, currmovenumber, hasfull, nps, tbhits, cpuload, string
    pub depth: Option<usize>,
    pub time: Option<usize>,
    pub nodes: Option<usize>,
    pub score: Option<Score>,
    pub principal_variation_line: Option<Vec<M>>
}

#[derive(Clone)]
pub struct SearchResult<M :Move> {
    // todo: ponder
    pub bestmove: M
}

pub type Search<M, B> = fn(&mut B, SearchInstruction, &Receiver<()>, &Sender<Response<M>>) -> SearchResult<M>;

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

impl<M: Move> fmt::Debug for SearchInfo<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        // turn score into a string
        let score = match &self.score {
            Option::None        => Option::None,
            Option::Some(score) => Option::Some(score.as_str())
        };

        // concat PV line into a single String (if there is any)
        let pv_line: Option<String> = match &self.principal_variation_line {
            Option::None => Option::None,
            Option::Some(v) => Option::Some(v.iter().map(|r#move| r#move.as_str()).collect::<Vec<_>>().join(" "))
        };

        println!("\nSearchInfo:");
        maybe_write!(f, "depth:    {}", self.depth);
        maybe_write!(f, "time:     {}", self.time);
        maybe_write!(f, "nodes:    {}", self.nodes);
        maybe_write!(f, "score:    {}", score);
        maybe_write!(f, "pv line:  {:?}", &pv_line);
        println!();

        return Ok(());
        
    }
}

impl<M: Move> fmt::Debug for SearchResult<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        println!("\nSearchResult:");
        writeln!(f, "bestmove: {}", self.bestmove.as_str())?;
        println!();

        return Ok(());

    }
}