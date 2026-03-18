
use crate::uci::SearchInstruction;
use crate::board::traits::{Move, Board};


pub trait Search<M: Move, B: Board<M>> {
    fn search(&mut self, search_instruction: SearchInstruction) -> M;
}