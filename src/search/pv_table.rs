
use crate::board::Move;

#[derive(Clone, Copy)]
pub struct PVTable<M: Move> {
    table: [[M; 64]; 64],
    lengths: [usize; 64]
}

impl<M: Move> PVTable<M> {
    pub fn new() -> Self {
        return Self {
            table: [[M::default(); 64]; 64],
            lengths: [0; 64]
        };
    }

    pub fn clear_at(&mut self, ply: usize) {
        self.lengths[ply] = 0;
    }

    pub fn store(&mut self, r#move: M, ply: usize) {

        // store move
        self.table[ply][ply] = r#move;

        // copy pv of child
        let childs_length = self.lengths[ply + 1];
        for i in 0..childs_length {
            self.table[ply][ply + 1 + i] = self.table[ply + 1][ply + 1 + i];
        }

        // update length
        self.lengths[ply] = childs_length + 1;

    }

    pub fn get_pv_line(&self) -> &[M] {
        return &self.table[0][0..self.lengths[0]];
    }

    pub fn get_pv_move(&self) -> M {
        return self.table[0][0];
    }

}