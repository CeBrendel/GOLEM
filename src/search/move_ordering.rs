
use crate::{
    board::{Piece, Move, Board},
    search::{SearchInfo, Value}
};

const fn piece_value(piece: Piece) -> i32 {
    return match piece {
        Piece::Pawn   => 100,
        Piece::Knight => 300,
        Piece::Bishop => 310,
        Piece::Rook   => 500,
        Piece::Queen  => 800,
        Piece::King   => 999
    };
}

const PV_SCORE: i32 = 10_000;

const PIECES: [Piece; 6] = [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King];

const fn init_mvv_lva_lookup() -> [[i32; 6]; 6] {

    // init empty array
    let mut array = [[0; 6]; 6];

    // loop through all possible victims and attackers and write scores to array
    let mut victim_idx = 0;
    let mut attacker_idx = 0;
    while victim_idx < 6 {
        while attacker_idx < 6 {

            // get victim and attacker
            let victim = PIECES[victim_idx];
            let attacker = PIECES[attacker_idx];

            // write difference in values to array
            let difference = piece_value(victim) - piece_value(attacker);
            array[victim as usize][attacker as usize] = 1_000 + difference;  // offset of 1_000 to ensure score is positive

            attacker_idx += 1;
        }
        victim_idx += 1;
    }
    
    return array;

}

const MVV_LVA_LOOKUP: [[i32; 6]; 6] = init_mvv_lva_lookup();


pub trait MVVLVAScorer<M: Move>: Board<M> {
    fn is_capture(&self, r#move: M) -> bool;
    fn get_victim_of(&self, r#move: M) -> Piece;
    fn get_attacker_of(&self, r#move: M) -> Piece;
}


pub struct MoveIterator<M: Move> {
    moves: Vec<M>,
    scores: Vec<i32>,
    head: usize,
    length: usize
}


impl<M: Move> MoveIterator<M> {

    pub fn from_vec<V: Value, B: MVVLVAScorer<M>>(moves: Vec<M>, search_info: &SearchInfo<M, V>, board: &B) -> Self {

        let n_moves = moves.len();
        
        // get pv move
        let pv_move = search_info.pv_table.get_pv_move();
        
        let mut scores = Vec::with_capacity(n_moves);
        for &r#move in moves.iter() {

            // init default score
            let mut score = 0;

            // check various conditions and adjust score
            if r#move == pv_move {
                // check if move is in the principal variation
                score += PV_SCORE;
            } else if board.is_capture(r#move) {
                // if the move is a capture, calculate its MVV-LVA-score
                let victim = board.get_victim_of(r#move);
                let attacker = board.get_attacker_of(r#move);
                score += MVV_LVA_LOOKUP[victim as usize][attacker as usize];
            }

            // remember score
            scores.push(score);
        }

        return Self {
            moves: moves,
            scores: scores,
            head: 0,
            length: n_moves
        };

    }

}

impl<M: Move> Iterator for MoveIterator<M> {
    type Item = M;

    fn next(&mut self) -> Option<Self::Item> {

        // check if there are any moves left to yield
        if self.head == self.length {
            return Option::None;
        }

        // get index of move with highest score
        let mut best_score = i32::MIN;
        let mut best_idx = self.head;
        for idx in self.head..self.length {
            if self.scores[idx] > best_score {
                best_score = self.scores[idx];
                best_idx = idx
            }
        }

        // swap moves and corresponding scores, so that next best move to search is at the position of the head
        (self.moves[self.head], self.moves[best_idx]) = (self.moves[best_idx], self.moves[self.head]);
        (self.scores[self.head], self.scores[best_idx]) = (self.scores[best_idx], self.scores[self.head]);

        // get move and increment head
        let r#move = self.moves[self.head];
        self.head += 1;

        // return move with highest score
        return Option::Some(r#move);

    }

}
