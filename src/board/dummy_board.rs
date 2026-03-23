
use crate::board::{Move, Board};

#[derive(Debug, Clone)]
pub struct DummyMove{in_algebraic: String}

#[derive(Default)]
pub struct DummyBoard{
    pub fen_like_base_position: String,  // either "startpos" or a FEN
    pub pushed_moves: Vec<DummyMove>
}

impl Move for DummyMove{
    fn as_string(&self) -> String {
        return self.in_algebraic.clone();
    }
    fn from_algebraic(s: &str) -> Self {
        Self{in_algebraic: s.to_owned()}
    }
}

impl Board<DummyMove> for DummyBoard{
    fn put_into_startpos(&mut self) {
        self.fen_like_base_position = String::from("startpos");
        self.pushed_moves = Vec::new();
    }
    fn put_into_fen(&mut self, fen: &str) {
        self.fen_like_base_position = String::from("fen ") + fen;
        self.pushed_moves = Vec::new();
    }
    fn make(&mut self, r#move: DummyMove) {
        self.pushed_moves.push(r#move);
    }
    fn unmake(&mut self) {
        self.pushed_moves.pop();
    }
    fn visualize(&self) {
        println!("Board state:");
        println!("  Base position: {}", self.fen_like_base_position);
        let pushed_moves = self.pushed_moves.iter().map(|r#move| r#move.as_string()).collect::<Vec<String>>();
        if pushed_moves.len() > 0 {
            println!("  Pushed moves:  {}", pushed_moves.join(" "));
        }
        println!();
    }
}
