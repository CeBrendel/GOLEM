
pub trait Move where {
    fn as_str(&self) -> &str;
    fn from_algebraic(s: &str) -> Self;
}

pub trait Board<M: Move> {
    fn put_into_startpos(&mut self);
    fn put_into_fen(&mut self, fen: &str);
    fn make(&mut self, r#move: M);
    fn unmake(&mut self);
    fn visualize(&self);
}
